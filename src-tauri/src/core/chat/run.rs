use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::repo::{ChatMessage, ChatRepo, MessageRole};
use crate::core::agent::Agent;
use crate::core::error::{CoreError, CoreResult};
use crate::core::llm::{
    ChatRole, ChatTurn, FinishReason, GenerateRequest, LlmEvent, ProviderRegistry,
};

pub type RunId = String;

/// Max number of prior messages included when prompting the model.
const HISTORY_WINDOW: usize = 20;
const DEFAULT_MAX_TOKENS: u32 = 4096;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStarted {
    pub run_id: RunId,
    pub assistant_message_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RunEvent {
    Delta {
        text: String,
    },
    Finish {
        reason: String,
        message: ChatMessage,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug)]
pub struct ChatRunHandle {
    pub cancel: CancellationToken,
}

#[derive(Clone, Debug)]
pub struct ChatRunner {
    app: AppHandle,
    repo: ChatRepo,
    registry: ProviderRegistry,
    active: Arc<Mutex<HashMap<RunId, ChatRunHandle>>>,
}

impl ChatRunner {
    pub fn new(app: AppHandle, repo: ChatRepo, registry: ProviderRegistry) -> Self {
        Self {
            app,
            repo,
            registry,
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a new run: persists the user message, spawns the LLM call, and
    /// returns the run-id + the assistant-message-id the UI should build up.
    pub async fn start(
        &self,
        agent: Agent,
        provider_id: String,
        model_id: String,
        user_text: String,
    ) -> CoreResult<RunStarted> {
        let user_msg = ChatMessage::new(MessageRole::User, user_text.clone());
        self.repo.append(&agent.id, &user_msg)?;

        let mut turns = self
            .repo
            .load(&agent.id)?
            .into_iter()
            .filter_map(|m| {
                let role = match m.role {
                    MessageRole::User => ChatRole::User,
                    MessageRole::Assistant => ChatRole::Assistant,
                    _ => return None,
                };
                Some(ChatTurn {
                    role,
                    content: m.content,
                })
            })
            .collect::<Vec<_>>();
        if turns.len() > HISTORY_WINDOW {
            turns.drain(0..turns.len() - HISTORY_WINDOW);
        }

        let system_prompt = if agent.system_prompt.trim().is_empty() {
            None
        } else {
            Some(agent.system_prompt.clone())
        };

        let request = GenerateRequest {
            model_id,
            system_prompt,
            turns,
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: None,
        };

        let provider = self.registry.get(&provider_id)?;
        let run_id = Uuid::new_v4().to_string();
        let assistant_id = Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();

        {
            let mut active = self.active.lock().await;
            active.insert(
                run_id.clone(),
                ChatRunHandle {
                    cancel: cancel.clone(),
                },
            );
        }

        let app = self.app.clone();
        let repo = self.repo.clone();
        let agent_id = agent.id.clone();
        let run_id_bg = run_id.clone();
        let assistant_id_bg = assistant_id.clone();
        let active = self.active.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel::<LlmEvent>(64);
            let cancel_for_provider = cancel.clone();
            let provider_bg = provider.clone();

            let gen_task = tokio::spawn(async move {
                provider_bg.generate(request, tx, cancel_for_provider).await
            });

            let mut buffer = String::new();
            let mut finish_reason: Option<FinishReason> = None;
            let mut terminal_error: Option<(String, String)> = None;

            while let Some(event) = rx.recv().await {
                match event {
                    LlmEvent::TextDelta { text } => {
                        buffer.push_str(&text);
                        let _ =
                            app.emit(&format!("chat:run:{run_id_bg}"), RunEvent::Delta { text });
                    }
                    LlmEvent::Finish { reason } => {
                        finish_reason = Some(reason);
                    }
                    LlmEvent::Error { code, message } => {
                        terminal_error = Some((code, message));
                    }
                }
            }

            // Await the provider task to surface any error the channel didn't carry.
            let gen_result = gen_task.await;

            let final_reason = finish_reason.unwrap_or(FinishReason::Stop);

            if let Some((code, message)) = terminal_error {
                let _ = app.emit(
                    &format!("chat:run:{run_id_bg}"),
                    RunEvent::Error { code, message },
                );
            } else {
                let assistant_msg = ChatMessage {
                    id: assistant_id_bg.clone(),
                    role: MessageRole::Assistant,
                    content: buffer,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                // Persist only if we actually produced something OR we weren't
                // cancelled mid-stream. Cancelled runs still save what we got.
                if !assistant_msg.content.is_empty() {
                    if let Err(e) = repo.append(&agent_id, &assistant_msg) {
                        tracing::warn!(error = %e, "failed to persist assistant message");
                    }
                }

                let _ = app.emit(
                    &format!("chat:run:{run_id_bg}"),
                    RunEvent::Finish {
                        reason: reason_str(final_reason).to_string(),
                        message: assistant_msg,
                    },
                );
            }

            if let Ok(Err(e)) = gen_result {
                if !matches!(e, CoreError::Cancelled) {
                    tracing::warn!(error = %e, "provider returned error after stream close");
                }
            }

            let mut active = active.lock().await;
            active.remove(&run_id_bg);
        });

        Ok(RunStarted {
            run_id,
            assistant_message_id: assistant_id,
        })
    }

    pub async fn cancel(&self, run_id: &str) {
        let active = self.active.lock().await;
        if let Some(handle) = active.get(run_id) {
            handle.cancel.cancel();
        }
    }
}

fn reason_str(r: FinishReason) -> &'static str {
    match r {
        FinishReason::Stop => "stop",
        FinishReason::MaxTokens => "max_tokens",
        FinishReason::Cancelled => "cancelled",
        FinishReason::Error => "error",
    }
}
