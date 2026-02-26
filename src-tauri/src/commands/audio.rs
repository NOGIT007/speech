use tauri::State;

use crate::managers::audio::AudioState;

#[tauri::command]
pub fn start_recording(audio: State<AudioState>) -> Result<String, String> {
    let path = crate::managers::audio::start_recording(&audio).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn stop_recording(audio: State<AudioState>) -> Result<String, String> {
    let path = crate::managers::audio::stop_recording(&audio).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_audio_level(audio: State<AudioState>) -> Result<f32, String> {
    Ok(audio.current_level())
}
