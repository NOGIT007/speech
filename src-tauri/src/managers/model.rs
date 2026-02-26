use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Engine types supported by the transcription system.
/// Matches the TypeScript EngineType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    Whisper,
    Parakeet,
    Moonshine,
    SenseVoice,
}

/// A model available for transcription.
/// Matches the TypeScript Model interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub engine: EngineType,
    pub name: String,
    pub display_name: String,
    pub size: String,
    pub languages: Vec<String>,
    #[serde(skip)]
    pub repo_id: String,
}

/// Supported transcription languages.
/// Matches AppState.swift:481-523.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
}

/// Manages the model registry, downloads, and storage.
pub struct ModelManager {
    models_dir: PathBuf,
    registry: Vec<ModelInfo>,
}

impl ModelManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let models_dir = app_data_dir.join("models");
        Self {
            models_dir,
            registry: Self::build_registry(),
        }
    }

    /// Build the initial model registry.
    /// Matches AppState.swift:234-276 (WhisperModel enum) plus new engines.
    fn build_registry() -> Vec<ModelInfo> {
        vec![
            // Whisper models (matching existing Swift WhisperModel enum)
            ModelInfo {
                id: "whisper-tiny".into(),
                engine: EngineType::Whisper,
                name: "tiny".into(),
                display_name: "Whisper Tiny (~75 MB) - Fastest".into(),
                size: "75MB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
            ModelInfo {
                id: "whisper-base".into(),
                engine: EngineType::Whisper,
                name: "base".into(),
                display_name: "Whisper Base (~142 MB) - Balanced".into(),
                size: "142MB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
            ModelInfo {
                id: "whisper-small".into(),
                engine: EngineType::Whisper,
                name: "small".into(),
                display_name: "Whisper Small (~466 MB) - Accurate".into(),
                size: "466MB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
            ModelInfo {
                id: "whisper-medium".into(),
                engine: EngineType::Whisper,
                name: "medium".into(),
                display_name: "Whisper Medium (~1.5 GB) - High Accuracy".into(),
                size: "1.5GB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
            ModelInfo {
                id: "whisper-large-v3-turbo".into(),
                engine: EngineType::Whisper,
                name: "large-v3-turbo".into(),
                display_name: "Whisper Large v3 Turbo (~1.1 GB) - Fast & Accurate".into(),
                size: "1.1GB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
            ModelInfo {
                id: "whisper-large-v3".into(),
                engine: EngineType::Whisper,
                name: "large-v3".into(),
                display_name: "Whisper Large v3 (~1.6 GB) - Best Accuracy".into(),
                size: "1.6GB".into(),
                languages: Self::whisper_languages(),
                repo_id: "ggerganov/whisper.cpp".into(),
            },
        ]
    }

    /// Whisper's multilingual language list.
    /// Matches AppState.swift:481-523.
    fn whisper_languages() -> Vec<String> {
        vec![
            "auto", "en", "es", "fr", "de", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko",
            "da", "no", "sv", "fi",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    /// Get all supported languages with display names.
    pub fn supported_languages() -> Vec<Language> {
        vec![
            Language { code: "auto".into(), name: "Auto-detect".into() },
            Language { code: "en".into(), name: "English".into() },
            Language { code: "es".into(), name: "Spanish".into() },
            Language { code: "fr".into(), name: "French".into() },
            Language { code: "de".into(), name: "German".into() },
            Language { code: "it".into(), name: "Italian".into() },
            Language { code: "pt".into(), name: "Portuguese".into() },
            Language { code: "nl".into(), name: "Dutch".into() },
            Language { code: "pl".into(), name: "Polish".into() },
            Language { code: "ru".into(), name: "Russian".into() },
            Language { code: "ja".into(), name: "Japanese".into() },
            Language { code: "zh".into(), name: "Chinese".into() },
            Language { code: "ko".into(), name: "Korean".into() },
            Language { code: "da".into(), name: "Danish".into() },
            Language { code: "no".into(), name: "Norwegian".into() },
            Language { code: "sv".into(), name: "Swedish".into() },
            Language { code: "fi".into(), name: "Finnish".into() },
        ]
    }

    /// List all models with their download status.
    pub fn list_models(&self) -> Vec<ModelStatus> {
        self.registry
            .iter()
            .map(|model| {
                let downloaded = self.is_model_downloaded(&model.id);
                ModelStatus {
                    info: model.clone(),
                    downloaded,
                    active: false, // Will be set by the caller
                }
            })
            .collect()
    }

    /// Check if a model is downloaded locally.
    pub fn is_model_downloaded(&self, model_id: &str) -> bool {
        let model_dir = self.models_dir.join(model_id);
        model_dir.exists()
    }

    /// Get the local path for a model.
    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(model_id)
    }

    /// Get model info by ID.
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.registry.iter().find(|m| m.id == model_id)
    }

    /// Get the download URL for a model's GGML file from HuggingFace.
    pub fn get_download_url(&self, model: &ModelInfo) -> String {
        // HuggingFace direct download URLs for whisper.cpp GGML models
        match model.name.as_str() {
            "tiny" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
            "base" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
            "small" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
            "medium" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
            "large-v3-turbo" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".into(),
            "large-v3" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".into(),
            _ => format!(
                "https://huggingface.co/{}/resolve/main/ggml-{}.bin",
                model.repo_id, model.name
            ),
        }
    }

    /// Delete a downloaded model.
    pub fn delete_model(&self, model_id: &str) -> Result<()> {
        let model_dir = self.models_dir.join(model_id);
        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)
                .context(format!("Failed to delete model {}", model_id))?;
        }
        Ok(())
    }

    /// Ensure the models directory exists.
    pub fn ensure_models_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.models_dir)
            .context("Failed to create models directory")?;
        Ok(())
    }
}

/// Model with its download/active status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    #[serde(flatten)]
    pub info: ModelInfo,
    pub downloaded: bool,
    pub active: bool,
}
