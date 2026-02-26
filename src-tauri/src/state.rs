use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::managers::audio::AudioState;
use crate::managers::paste::PasteManager;
use crate::text_cleaner;

// Frontend events emitted during the recording lifecycle.
pub const EVENT_PHASE_CHANGED: &str = "phase-changed";
pub const EVENT_TRANSCRIPTION: &str = "transcription-result";
pub const EVENT_ERROR: &str = "app-error";
pub const EVENT_HISTORY_UPDATED: &str = "history-updated";

/// Application phase matching AppState.swift's isRecording/isTranscribing states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppPhase {
    Idle,
    Recording,
    Processing,
}

/// A single transcription history entry.
/// Matches AppState.swift:394-411 TranscriptionItem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionItem {
    pub id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub preview: String,
}

impl TranscriptionItem {
    pub fn new(text: String) -> Self {
        let preview = if text.len() <= 50 {
            text.clone()
        } else {
            format!("{}...", &text[..47])
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            timestamp: Utc::now(),
            preview,
        }
    }
}

/// Central coordinator for the recording flow.
/// Manages the state machine: Idle -> Recording -> Processing -> Idle.
/// Matches AppState.swift:109-215.
pub struct RecordingCoordinator {
    phase: AppPhase,
    history: Vec<TranscriptionItem>,
    /// Currently selected language for transcription
    selected_language: String,
    /// Whether to clean filler words from transcription
    remove_filler_words: bool,
    /// Whether to auto-paste transcribed text
    auto_paste: bool,
}

/// Tauri managed state wrapper for the coordinator.
pub struct CoordinatorState(pub Mutex<RecordingCoordinator>);

/// Tauri managed state wrapper for the paste manager.
pub struct PasteState(pub Mutex<PasteManager>);

impl RecordingCoordinator {
    pub fn new() -> Self {
        Self {
            phase: AppPhase::Idle,
            history: Vec::new(),
            selected_language: "auto".to_string(),
            remove_filler_words: true,
            auto_paste: true,
        }
    }

    pub fn phase(&self) -> AppPhase {
        self.phase
    }

    pub fn history(&self) -> &[TranscriptionItem] {
        &self.history
    }

    pub fn set_language(&mut self, language: String) {
        self.selected_language = language;
    }

    pub fn set_remove_filler_words(&mut self, remove: bool) {
        self.remove_filler_words = remove;
    }

    pub fn set_auto_paste(&mut self, auto_paste: bool) {
        self.auto_paste = auto_paste;
    }

