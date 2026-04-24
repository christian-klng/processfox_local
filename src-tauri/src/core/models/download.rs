use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::core::error::{CoreError, CoreResult};
use crate::core::storage::AppPaths;

const GGUF_MAGIC: &[u8; 4] = b"GGUF";

/// Tauri event names only allow a restricted character set. Replace every
/// other character with `_` so IDs with dots or other symbols can still be
/// used as logical identifiers.
fn event_channel(download_id: &str) -> String {
    let safe: String = download_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '/' | ':' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("model:download:{safe}")
}

/// A single active download, keyed by its assigned id (the catalog entry id
/// for catalog downloads, or a UUID for custom-URL downloads).
#[derive(Debug)]
pub struct DownloadHandle {
    pub cancel: CancellationToken,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DownloadEvent {
    Started { total_bytes: Option<u64> },
    Progress { received: u64, total: Option<u64> },
    Finished { path: PathBuf, size_bytes: u64 },
    Error { message: String },
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DownloadRunner {
    app: AppHandle,
    paths: AppPaths,
    http: reqwest::Client,
    active: Arc<Mutex<HashMap<String, DownloadHandle>>>,
}

impl DownloadRunner {
    pub fn new(app: AppHandle, paths: AppPaths) -> CoreResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent("ProcessFox/0.1")
            .build()
            .map_err(|e| CoreError::Http(e.to_string()))?;
        Ok(Self {
            app,
            paths,
            http,
            active: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start a new download. Returns immediately; progress and completion are
    /// reported via Tauri events on channel `model:download:<id>`.
    pub async fn start(&self, id: String, url: String, filename: String) -> CoreResult<()> {
        // Basic sanity on the target filename. Preventing path traversal or
        // overwrite of unrelated files.
        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            return Err(CoreError::PathInvalid(filename));
        }

        {
            let active = self.active.lock().await;
            if active.contains_key(&id) {
                return Err(CoreError::Llm(format!(
                    "Download für '{id}' läuft bereits."
                )));
            }
        }

        let target_dir = self.paths.models_downloads_dir();
        std::fs::create_dir_all(&target_dir)?;
        let final_path = target_dir.join(&filename);
        let partial_path = target_dir.join(format!("{filename}.partial"));

        if final_path.exists() {
            return Err(CoreError::Llm(format!(
                "Datei existiert bereits: {filename}"
            )));
        }

        let cancel = CancellationToken::new();
        {
            let mut active = self.active.lock().await;
            active.insert(
                id.clone(),
                DownloadHandle {
                    cancel: cancel.clone(),
                },
            );
        }

        let app = self.app.clone();
        let http = self.http.clone();
        let active = self.active.clone();
        let id_bg = id.clone();

        tokio::spawn(async move {
            let channel = event_channel(&id_bg);
            let result = run_download(&http, &url, &partial_path, &final_path, &cancel, |event| {
                let _ = app.emit(&channel, event);
            })
            .await;

            match result {
                Ok(size_bytes) => {
                    let _ = app.emit(
                        &channel,
                        DownloadEvent::Finished {
                            path: final_path,
                            size_bytes,
                        },
                    );
                }
                Err(e) => {
                    // Best-effort cleanup of the .partial file on error/cancel.
                    let _ = std::fs::remove_file(&partial_path);
                    let event = if matches!(e, CoreError::Cancelled) {
                        DownloadEvent::Cancelled
                    } else {
                        DownloadEvent::Error {
                            message: e.to_string(),
                        }
                    };
                    let _ = app.emit(&channel, event);
                }
            }

            let mut active = active.lock().await;
            active.remove(&id_bg);
        });

        Ok(())
    }

    pub async fn cancel(&self, id: &str) {
        let active = self.active.lock().await;
        if let Some(handle) = active.get(id) {
            handle.cancel.cancel();
        }
    }
}

async fn run_download<F>(
    http: &reqwest::Client,
    url: &str,
    partial_path: &PathBuf,
    final_path: &PathBuf,
    cancel: &CancellationToken,
    mut emit: F,
) -> CoreResult<u64>
where
    F: FnMut(DownloadEvent),
{
    let response = tokio::select! {
        r = http.get(url).send() => r.map_err(|e| CoreError::Http(e.to_string()))?,
        _ = cancel.cancelled() => return Err(CoreError::Cancelled),
    };

    if !response.status().is_success() {
        return Err(CoreError::Http(format!(
            "HTTP {} beim Laden von {url}",
            response.status()
        )));
    }

    let total_bytes = response.content_length();
    emit(DownloadEvent::Started { total_bytes });

    let mut file = tokio::fs::File::create(partial_path).await?;
    let mut received: u64 = 0;
    let mut magic_checked = false;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = tokio::select! {
        c = stream.next() => c,
        _ = cancel.cancelled() => None,
    } {
        if cancel.is_cancelled() {
            return Err(CoreError::Cancelled);
        }
        let chunk = chunk.map_err(|e| CoreError::Http(e.to_string()))?;

        if !magic_checked {
            if chunk.len() < 4 || &chunk[..4] != GGUF_MAGIC {
                return Err(CoreError::Llm(format!(
                    "Datei ist kein GGUF (Magic-Bytes fehlen): {url}"
                )));
            }
            magic_checked = true;
        }

        file.write_all(&chunk).await?;
        received += chunk.len() as u64;
        emit(DownloadEvent::Progress {
            received,
            total: total_bytes,
        });
    }

    if !magic_checked {
        return Err(CoreError::Llm("Leere Antwort vom Server.".to_string()));
    }

    file.flush().await?;
    drop(file);

    // Atomic rename so half-written files never look complete.
    tokio::fs::rename(partial_path, final_path).await?;
    Ok(received)
}
