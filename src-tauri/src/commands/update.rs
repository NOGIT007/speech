use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::managers::update::UpdateInfo;

/// Check for updates using tauri-plugin-updater.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".into());

    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            current_version: current_version.clone(),
            latest_version: update.version.clone(),
            update_available: true,
            release_notes: update.body.clone(),
        }),
        Ok(None) => Ok(UpdateInfo {
            current_version: current_version.clone(),
            latest_version: current_version,
            update_available: false,
            release_notes: None,
        }),
        Err(e) => {
            tracing::warn!("Update check failed: {}", e);
            Ok(UpdateInfo {
                current_version: current_version.clone(),
                latest_version: current_version,
                update_available: false,
                release_notes: None,
            })
        }
    }
}

/// Install an available update and relaunch.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            update
                .download_and_install(|_, _| {}, || {})
                .await
                .map_err(|e| e.to_string())?;
            app.restart();
        }
        Ok(None) => Err("No update available".into()),
        Err(e) => Err(e.to_string()),
    }
}
