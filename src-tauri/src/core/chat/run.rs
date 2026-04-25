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
    ChatRole, ChatTurn, FinishReason, GenerateRequest, LlmEvent, ProviderRegistry, ToolCall,
    ToolResult,
};
use crate::core::skill::SkillRegistry;
use crate::core::tool::{ToolContext, ToolRegistry, ToolSchema};

pub type RunId = String;

/// Max number of prior messages included when prompting the model.
const HISTORY_WINDOW: usize = 20;
const DEFAULT_MAX_TOKENS: u32 = 4096;
const MAX_REACT_ITERATIONS: u32 = 12;

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
    ReasoningDelta {
        text: String,
    },
    ToolCallStarted {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolCallCompleted {
        id: String,
        content: String,
        is_error: bool,
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
    tools: ToolRegistry,
    skills: SkillRegistry,
    active: Arc<Mutex<HashMap<RunId, ChatRunHandle>>>,
}

impl ChatRunner {
    pub fn new(
        app: AppHandle,
        repo: ChatRepo,
        registry: ProviderRegistry,
        tools: ToolRegistry,
        skills: SkillRegistry,
    ) -> Self {
        Self {
            app,
            repo,
            registry,
            tools,
            skills,
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Kick off a ReAct-style run: persist the user message, then loop LLM →
    /// tool calls → tool results → LLM until the model finishes without
    /// another tool call (or we hit the iteration cap).
    pub async fn start(
        &self,
        agent: Agent,
        provider_id: String,
        model_id: String,
        user_text: String,
    ) -> CoreResult<RunStarted> {
        let user_msg = ChatMessage::new(MessageRole::User, user_text.clone());
        self.repo.append(&agent.id, &user_msg)?;

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
        let tools = self.tools.clone();
        let skills = self.skills.clone();
        let agent_id = agent.id.clone();
        let run_id_bg = run_id.clone();
        let assistant_id_bg = assistant_id.clone();
        let active = self.active.clone();

        tokio::spawn(async move {
            use futures_util::FutureExt;
            use std::panic::AssertUnwindSafe;

            let channel = format!("chat:run:{run_id_bg}");

            // Wrap the loop in catch_unwind so a panic deep in the provider
            // (e.g. an unsupported GGUF architecture in the inference engine)
            // surfaces as a clean error event in the UI instead of a silent
            // worker-thread death.
            let outcome = AssertUnwindSafe(react_loop(
                &app,
                &channel,
                provider,
                &repo,
                &tools,
                &skills,
                &agent,
                &model_id,
                &assistant_id_bg,
                cancel.clone(),
            ))
            .catch_unwind()
            .await;

            match outcome {
                Ok(Ok(final_msg)) => {
                    let _ = app.emit(
                        &channel,
                        RunEvent::Finish {
                            reason: "stop".to_string(),
                            message: final_msg,
                        },
                    );
                }
                Ok(Err(e)) => {
                    let code = match &e {
                        CoreError::Cancelled => "cancelled",
                        _ => "llm_error",
                    };
                    let _ = app.emit(
                        &channel,
                        RunEvent::Error {
                            code: code.to_string(),
                            message: e.to_string(),
                        },
                    );
                }
                Err(panic) => {
                    let panic_msg = panic
                        .downcast_ref::<&str>()
                        .map(|s| (*s).to_string())
                        .or_else(|| panic.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "panic without payload".to_string());
                    tracing::error!(panic = %panic_msg, "chat run panicked");
                    let _ = app.emit(
                        &channel,
                        RunEvent::Error {
                            code: "panic".to_string(),
                            message: format!(
                                "Provider crashte intern. Bitte einen anderen Provider/ein anderes Modell wählen. Detail: {panic_msg}"
                            ),
                        },
                    );
                }
            }

            let _ = agent_id;
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

#[allow(clippy::too_many_arguments)]
async fn react_loop(
    app: &AppHandle,
    channel: &str,
    provider: Arc<dyn crate::core::llm::LlmProvider>,
    repo: &ChatRepo,
    tools: &ToolRegistry,
    skills: &SkillRegistry,
    agent: &Agent,
    model_id: &str,
    final_msg_id: &str,
    cancel: CancellationToken,
) -> CoreResult<ChatMessage> {
    let agent_folder = agent.folder.clone();
    let system_prompt = compose_system_prompt(agent, skills);
    let tool_schemas: Vec<ToolSchema> = if provider.supports_tools() {
        collect_tool_schemas(agent, skills, tools)
    } else {
        Vec::new()
    };

    // Build initial turn list from persisted history, trimmed to the window.
    let history = repo.load(&agent.id)?;
    let mut turns = history_to_turns(&history);
    if turns.len() > HISTORY_WINDOW {
        turns.drain(0..turns.len() - HISTORY_WINDOW);
    }

    for _ in 0..MAX_REACT_ITERATIONS {
        let (text, reasoning, tool_calls, finish_reason) = run_single_call(
            app,
            channel,
            provider.clone(),
            &tool_schemas,
            model_id,
            system_prompt.clone(),
            turns.clone(),
            cancel.clone(),
        )
        .await?;

        // Record the assistant turn we just produced, either in-memory for
        // the next LLM call or (on final iteration) as the persisted message.
        if tool_calls.is_empty() {
            let assistant_msg = ChatMessage {
                id: final_msg_id.to_string(),
                role: MessageRole::Assistant,
                content: text.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                tool_calls: Vec::new(),
                tool_results: Vec::new(),
                reasoning: reasoning.clone(),
            };
            if !assistant_msg.content.is_empty() || reasoning.is_some() {
                let _ = repo.append(&agent.id, &assistant_msg);
            }
            return Ok(assistant_msg);
        }

        // Assistant asked for tool calls — keep the assistant turn in the
        // history (text + tool_calls) and append a user turn carrying the
        // results of each tool call, then loop.
        turns.push(ChatTurn {
            role: ChatRole::Assistant,
            content: text.clone(),
            tool_calls: tool_calls.clone(),
            tool_results: Vec::new(),
        });

        let assistant_persist = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: text,
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_calls: tool_calls.clone(),
            tool_results: Vec::new(),
            reasoning: reasoning.clone(),
        };
        let _ = repo.append(&agent.id, &assistant_persist);

        // Execute each tool and collect results.
        let mut results = Vec::with_capacity(tool_calls.len());
        for call in &tool_calls {
            if cancel.is_cancelled() {
                return Err(CoreError::Cancelled);
            }
            let folder = agent_folder.clone().ok_or_else(|| {
                CoreError::Llm("Agent hat keinen Ordner konfiguriert.".to_string())
            })?;
            let ctx = ToolContext {
                agent_id: agent.id.clone(),
                agent_folder: folder,
                app: app.clone(),
            };
            let (content, is_error) = match tools.get(&call.name) {
                Ok(tool) => match tool.execute(call.arguments.clone(), &ctx).await {
                    Ok(out) => (out.content, false),
                    Err(e) => (format!("tool error: {e}"), true),
                },
                Err(e) => (format!("unknown tool: {e}"), true),
            };
            let _ = app.emit(
                channel,
                RunEvent::ToolCallCompleted {
                    id: call.id.clone(),
                    content: content.clone(),
                    is_error,
                },
            );
            results.push(ToolResult {
                tool_use_id: call.id.clone(),
                content,
                is_error,
            });
        }

        let tool_turn = ChatTurn {
            role: ChatRole::User,
            content: String::new(),
            tool_calls: Vec::new(),
            tool_results: results.clone(),
        };
        turns.push(tool_turn);

        let tool_persist = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Tool,
            content: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_calls: Vec::new(),
            tool_results: results,
            reasoning: None,
        };
        let _ = repo.append(&agent.id, &tool_persist);

        if finish_reason == FinishReason::Stop {
            // LLM said "stop" even though it also produced tool calls. Feed
            // the results back in any case and let it continue if needed.
        }
    }

    Err(CoreError::Llm(format!(
        "ReAct-Loop-Limit ({MAX_REACT_ITERATIONS}) erreicht."
    )))
}

#[allow(clippy::too_many_arguments)]
async fn run_single_call(
    app: &AppHandle,
    channel: &str,
    provider: Arc<dyn crate::core::llm::LlmProvider>,
    tool_schemas: &[ToolSchema],
    model_id: &str,
    system_prompt: Option<String>,
    turns: Vec<ChatTurn>,
    cancel: CancellationToken,
) -> CoreResult<(String, Option<String>, Vec<ToolCall>, FinishReason)> {
    let (tx, mut rx) = mpsc::channel::<LlmEvent>(64);

    let request = GenerateRequest {
        model_id: model_id.to_string(),
        system_prompt,
        turns,
        tools: tool_schemas.to_vec(),
        max_tokens: DEFAULT_MAX_TOKENS,
        temperature: None,
    };

    let provider_bg = provider.clone();
    let cancel_for_provider = cancel.clone();
    let gen_task =
        tokio::spawn(async move { provider_bg.generate(request, tx, cancel_for_provider).await });

    let mut buffer = String::new();
    let mut reasoning_buffer = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut finish_reason: Option<FinishReason> = None;
    let mut terminal_error: Option<(String, String)> = None;

    while let Some(event) = rx.recv().await {
        match event {
            LlmEvent::TextDelta { text } => {
                buffer.push_str(&text);
                let _ = app.emit(channel, RunEvent::Delta { text });
            }
            LlmEvent::ReasoningDelta { text } => {
                reasoning_buffer.push_str(&text);
                let _ = app.emit(channel, RunEvent::ReasoningDelta { text });
            }
            LlmEvent::ToolCall(call) => {
                let _ = app.emit(
                    channel,
                    RunEvent::ToolCallStarted {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        arguments: call.arguments.clone(),
                    },
                );
                tool_calls.push(call);
            }
            LlmEvent::Finish { reason } => {
                finish_reason = Some(reason);
            }
            LlmEvent::Error { code, message } => {
                terminal_error = Some((code, message));
            }
        }
    }

    let _ = gen_task.await;
    if let Some((code, message)) = terminal_error {
        return Err(CoreError::Llm(format!("{code}: {message}")));
    }
    let reasoning = if reasoning_buffer.trim().is_empty() {
        None
    } else {
        Some(reasoning_buffer)
    };
    Ok((
        buffer,
        reasoning,
        tool_calls,
        finish_reason.unwrap_or(FinishReason::Stop),
    ))
}

fn compose_system_prompt(agent: &Agent, skills: &SkillRegistry) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if !agent.system_prompt.trim().is_empty() {
        parts.push(agent.system_prompt.trim().to_string());
    }
    for skill_name in &agent.skills {
        if let Some(skill) = skills.get(skill_name) {
            let mut block = format!("## Skill: {}\n", skill.title);
            block.push_str(&skill.description);
            if !skill.body.trim().is_empty() {
                block.push_str("\n\n");
                block.push_str(skill.body.trim());
            }
            parts.push(block);
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n"))
    }
}

fn collect_tool_schemas(
    agent: &Agent,
    skills: &SkillRegistry,
    tools: &ToolRegistry,
) -> Vec<ToolSchema> {
    let mut wanted: Vec<String> = Vec::new();
    for skill_name in &agent.skills {
        if let Some(skill) = skills.get(skill_name) {
            for t in &skill.tools {
                if !wanted.contains(t) {
                    wanted.push(t.clone());
                }
            }
        }
    }
    tools.schemas_for(&wanted)
}

fn history_to_turns(history: &[ChatMessage]) -> Vec<ChatTurn> {
    let mut turns = Vec::new();
    for m in history {
        let role = match m.role {
            MessageRole::User => ChatRole::User,
            MessageRole::Assistant => ChatRole::Assistant,
            MessageRole::Tool => ChatRole::User,
            MessageRole::System => continue,
        };
        turns.push(ChatTurn {
            role,
            content: m.content.clone(),
            tool_calls: m.tool_calls.clone(),
            tool_results: m.tool_results.clone(),
        });
    }
    turns
}
