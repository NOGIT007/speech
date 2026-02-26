use crate::managers::permissions;

/// Check all permission statuses.
/// Returns { microphone: bool, accessibility: bool, inputMonitoring: bool }.
#[tauri::command]
pub fn check_permissions() -> permissions::PermissionStatus {
    permissions::check_permissions()
}

/// Open permission settings for the given type.
/// Types: "microphone", "accessibility", "inputMonitoring".
#[tauri::command]
pub fn open_permission_settings(permission_type: String) {
    match permission_type.as_str() {
        "microphone" => permissions::open_microphone_settings(),
        "accessibility" => permissions::prompt_accessibility(),
        "inputMonitoring" => permissions::open_input_monitoring_settings(),
        _ => tracing::warn!("Unknown permission type: {}", permission_type),
    }
}

/// Reset TCC permissions and relaunch the app.
/// Matches SettingsView.swift:363-386.
#[tauri::command]
pub fn reset_permissions(app: tauri::AppHandle) {
    let bundle_id = app
        .config()
        .identifier
        .clone();

    permissions::reset_permissions_and_relaunch(&bundle_id);

    // Relaunch the app
    #[cfg(target_os = "macos")]
    {
        if let Ok(exe) = std::env::current_exe() {
            // Find the .app bundle (go up from binary to Contents/MacOS -> Contents -> .app)
            let app_bundle = exe
                .parent() // MacOS
                .and_then(|p| p.parent()) // Contents
                .and_then(|p| p.parent()); // .app

            if let Some(app_path) = app_bundle {
                let _ = std::process::Command::new("open")
                    .arg(app_path)
                    .spawn();
            }
        }

        // Give the new instance time to start, then quit
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::exit(0);
        });
    }
}
