/// Placeholder command to verify Tauri IPC works.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Speech v3 is running.", name)
}
