use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::core::error::{CoreError, CoreResult};
use crate::core::tool::{Tool, ToolContext, ToolOutput, ToolSchema};

#[derive(Debug, Default)]
pub struct AskUserTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Schema-only — react_loop intercepts ask_user by name and never calls execute.
struct Input {
    question: String,
}

/// `ask_user` is special: the chat-runner intercepts calls by name in
/// `react_loop` and resolves them via an interactive Tauri event. The tool's
/// `execute` body should never run; if it does (e.g. someone calls it
/// outside the loop), we return a clean error.
#[async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &'static str {
        "ask_user"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description:
                "Ask the user a clarifying question and wait for their typed answer. Use this \
                 when the user's request is ambiguous, when you need to choose between \
                 reasonable options, or when you'd otherwise have to guess (e.g. which file, \
                 which date, which numeric value). Be specific — vague questions like \
                 'What do you want?' annoy the user; questions like 'Soll ich Marketing auf \
                 12000 oder 15000 setzen?' move the conversation forward. The answer is \
                 returned to you as the tool result."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to show the user. Plain text; line breaks are preserved."
                    }
                },
                "required": ["question"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, input: JsonValue, _ctx: &ToolContext) -> CoreResult<ToolOutput> {
        // Should be unreachable — react_loop intercepts ask_user by name and
        // resolves via the question-pipeline. If we ever land here, surface
        // a clean error so the run doesn't get stuck.
        let _: Input = serde_json::from_value(input).map_err(CoreError::from)?;
        Err(CoreError::Llm(
            "ask_user must be resolved by the chat runner; tool.execute should not run."
                .to_string(),
        ))
    }
}
