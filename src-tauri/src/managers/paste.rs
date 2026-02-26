use anyhow::{Context, Result};
use std::sync::Mutex;

/// Manages text injection via clipboard + CGEvent Cmd+V simulation.
/// Port of TextInjector.swift:1-138.
pub struct PasteManager {
    /// PID of the app that was focused before recording started
    previous_app_pid: Mutex<Option<i32>>,
}

impl PasteManager {
    pub fn new() -> Self {
        Self {
            previous_app_pid: Mutex::new(None),
        }
    }

    /// Save the currently focused app's PID before recording.
    /// Matches TextInjector.swift:15-17.
    pub fn save_focused_app(&self) {
        #[cfg(target_os = "macos")]
        {
            let pid = get_frontmost_app_pid();
            *self.previous_app_pid.lock().unwrap() = pid;
            if let Some(pid) = pid {
                tracing::debug!("Saved focused app PID: {}", pid);
            }
        }
    }

    /// Clear the saved previous app reference.
    pub fn clear_previous_app(&self) {
        *self.previous_app_pid.lock().unwrap() = None;
    }

    /// Inject text into the previously focused application.
    /// Matches TextInjector.swift:23-77.
    pub async fn inject_text(&self, text: &str, auto_paste: bool) -> Result<()> {
        // Save current clipboard contents for restoration
        let saved_clipboard = get_clipboard_text();

        // Set clipboard to transcribed text
        set_clipboard_text(text)?;

        // Wait for clipboard to propagate (100ms, matching TextInjector.swift:33)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Restore focus to previous app
        let prev_pid = *self.previous_app_pid.lock().unwrap();
        if let Some(pid) = prev_pid {
            activate_app_by_pid(pid);

            // Wait for app to become active (matching TextInjector.swift:40-44)
            for _ in 0..20 {
                if get_frontmost_app_pid() == Some(pid) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        }

        // Buffer after focus restoration (100ms, matching TextInjector.swift:48)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Set clipboard AGAIN after focus restore (matching TextInjector.swift:51)
        set_clipboard_text(text)?;

        // Final buffer (50ms, matching TextInjector.swift:54)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        if auto_paste {
            // Wait for modifier release (matching TextInjector.swift:82-95)
            wait_for_modifier_release().await;

            // Simulate Cmd+V (matching TextInjector.swift:98-113)
            if simulate_paste() {
                // Restore original clipboard after paste (100ms delay, matching TextInjector.swift:64-69)
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(saved) = saved_clipboard {
                    let _ = set_clipboard_text(&saved);
                } else {
                    let _ = clear_clipboard();
                }
            } else {
                tracing::warn!("Failed to simulate Cmd+V paste");
            }
        }

        // Clear previous app reference
        self.clear_previous_app();

        Ok(())
    }
}

/// Get the frontmost application's PID via NSWorkspace.
#[cfg(target_os = "macos")]
fn get_frontmost_app_pid() -> Option<i32> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let workspace_class = Class::get("NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        if workspace.is_null() {
            return None;
        }
        let app: *mut Object = msg_send![workspace, frontmostApplication];
        if app.is_null() {
            return None;
        }
        let pid: i32 = msg_send![app, processIdentifier];
        Some(pid)
    }
}

/// Activate an app by its PID.
/// Matches TextInjector.swift:37-38.
#[cfg(target_os = "macos")]
fn activate_app_by_pid(pid: i32) {
    use objc::runtime::{Class, Object, BOOL};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let running_app_class = Class::get("NSRunningApplication").unwrap();
        let app: *mut Object =
            msg_send![running_app_class, runningApplicationWithProcessIdentifier: pid];
        if !app.is_null() {
            let options: u64 = 1 << 1; // NSApplicationActivateIgnoringOtherApps
            let _: BOOL = msg_send![app, activateWithOptions: options];
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn get_frontmost_app_pid() -> Option<i32> {
    None
}

#[cfg(not(target_os = "macos"))]
fn activate_app_by_pid(_pid: i32) {}

/// Get clipboard text via arboard.
fn get_clipboard_text() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
}

/// Set clipboard text via arboard.
fn set_clipboard_text(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to open clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to set clipboard text")?;
    Ok(())
}

/// Clear the clipboard.
fn clear_clipboard() -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to open clipboard")?;
    clipboard.clear().context("Failed to clear clipboard")?;
    Ok(())
}

/// Wait for all modifier keys to be released.
/// Matches TextInjector.swift:82-95.
async fn wait_for_modifier_release() {
    #[cfg(target_os = "macos")]
    {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let relevant_modifiers: u64 = (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20);
        // shift=17, control=18, option=19, command=20

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(1);

        loop {
            let flags: u64 = unsafe {
                let event_class = Class::get("NSEvent").unwrap();
                msg_send![event_class, modifierFlags]
            };

            if flags & relevant_modifiers == 0 {
                break;
            }

            if start.elapsed() >= timeout {
                tracing::warn!("Modifier release timeout after 1s");
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // Small buffer after modifiers released (50ms, matching TextInjector.swift:94)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// Simulate Cmd+V paste via CGEvent.
/// Matches TextInjector.swift:98-113.
fn simulate_paste() -> bool {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
            Ok(s) => s,
            Err(_) => {
                tracing::error!("Failed to create CGEventSource");
                return false;
            }
        };

        // Virtual key code for 'V' = 0x09 (matching TextInjector.swift:99)
        let v_keycode: u16 = 0x09;

        let key_down = CGEvent::new_keyboard_event(source.clone(), v_keycode, true);
        let key_up = CGEvent::new_keyboard_event(source, v_keycode, false);

        match (key_down, key_up) {
            (Ok(down), Ok(up)) => {
                down.set_flags(CGEventFlags::CGEventFlagCommand);
                up.set_flags(CGEventFlags::CGEventFlagCommand);

                // Post to cghidEventTap (matching TextInjector.swift:109-110)
                down.post(CGEventTapLocation::HID);
                up.post(CGEventTapLocation::HID);

                true
            }
            _ => {
                tracing::error!("Failed to create CGEvent for Cmd+V");
                false
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
