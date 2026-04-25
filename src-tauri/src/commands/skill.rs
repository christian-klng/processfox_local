use tauri::State;

use crate::core::error::CommandError;
use crate::core::skill::Skill;
use crate::state::AppState;

#[tauri::command]
pub async fn list_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, CommandError> {
    Ok(state.skills.all().to_vec())
}
