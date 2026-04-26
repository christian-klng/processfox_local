use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use tauri::{AppHandle, Emitter};

use crate::core::error::{CoreError, CoreResult};

/// Single, replace-on-watch FS watcher. Whenever the active agent's folder
/// changes (or the folder switches), `watch` drops the previous debouncer
/// and arms a new one. Filesystem activity is debounced (400 ms) and emitted
/// to the frontend as `"fs-changed"` events so the FileTree can reload
/// without the user having to interact.
#[derive(Clone)]
pub struct FolderWatcher {
    inner: Arc<Mutex<Option<Debouncer<notify::RecommendedWatcher>>>>,
    app: AppHandle,
}

impl std::fmt::Debug for FolderWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FolderWatcher").finish()
    }
}

impl FolderWatcher {
    pub fn new(app: AppHandle) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            app,
        }
    }

    pub fn watch(&self, path: &Path) -> CoreResult<()> {
        let app_for_callback = self.app.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(400),
            move |result: DebounceEventResult| match result {
                Ok(_events) => {
                    let _ = app_for_callback.emit("fs-changed", ());
                }
                Err(e) => {
                    tracing::warn!(error = %e, "fs watcher error");
                }
            },
        )
        .map_err(|e| CoreError::Llm(format!("watcher init failed: {e}")))?;

        debouncer
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| CoreError::Llm(format!("watch path failed: {e}")))?;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| CoreError::Llm("watcher mutex poisoned".to_string()))?;
        *guard = Some(debouncer);
        Ok(())
    }

    pub fn unwatch(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
    }
}
