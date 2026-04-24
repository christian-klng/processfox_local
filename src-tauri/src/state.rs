use std::sync::{Arc, OnceLock};

use crate::core::agent::AgentRepo;
use crate::core::chat::{ChatRepo, ChatRunner};
use crate::core::llm::ProviderRegistry;
use crate::core::settings::SettingsStore;
use crate::core::storage::AppPaths;

/// Application-wide state managed by Tauri. Held via `tauri::Manager::manage`
/// and accessed from commands via `State<'_, AppState>`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub providers: ProviderRegistry,
    /// Initialized lazily on first chat run, because the `ChatRunner` needs
    /// an `AppHandle` that only exists after Tauri has finished `setup`.
    runner: Arc<OnceLock<ChatRunner>>,
}

impl AppState {
    pub fn new(paths: AppPaths, providers: ProviderRegistry) -> Self {
        Self {
            paths,
            providers,
            runner: Arc::new(OnceLock::new()),
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

    pub fn chat_runner(&self, app: &tauri::AppHandle) -> ChatRunner {
        self.runner
            .get_or_init(|| ChatRunner::new(app.clone(), self.chat_repo(), self.providers.clone()))
            .clone()
    }
}
