//! Shared type stubs used across core modules.
//!
//! These are intentionally minimal in Phase 1; Phase 2 (LLM) and Phase 3
//! (Tools/Skills) will fill out the fields.

use serde::{Deserialize, Serialize};

/// A single chat message. Filled out in Phase 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// A skill surfaced to the UI. Full struct lands in Phase 3.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon: Option<String>,
}

/// JSON-Schema descriptor for a tool, used for LLM function-calling. Phase 3.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
