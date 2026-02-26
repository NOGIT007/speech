use tauri::{AppHandle, State};

use crate::state::{
    AppPhase, CoordinatorState, TranscriptionItem,
};

#[tauri::command]
pub fn get_phase(coord: State<CoordinatorState>) -> Result<AppPhase, String> {
    let coord = coord.0.lock().map_err(|e| e.to_string())?;
    Ok(coord.phase())
}

#[tauri::command]
pub fn get_history(coord: State<CoordinatorState>) -> Result<Vec<TranscriptionItem>, String> {
    let coord = coord.0.lock().map_err(|e| e.to_string())?;
    Ok(coord.history().to_vec())
}

#[tauri::command]
pub fn delete_history_item(
    coord: State<CoordinatorState>,
    id: String,
) -> Result<(), String> {
    let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
    coord.delete_history_item(&id);
    Ok(())
}

#[tauri::command]
pub fn clear_history(coord: State<CoordinatorState>) -> Result<(), String> {
    let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
    coord.clear_history();
    Ok(())
}

#[tauri::command]
pub fn set_language(coord: State<CoordinatorState>, language: String) -> Result<(), String> {
    let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
    coord.set_language(language);
    Ok(())
}

#[tauri::command]
pub fn set_remove_filler_words(
    coord: State<CoordinatorState>,
    remove: bool,
) -> Result<(), String> {
    let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
    coord.set_remove_filler_words(remove);
    Ok(())
}

#[tauri::command]
pub fn set_auto_paste(coord: State<CoordinatorState>, auto_paste: bool) -> Result<(), String> {
    let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
    coord.set_auto_paste(auto_paste);
    Ok(())
}

/// Manually trigger start recording from the frontend (e.g., a record button).
#[tauri::command]
pub fn cmd_start_recording(app: AppHandle) -> Result<(), String> {
    crate::state::start_recording(&app)
}

/// Manually trigger stop + transcribe from the frontend.
#[tauri::command]
pub fn cmd_stop_and_transcribe(app: AppHandle) -> Result<(), String> {
    crate::state::stop_and_transcribe(&app)
}

/// Manually trigger cancel recording from the frontend.
#[tauri::command]
pub fn cmd_cancel_recording(app: AppHandle) -> Result<(), String> {
    crate::state::cancel_recording(&app)
}
