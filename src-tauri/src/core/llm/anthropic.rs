use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{ChatRole, ChatTurn, FinishReason, GenerateRequest, LlmEvent, LlmProvider};
use crate::core::error::{CoreError, CoreResult};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

#[derive(Debug)]
pub struct AnthropicProvider {
    http: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new() -> CoreResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent("ProcessFox/0.1")
            .build()
            .map_err(|e| CoreError::Http(e.to_string()))?;
        Ok(Self { http })
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new().expect("Anthropic HTTP client must build")
    }
}

#[derive(Serialize)]
struct Request {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<WireMessage>,
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
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent {
    MessageStart {},
    ContentBlockStart {},
    Ping {},
    ContentBlockDelta {
        delta: ContentDelta,
    },
    ContentBlockStop {},
    MessageDelta {
        delta: MessageDelta,
    },
    MessageStop {},
    Error {
        error: ApiErrorBody,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentDelta {
    TextDelta {
        text: String,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
struct MessageDelta {
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiErrorBody {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

fn map_role(role: ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "user", // fallback — system is sent via top-level `system`
    }
}

fn map_stop_reason(raw: Option<&str>) -> FinishReason {
    match raw {
        Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
        Some("max_tokens") => FinishReason::MaxTokens,
        _ => FinishReason::Stop,
    }
}

fn to_wire(turns: &[ChatTurn]) -> Vec<WireMessage> {
    turns
        .iter()
        .filter(|t| t.role != ChatRole::System) // system prompt goes in a dedicated field
        .map(|t| WireMessage {
            role: map_role(t.role),
            content: t.content.clone(),
        })
        .collect()
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    async fn validate(&self) -> CoreResult<()> {
        let api_key = crate::core::secrets::get_api_key("anthropic")?
            .ok_or_else(|| CoreError::MissingApiKey("anthropic".to_string()))?;

        let resp = self
            .http
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", &api_key)
            .header("anthropic-version", API_VERSION)
            .send()
            .await
            .map_err(|e| CoreError::Http(e.to_string()))?;

        if resp.status().is_success() {
            return Ok(());
        }

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(CoreError::Llm(format!(
            "Anthropic-Key abgelehnt ({status}): {}",
            body.chars().take(200).collect::<String>()
        )))
    }

    async fn generate(
        &self,
        request: GenerateRequest,
        sink: mpsc::Sender<LlmEvent>,
        cancel: CancellationToken,
    ) -> CoreResult<()> {
        let api_key = crate::core::secrets::get_api_key("anthropic")?
            .ok_or_else(|| CoreError::MissingApiKey("anthropic".to_string()))?;

        let body = Request {
            model: request.model_id.clone(),
            max_tokens: request.max_tokens,
            system: request.system_prompt.clone(),
            messages: to_wire(&request.turns),
            stream: true,
            temperature: request.temperature,
        };

        let response = tokio::select! {
            r = self
                .http
                .post(API_URL)
                .header("x-api-key", &api_key)
                .header("anthropic-version", API_VERSION)
                .header("content-type", "application/json")
                .header("accept", "text/event-stream")
                .json(&body)
                .send() => r.map_err(|e| CoreError::Http(e.to_string()))?,
            _ = cancel.cancelled() => {
                let _ = sink.send(LlmEvent::Finish { reason: FinishReason::Cancelled }).await;
                return Err(CoreError::Cancelled);
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            let msg = format!("Anthropic {status}: {text}");
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

            let parsed: StreamEvent = match serde_json::from_str(&event.data) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(data = %event.data, error = %e, "failed to parse Anthropic SSE event");
                    continue;
                }
            };

            match parsed {
                StreamEvent::ContentBlockDelta {
                    delta: ContentDelta::TextDelta { text },
                } if !text.is_empty() => {
                    if sink.send(LlmEvent::TextDelta { text }).await.is_err() {
                        return Ok(());
                    }
                }
                StreamEvent::MessageDelta { delta } => {
                    if let Some(reason) = delta.stop_reason.as_deref() {
                        let _ = sink
                            .send(LlmEvent::Finish {
                                reason: map_stop_reason(Some(reason)),
                            })
                            .await;
                    }
                }
                StreamEvent::MessageStop {} => {
                    // MessageDelta typically emitted the final reason already; if not, send Stop.
                    let _ = sink
                        .send(LlmEvent::Finish {
                            reason: FinishReason::Stop,
                        })
                        .await;
                    return Ok(());
                }
                StreamEvent::Error { error } => {
                    let msg = error.message.unwrap_or_else(|| "unknown error".into());
                    let code = error.kind.unwrap_or_else(|| "provider_error".into());
                    let _ = sink
                        .send(LlmEvent::Error {
                            code: code.clone(),
                            message: msg.clone(),
                        })
                        .await;
                    return Err(CoreError::Llm(format!("{code}: {msg}")));
                }
                _ => {}
            }
        }

        // Stream ended without explicit MessageStop (unusual).
        let _ = sink
            .send(LlmEvent::Finish {
                reason: FinishReason::Stop,
            })
            .await;
        Ok(())
    }
}
