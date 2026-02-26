use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Events emitted by the hotkey manager.
pub const EVENT_RECORD_DOWN: &str = "hotkey-record-down";
pub const EVENT_RECORD_UP: &str = "hotkey-record-up";
pub const EVENT_RECORD_CANCEL: &str = "hotkey-record-cancel";
pub const EVENT_SWITCH_PROFILE: &str = "hotkey-switch-profile";

/// Manages global hotkeys for recording and profile switching.
/// Porting from HotkeyManager.swift:1-153.
pub struct HotkeyManager {
    /// Whether the record key is currently held down
    is_key_down: Arc<AtomicBool>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            is_key_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Register the record hotkey (hold-to-record pattern).
    /// Default: Alt+Space (matching AppState.swift:284-287 HotkeyConfig.default).
    pub fn register_record_hotkey(&self, app: &AppHandle, shortcut_str: &str) -> Result<()> {
        let shortcut: Shortcut = shortcut_str
            .parse()
            .context(format!("Invalid shortcut: {}", shortcut_str))?;

        let is_key_down = Arc::clone(&self.is_key_down);
        let app_handle = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                match event.state {
                    ShortcutState::Pressed => {
                        if !is_key_down.swap(true, Ordering::Relaxed) {
                            // First press - start recording
                            // Matches HotkeyManager.swift:96-100 handleKeyDown
                            tracing::info!("Record hotkey pressed");
                            let _ = app_handle.emit(EVENT_RECORD_DOWN, ());
                        }
                    }
                    ShortcutState::Released => {
                        if is_key_down.swap(false, Ordering::Relaxed) {
                            // Released - stop recording
                            // Matches HotkeyManager.swift:102-106 handleKeyUp
                            tracing::info!("Record hotkey released");
                            let _ = app_handle.emit(EVENT_RECORD_UP, ());
                        }
                    }
                }
            })
            .context("Failed to register record hotkey")?;

        tracing::info!("Record hotkey registered: {}", shortcut_str);
        Ok(())
    }

    /// Register the profile switch hotkey (single press cycles profiles).
    /// Matches HotkeyManager.swift:55-69.
    pub fn register_switch_hotkey(&self, app: &AppHandle, shortcut_str: &str) -> Result<()> {
        let shortcut: Shortcut = shortcut_str
            .parse()
            .context(format!("Invalid shortcut: {}", shortcut_str))?;

        let app_handle = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::info!("Switch profile hotkey pressed");
                    let _ = app_handle.emit(EVENT_SWITCH_PROFILE, ());
                }
            })
            .context("Failed to register switch hotkey")?;

        tracing::info!("Switch hotkey registered: {}", shortcut_str);
        Ok(())
    }

    /// Cancel the current recording (triggered by Escape key).
    /// Matches HotkeyManager.swift:110-113.
    pub fn cancel_recording(&self, app: &AppHandle) {
        if self.is_key_down.swap(false, Ordering::Relaxed) {
            tracing::info!("Recording cancelled via Escape");
            let _ = app.emit(EVENT_RECORD_CANCEL, ());
        }
    }

    /// Register Escape key as cancel during recording.
    pub fn register_escape_cancel(&self, app: &AppHandle) -> Result<()> {
        let shortcut: Shortcut = "Escape"
            .parse()
            .context("Failed to parse Escape shortcut")?;

        let is_key_down = Arc::clone(&self.is_key_down);
        let app_handle = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed
                    && is_key_down.load(Ordering::Relaxed)
                {
                    is_key_down.store(false, Ordering::Relaxed);
                    tracing::info!("Recording cancelled via Escape");
                    let _ = app_handle.emit(EVENT_RECORD_CANCEL, ());
                }
            })
            .context("Failed to register Escape cancel")?;

        Ok(())
    }

    /// Unregister all hotkeys.
    pub fn unregister_all(&self, app: &AppHandle) -> Result<()> {
        app.global_shortcut()
            .unregister_all()
            .context("Failed to unregister hotkeys")?;
        self.is_key_down.store(false, Ordering::Relaxed);
        tracing::info!("All hotkeys unregistered");
        Ok(())
    }

    /// Check if the record key is currently held down.
    pub fn is_recording_active(&self) -> bool {
        self.is_key_down.load(Ordering::Relaxed)
    }
}
