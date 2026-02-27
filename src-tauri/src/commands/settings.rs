use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

use crate::commands::model::{ModelState, TranscriptionState};
use crate::managers::settings::AppSettings;

const STORE_FILE: &str = "settings.json";

/// Load the selected model into the TranscriptionManager.
/// Called at startup and when the selected model changes.
/// Falls back to the active profile's model or any downloaded model if the
/// selected model is not in the registry or not downloaded.
pub fn load_selected_model(app: &tauri::AppHandle) {
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to open store for model loading: {}", e);
            return;
        }
    };

    let mut model_id = store
        .get("selectedModel")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "whisper-small".into());

    // Get model path from ModelManager (scoped to release State borrow)
    let model_path = {
        let model_mgr = app.state::<ModelState>();
        let mgr = match model_mgr.0.lock() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to lock ModelManager: {}", e);
                return;
            }
        };

        // If selected model isn't in registry or not downloaded, find a fallback
        if mgr.get_model(&model_id).is_none() || !mgr.is_model_downloaded(&model_id) {
            tracing::warn!(
                "Selected model '{}' not available, searching for fallback",
                model_id
            );

            // Try active profile's model first
            let profile_model = store
                .get("profiles")
                .and_then(|v| serde_json::from_value::<Vec<serde_json::Value>>(v.clone()).ok())
                .and_then(|profiles| {
                    let idx = store
                        .get("activeProfileIndex")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    profiles
                        .get(idx)
                        .and_then(|p| p.get("modelId"))
                        .and_then(|v| v.as_str().map(String::from))
                });

            let mut found_fallback = false;
            if let Some(ref pm) = profile_model {
                if mgr.get_model(pm).is_some() && mgr.is_model_downloaded(pm) {
                    tracing::info!("Falling back to active profile's model: {}", pm);
                    model_id = pm.clone();
                    store.set("selectedModel", serde_json::json!(model_id));
                    found_fallback = true;
                }
            }

            // If still not available, find any downloaded model
            if !found_fallback {
                let models = mgr.list_models();
                if let Some(downloaded) = models.iter().find(|m| m.downloaded) {
                    tracing::info!(
                        "Falling back to first downloaded model: {}",
                        downloaded.info.id
                    );
                    model_id = downloaded.info.id.clone();
                    store.set("selectedModel", serde_json::json!(model_id));
                } else {
                    tracing::warn!("No models downloaded, cannot load any model");
                    return;
                }
            }
        }

        mgr.get_model_path(&model_id)
    };

    // Load into TranscriptionManager (scoped to release State borrow)
    {
        let transcription = app.state::<TranscriptionState>();
        match transcription.0.lock() {
            Ok(mut tm) => {
                if let Err(e) = tm.load_model(&model_id, model_path) {
                    tracing::error!("Failed to load model '{}': {}", model_id, e);
                } else {
                    tracing::info!("Model '{}' loaded successfully", model_id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to lock TranscriptionManager: {}", e);
            }
        };
    }
}

/// Sync coordinator settings from the store at startup.
/// Ensures auto_paste, language, and filler word settings match stored values.
pub fn sync_coordinator_settings(app: &tauri::AppHandle) {
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to open store for coordinator sync: {}", e);
            return;
        }
    };

    let auto_paste = store
        .get("autoPaste")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let remove_fillers = store
        .get("removeFillerWords")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let language = store
        .get("selectedLanguage")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "auto".into());

    let coord = app.state::<crate::state::CoordinatorState>();
    if let Ok(mut c) = coord.0.lock() {
        c.set_auto_paste(auto_paste);
        c.set_remove_filler_words(remove_fillers);
        c.set_language(language.clone());
    }

    tracing::info!(
        "Coordinator synced: auto_paste={}, remove_fillers={}, language={}",
        auto_paste,
        remove_fillers,
        language
    );
}

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
        "selectedModel" => {
            load_selected_model(&app);
        }
        "launchAtLogin" => {
            if let Some(enabled) = stored_value.and_then(|v| v.as_bool()) {
                let autostart = app.autolaunch();
                if enabled {
                    let _ = autostart.enable();
                } else {
                    let _ = autostart.disable();
                }
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
