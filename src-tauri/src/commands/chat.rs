use tauri::{AppHandle, State};

use crate::core::chat::repo::ChatMessage;
use crate::core::chat::run::RunStarted;
use crate::core::error::{CommandError, CoreError};
use crate::state::AppState;

#[tauri::command]
pub async fn list_messages(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChatMessage>, CommandError> {
    state.chat_repo().load(&agent_id).map_err(Into::into)
}

#[tauri::command]
pub async fn send_message(
    agent_id: String,
    provider: String,
    model_id: String,
    text: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RunStarted, CommandError> {
    let agent = state.agent_repo().get(&agent_id)?;

    // Local GGUF inference has no API key concept; every other provider does.
    if provider != "local" && !crate::core::secrets::has_api_key(&provider)? {
        return Err(CoreError::MissingApiKey(provider).into());
    }

    let runner = state.chat_runner(&app);
    runner
        .start(agent, provider, model_id, text)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn cancel_run(
    run_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let runner = state.chat_runner(&app);
    runner.cancel(&run_id).await;
    Ok(())
}

#[tauri::command]
pub async fn approve_hitl(
    hitl_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let runner = state.chat_runner(&app);
    runner
        .resolve_hitl(&hitl_id, crate::core::tool::HitlDecision::Approve)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn reject_hitl(
    hitl_id: String,
    reason: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let runner = state.chat_runner(&app);
    runner
        .resolve_hitl(&hitl_id, crate::core::tool::HitlDecision::Reject { reason })
        .await;
    Ok(())
}
