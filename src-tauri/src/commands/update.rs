use tauri::AppHandle;

use crate::managers::update::UpdateInfo;

/// Check for updates.
/// TODO: Enable once tauri-plugin-updater signing keys are configured.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".into());

    Ok(UpdateInfo {
        current_version: current_version.clone(),
        latest_version: current_version,
        update_available: false,
        release_notes: None,
    })
}

/// Install an available update and relaunch.
/// TODO: Enable once tauri-plugin-updater signing keys are configured.
#[tauri::command]
pub async fn install_update(_app: AppHandle) -> Result<(), String> {
    Err("Updater not configured yet".into())
}
