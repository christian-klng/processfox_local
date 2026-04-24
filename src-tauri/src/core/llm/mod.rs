pub mod anthropic;
pub mod local_gguf;
pub mod openai;
pub mod openai_compat;
pub mod openrouter;
pub mod registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::error::CoreResult;

pub use registry::{ProviderId, ProviderRegistry};

/// A single turn in the conversation, as sent to the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub model_id: String,
    pub system_prompt: Option<String>,
    pub turns: Vec<ChatTurn>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
}

/// Events emitted by a provider during a single generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LlmEvent {
    TextDelta { text: String },
    Finish { reason: FinishReason },
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    MaxTokens,
    Cancelled,
    Error,
}

/// Implementation-agnostic interface for producing text from an LLM.
///
/// `generate` streams `LlmEvent`s into `sink` and must respect `cancel`
/// promptly. It returns when the stream is exhausted, an unrecoverable error
/// occurs, or cancellation is observed.
#[async_trait]
pub trait LlmProvider: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &'static str;

    async fn generate(
        &self,
        request: GenerateRequest,
        sink: mpsc::Sender<LlmEvent>,
        cancel: CancellationToken,
    ) -> CoreResult<()>;

    /// Verify that the currently stored API key is valid by issuing a cheap
    /// request to the provider (e.g. listing models). Returns Ok(()) on
    /// success, a descriptive `CoreError` otherwise.
    async fn validate(&self) -> CoreResult<()>;
}
