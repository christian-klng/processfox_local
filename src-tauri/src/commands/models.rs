use tauri::{AppHandle, State};

use crate::core::error::{CommandError, CoreError};
use crate::core::hardware::HardwareInfo;
use crate::core::models::{CatalogEntry, InstalledModel};
use crate::state::AppState;

#[tauri::command]
pub async fn list_catalog(state: State<'_, AppState>) -> Result<Vec<CatalogEntry>, CommandError> {
    Ok(state.catalog.models.clone())
}

#[tauri::command]
pub async fn list_installed_models(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledModel>, CommandError> {
    state
        .installed_scanner()
        .scan(&state.catalog)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_hardware_info(state: State<'_, AppState>) -> Result<HardwareInfo, CommandError> {
    Ok(HardwareInfo::detect(&state.catalog))
}

#[tauri::command]
pub async fn download_from_catalog(
    catalog_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let entry = state
        .catalog
        .find(&catalog_id)
        .ok_or_else(|| CoreError::Llm(format!("Modell nicht im Katalog: {catalog_id}")))?
        .clone();

    let runner = state.download_runner(&app)?;
    runner
        .start(entry.id.clone(), entry.hf_url, entry.filename)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn download_from_url(
    download_id: String,
    url: String,
    filename: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let runner = state.download_runner(&app)?;
    runner
        .start(download_id, url, filename)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn cancel_download(
    download_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let runner = state.download_runner(&app)?;
    runner.cancel(&download_id).await;
    Ok(())
}

#[tauri::command]
pub async fn delete_model(
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .installed_scanner()
        .delete(&filename)
        .map_err(Into::into)
}
