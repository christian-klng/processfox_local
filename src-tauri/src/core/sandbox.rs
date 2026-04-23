use std::path::{Path, PathBuf};

use super::error::{CoreError, CoreResult};

/// Ensure `requested` lies inside `agent_folder`. Returns the canonical,
/// absolute path on success.
///
/// Resolves symlinks via `canonicalize` to prevent escape via symlinks
/// pointing outside the agent folder.
pub fn ensure_in_agent_folder(agent_folder: &Path, requested: &Path) -> CoreResult<PathBuf> {
    let absolute = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        agent_folder.join(requested)
    };

    let canonical_requested = absolute
        .canonicalize()
        .map_err(|e| CoreError::PathInvalid(e.to_string()))?;
    let canonical_root = agent_folder
        .canonicalize()
        .map_err(|e| CoreError::PathInvalid(e.to_string()))?;

    if !canonical_requested.starts_with(&canonical_root) {
        return Err(CoreError::PathOutsideAgentFolder);
    }

    Ok(canonical_requested)
}
