use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::managers::settings::AppSettings;

const STORE_FILE: &str = "settings.json";

/// Get all settings.
#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;

    let settings = AppSettings {
        launch_at_login: store
            .get("launchAtLogin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        auto_paste: store
            .get("autoPaste")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        remove_filler_words: store
            .get("removeFillerWords")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        record_hotkey: store
            .get("recordHotkey")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Alt+Space".to_string()),
        switch_hotkey: store
            .get("switchHotkey")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Alt+Shift+Space".to_string()),
        switch_hotkey_enabled: store
            .get("switchHotkeyEnabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        selected_language: store
            .get("selectedLanguage")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "auto".to_string()),
        selected_model: store
            .get("selectedModel")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "whisper-small".to_string()),
        active_profile_index: store
            .get("activeProfileIndex")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0),
    };

    Ok(settings)
}

/// Update a single setting by key.
#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: serde_json::Value) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(&key, value);

    // Read the value back from the store before dropping the store reference
    let stored_value = store.get(&key);

    // Apply side-effects for certain settings
    match key.as_str() {
        "autoPaste" => {
            if let Some(auto_paste) = stored_value.and_then(|v| v.as_bool()) {
                let coord = app.state::<crate::state::CoordinatorState>();
                let _ = coord.0.lock().map(|mut c| c.set_auto_paste(auto_paste));
            }
        }
        "removeFillerWords" => {
            if let Some(remove) = stored_value.and_then(|v| v.as_bool()) {
                let coord = app.state::<crate::state::CoordinatorState>();
                let _ = coord.0.lock().map(|mut c| c.set_remove_filler_words(remove));
            }
        }
        "selectedLanguage" => {
            if let Some(lang) = stored_value.and_then(|v| v.as_str().map(|s| s.to_string())) {
                let coord = app.state::<crate::state::CoordinatorState>();
                let _ = coord.0.lock().map(|mut c| c.set_language(lang));
            }
        }
        _ => {}
    }

    Ok(())
}

/// Open the settings window.
#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    // Check if settings window already exists
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    // Create new settings window
    let _window = tauri::WebviewWindowBuilder::new(&app, "settings", tauri::WebviewUrl::App("/".into()))
        .title("Speech Settings")
        .inner_size(450.0, 560.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    // Activate the app so the window is visible
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication};
        unsafe {
            let ns_app = NSApp();
            #[allow(deprecated)]
            ns_app.activateIgnoringOtherApps_(true);
        }
    }

    Ok(())
}
