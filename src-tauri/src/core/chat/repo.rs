use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::CoreResult;
use crate::core::storage::AppPaths;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: String,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content: content.into(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Append-only JSONL persistence for per-agent chat history.
#[derive(Debug, Clone)]
pub struct ChatRepo {
    dir: PathBuf,
}

impl ChatRepo {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            dir: paths.agents_dir(),
        }
    }

    fn file_for(&self, agent_id: &str) -> PathBuf {
        self.dir.join(format!("{agent_id}.chat.jsonl"))
    }

    pub fn load(&self, agent_id: &str) -> CoreResult<Vec<ChatMessage>> {
        let path = self.file_for(agent_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let f = std::fs::File::open(&path)?;
        let reader = BufReader::new(f);
        let mut messages = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<ChatMessage>(&line) {
                Ok(msg) => messages.push(msg),
                Err(e) => {
                    tracing::warn!(agent_id, error = %e, "skipping malformed chat line");
                }
            }
        }
        Ok(messages)
    }

    pub fn append(&self, agent_id: &str, message: &ChatMessage) -> CoreResult<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.file_for(agent_id);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let line = serde_json::to_string(message)?;
        writeln!(f, "{line}")?;
        Ok(())
    }
}
