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
    /// Direct download URL for the model file or archive.
    pub download_url: String,
    /// Whether the model is a directory (tar.gz archive) rather than a single file.
    pub is_directory: bool,
}

/// Supported transcription languages.
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
    /// Includes Whisper (GGML) and Parakeet V3 (ONNX) engines.
    fn build_registry() -> Vec<ModelInfo> {
        vec![
            // -- Whisper models (ggerganov/whisper.cpp GGML) --
            ModelInfo {
                id: "whisper-small".into(),
                engine: EngineType::Whisper,
                name: "small".into(),
                display_name: "Whisper Small (~466 MB) - Accurate".into(),
                size: "466MB".into(),
                languages: Self::whisper_languages(),
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
                is_directory: false,
            },
            ModelInfo {
                id: "whisper-medium".into(),
                engine: EngineType::Whisper,
                name: "medium".into(),
                display_name: "Whisper Medium (~1.5 GB) - High Accuracy".into(),
                size: "1.5GB".into(),
                languages: Self::whisper_languages(),
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
                is_directory: false,
            },
            ModelInfo {
                id: "whisper-large-v3-turbo".into(),
                engine: EngineType::Whisper,
                name: "large-v3-turbo".into(),
                display_name: "Whisper Large v3 Turbo (~1.1 GB) - Fast & Accurate".into(),
                size: "1.1GB".into(),
                languages: Self::whisper_languages(),
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".into(),
                is_directory: false,
            },
            ModelInfo {
                id: "whisper-large-v3".into(),
                engine: EngineType::Whisper,
                name: "large-v3".into(),
                display_name: "Whisper Large v3 (~1.6 GB) - Best Accuracy".into(),
                size: "1.6GB".into(),
                languages: Self::whisper_languages(),
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".into(),
                is_directory: false,
            },

            // -- Parakeet V3 (NVIDIA ONNX, int8 quantized) --
            ModelInfo {
                id: "parakeet-v3".into(),
                engine: EngineType::Parakeet,
                name: "v3".into(),
                display_name: "Parakeet V3 (~478 MB) - Fast & Multilingual".into(),
                size: "478MB".into(),
                languages: Self::parakeet_v3_languages(),
                download_url: "https://blob.handy.computer/models/parakeet-tdt_ctc-110m-onnx-int8.tar.gz".into(),
                is_directory: true,
            },
        ]
    }

    /// Whisper's multilingual language list.
    fn whisper_languages() -> Vec<String> {
        vec![
            "auto", "en", "es", "fr", "de", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ko",
            "da", "no", "sv", "fi",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    /// Parakeet V3's 25 European language list.
    fn parakeet_v3_languages() -> Vec<String> {
        vec![
            "en", "es", "fr", "de", "it", "pt", "nl", "pl", "ru", "da", "no", "sv", "fi",
            "cs", "uk", "hu", "ro", "bg", "hr", "sk", "sl", "lt", "lv", "et", "ca",
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
            Language { code: "cs".into(), name: "Czech".into() },
            Language { code: "uk".into(), name: "Ukrainian".into() },
            Language { code: "hu".into(), name: "Hungarian".into() },
            Language { code: "ro".into(), name: "Romanian".into() },
            Language { code: "bg".into(), name: "Bulgarian".into() },
            Language { code: "hr".into(), name: "Croatian".into() },
            Language { code: "sk".into(), name: "Slovak".into() },
            Language { code: "sl".into(), name: "Slovenian".into() },
            Language { code: "lt".into(), name: "Lithuanian".into() },
            Language { code: "lv".into(), name: "Latvian".into() },
            Language { code: "et".into(), name: "Estonian".into() },
            Language { code: "ca".into(), name: "Catalan".into() },
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
    /// Verifies the directory contains at least one model file (.bin, .gguf, or .onnx),
    /// not just that the directory exists (which can happen after a failed download).
    pub fn is_model_downloaded(&self, model_id: &str) -> bool {
        let model_dir = self.models_dir.join(model_id);
        if !model_dir.exists() {
            return false;
        }
        // Check for actual model files inside (recursively for directory models)
        Self::has_model_file(&model_dir)
    }

    /// Recursively check if a directory contains a model file (.bin, .gguf, .onnx).
    fn has_model_file(dir: &std::path::Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if Self::has_model_file(&path) {
                        return true;
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "bin" || ext == "gguf" || ext == "onnx" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get the local path for a model.
    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(model_id)
    }

    /// Get model info by ID.
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.registry.iter().find(|m| m.id == model_id)
    }

    /// List models grouped by engine type.
    pub fn list_models_grouped(&self) -> Vec<EngineGroup> {
        let engines = [
            EngineType::Whisper,
            EngineType::Parakeet,
        ];

        engines
            .iter()
            .map(|&engine| {
                let models: Vec<ModelStatus> = self
                    .registry
                    .iter()
                    .filter(|m| m.engine == engine)
                    .map(|model| ModelStatus {
                        info: model.clone(),
                        downloaded: self.is_model_downloaded(&model.id),
                        active: false,
                    })
                    .collect();

                EngineGroup {
                    engine,
                    display_name: Self::engine_display_name(engine).to_string(),
                    description: Self::engine_description(engine).to_string(),
                    models,
                }
            })
            .collect()
    }

    /// Get a display name for an engine type.
    pub fn engine_display_name(engine: EngineType) -> &'static str {
        match engine {
            EngineType::Whisper => "Whisper",
            EngineType::Parakeet => "Parakeet",
        }
    }

    /// Get a description for an engine type.
    pub fn engine_description(engine: EngineType) -> &'static str {
        match engine {
            EngineType::Whisper => "OpenAI - Multilingual, most versatile",
            EngineType::Parakeet => "NVIDIA - Fast CTC-based, 25 European languages",
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

/// A group of models by engine type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineGroup {
    pub engine: EngineType,
    pub display_name: String,
    pub description: String,
    pub models: Vec<ModelStatus>,
}
