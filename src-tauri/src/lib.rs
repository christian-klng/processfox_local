pub mod commands;
pub mod core;
pub mod state;

use tauri::Manager;
use tracing_subscriber::{fmt, EnvFilter};

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
            app.manage(AppState::new(paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::list_agents,
            commands::agent::get_agent,
            commands::agent::create_agent,
            commands::agent::update_agent,
            commands::agent::delete_agent,
            commands::file::list_agent_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
