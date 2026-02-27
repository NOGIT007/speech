use std::collections::HashSet;
use std::sync::Mutex;

use tauri::{Emitter, State};

use crate::managers::model::{EngineGroup, ModelManager, ModelStatus};
use crate::managers::transcription::TranscriptionManager;

pub struct ModelState(pub Mutex<ModelManager>);
pub struct TranscriptionState(pub Mutex<TranscriptionManager>);

/// Tracks which models are currently being downloaded to prevent concurrent downloads.
pub struct DownloadTrackerState(pub Mutex<HashSet<String>>);

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

/// Download a model file from its configured URL.
/// Prevents concurrent downloads of the same model.
/// For directory models (is_directory=true), extracts tar.gz after download.
/// Emits progress events: model-download-progress, model-download-complete, model-download-error.
#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    model_mgr: State<'_, ModelState>,
    tracker: State<'_, DownloadTrackerState>,
    model_id: String,
) -> Result<(), String> {
    // Check if already downloading this model
    {
        let mut downloading = tracker.0.lock().map_err(|e| e.to_string())?;
        if downloading.contains(&model_id) {
            return Err(format!("Model {} is already being downloaded", model_id));
        }
        downloading.insert(model_id.clone());
    }

    // Get download URL, target path, and is_directory flag from model manager
    let (download_url, model_path, is_directory) = {
        let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
        let model = mgr
            .get_model(&model_id)
            .ok_or_else(|| format!("Unknown model: {}", model_id))?;
        let url = model.download_url.clone();
        let path = mgr.get_model_path(&model_id);
        let is_dir = model.is_directory;
        mgr.ensure_models_dir().map_err(|e| e.to_string())?;
        (url, path, is_dir)
    };

    // Clone values for the async task
    let mid = model_id.clone();
    let app_handle = app.clone();
    let tracker_arc = {
        // We need to get the inner Arc from the State. Since DownloadTrackerState
        // wraps a Mutex directly, we need to clone the tracker handle differently.
        // We'll pass a reference to the app handle and use it to access state later.
        app.clone()
    };

    // Spawn download in background task
    tauri::async_runtime::spawn(async move {
        match download_model_file(&app_handle, &mid, &download_url, &model_path, is_directory)
            .await
        {
            Ok(()) => {
                let _ = app_handle.emit(
                    "model-download-complete",
                    serde_json::json!({ "modelId": mid }),
                );
                tracing::info!("Model {} downloaded successfully", mid);
            }
            Err(e) => {
                // Clean up empty/partial model directory on failure
                if model_path.exists() {
                    let _ = std::fs::remove_dir_all(&model_path);
                    tracing::info!(
                        "Cleaned up failed download directory: {}",
                        model_path.display()
                    );
                }
                let _ = app_handle.emit(
                    "model-download-error",
                    serde_json::json!({ "modelId": mid, "error": e.to_string() }),
                );
                tracing::error!("Model {} download failed: {}", mid, e);
            }
        }

        // Remove from download tracker
        {
            use tauri::Manager;
            let tracker = tracker_arc.state::<DownloadTrackerState>();
            let lock_result = tracker.0.lock();
            if let Ok(mut downloading) = lock_result {
                downloading.remove(&mid);
            }
        }
    });

    Ok(())
}

/// Download a model file with progress reporting.
/// For directory models, downloads as tar.gz and extracts.
async fn download_model_file(
    app: &tauri::AppHandle,
    model_id: &str,
    url: &str,
    dest_dir: &std::path::Path,
    is_directory: bool,
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

    if is_directory {
        // Download tar.gz to a temp file, then extract
        let tar_path = dest_dir.join("_download.tar.gz");
        let mut file = tokio::fs::File::create(&tar_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut last_progress_emit = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            downloaded += chunk.len() as u64;

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

        // Emit 100% download progress
        let _ = app.emit(
            "model-download-progress",
            serde_json::json!({
                "modelId": model_id,
                "progress": 1.0,
            }),
        );

        // Emit extracting event
        let _ = app.emit(
            "model-extracting",
            serde_json::json!({ "modelId": model_id }),
        );

        // Extract tar.gz (blocking I/O, run in spawn_blocking)
        let tar_path_clone = tar_path.clone();
        let dest_dir_owned = dest_dir.to_path_buf();
        tokio::task::spawn_blocking(move || extract_tar_gz(&tar_path_clone, &dest_dir_owned))
            .await??;

        // Clean up the tar.gz file
        let _ = tokio::fs::remove_file(&tar_path).await;

        // Emit extraction complete
        let _ = app.emit(
            "model-extraction-complete",
            serde_json::json!({ "modelId": model_id }),
        );
    } else {
        // Single file download (Whisper GGML .bin)
        let file_name = url.split('/').last().unwrap_or("model.bin");
        let file_path = dest_dir.join(file_name);

        let mut file = tokio::fs::File::create(&file_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut last_progress_emit = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            downloaded += chunk.len() as u64;

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
    }

    Ok(())
}

/// Extract a tar.gz archive into the destination directory.
fn extract_tar_gz(
    tar_path: &std::path::Path,
    dest_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = std::fs::File::open(tar_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    archive.unpack(dest_dir)?;

    tracing::info!(
        "Extracted tar.gz {} into {}",
        tar_path.display(),
        dest_dir.display()
    );

    Ok(())
}
