use tauri::State;

use crate::core::agent::{Agent, AgentDraft, AgentUpdate};
use crate::core::error::CommandError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, CommandError> {
    let repo = state.agent_repo();
    repo.list().map_err(Into::into)
}

#[tauri::command]
pub async fn get_agent(id: String, state: State<'_, AppState>) -> Result<Agent, CommandError> {
    let repo = state.agent_repo();
    repo.get(&id).map_err(Into::into)
}

#[tauri::command]
pub async fn create_agent(
    draft: AgentDraft,
    state: State<'_, AppState>,
) -> Result<Agent, CommandError> {
    let repo = state.agent_repo();
    let agent = Agent::from_draft(draft);
    repo.save(&agent)?;
    Ok(agent)
}

#[tauri::command]
pub async fn update_agent(
    id: String,
    update: AgentUpdate,
    state: State<'_, AppState>,
) -> Result<Agent, CommandError> {
    let repo = state.agent_repo();
    let mut agent = repo.get(&id)?;
    agent.apply_update(update);
    repo.save(&agent)?;
    Ok(agent)
}

#[tauri::command]
pub async fn delete_agent(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let repo = state.agent_repo();
    repo.delete(&id).map_err(Into::into)
}
