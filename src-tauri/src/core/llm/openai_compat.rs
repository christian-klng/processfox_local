//! Shared implementation for OpenAI-compatible chat-completions APIs.
//!
//! Used by `openai` (OpenAI directly) and `openrouter` (OpenRouter proxy).
//! Both endpoints speak the same request/response shape, differing only in
//! base URL and a couple of recommended headers.

use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{ChatRole, FinishReason, GenerateRequest, LlmEvent};
use crate::core::error::{CoreError, CoreResult};

pub struct OpenAiCompat {
    pub provider_id: &'static str,
    pub secret_key: &'static str,
    pub api_base: &'static str,
    pub extra_headers: &'static [(&'static str, &'static str)],
    pub http: reqwest::Client,
}

#[derive(Serialize)]
struct Request {
    model: String,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct WireMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize, Debug)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    #[serde(default)]
    delta: Delta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

fn map_role(role: ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
    }
}

fn map_finish(raw: Option<&str>) -> FinishReason {
    match raw {
        Some("stop") => FinishReason::Stop,
        Some("length") => FinishReason::MaxTokens,
        _ => FinishReason::Stop,
    }
}

fn compose_wire(request: &GenerateRequest) -> Vec<WireMessage> {
    let mut messages = Vec::new();
    if let Some(sys) = request.system_prompt.as_deref() {
        if !sys.trim().is_empty() {
            messages.push(WireMessage {
                role: "system",
                content: sys.to_string(),
            });
        }
    }
    for turn in &request.turns {
        messages.push(WireMessage {
            role: map_role(turn.role),
            content: turn.content.clone(),
        });
    }
    messages
}

impl OpenAiCompat {
    pub fn new(
        provider_id: &'static str,
        secret_key: &'static str,
        api_base: &'static str,
        extra_headers: &'static [(&'static str, &'static str)],
    ) -> CoreResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent("ProcessFox/0.1")
            .build()
            .map_err(|e| CoreError::Http(e.to_string()))?;
        Ok(Self {
            provider_id,
            secret_key,
            api_base,
            extra_headers,
            http,
        })
    }

    fn apply_extras(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        for (k, v) in self.extra_headers {
            req = req.header(*k, *v);
        }
        req
    }

    pub async fn validate(&self) -> CoreResult<()> {
        let api_key = crate::core::secrets::get_api_key(self.secret_key)?
            .ok_or_else(|| CoreError::MissingApiKey(self.secret_key.to_string()))?;

        let url = format!("{}/models", self.api_base);
        let req = self
            .http
            .get(&url)
            .header("authorization", format!("Bearer {api_key}"));
        let req = self.apply_extras(req);

        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Http(e.to_string()))?;

        if resp.status().is_success() {
            return Ok(());
        }

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(CoreError::Llm(format!(
            "{}-Key abgelehnt ({status}): {}",
            self.provider_id,
            body.chars().take(200).collect::<String>()
        )))
    }

    pub async fn generate(
        &self,
        request: GenerateRequest,
        sink: mpsc::Sender<LlmEvent>,
        cancel: CancellationToken,
    ) -> CoreResult<()> {
        let api_key = crate::core::secrets::get_api_key(self.secret_key)?
            .ok_or_else(|| CoreError::MissingApiKey(self.secret_key.to_string()))?;

        let body = Request {
            model: request.model_id.clone(),
            messages: compose_wire(&request),
            max_tokens: Some(request.max_tokens),
            stream: true,
            temperature: request.temperature,
        };

        let url = format!("{}/chat/completions", self.api_base);
        let req = self
            .http
            .post(&url)
            .header("authorization", format!("Bearer {api_key}"))
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .json(&body);
        let req = self.apply_extras(req);

        let response = tokio::select! {
            r = req.send() => r.map_err(|e| CoreError::Http(e.to_string()))?,
            _ = cancel.cancelled() => {
                let _ = sink
                    .send(LlmEvent::Finish { reason: FinishReason::Cancelled })
                    .await;
                return Err(CoreError::Cancelled);
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let msg = format!("{} {status}: {text}", self.provider_id);
            let _ = sink
                .send(LlmEvent::Error {
                    code: "http_error".to_string(),
                    message: msg.clone(),
                })
                .await;
            return Err(CoreError::Llm(msg));
        }

        let mut stream = response.bytes_stream().eventsource();

        while let Some(event) = tokio::select! {
            e = stream.next() => e,
            _ = cancel.cancelled() => None,
        } {
            if cancel.is_cancelled() {
                let _ = sink
                    .send(LlmEvent::Finish {
                        reason: FinishReason::Cancelled,
                    })
                    .await;
                return Err(CoreError::Cancelled);
            }

            let event = match event {
                Ok(e) => e,
                Err(e) => {
                    let msg = format!("SSE decode error: {e}");
                    let _ = sink
                        .send(LlmEvent::Error {
                            code: "sse_error".into(),
                            message: msg.clone(),
                        })
                        .await;
                    return Err(CoreError::Llm(msg));
                }
            };

            if event.data.is_empty() {
                continue;
            }
            // Sentinel that OpenAI / OpenRouter send at end of stream.
            if event.data.trim() == "[DONE]" {
                let _ = sink
                    .send(LlmEvent::Finish {
                        reason: FinishReason::Stop,
                    })
                    .await;
                return Ok(());
            }

            let chunk: StreamChunk = match serde_json::from_str(&event.data) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        provider = self.provider_id,
                        data = %event.data,
                        error = %e,
                        "failed to parse SSE chunk"
                    );
                    continue;
                }
            };

            for choice in chunk.choices {
                if let Some(text) = choice.delta.content {
                    if !text.is_empty() && sink.send(LlmEvent::TextDelta { text }).await.is_err() {
                        return Ok(());
                    }
                }
                if let Some(reason) = choice.finish_reason.as_deref() {
                    let _ = sink
                        .send(LlmEvent::Finish {
                            reason: map_finish(Some(reason)),
                        })
                        .await;
                }
            }
        }

        let _ = sink
            .send(LlmEvent::Finish {
                reason: FinishReason::Stop,
            })
            .await;
        Ok(())
    }
}

impl std::fmt::Debug for OpenAiCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiCompat")
            .field("provider_id", &self.provider_id)
            .field("api_base", &self.api_base)
            .finish()
    }
}
