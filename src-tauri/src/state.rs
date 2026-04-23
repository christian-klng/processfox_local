use crate::core::agent::AgentRepo;
use crate::core::storage::AppPaths;

/// Application-wide state managed by Tauri. Held as `Arc<Self>` via
/// `tauri::Manager::manage` and accessed from commands via `State<'_, AppState>`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: AppPaths,
}

impl AppState {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub fn agent_repo(&self) -> AgentRepo {
        AgentRepo::new(&self.paths)
    }
}
