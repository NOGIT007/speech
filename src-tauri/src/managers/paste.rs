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
    /// Skips saving if the frontmost app is Speech itself (accessory app edge case).
    pub fn save_focused_app(&self) {
        #[cfg(target_os = "macos")]
        {
            let pid = get_frontmost_app_pid();
            let my_pid = std::process::id() as i32;

            if let Some(p) = pid {
                if p == my_pid {
                    tracing::warn!("Frontmost app is Speech itself (PID {}), keeping previous saved PID", p);
                    return; // Don't overwrite with our own PID
                }
            }

            *self.previous_app_pid.lock().unwrap() = pid;
            if let Some(pid) = pid {
                tracing::info!("Saved focused app PID: {}", pid);
            } else {
                tracing::warn!("No frontmost app found to save");
            }
        }
    }

    /// Clear the saved previous app reference.
    pub fn clear_previous_app(&self) {
        *self.previous_app_pid.lock().unwrap() = None;
    }

    /// Get the saved previous app PID (used to pass to standalone inject function).
    pub fn get_previous_app_pid(&self) -> Option<i32> {
        *self.previous_app_pid.lock().unwrap()
    }
}

/// Standalone async text injection that takes a pre-extracted PID.
/// This avoids holding a MutexGuard across await points.
/// Matches TextInjector.swift:23-77.
pub async fn inject_text_with_pid(text: &str, auto_paste: bool, prev_pid: Option<i32>) -> Result<()> {
    tracing::info!(
        "inject_text_with_pid: auto_paste={}, prev_pid={:?}, text_len={}",
        auto_paste, prev_pid, text.len()
    );

    // Set clipboard to transcribed text (kept for manual Cmd+V fallback)
    set_clipboard_text(text)?;
    tracing::debug!("Clipboard set with transcribed text");

    // Wait for clipboard to propagate (100ms, matching TextInjector.swift:33)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Restore focus to previous app
    if let Some(pid) = prev_pid {
        tracing::info!("Activating previous app PID: {}", pid);
        activate_app_by_pid(pid);

        // Wait for app to become active (up to 750ms, matching TextInjector.swift:40-44)
        let mut focus_restored = false;
        for i in 0..30 {
            if get_frontmost_app_pid() == Some(pid) {
                tracing::info!("Focus restored to PID {} after {}ms", pid, i * 25);
                focus_restored = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        if !focus_restored {
            let current = get_frontmost_app_pid();
            tracing::warn!(
                "Failed to restore focus to PID {}. Current frontmost: {:?}",
                pid, current
            );
            // Try activation again
            activate_app_by_pid(pid);
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    } else {
        tracing::warn!("No previous app PID saved, cannot restore focus");
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

        // Verify focus is on target app before pasting
        if let Some(pid) = prev_pid {
            let current = get_frontmost_app_pid();
            if current != Some(pid) {
                tracing::warn!(
                    "Focus drifted before paste! Expected PID {}, got {:?}. Re-activating.",
                    pid, current
                );
                activate_app_by_pid(pid);
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
        }

        // Simulate Cmd+V (matching TextInjector.swift:98-113)
        if simulate_paste() {
            tracing::info!("Cmd+V paste simulated successfully");
        } else {
            tracing::error!("Failed to simulate Cmd+V paste");
        }
        // Transcribed text stays on clipboard so user can manually Cmd+V elsewhere
    } else {
        tracing::info!("auto_paste disabled, text is on clipboard only");
    }

    Ok(())
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
        let running_app_class = match Class::get("NSRunningApplication") {
            Some(c) => c,
            None => {
                tracing::error!("NSRunningApplication class not found");
                return;
            }
        };
        let app: *mut Object =
            msg_send![running_app_class, runningApplicationWithProcessIdentifier: pid];
        if !app.is_null() {
            let options: u64 = 1 << 1; // NSApplicationActivateIgnoringOtherApps
            let result: BOOL = msg_send![app, activateWithOptions: options];
            tracing::debug!("activateWithOptions for PID {}: result={}", pid, result);
        } else {
            tracing::warn!("No NSRunningApplication found for PID {}", pid);
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
        use objc::runtime::Class;
        use objc::{msg_send, sel, sel_impl};

        let relevant_modifiers: u64 = (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20);
        // shift=17, control=18, option=19, command=20

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(2);

        loop {
            let flags: u64 = unsafe {
                let event_class = Class::get("NSEvent").unwrap();
                msg_send![event_class, modifierFlags]
            };

            if flags & relevant_modifiers == 0 {
                tracing::debug!("Modifiers released after {:?}", start.elapsed());
                break;
            }

            if start.elapsed() >= timeout {
                tracing::warn!("Modifier release timeout after 2s, flags: {:#x}", flags);
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // Small buffer after modifiers released (50ms, matching TextInjector.swift:94)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// Simulate Cmd+V paste via CGEvent.
/// Uses separate modifier key events (Cmd down → V down → V up → Cmd up)
/// for broader app compatibility, matching the approach used by the enigo crate.
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

        // Virtual key codes
        let cmd_keycode: u16 = 0x37; // Left Command
        let v_keycode: u16 = 0x09;   // V key

        // Create all events
        let cmd_down = CGEvent::new_keyboard_event(source.clone(), cmd_keycode, true);
        let v_down = CGEvent::new_keyboard_event(source.clone(), v_keycode, true);
        let v_up = CGEvent::new_keyboard_event(source.clone(), v_keycode, false);
        let cmd_up = CGEvent::new_keyboard_event(source, cmd_keycode, false);

        match (cmd_down, v_down, v_up, cmd_up) {
            (Ok(cd), Ok(vd), Ok(vu), Ok(cu)) => {
                // Set Command flag on all events
                cd.set_flags(CGEventFlags::CGEventFlagCommand);
                vd.set_flags(CGEventFlags::CGEventFlagCommand);
                vu.set_flags(CGEventFlags::CGEventFlagCommand);
                // No flags on cmd release

                // Post sequence: Cmd down → V down → V up → Cmd up
                cd.post(CGEventTapLocation::HID);
                vd.post(CGEventTapLocation::HID);
                vu.post(CGEventTapLocation::HID);
                // Small delay before releasing modifier (matches enigo behavior)
                std::thread::sleep(std::time::Duration::from_millis(50));
                cu.post(CGEventTapLocation::HID);

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
