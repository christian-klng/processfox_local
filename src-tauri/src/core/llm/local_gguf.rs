//! Local GGUF inference provider backed by mistral.rs.
//!
//! One loaded model at a time; switching models triggers an unload + reload.
//! Metal acceleration is intentionally disabled (see Cargo.toml) so the crate
//! builds without the full Xcode install.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use mistralrs::{
    ChatCompletionChunkResponse, ChunkChoice, Delta, DeviceMapSetting, GgufModelBuilder, Model,
    RequestBuilder, Response, TextMessageRole,
};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

use super::{ChatRole, FinishReason, GenerateRequest, LlmEvent, LlmProvider};
use crate::core::error::{CoreError, CoreResult};

pub const PROVIDER_ID: &str = "local";

struct Loaded {
    filename: String,
    model: Arc<Model>,
}

impl std::fmt::Debug for Loaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Loaded")
            .field("filename", &self.filename)
            .finish()
    }
}

#[derive(Debug)]
pub struct LocalGgufProvider {
    models_dir: PathBuf,
    loaded: Arc<Mutex<Option<Loaded>>>,
}

impl LocalGgufProvider {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            loaded: Arc::new(Mutex::new(None)),
        }
    }

    /// Drop the currently loaded model, freeing its memory.
    pub async fn unload(&self) {
        *self.loaded.lock().await = None;
    }

    /// Return a shared handle to the requested model, loading it if it's not
    /// already resident. Switching to a different `filename` triggers an
    /// unload of the previous model first.
    async fn ensure_loaded(&self, filename: &str) -> CoreResult<Arc<Model>> {
        {
            let guard = self.loaded.lock().await;
            if let Some(current) = guard.as_ref() {
                if current.filename == filename {
                    return Ok(Arc::clone(&current.model));
                }
            }
        }

        // Drop any previous model before building a new one. A fresh Model
        // takes multi-GB of RAM; holding two simultaneously would OOM small
        // machines.
        {
            let mut guard = self.loaded.lock().await;
            *guard = None;
        }

        let dir = self.models_dir.to_string_lossy().into_owned();
        tracing::info!(filename, dir = %dir, "loading local GGUF model");

        // Skip mistralrs's auto-device-mapper. It has a bug in 0.8.1 where
        // CPU memory is mis-detected as 0 MB (calls `refresh_cpu_all` instead
        // of `refresh_memory` in memory_usage.rs), making it refuse to load
        // any model larger than zero bytes. The `dummy` mapping puts every
        // layer on the primary device and skips the memory arithmetic.
        let model = GgufModelBuilder::new(dir, vec![filename.to_string()])
            .with_logging()
            .with_device_mapping(DeviceMapSetting::dummy())
            .build()
            .await
            .map_err(|e| CoreError::Llm(format!("Modell-Load fehlgeschlagen: {e}")))?;
        let model = Arc::new(model);

        let mut guard = self.loaded.lock().await;
        *guard = Some(Loaded {
            filename: filename.to_string(),
            model: Arc::clone(&model),
        });
        Ok(model)
    }
}

fn map_role(role: ChatRole) -> TextMessageRole {
    match role {
        ChatRole::User => TextMessageRole::User,
        ChatRole::Assistant => TextMessageRole::Assistant,
        ChatRole::System => TextMessageRole::System,
    }
}

#[async_trait]
impl LlmProvider for LocalGgufProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    async fn generate(
        &self,
        request: GenerateRequest,
        sink: mpsc::Sender<LlmEvent>,
        cancel: CancellationToken,
    ) -> CoreResult<()> {
        if request.model_id.trim().is_empty() {
            return Err(CoreError::Llm(
                "Kein lokales Modell ausgewählt.".to_string(),
            ));
        }

        let model = tokio::select! {
            r = self.ensure_loaded(&request.model_id) => r?,
            _ = cancel.cancelled() => {
                let _ = sink
                    .send(LlmEvent::Finish { reason: FinishReason::Cancelled })
                    .await;
                return Err(CoreError::Cancelled);
            }
        };

        let mut builder = RequestBuilder::new().set_sampler_max_len(request.max_tokens as usize);
        if let Some(sys) = request.system_prompt.as_deref() {
            if !sys.trim().is_empty() {
                builder = builder.add_message(TextMessageRole::System, sys);
            }
        }
        for turn in &request.turns {
            builder = builder.add_message(map_role(turn.role), &turn.content);
        }

        let mut stream = tokio::select! {
            r = model.stream_chat_request(builder) => r.map_err(|e| CoreError::Llm(format!("Stream-Start: {e}")))?,
            _ = cancel.cancelled() => {
                let _ = sink
                    .send(LlmEvent::Finish { reason: FinishReason::Cancelled })
                    .await;
                return Err(CoreError::Cancelled);
            }
        };

        while let Some(response) = tokio::select! {
            r = stream.next() => r,
            _ = cancel.cancelled() => None,
        } {
            if cancel.is_cancelled() {
                let _ = sink
                    .send(LlmEvent::Finish {
                        reason: FinishReason::Cancelled,
                    })
                    .await;
                return Err(CoreError::Cancelled);
            }

            if let Response::Chunk(ChatCompletionChunkResponse { choices, .. }) = response {
                for choice in choices {
                    let ChunkChoice {
                        delta: Delta { content, .. },
                        finish_reason,
                        ..
                    } = choice;
                    if let Some(text) = content {
                        if !text.is_empty()
                            && sink.send(LlmEvent::TextDelta { text }).await.is_err()
                        {
                            return Ok(());
                        }
                    }
                    if let Some(reason) = finish_reason {
                        let mapped = match reason.as_str() {
                            "length" => FinishReason::MaxTokens,
                            _ => FinishReason::Stop,
                        };
                        let _ = sink.send(LlmEvent::Finish { reason: mapped }).await;
                    }
                }
            }
        }

        let _ = sink
            .send(LlmEvent::Finish {
                reason: FinishReason::Stop,
            })
            .await;
        Ok(())
    }

    async fn validate(&self) -> CoreResult<()> {
        if !self.models_dir.exists() {
            return Err(CoreError::Llm(format!(
                "Modell-Ordner existiert nicht: {}",
                self.models_dir.display()
            )));
        }
        Ok(())
    }
}
