use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Result of a transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub text: String,
    pub duration_ms: u64,
    pub model_id: String,
}

/// Manages transcription using loaded models.
/// Currently a stub that will be wired to transcribe-rs when the dependency is enabled.
pub struct TranscriptionManager {
    loaded_model_id: Option<String>,
    loaded_model_path: Option<PathBuf>,
}

impl TranscriptionManager {
    pub fn new() -> Self {
        Self {
            loaded_model_id: None,
            loaded_model_path: None,
        }
    }

    /// Load a model for transcription.
    pub fn load_model(&mut self, model_id: &str, model_path: PathBuf) -> Result<()> {
        tracing::info!("Loading model: {} from {}", model_id, model_path.display());

        // TODO: Initialize transcribe-rs engine here when dependency is enabled
        // For now, just track what's loaded
        self.loaded_model_id = Some(model_id.to_string());
        self.loaded_model_path = Some(model_path);

        tracing::info!("Model loaded: {}", model_id);
        Ok(())
    }

    /// Transcribe an audio file.
    pub fn transcribe(&self, audio_path: &PathBuf, language: &str) -> Result<TranscriptionResult> {
        let model_id = self
            .loaded_model_id
            .as_ref()
            .context("No model loaded")?;

        tracing::info!(
            "Transcribing {} with model {} (lang: {})",
            audio_path.display(),
            model_id,
            language
        );

        let start = std::time::Instant::now();

        // TODO: Actually call transcribe-rs here
        // For now, return a placeholder
        let text = format!(
            "[Transcription placeholder - model: {}, lang: {}]",
            model_id, language
        );

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(TranscriptionResult {
            text,
            duration_ms,
            model_id: model_id.clone(),
        })
    }

    /// Unload the current model.
    pub fn unload(&mut self) {
        if let Some(id) = &self.loaded_model_id {
            tracing::info!("Unloading model: {}", id);
        }
        self.loaded_model_id = None;
        self.loaded_model_path = None;
    }

    /// Check if a model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded_model_id.is_some()
    }

    /// Get the ID of the currently loaded model.
    pub fn loaded_model_id(&self) -> Option<&str> {
        self.loaded_model_id.as_deref()
    }
}
