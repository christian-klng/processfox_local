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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "processfox_sandbox_{prefix}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn relative_path_inside_resolves() {
        let root = tmp_dir("relative");
        let inner = root.join("notes.md");
        fs::write(&inner, "hi").unwrap();

        let resolved = ensure_in_agent_folder(&root, Path::new("notes.md")).unwrap();
        assert_eq!(resolved, inner.canonicalize().unwrap());
    }

    #[test]
    fn absolute_path_outside_is_rejected() {
        let root = tmp_dir("absolute");
        let outside = std::env::temp_dir().join("processfox_sandbox_outside_target");
        fs::write(&outside, "nope").unwrap();

        let err = ensure_in_agent_folder(&root, &outside).unwrap_err();
        assert!(matches!(err, CoreError::PathOutsideAgentFolder));
    }

    #[test]
    fn parent_traversal_is_rejected() {
        let root = tmp_dir("traversal");
        let sibling = root
            .parent()
            .unwrap()
            .join(format!("processfox_sandbox_sibling_{}", std::process::id()));
        fs::create_dir_all(&sibling).unwrap();
        let leak = sibling.join("leak.txt");
        fs::write(&leak, "leak").unwrap();

        let relative = Path::new("../")
            .join(sibling.file_name().unwrap())
            .join("leak.txt");
        let err = ensure_in_agent_folder(&root, &relative).unwrap_err();
        assert!(matches!(err, CoreError::PathOutsideAgentFolder));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = tmp_dir("symlink");
        let outside = std::env::temp_dir().join(format!(
            "processfox_sandbox_sym_target_{}",
            std::process::id()
        ));
        fs::create_dir_all(&outside).unwrap();
        let target = outside.join("secret.txt");
        fs::write(&target, "secret").unwrap();

        let link = root.join("link-out");
        symlink(&outside, &link).unwrap();

        // Accessing through the symlink should be refused — canonicalize
        // resolves it to the outside target.
        let err = ensure_in_agent_folder(&root, Path::new("link-out/secret.txt")).unwrap_err();
        assert!(matches!(err, CoreError::PathOutsideAgentFolder));
    }

    #[test]
    fn nonexistent_path_fails_cleanly() {
        let root = tmp_dir("missing");
        let err = ensure_in_agent_folder(&root, Path::new("does-not-exist")).unwrap_err();
        assert!(matches!(err, CoreError::PathInvalid(_)));
    }
}
