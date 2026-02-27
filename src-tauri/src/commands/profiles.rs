use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

/// A model profile - preset model + language combination.
/// Matches AppState.swift:465-477 ModelProfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub language: String,
}

impl ModelProfile {
    pub fn new(name: String, model_id: String, language: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            model_id,
            language,
        }
    }
}

/// Get all profiles from the store.
#[tauri::command]
pub fn list_profiles(app: tauri::AppHandle) -> Result<Vec<ModelProfile>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();
    Ok(profiles)
}

/// Get the active profile index.
#[tauri::command]
pub fn get_active_profile_index(app: tauri::AppHandle) -> Result<usize, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let idx = store
        .get("activeProfileIndex")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    Ok(idx)
}

/// Create a new profile.
#[tauri::command]
pub fn create_profile(
    app: tauri::AppHandle,
    name: String,
    model_id: String,
    language: String,
) -> Result<ModelProfile, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let mut profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    let profile = ModelProfile::new(name, model_id, language);
    profiles.push(profile.clone());

    store.set(
        "profiles",
        serde_json::to_value(&profiles).map_err(|e| e.to_string())?,
    );

    Ok(profile)
}

/// Update an existing profile.
#[tauri::command]
pub fn update_profile(
    app: tauri::AppHandle,
    id: String,
    name: Option<String>,
    model_id: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let mut profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    if let Some(profile) = profiles.iter_mut().find(|p| p.id == id) {
        if let Some(n) = name {
            profile.name = n;
        }
        if let Some(m) = model_id {
            profile.model_id = m;
        }
        if let Some(l) = language {
            profile.language = l;
        }
    }

    store.set(
        "profiles",
        serde_json::to_value(&profiles).map_err(|e| e.to_string())?,
    );

    Ok(())
}

/// Delete a profile by ID. Adjusts active index if needed.
#[tauri::command]
pub fn delete_profile(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let mut profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    // Don't delete if only 1 profile
    if profiles.len() <= 1 {
        return Err("Cannot delete the last profile".into());
    }

    let active_idx = store
        .get("activeProfileIndex")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let delete_idx = profiles.iter().position(|p| p.id == id);
    if let Some(idx) = delete_idx {
        profiles.remove(idx);

        // Adjust active index (matching SettingsView.swift:142-148)
        let new_active = if active_idx == idx {
            idx.saturating_sub(1).min(profiles.len().saturating_sub(1))
        } else if active_idx > idx {
            active_idx - 1
        } else {
            active_idx
        };

        store.set(
            "profiles",
            serde_json::to_value(&profiles).map_err(|e| e.to_string())?,
        );
        store.set("activeProfileIndex", serde_json::json!(new_active));
    }

    Ok(())
}

/// Set the active profile index and apply it.
#[tauri::command]
pub fn set_active_profile(app: tauri::AppHandle, index: usize) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("activeProfileIndex", serde_json::json!(index));

    apply_active_profile(&app, &store)?;
    Ok(())
}

/// Switch to the next profile (cycle). Matches AppState.swift:441-445.
#[tauri::command]
pub fn switch_to_next_profile(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    if profiles.len() < 2 {
        return Ok(());
    }

    let active_idx = store
        .get("activeProfileIndex")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let new_idx = (active_idx + 1) % profiles.len();
    store.set("activeProfileIndex", serde_json::json!(new_idx));

    apply_active_profile(&app, &store)?;
    Ok(())
}

/// Apply the active profile's model and language to the coordinator.
/// Shows the switch overlay briefly.
/// Matches AppState.swift:447-460.
fn apply_active_profile(
    app: &tauri::AppHandle,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    let profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    let active_idx = store
        .get("activeProfileIndex")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let profile = profiles.get(active_idx).ok_or("Invalid profile index")?;

    // Update coordinator settings
    {
        let coord = app.state::<crate::state::CoordinatorState>();
        let mut c = coord.0.lock().map_err(|e| e.to_string())?;
        c.set_language(profile.language.clone());
    }

    // Update selected model in store
    store.set("selectedModel", serde_json::json!(profile.model_id));
    store.set("selectedLanguage", serde_json::json!(profile.language));

    // Load the new model into the transcription engine
    crate::commands::settings::load_selected_model(app);

    // Show switch overlay
    let _ = app.emit(
        "switch-profile",
        serde_json::json!({
            "profileName": profile.name,
            "modelId": profile.model_id,
            "language": profile.language,
        }),
    );

    // Show and auto-hide the switch overlay window
    if let Some(window) = app.get_webview_window("switch-overlay") {
        let _ = window.show();
        let _ = window.set_focus();
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            if let Some(w) = app_handle.get_webview_window("switch-overlay") {
                let _ = w.hide();
            }
        });
    }

    Ok(())
}

/// Auto-migrate: create default profile from current settings on first launch.
/// Matches AppState.swift:431-439.
#[tauri::command]
pub fn migrate_profiles(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let profiles = store
        .get("profiles")
        .and_then(|v| serde_json::from_value::<Vec<ModelProfile>>(v).ok())
        .unwrap_or_default();

    if !profiles.is_empty() {
        return Ok(());
    }

    let language = store
        .get("selectedLanguage")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "auto".into());

    let model_id = store
        .get("selectedModel")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "whisper-small".into());

    let language_name = match language.as_str() {
        "auto" => "Auto-detect",
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        other => other,
    };

    let profile = ModelProfile::new(language_name.to_string(), model_id, language);
    store.set(
        "profiles",
        serde_json::to_value(vec![profile]).map_err(|e| e.to_string())?,
    );
    store.set("activeProfileIndex", serde_json::json!(0));

    Ok(())
}
