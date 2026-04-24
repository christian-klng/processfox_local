use std::sync::{Arc, OnceLock};

use crate::core::agent::AgentRepo;
use crate::core::chat::{ChatRepo, ChatRunner};
use crate::core::llm::ProviderRegistry;
use crate::core::models::{DownloadRunner, InstalledScanner, ModelCatalog};
use crate::core::settings::SettingsStore;
use crate::core::storage::AppPaths;

/// Application-wide state managed by Tauri. Held via `tauri::Manager::manage`
/// and accessed from commands via `State<'_, AppState>`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub providers: ProviderRegistry,
    pub catalog: ModelCatalog,
    /// Initialized lazily on first chat run, because the `ChatRunner` needs
    /// an `AppHandle` that only exists after Tauri has finished `setup`.
    chat_runner: Arc<OnceLock<ChatRunner>>,
    download_runner: Arc<OnceLock<DownloadRunner>>,
}

impl AppState {
    pub fn new(paths: AppPaths, providers: ProviderRegistry, catalog: ModelCatalog) -> Self {
        Self {
            paths,
            providers,
            catalog,
            chat_runner: Arc::new(OnceLock::new()),
            download_runner: Arc::new(OnceLock::new()),
        }
    }

    pub fn agent_repo(&self) -> AgentRepo {
        AgentRepo::new(&self.paths)
    }

    pub fn chat_repo(&self) -> ChatRepo {
        ChatRepo::new(&self.paths)
    }

    pub fn settings(&self) -> SettingsStore {
        SettingsStore::new(&self.paths)
    }

    pub fn installed_scanner(&self) -> InstalledScanner {
        InstalledScanner::new(&self.paths)
    }

    pub fn chat_runner(&self, app: &tauri::AppHandle) -> ChatRunner {
        self.chat_runner
            .get_or_init(|| ChatRunner::new(app.clone(), self.chat_repo(), self.providers.clone()))
            .clone()
    }

    pub fn download_runner(
        &self,
        app: &tauri::AppHandle,
    ) -> crate::core::error::CoreResult<DownloadRunner> {
        if let Some(runner) = self.download_runner.get() {
            return Ok(runner.clone());
        }
        let runner = DownloadRunner::new(app.clone(), self.paths.clone())?;
        let _ = self.download_runner.set(runner.clone());
        Ok(runner)
    }
}
