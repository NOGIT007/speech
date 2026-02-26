use std::sync::Mutex;

use tauri::State;

use crate::managers::model::{ModelManager, ModelStatus};
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
pub fn delete_model(model_mgr: State<ModelState>, model_id: String) -> Result<(), String> {
    let mgr = model_mgr.0.lock().map_err(|e| e.to_string())?;
    mgr.delete_model(&model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_supported_languages() -> Vec<crate::managers::model::Language> {
    crate::managers::model::ModelManager::supported_languages()
}
