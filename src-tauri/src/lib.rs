pub mod commands;
pub mod managers;
pub mod text_cleaner;
pub mod tray;

use std::sync::Mutex;
use tauri::Manager;

use commands::model::{ModelState, TranscriptionState};
use managers::audio::AudioState;
use managers::model::ModelManager;
use managers::transcription::TranscriptionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(AudioState::new())
        .manage(TranscriptionState(Mutex::new(TranscriptionManager::new())))
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

            // Initialize model manager with app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            app.manage(ModelState(Mutex::new(ModelManager::new(app_data_dir))));

            // Set up system tray
            tray::setup_tray(app.handle())?;

            tracing::info!("Speech v3.0.0 started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::audio::get_audio_level,
            commands::model::list_models,
            commands::model::delete_model,
            commands::model::get_supported_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speech");
}
