use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::core::error::{CoreError, CoreResult};
use crate::core::sandbox::ensure_in_agent_folder;
use crate::core::tool::{Tool, ToolContext, ToolOutput, ToolSchema};

/// Safety caps: max files scanned per call, max file size considered, max
/// hits returned.
const MAX_FILES: usize = 300;
const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;
const MAX_HITS: usize = 100;
const MAX_DEPTH: usize = 8;
/// Text-ish extensions we bother scanning. We skip binaries explicitly to
/// avoid noise and false UTF-8 errors.
const SCAN_EXTENSIONS: &[&str] = &[
    "md", "txt", "csv", "json", "yaml", "yml", "toml", "html", "htm", "xml", "rs", "ts", "tsx",
    "js", "jsx", "py", "go", "c", "cpp", "h", "hpp", "sh",
];

#[derive(Debug, Default)]
pub struct GrepInFilesTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    case_sensitive: Option<bool>,
}

#[async_trait]
impl Tool for GrepInFilesTool {
    fn name(&self) -> &'static str {
        "grep_in_files"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description:
                "Search inside text files in the agent's folder for a regular expression. \
                 Returns up to 100 hits with file path and line number. Scans .md, .txt, .csv, \
                 .json, .yaml, .toml, .html, .xml and common source files; skips binaries."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regular expression (Rust `regex` syntax)."
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional relative subfolder to limit the search."
                    },
                    "caseSensitive": {
                        "type": "boolean",
                        "description": "Whether the match is case-sensitive. Defaults to false."
                    }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, input: JsonValue, ctx: &ToolContext) -> CoreResult<ToolOutput> {
        let parsed: Input = serde_json::from_value(input).map_err(CoreError::from)?;

        let regex_src = if parsed.case_sensitive.unwrap_or(false) {
            parsed.pattern.clone()
        } else {
            format!("(?i){}", parsed.pattern)
        };
        let re =
            Regex::new(&regex_src).map_err(|e| CoreError::Llm(format!("Ungültiges Regex: {e}")))?;

        let root = match parsed.path.as_deref() {
            Some(p) if !p.trim().is_empty() => {
                ensure_in_agent_folder(&ctx.agent_folder, &PathBuf::from(p))?
            }
            _ => ctx
                .agent_folder
                .canonicalize()
                .map_err(|e| CoreError::PathInvalid(e.to_string()))?,
        };

        let mut files_scanned = 0usize;
        let mut hits: Vec<String> = Vec::new();
        walk(
            &root,
            &ctx.agent_folder,
            0,
            &mut files_scanned,
            &re,
            &mut hits,
        );

        let body = if hits.is_empty() {
            format!(
                "No matches for /{}/ in {} files.",
                parsed.pattern, files_scanned
            )
        } else {
            let mut out = format!(
                "{} matches for /{}/ in {} files:\n\n",
                hits.len(),
                parsed.pattern,
                files_scanned
            );
            for h in &hits {
                out.push_str(h);
                out.push('\n');
            }
            if hits.len() >= MAX_HITS {
                out.push_str("\n[hit cap reached — narrow the pattern or the path]");
            }
            out
        };

        Ok(ToolOutput::text(body))
    }
}

fn walk(
    dir: &Path,
    agent_folder: &Path,
    depth: usize,
    files_scanned: &mut usize,
    re: &Regex,
    hits: &mut Vec<String>,
) {
    if depth > MAX_DEPTH || hits.len() >= MAX_HITS || *files_scanned >= MAX_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if hits.len() >= MAX_HITS || *files_scanned >= MAX_FILES {
            return;
        }
        let path = entry.path();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden and common noise dirs.
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            walk(&path, agent_folder, depth + 1, files_scanned, re, hits);
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let ext_ok = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| SCAN_EXTENSIONS.contains(&e.to_lowercase().as_str()))
            .unwrap_or(false);
        if !ext_ok {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.len() > MAX_FILE_SIZE {
            continue;
        }
        *files_scanned += 1;
        let Ok(file) = std::fs::File::open(&path) else {
            continue;
        };
        let reader = BufReader::new(file);
        let rel = path
            .strip_prefix(agent_folder)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        for (i, line) in reader.lines().enumerate() {
            if hits.len() >= MAX_HITS {
                return;
            }
            let Ok(line) = line else {
                break;
            };
            if re.is_match(&line) {
                let snippet: String = line.chars().take(200).collect();
                hits.push(format!("{rel}:{}: {}", i + 1, snippet));
            }
        }
    }
}
