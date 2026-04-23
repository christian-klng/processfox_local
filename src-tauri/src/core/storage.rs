use std::path::{Path, PathBuf};

use directories::BaseDirs;

use super::error::{CoreError, CoreResult};

/// Root of all ProcessFox application data on disk.
///
/// macOS:   ~/Library/Application Support/ProcessFox/
/// Windows: %APPDATA%/ProcessFox/
/// Linux:   ~/.config/ProcessFox/ (XDG config dir)
#[derive(Debug, Clone)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    pub fn discover() -> CoreResult<Self> {
        let base = BaseDirs::new().ok_or(CoreError::AppSupportUnavailable)?;
        // On macOS, `config_dir` points at Library/Application Support, matching
        // our spec. On Windows it's AppData/Roaming. On Linux it's XDG_CONFIG_HOME.
        let root = base.config_dir().join("ProcessFox");
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn agents_dir(&self) -> PathBuf {
        self.root.join("agents")
    }

    pub fn skills_user_dir(&self) -> PathBuf {
        self.root.join("skills").join("user")
    }

    pub fn models_downloads_dir(&self) -> PathBuf {
        self.root.join("models").join("downloads")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    /// Ensure every directory this app relies on exists. Idempotent.
    pub fn ensure_dirs(&self) -> CoreResult<()> {
        for dir in [
            self.agents_dir(),
            self.skills_user_dir(),
            self.models_downloads_dir(),
            self.logs_dir(),
        ] {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(())
    }
}
