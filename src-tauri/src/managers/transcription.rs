use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use transcribe_rs::{
    engines::{
        parakeet::{
            ParakeetEngine, ParakeetInferenceParams, ParakeetModelParams, TimestampGranularity,
        },
        whisper::{WhisperEngine, WhisperInferenceParams},
    },
    TranscriptionEngine,
};

use crate::managers::model::EngineType;

/// Result of a transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub text: String,
    pub duration_ms: u64,
    pub model_id: String,
}

/// Currently loaded engine variant.
enum LoadedEngine {
    Whisper(WhisperEngine),
    Parakeet(ParakeetEngine),
}

/// Manages transcription using transcribe-rs (Whisper + Parakeet).
pub struct TranscriptionManager {
    loaded_model_id: Option<String>,
    loaded_engine_type: Option<EngineType>,
    engine: Option<LoadedEngine>,
}

impl TranscriptionManager {
    pub fn new() -> Self {
        Self {
            loaded_model_id: None,
            loaded_engine_type: None,
            engine: None,
        }
    }

    /// Load a model for transcription.
    /// `model_path` is the model directory.
    /// For Whisper: scans for .bin file and loads via WhisperEngine.
    /// For Parakeet: loads directory with ONNX files via ParakeetEngine.
    pub fn load_model(
        &mut self,
        model_id: &str,
        model_path: PathBuf,
        engine_type: EngineType,
    ) -> Result<()> {
        tracing::info!(
            "Loading model: {} (engine: {:?}) from {}",
            model_id,
            engine_type,
            model_path.display()
        );

        // Unload any existing model first
        self.unload();

        let loaded = match engine_type {
            EngineType::Whisper => {
                let model_file = Self::find_model_file(&model_path, &["bin", "gguf"])
                    .with_context(|| {
                        format!("No .bin/.gguf file found in {}", model_path.display())
                    })?;
                tracing::info!("Found whisper model file: {}", model_file.display());

                let mut engine = WhisperEngine::new();
                engine
                    .load_model(&model_file)
                    .map_err(|e| anyhow::anyhow!("Failed to load Whisper model: {}", e))?;
                LoadedEngine::Whisper(engine)
            }
            EngineType::Parakeet => {
                // Parakeet expects the directory containing ONNX files.
                // If extracted from tar.gz, there may be a nested subdirectory.
                let onnx_dir = Self::find_onnx_directory(&model_path)
                    .with_context(|| {
                        format!("No ONNX model directory found in {}", model_path.display())
                    })?;
                tracing::info!("Found parakeet model dir: {}", onnx_dir.display());

                let mut engine = ParakeetEngine::new();
                engine
                    .load_model_with_params(&onnx_dir, ParakeetModelParams::int8())
                    .map_err(|e| anyhow::anyhow!("Failed to load Parakeet model: {}", e))?;
                LoadedEngine::Parakeet(engine)
            }
        };

        self.loaded_model_id = Some(model_id.to_string());
        self.loaded_engine_type = Some(engine_type);
        self.engine = Some(loaded);

        tracing::info!("Model loaded: {} ({:?})", model_id, engine_type);
        Ok(())
    }

    /// Find the first file with one of the given extensions in a directory.
    fn find_model_file(dir: &PathBuf, extensions: &[&str]) -> Result<PathBuf> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Cannot read model directory: {}", dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!(
            "No file with extensions {:?} found in {}",
            extensions,
            dir.display()
        )
    }

    /// Find the directory containing ONNX model files.
    /// Handles both flat layout (model_dir/*.onnx) and nested (model_dir/subdir/*.onnx).
    fn find_onnx_directory(model_path: &PathBuf) -> Result<PathBuf> {
        // First check if the model_path itself contains .onnx files
        if Self::dir_contains_onnx(model_path) {
            return Ok(model_path.clone());
        }

        // Check one level of subdirectories
        if let Ok(entries) = std::fs::read_dir(model_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && Self::dir_contains_onnx(&path) {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!("No ONNX model files found in {}", model_path.display())
    }

    /// Check if a directory directly contains .onnx files.
    fn dir_contains_onnx(dir: &std::path::Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext == "onnx" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Transcribe an audio file using the loaded model.
    pub fn transcribe(
        &mut self,
        audio_path: &PathBuf,
        language: &str,
    ) -> Result<TranscriptionResult> {
        let model_id = self
            .loaded_model_id
            .as_ref()
            .context("No model loaded")?
            .clone();

        let engine = self.engine.as_mut().context("No engine loaded")?;

        tracing::info!(
            "Transcribing {} with model {} (lang: {})",
            audio_path.display(),
            model_id,
            language
        );

        let start = std::time::Instant::now();

        // Read WAV file and convert to f32 samples
        let samples = Self::read_wav_as_f32(audio_path)?;

        tracing::info!(
            "Audio loaded: {} samples ({:.1}s at 16kHz)",
            samples.len(),
            samples.len() as f64 / 16000.0
        );

        // Dispatch transcription to the appropriate engine
        let text = match engine {
            LoadedEngine::Whisper(whisper) => {
                let whisper_language = if language == "auto" {
                    None
                } else {
                    Some(language.to_string())
                };

                let params = WhisperInferenceParams {
                    language: whisper_language,
                    ..Default::default()
                };

                let result = whisper
                    .transcribe_samples(samples, Some(params))
                    .map_err(|e| anyhow::anyhow!("Whisper transcription failed: {}", e))?;

                result.text
            }
            LoadedEngine::Parakeet(parakeet) => {
                let params = ParakeetInferenceParams {
                    timestamp_granularity: TimestampGranularity::Segment,
                    ..Default::default()
                };

                let result = parakeet
                    .transcribe_samples(samples, Some(params))
                    .map_err(|e| anyhow::anyhow!("Parakeet transcription failed: {}", e))?;

                result.text
            }
        };

        let text = text.trim().to_string();
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            "Transcription complete in {}ms: \"{}\"",
            duration_ms,
            text
        );

        Ok(TranscriptionResult {
            text,
            duration_ms,
            model_id,
        })
    }

    /// Read a 16kHz mono 16-bit PCM WAV file and convert to f32 samples.
    fn read_wav_as_f32(path: &PathBuf) -> Result<Vec<f32>> {
        let reader = hound::WavReader::open(path)
            .with_context(|| format!("Failed to open WAV file: {}", path.display()))?;

        let spec = reader.spec();
        tracing::debug!(
            "WAV spec: {}ch, {}Hz, {:?} {}bit",
            spec.channels,
            spec.sample_rate,
            spec.sample_format,
            spec.bits_per_sample
        );

        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect();

        Ok(samples)
    }

    /// Unload the current model, freeing memory.
    pub fn unload(&mut self) {
        if let Some(id) = &self.loaded_model_id {
            tracing::info!("Unloading model: {}", id);
        }
        // Unload engine-specific resources
        if let Some(engine) = self.engine.as_mut() {
            match engine {
                LoadedEngine::Whisper(e) => e.unload_model(),
                LoadedEngine::Parakeet(e) => e.unload_model(),
            }
        }
        self.engine = None;
        self.loaded_model_id = None;
        self.loaded_engine_type = None;
    }

    /// Check if a model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.engine.is_some()
    }

    /// Get the ID of the currently loaded model.
    pub fn loaded_model_id(&self) -> Option<&str> {
        self.loaded_model_id.as_deref()
    }
}
