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

/// Human-in-the-loop preview shape — what the chat runner shows to the user
/// before a write tool actually executes. The runner emits this in a
/// `RunEvent::HitlRequest` and waits for the user's `HitlDecision`.
#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum HitlPreview {
    /// Append text to an existing file (or create it if missing).
    AppendToFile {
        path: String,
        content: String,
        /// True when the target file does not yet exist; UI can surface
        /// "neu erstellen" vs "anhängen".
        creates_file: bool,
        /// Last few lines of the existing file (max ~600 chars). Helps the
        /// reviewer spot a format mismatch before approving. `None` for new
        /// files or when the tail can't be read.
        #[serde(skip_serializing_if = "Option::is_none")]
        existing_tail: Option<String>,
    },
    /// Create a new .docx (or overwrite an existing one). The preview shows
    /// a plaintext rendering of the first few blocks so the user can sanity
    /// check structure before approving.
    WriteDocx {
        path: String,
        block_count: usize,
        preview_text: String,
        creates_file: bool,
    },
    /// Append blocks to an existing .docx (or create one if missing). The
    /// existing document is round-tripped through docx-rs so its formatting
    /// is preserved as far as the parser supports it.
    AppendToDocx {
        path: String,
        block_count: usize,
        preview_text: String,
        creates_file: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        existing_tail: Option<String>,
    },
    /// Replace the entire contents of a text or markdown file. The frontend
    /// renders `before` and `after` as a unified diff so the user can spot
    /// every line that changes.
    RewriteFile {
        path: String,
        before: String,
        after: String,
        creates_file: bool,
    },
    // Future variants in later Phase-4 etappes:
    //   UpdateCells  { path, sheet, changes: Vec<CellChange> }
}

/// Decision returned for a pending HITL request.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum HitlDecision {
    Approve,
    Reject {
        #[serde(default)]
        reason: Option<String>,
    },
}

/// Individual tool. Stateless — each call gets a fresh `ToolContext`. Tools
/// must be `Send + Sync` because the registry is cloned into every chat run.
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn schema(&self) -> ToolSchema;

    async fn execute(&self, input: JsonValue, ctx: &ToolContext) -> CoreResult<ToolOutput>;

    /// Probe before execution: if the tool would mutate the agent's folder,
    /// return a preview. The chat runner emits this to the UI as a HITL
    /// request and only calls `execute` after the user approves. Read-only
    /// tools return `None` (the default). `ctx` lets the tool read existing
    /// file state so the preview can show before/after context.
    fn requires_approval(&self, _input: &JsonValue, _ctx: &ToolContext) -> Option<HitlPreview> {
        None
    }
}
