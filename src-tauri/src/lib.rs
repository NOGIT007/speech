pub mod commands;
pub mod managers;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Set activation policy to Accessory (no dock icon, like LSUIElement)
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                unsafe {
                    let ns_app = NSApp();
                    ns_app.setActivationPolicy_(
                        NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
                    );
                }
            }

            // Hide the main window initially (will be shown by tray click)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            // Hide overlay windows initially
            if let Some(window) = app.get_webview_window("recording-overlay") {
                let _ = window.hide();
            }
            if let Some(window) = app.get_webview_window("switch-overlay") {
                let _ = window.hide();
            }

            tracing::info!("Speech v3.0.0 started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speech");
}
