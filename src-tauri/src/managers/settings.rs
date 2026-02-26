use serde::{Deserialize, Serialize};

/// Application settings persisted via tauri-plugin-store.
/// Matches the Swift UserDefaults/AppStorage keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub auto_paste: bool,
    pub remove_filler_words: bool,
    pub record_hotkey: String,
    pub switch_hotkey: String,
    pub switch_hotkey_enabled: bool,
    pub selected_language: String,
    pub selected_model: String,
    pub active_profile_index: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            auto_paste: true,
            remove_filler_words: true,
            record_hotkey: "Alt+Space".to_string(),
            switch_hotkey: "Alt+Shift+Space".to_string(),
            switch_hotkey_enabled: false,
            selected_language: "auto".to_string(),
            selected_model: "whisper-small".to_string(),
            active_profile_index: 0,
        }
    }
}
