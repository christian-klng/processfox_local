use serde::Serialize;
use thiserror::Error;

/// Serializable error shape returned to the frontend from every Tauri command.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("IO-Fehler: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialisierungs-Fehler: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Pfad liegt außerhalb des Agenten-Ordners")]
    PathOutsideAgentFolder,

    #[error("Pfad ungültig: {0}")]
    PathInvalid(String),

    #[error("Agent nicht gefunden: {0}")]
    AgentNotFound(String),

    #[error("App-Support-Ordner konnte nicht ermittelt werden")]
    AppSupportUnavailable,
}

impl From<CoreError> for CommandError {
    fn from(err: CoreError) -> Self {
        let code = match &err {
            CoreError::Io(_) => "io_error",
            CoreError::Json(_) => "serialization_error",
            CoreError::PathOutsideAgentFolder => "path_outside_agent_folder",
            CoreError::PathInvalid(_) => "path_invalid",
            CoreError::AgentNotFound(_) => "agent_not_found",
            CoreError::AppSupportUnavailable => "app_support_unavailable",
        };
        CommandError::new(code, err.to_string())
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
