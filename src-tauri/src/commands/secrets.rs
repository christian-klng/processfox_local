use serde::Serialize;
use tauri::State;

use crate::core::error::CommandError;
use crate::core::secrets;
use crate::state::AppState;

#[tauri::command]
pub async fn set_api_key(provider: String, value: String) -> Result<(), CommandError> {
    secrets::set_api_key(&provider, &value).map_err(Into::into)
}

#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<bool, CommandError> {
    secrets::has_api_key(&provider).map_err(Into::into)
}

#[tauri::command]
pub async fn clear_api_key(provider: String) -> Result<(), CommandError> {
    secrets::clear_api_key(&provider).map_err(Into::into)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[tauri::command]
pub async fn validate_api_key(
    provider: String,
    state: State<'_, AppState>,
) -> Result<ValidationResult, CommandError> {
    let p = state.providers.get(&provider)?;
    match p.validate().await {
        Ok(()) => Ok(ValidationResult {
            ok: true,
            error: None,
        }),
        Err(e) => Ok(ValidationResult {
            ok: false,
            error: Some(e.to_string()),
        }),
    }
}
