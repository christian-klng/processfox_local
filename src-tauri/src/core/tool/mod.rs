pub mod registry;
pub mod tools;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tauri::AppHandle;

use super::error::CoreResult;

pub use registry::ToolRegistry;

/// JSON-Schema description of a tool, used to tell the LLM what tools are
/// available and what shape their arguments take.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    /// A JSON-Schema (draft-2020-12-ish) describing the tool's input.
    pub input_schema: JsonValue,
}

/// Runtime context handed to each tool on execution. Tools must use
/// `agent_folder` as the sandbox root; writes and reads outside it are
/// rejected in `ensure_in_agent_folder`.
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub agent_folder: PathBuf,
    pub app: AppHandle,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutput {
    /// Text payload handed back to the LLM. Keep it compact — it's counted
    /// toward the context window.
    pub content: String,
    /// Optional structured payload for the UI to render (e.g. a list of
    /// files, a diff). Not sent to the LLM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_hint: Option<JsonValue>,
}

impl ToolOutput {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ui_hint: None,
        }
    }
}

/// Individual tool. Stateless — each call gets a fresh `ToolContext`. Tools
/// must be `Send + Sync` because the registry is cloned into every chat run.
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn schema(&self) -> ToolSchema;

    async fn execute(&self, input: JsonValue, ctx: &ToolContext) -> CoreResult<ToolOutput>;
}
