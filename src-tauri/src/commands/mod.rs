pub mod audio;
pub mod model;
pub mod permissions;
pub mod settings;
pub mod state;

/// Placeholder command to verify Tauri IPC works.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Speech v3 is running.", name)
}
