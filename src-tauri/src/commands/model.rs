use std::sync::Mutex;

use tauri::{Emitter, State};

use crate::managers::model::{EngineGroup, ModelManager, ModelStatus};
use crate::managers::transcription::TranscriptionManager;

pub struct ModelState(pub Mutex<ModelManager>);
pub struct TranscriptionState(pub Mutex<TranscriptionManager>);

#[tauri::command]
pub fn list_models(
    model_mgr: State<ModelState>,
    active_model_id: Option<String>,
) -> Result<Vec<ModelStatus>, String> {
    let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
    let mut models = mgr.list_models();

    // Mark the active model
    if let Some(active_id) = active_model_id {
        for model in &mut models {
            model.active = model.info.id == active_id;
        }
    }

    Ok(models)
}

#[tauri::command]
pub fn list_models_grouped(
    model_mgr: State<ModelState>,
    active_model_id: Option<String>,
) -> Result<Vec<EngineGroup>, String> {
    let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
    let mut groups = mgr.list_models_grouped();

    // Mark the active model
    if let Some(active_id) = active_model_id {
        for group in &mut groups {
            for model in &mut group.models {
                model.active = model.info.id == active_id;
            }
        }
    }

    Ok(groups)
}

#[tauri::command]
pub fn delete_model(model_mgr: State<ModelState>, model_id: String) -> Result<(), String> {
    let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
    mgr.delete_model(&model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_supported_languages() -> Vec<crate::managers::model::Language> {
    crate::managers::model::ModelManager::supported_languages()
}

/// Download a Whisper GGML model from HuggingFace.
/// Emits progress events: model-download-progress, model-download-complete, model-download-error.
#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    model_mgr: State<'_, ModelState>,
    model_id: String,
) -> Result<(), String> {
    // Get download URL and target path from model manager
    let (download_url, model_path) = {
        let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
        let model = mgr
            .get_model(&model_id)
            .ok_or_else(|| format!("Unknown model: {}", model_id))?;
        let url = mgr.get_download_url(model);
        let path = mgr.get_model_path(&model_id);
        mgr.ensure_models_dir().map_err(|e| e.to_string())?;
        (url, path)
    };

    // Clone values for the async task
    let mid = model_id.clone();
    let app_handle = app.clone();

    // Spawn download in background task
    tauri::async_runtime::spawn(async move {
        match download_model_file(&app_handle, &mid, &download_url, &model_path).await {
            Ok(()) => {
                let _ = app_handle.emit(
                    "model-download-complete",
                    serde_json::json!({ "modelId": mid }),
                );
                tracing::info!("Model {} downloaded successfully", mid);
            }
            Err(e) => {
                let _ = app_handle.emit("model-download-error", e.to_string());
                tracing::error!("Model {} download failed: {}", mid, e);
            }
        }
    });

    Ok(())
}

/// Download a model file with progress reporting.
async fn download_model_file(
    app: &tauri::AppHandle,
    model_id: &str,
    url: &str,
    dest_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::StreamExt;

    // Create the model directory
    std::fs::create_dir_all(dest_dir)?;

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let file_name = url
        .split('/')
        .last()
        .unwrap_or("model.bin");
    let file_path = dest_dir.join(file_name);

    let mut file = tokio::fs::File::create(&file_path).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_progress_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        downloaded += chunk.len() as u64;

        // Emit progress at most every 100ms to avoid flooding
        if last_progress_emit.elapsed() > std::time::Duration::from_millis(100) {
            let progress = if total_size > 0 {
                downloaded as f64 / total_size as f64
            } else {
                0.0
            };
            let _ = app.emit(
                "model-download-progress",
                serde_json::json!({
                    "modelId": model_id,
                    "progress": progress,
                }),
            );
            last_progress_emit = std::time::Instant::now();
        }
    }

    // Final progress emit at 100%
    let _ = app.emit(
        "model-download-progress",
        serde_json::json!({
            "modelId": model_id,
            "progress": 1.0,
        }),
    );

    Ok(())
}
