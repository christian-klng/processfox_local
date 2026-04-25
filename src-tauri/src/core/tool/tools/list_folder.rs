use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::path::PathBuf;

use crate::core::error::{CoreError, CoreResult};
use crate::core::sandbox::ensure_in_agent_folder;
use crate::core::tool::{Tool, ToolContext, ToolOutput, ToolSchema};

/// Limit on how many entries to return in one call. Keeps the LLM context
/// bounded even in huge folders.
const MAX_ENTRIES: usize = 200;

#[derive(Debug, Default)]
pub struct ListFolderTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    #[serde(default)]
    path: Option<String>,
}

#[async_trait]
impl Tool for ListFolderTool {
    fn name(&self) -> &'static str {
        "list_folder"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description:
                "List the files and subfolders inside the agent's working folder. Use an empty \
                 or omitted 'path' to list the folder root; pass a relative subfolder to list \
                 that subfolder. Paths are always resolved inside the agent's folder."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Optional relative subfolder (e.g. 'pdfs' or 'reports/2026')."
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, input: JsonValue, ctx: &ToolContext) -> CoreResult<ToolOutput> {
        let parsed: Input = serde_json::from_value(input).map_err(CoreError::from)?;
        let target = match parsed.path.as_deref() {
            Some(p) if !p.trim().is_empty() => {
                ensure_in_agent_folder(&ctx.agent_folder, &PathBuf::from(p))?
            }
            _ => ctx
                .agent_folder
                .canonicalize()
                .map_err(|e| CoreError::PathInvalid(e.to_string()))?,
        };

        if !target.is_dir() {
            return Err(CoreError::PathInvalid(format!(
                "'{}' is not a directory",
                target.display()
            )));
        }

        let mut entries = Vec::new();
        let mut total = 0usize;
        for entry in std::fs::read_dir(&target)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if matches!(name.as_str(), ".DS_Store" | "Thumbs.db" | ".Spotlight-V100") {
                continue;
            }
            total += 1;
            if entries.len() >= MAX_ENTRIES {
                continue;
            }
            let file_type = entry.file_type()?;
            let size = if file_type.is_file() {
                entry.metadata()?.len()
            } else {
                0
            };
            entries.push((name, file_type.is_dir(), size));
        }

        entries.sort_by(|a, b| match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        });

        let body = if entries.is_empty() {
            "(empty folder)".to_string()
        } else {
            let mut out = String::new();
            for (name, is_dir, size) in &entries {
                if *is_dir {
                    out.push_str(&format!("📁 {name}/\n"));
                } else {
                    out.push_str(&format!("📄 {name}  ({})\n", human_bytes(*size)));
                }
            }
            if total > entries.len() {
                out.push_str(&format!(
                    "… {} more entries omitted (cap {MAX_ENTRIES}).\n",
                    total - entries.len()
                ));
            }
            out
        };

        Ok(ToolOutput::text(body))
    }
}

fn human_bytes(b: u64) -> String {
    const K: u64 = 1024;
    if b < K {
        format!("{b} B")
    } else if b < K * K {
        format!("{:.1} KB", b as f64 / K as f64)
    } else if b < K * K * K {
        format!("{:.1} MB", b as f64 / (K * K) as f64)
    } else {
        format!("{:.2} GB", b as f64 / (K * K * K) as f64)
    }
}
