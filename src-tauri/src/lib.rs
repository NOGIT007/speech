pub mod commands;
pub mod managers;
pub mod state;
pub mod text_cleaner;
pub mod tray;

use std::sync::Mutex;
use tauri::Manager;

use commands::model::{ModelState, TranscriptionState};
use managers::audio::AudioState;
use managers::model::ModelManager;
use managers::paste::PasteManager;
use managers::transcription::TranscriptionManager;
use state::{CoordinatorState, PasteState, RecordingCoordinator};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(AudioState::new())
        .manage(TranscriptionState(Mutex::new(TranscriptionManager::new())))
        .manage(CoordinatorState(Mutex::new(RecordingCoordinator::new())))
        .manage(PasteState(Mutex::new(PasteManager::new())))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        // TODO: Enable updater once signing keys are configured
        // .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                // Load AVFoundation so the permissions plugin can find AVCaptureDevice
                extern "C" {
                    fn dlopen(filename: *const std::ffi::c_char, flags: i32) -> *mut std::ffi::c_void;
                }
                const RTLD_LAZY: i32 = 0x1;
                unsafe {
                    dlopen(
                        b"/System/Library/Frameworks/AVFoundation.framework/AVFoundation\0".as_ptr()
                            as *const _,
                        RTLD_LAZY,
                    );
                }

                // Set activation policy to Accessory (no dock icon, like LSUIElement)
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

            // Load the selected model into the transcription engine
            commands::settings::load_selected_model(app.handle());

            // Sync coordinator settings from store (auto_paste, language, filler words)
            commands::settings::sync_coordinator_settings(app.handle());

            // Set up system tray
            tray::setup_tray(app.handle())?;

            // Set up hotkey listeners (record, cancel, switch)
            // Matches AppDelegate.swift:44-48 setupHotkey()
            if let Err(e) = state::setup_hotkey_listeners(app.handle()) {
                tracing::error!("Failed to set up hotkey listeners: {}", e);
            }

            tracing::info!("Speech v3.0.0 started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::audio::get_audio_level,
            commands::model::list_models,
            commands::model::list_models_grouped,
            commands::model::delete_model,
            commands::model::download_model,
            commands::model::get_supported_languages,
            commands::state::get_phase,
            commands::state::get_history,
            commands::state::delete_history_item,
            commands::state::clear_history,
            commands::state::set_language,
            commands::state::set_remove_filler_words,
            commands::state::set_auto_paste,
            commands::state::cmd_start_recording,
            commands::state::cmd_stop_and_transcribe,
            commands::state::cmd_cancel_recording,
            commands::state::copy_to_clipboard,
            commands::state::quit_app,
            commands::state::relaunch_app,
            commands::profiles::list_profiles,
            commands::profiles::get_active_profile_index,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::set_active_profile,
            commands::profiles::switch_to_next_profile,
            commands::profiles::migrate_profiles,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::open_settings,
            commands::permissions::check_permissions,
            commands::permissions::reset_permissions,
            commands::update::check_for_update,
            commands::update::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speech");
}
