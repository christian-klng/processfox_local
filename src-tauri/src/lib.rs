pub mod commands;
pub mod core;
pub mod state;

use std::sync::Arc;

use tauri::Manager;
use tracing_subscriber::{fmt, EnvFilter};

use crate::core::llm::{
    anthropic::AnthropicProvider, openai::OpenAiProvider, openrouter::OpenRouterProvider,
    ProviderRegistry,
};
use crate::core::models::ModelCatalog;
use crate::core::storage::AppPaths;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let paths = AppPaths::discover()?;
            paths.ensure_dirs()?;
            tracing::info!(root = %paths.root().display(), "ProcessFox app data initialized");

            let mut registry = ProviderRegistry::new();
            registry.register(Arc::new(AnthropicProvider::new()?));
            registry.register(Arc::new(OpenAiProvider::new()?));
            registry.register(Arc::new(OpenRouterProvider::new()?));

            let catalog = ModelCatalog::embedded()?;
            tracing::info!(models = catalog.models.len(), "model catalog loaded");

            app.manage(AppState::new(paths, registry, catalog));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::list_agents,
            commands::agent::get_agent,
            commands::agent::create_agent,
            commands::agent::update_agent,
            commands::agent::delete_agent,
            commands::file::list_agent_folder,
            commands::settings::get_settings,
            commands::settings::set_default_provider,
            commands::settings::set_default_model,
            commands::settings::set_first_run_done,
            commands::settings::available_providers,
            commands::secrets::set_api_key,
            commands::secrets::has_api_key,
            commands::secrets::clear_api_key,
            commands::secrets::validate_api_key,
            commands::chat::list_messages,
            commands::chat::send_message,
            commands::chat::cancel_run,
            commands::models::list_catalog,
            commands::models::list_installed_models,
            commands::models::get_hardware_info,
            commands::models::download_from_catalog,
            commands::models::download_from_url,
            commands::models::cancel_download,
            commands::models::delete_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
