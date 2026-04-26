use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::core::error::{CommandError, CoreError};
use crate::core::sandbox::ensure_in_agent_folder;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// List the contents of the given agent's folder (one level, sorted:
/// directories first, then files, both alphabetically).
///
/// If `sub_path` is provided it must resolve inside the agent folder.
#[tauri::command]
pub async fn list_agent_folder(
    agent_id: String,
    sub_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, CommandError> {
    let repo = state.agent_repo();
    let agent = repo.get(&agent_id)?;

    let root = agent.folder.ok_or_else(|| {
        CommandError::new(
            "agent_has_no_folder",
            "Für diesen Agenten ist kein Ordner konfiguriert.",
        )
    })?;

    let target = match sub_path {
        Some(p) => ensure_in_agent_folder(&root, &PathBuf::from(p))?,
        None => root
            .canonicalize()
            .map_err(|e| CoreError::PathInvalid(e.to_string()))?,
    };

    if !target.is_dir() {
        return Err(CommandError::new(
            "not_a_directory",
            "Der angefragte Pfad ist kein Verzeichnis.",
        )
        .with_details(target.display().to_string()));
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    for entry in std::fs::read_dir(&target).map_err(CoreError::from)? {
        let entry = entry.map_err(CoreError::from)?;
        let file_type = entry.file_type().map_err(CoreError::from)?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip noisy OS-level metadata files; never surface or manipulate them.
        if matches!(name.as_str(), ".DS_Store" | "Thumbs.db" | ".Spotlight-V100") {
            continue;
        }

        entries.push(FileEntry {
            name,
            path: entry.path(),
            is_dir: file_type.is_dir(),
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Start (or replace) the FS watch on the given agent's folder. Subsequent
/// changes inside that folder emit a debounced `"fs-changed"` Tauri event.
#[tauri::command]
pub async fn watch_agent_folder(
    agent_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let agent = state.agent_repo().get(&agent_id)?;
    let folder = agent.folder.ok_or_else(|| {
        CommandError::new(
            "agent_has_no_folder",
            "Für diesen Agenten ist kein Ordner konfiguriert.",
        )
    })?;
    state
        .folder_watcher(&app)
        .watch(&folder)
        .map_err(CommandError::from)?;
    Ok(())
}

/// Stop the FS watcher (e.g. when no agent is active).
#[tauri::command]
pub async fn unwatch_agent_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state.folder_watcher(&app).unwatch();
    Ok(())
}