    pub fn delete_history_item(&mut self, id: &str) {
        self.history.retain(|item| item.id != id);
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Start recording: save focused app, start audio, change phase.
/// Matches AppState.swift:109-128 startRecording().
pub fn start_recording(app: &AppHandle) -> Result<(), String> {
    // Check we're idle
    {
        let coord = app.state::<CoordinatorState>();
        let coord = coord.0.lock().map_err(|e| e.to_string())?;
        if coord.phase != AppPhase::Idle {
            return Err("Not in idle state".into());
        }
    }

    // Save focused app before any async work (matching TextInjector.swift:15-17)
    {
        let paste = app.state::<PasteState>();
        let paste = paste.0.lock().map_err(|e| e.to_string())?;
        paste.save_focused_app();
    }

    // Start audio recording
    {
        let audio = app.state::<AudioState>();
        crate::managers::audio::start_recording(&audio).map_err(|e| e.to_string())?;
    }

    // Update phase to Recording
    {
        let coord = app.state::<CoordinatorState>();
        let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
        coord.phase = AppPhase::Recording;
    }

    // Emit phase change to frontend
    let _ = app.emit(EVENT_PHASE_CHANGED, AppPhase::Recording);

    // Show recording overlay (positioned on cursor screen)
    if let Some(window) = app.get_webview_window("recording-overlay") {
        position_overlay_on_cursor_screen(&window);
        let _ = window.show();
        let _ = window.set_ignore_cursor_events(true);
    }

    // Start audio level monitoring at ~30fps (matching AppState.swift:217-228)
    // We clone the Arc-wrapped state directly so we don't hold State borrows across awaits.
    let level_arc = {
        let audio = app.state::<AudioState>();
        std::sync::Arc::clone(&audio.level)
    };
    let is_rec_arc = {
        let audio = app.state::<AudioState>();
        std::sync::Arc::clone(&audio.is_recording)
    };
    let app_for_level = app.clone();
    tauri::async_runtime::spawn(async move {
        use std::sync::atomic::Ordering;
        loop {
            if !is_rec_arc.load(Ordering::Relaxed) {
                break;
            }

            let level = f32::from_bits(level_arc.load(Ordering::Relaxed));
            let _ = app_for_level.emit("audio-level", level);

            tokio::time::sleep(std::time::Duration::from_millis(33)).await; // ~30fps
        }
    });

    tracing::info!("Recording started");
    Ok(())
}

/// Cancel recording: stop audio, delete temp file, hide overlay, reset state.
/// Matches AppState.swift:130-144 cancelRecording().
pub fn cancel_recording(app: &AppHandle) -> Result<(), String> {
    let was_recording;
    {
        let coord = app.state::<CoordinatorState>();
        let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
        was_recording = coord.phase == AppPhase::Recording;
        coord.phase = AppPhase::Idle;
    }

    if was_recording {
        // Stop audio and get the file path
        let audio = app.state::<AudioState>();
        if let Ok(path) = crate::managers::audio::stop_recording(&audio) {
            // Clean up temp file
            let _ = std::fs::remove_file(&path);
        }
    }

    // Hide overlay
    if let Some(window) = app.get_webview_window("recording-overlay") {
        let _ = window.hide();
    }

    // Clear saved focused app
    {
        let paste = app.state::<PasteState>();
        let paste = paste.0.lock().map_err(|e| e.to_string())?;
        paste.clear_previous_app();
    }

    // Emit phase change
    let _ = app.emit(EVENT_PHASE_CHANGED, AppPhase::Idle);

    tracing::info!("Recording cancelled");
    Ok(())
}

/// Stop recording and start transcription.
/// Matches AppState.swift:146-160 stopRecordingAndTranscribe().
pub fn stop_and_transcribe(app: &AppHandle) -> Result<(), String> {
    // Check we're recording
    {
        let coord = app.state::<CoordinatorState>();
        let coord = coord.0.lock().map_err(|e| e.to_string())?;
        if coord.phase != AppPhase::Recording {
            return Err("Not recording".into());
        }
    }

    // Transition to processing
    {
        let coord = app.state::<CoordinatorState>();
        let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
        coord.phase = AppPhase::Processing;
    }
    let _ = app.emit(EVENT_PHASE_CHANGED, AppPhase::Processing);
    let _ = app.emit("overlay-mode", "processing");

    // Stop audio recording
    let audio_path;
    {
        let audio = app.state::<AudioState>();
        audio_path = crate::managers::audio::stop_recording(&audio).map_err(|e| e.to_string())?;
    }

    tracing::info!("Recording stopped, transcribing: {}", audio_path.display());

    // Get settings from coordinator
    let (language, remove_fillers, auto_paste);
    {
        let coord = app.state::<CoordinatorState>();
        let coord = coord.0.lock().map_err(|e| e.to_string())?;
        language = coord.selected_language.clone();
        remove_fillers = coord.remove_filler_words;
        auto_paste = coord.auto_paste;
    }

    // Perform transcription (currently a stub, will be wired to transcribe-rs)
    let result;
    {
        let ts = app.state::<crate::commands::model::TranscriptionState>();
        let ts = ts.0.lock().map_err(|e| e.to_string())?;
        result = ts
            .transcribe(&audio_path, &language)
            .map_err(|e| e.to_string())?;
    }

    let mut text = result.text;

    // Apply filler word removal if enabled (matching AppState.swift:175-177)
    if remove_fillers {
        text = text_cleaner::clean(&text);
    }

    tracing::info!("Transcription: {} ({}ms)", text, result.duration_ms);

    // Add to history (keep last 5, matching AppState.swift:183-190)
    if !text.is_empty() {
        let item = TranscriptionItem::new(text.clone());
        {
            let coord = app.state::<CoordinatorState>();
            let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
            coord.history.insert(0, item);
            if coord.history.len() > 5 {
                coord.history.truncate(5);
            }
        }
        // Emit updated history
        let history;
        {
            let coord = app.state::<CoordinatorState>();
            let coord = coord.0.lock().map_err(|e| e.to_string())?;
            history = coord.history.clone();
        }
        let _ = app.emit(EVENT_HISTORY_UPDATED, &history);
    }

    // Emit transcription result to frontend
    let _ = app.emit(EVENT_TRANSCRIPTION, &text);

    // Inject text via paste manager (matching AppState.swift:195-196)
    // We extract the saved PID from the PasteManager while holding the lock,
    // then pass it to the async inject function to avoid holding the lock across await.
    if !text.is_empty() {
        let app_clone = app.clone();
        let text_clone = text.clone();

        // Extract the previous app PID while we have the lock
        let prev_pid = {
            let paste = app.state::<PasteState>();
            let paste_mgr = paste.0.lock().map_err(|e| e.to_string())?;
            paste_mgr.get_previous_app_pid()
        };

        tauri::async_runtime::spawn(async move {
            if let Err(e) =
                crate::managers::paste::inject_text_with_pid(&text_clone, auto_paste, prev_pid)
                    .await
            {
                tracing::error!("Failed to inject text: {}", e);
                let _ = app_clone.emit(EVENT_ERROR, format!("Paste failed: {}", e));
            }
            // Clear the saved app reference after injection
            {
                let paste = app_clone.state::<PasteState>();
                let _ = paste.0.lock().map(|mgr| mgr.clear_previous_app());
            }
        });
    }

    // Clean up audio file
    let _ = std::fs::remove_file(&audio_path);

    // Return to idle
    {
        let coord = app.state::<CoordinatorState>();
        let mut coord = coord.0.lock().map_err(|e| e.to_string())?;
        coord.phase = AppPhase::Idle;
    }

    let _ = app.emit(EVENT_PHASE_CHANGED, AppPhase::Idle);

    // Show "ready" overlay briefly, then hide (matching RecordingOverlayController:169-178)
    let _ = app.emit("overlay-mode", "ready");

    let auto_paste_clone = auto_paste;
    let app_hide = app.clone();
    tauri::async_runtime::spawn(async move {
        let delay_ms = if auto_paste_clone { 500 } else { 1000 };
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        if let Some(window) = app_hide.get_webview_window("recording-overlay") {
            let _ = window.hide();
        }
    });

    tracing::info!("Transcription flow complete");
    Ok(())
}

/// Set up event listeners that connect hotkey events to the recording coordinator.
/// This wires HotkeyManager events -> RecordingCoordinator actions.
/// Matches AppDelegate.swift:44-48 setupHotkey().
pub fn setup_hotkey_listeners(app: &AppHandle) -> Result<(), String> {
    use crate::managers::hotkey;

    // Initialize hotkey manager and register hotkeys
    let hotkey_mgr = hotkey::HotkeyManager::new();

    // Register default record hotkey: Alt+Space (matching AppState.swift:284-287)
    hotkey_mgr
        .register_record_hotkey(app, "Alt+Space")
        .map_err(|e| e.to_string())?;

    // Register Escape as cancel during recording
    hotkey_mgr
        .register_escape_cancel(app)
        .map_err(|e| e.to_string())?;

    // Listen for hotkey-record-down -> start recording
    let app_handle = app.clone();
    app.listen(hotkey::EVENT_RECORD_DOWN, move |_event| {
        if let Err(e) = start_recording(&app_handle) {
            tracing::error!("Failed to start recording: {}", e);
            let _ = app_handle.emit(EVENT_ERROR, e);
        }
    });

    // Listen for hotkey-record-up -> stop and transcribe
    let app_handle = app.clone();
    app.listen(hotkey::EVENT_RECORD_UP, move |_event| {
        if let Err(e) = stop_and_transcribe(&app_handle) {
            tracing::error!("Failed to stop and transcribe: {}", e);
            let _ = app_handle.emit(EVENT_ERROR, e);
        }
    });

    // Listen for hotkey-record-cancel -> cancel recording
    let app_handle = app.clone();
    app.listen(hotkey::EVENT_RECORD_CANCEL, move |_event| {
        if let Err(e) = cancel_recording(&app_handle) {
            tracing::error!("Failed to cancel recording: {}", e);
            let _ = app_handle.emit(EVENT_ERROR, e);
        }
    });

    tracing::info!("Hotkey listeners set up");
    Ok(())
}

/// Position the overlay window centered on the screen containing the cursor.
/// Matches RecordingOverlay.swift:32-37.
fn position_overlay_on_cursor_screen(window: &tauri::WebviewWindow) {
    // Try to get cursor position and center the overlay on that screen
    if let Ok(cursor_pos) = window.cursor_position() {
        let width = 380.0;
        let height = 220.0;
        let x = cursor_pos.x - width / 2.0;
        let y = cursor_pos.y - height / 2.0;
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            x.max(0.0),
            y.max(0.0),
        )));
    }
}
