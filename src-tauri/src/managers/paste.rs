use anyhow::{Context, Result};
use std::sync::Mutex;

/// Manages text injection via clipboard + CGEvent Cmd+V simulation.
/// Port of TextInjector.swift:1-138.
pub struct PasteManager {
    /// PID of the app that was focused before recording started
    previous_app_pid: Mutex<Option<i32>>,
    /// Generation counter for safe async clearing (A6)
    generation: Mutex<u64>,
}

impl PasteManager {
    pub fn new() -> Self {
        Self {
            previous_app_pid: Mutex::new(None),
            generation: Mutex::new(0),
        }
    }

    /// Save the currently focused app's PID before recording.
    /// Matches TextInjector.swift:15-17.
    /// Skips saving if the frontmost app is Speech itself (accessory app edge case).
    /// Returns the new generation counter value.
    pub fn save_focused_app(&self) -> u64 {
        #[cfg(target_os = "macos")]
        {
            let pid = get_frontmost_app_pid();
            let my_pid = std::process::id() as i32;

            if let Some(p) = pid {
                if p == my_pid {
                    tracing::warn!("Frontmost app is Speech itself (PID {}), keeping previous saved PID", p);
                    return *self.generation.lock().unwrap();
                }
            }

            *self.previous_app_pid.lock().unwrap() = pid;
            let mut gen = self.generation.lock().unwrap();
            *gen += 1;
            if let Some(pid) = pid {
                tracing::info!("Saved focused app PID: {} (gen={})", pid, *gen);
            } else {
                tracing::warn!("No frontmost app found to save (gen={})", *gen);
            }
            *gen
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut gen = self.generation.lock().unwrap();
            *gen += 1;
            *gen
        }
    }

    /// Clear the saved previous app reference.
    pub fn clear_previous_app(&self) {
        *self.previous_app_pid.lock().unwrap() = None;
    }

    /// Clear the saved previous app reference only if the generation matches.
    /// Prevents a stale async clear from wiping a newer save.
    pub fn clear_previous_app_gen(&self, expected_gen: u64) {
        let gen = self.generation.lock().unwrap();
        if *gen == expected_gen {
            *self.previous_app_pid.lock().unwrap() = None;
        } else {
            tracing::debug!(
                "Skipping clear: expected gen={}, current gen={}",
                expected_gen, *gen
            );
        }
    }

    /// Get the saved previous app PID (used to pass to standalone inject function).
    pub fn get_previous_app_pid(&self) -> Option<i32> {
        *self.previous_app_pid.lock().unwrap()
    }

    /// Get the current generation counter.
    pub fn get_generation(&self) -> u64 {
        *self.generation.lock().unwrap()
    }
}

/// Standalone async text injection that takes a pre-extracted PID.
/// This avoids holding a MutexGuard across await points.
/// Matches TextInjector.swift:23-77.
pub async fn inject_text_with_pid(text: &str, auto_paste: bool, prev_pid: Option<i32>) -> Result<()> {
    let fn_start = std::time::Instant::now();

    tracing::info!(
        "inject_text_with_pid: auto_paste={}, prev_pid={:?}, text_len={}",
        auto_paste, prev_pid, text.len()
    );

    // Set clipboard to transcribed text (kept for manual Cmd+V fallback)
    set_clipboard_text(text)?;
    tracing::debug!("Clipboard set with transcribed text");

    // Wait for clipboard to propagate (reduced from 100ms to 30ms)
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    // Structured 2-attempt focus restoration
    let focus_start = std::time::Instant::now();
    if let Some(pid) = prev_pid {
        let mut focus_restored = false;
        let attempts: [(u64, &str); 2] = [(500, "first"), (300, "second")];

        for (timeout_ms, label) in &attempts {
            tracing::info!("Focus restore {} attempt: activating PID {}", label, pid);
            activate_app_by_pid(pid);

            let attempt_start = std::time::Instant::now();
            let timeout = std::time::Duration::from_millis(*timeout_ms);

            loop {
                if get_frontmost_app_pid() == Some(pid) {
                    tracing::info!(
                        "Focus restored to PID {} after {:?} ({} attempt)",
                        pid, attempt_start.elapsed(), label
                    );
                    focus_restored = true;
                    break;
                }
                if attempt_start.elapsed() >= timeout {
                    tracing::warn!(
                        "Focus restore timeout after {}ms ({} attempt)",
                        timeout_ms, label
                    );
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }

            if focus_restored {
                break;
            }
        }
        tracing::info!("Focus restoration took {:?}", focus_start.elapsed());

        if !focus_restored {
            let current = get_frontmost_app_pid();
            tracing::warn!(
                "Failed to restore focus to PID {} after 2 attempts. Current frontmost: {:?}. Text remains on clipboard for manual Cmd+V.",
                pid, current
            );
            tracing::info!("inject_text_with_pid total elapsed: {:?}", fn_start.elapsed());
            return Err(anyhow::anyhow!(
                "Focus restoration failed: target PID {} not frontmost after 2 attempts (current: {:?})",
                pid, current
            ));
        }
    } else {
        tracing::warn!("No previous app PID saved, cannot restore focus");
        tracing::info!("Focus restoration section took {:?}", focus_start.elapsed());
    }

    // Buffer after focus restoration (reduced from 100ms to 30ms)
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    // Clipboard verification: only re-set if contents don't match
    let needs_reclip = get_clipboard_text().as_deref() != Some(text);
    if needs_reclip {
        set_clipboard_text(text)?;
        tracing::debug!("Clipboard re-set after focus restore (contents didn't match)");
    } else {
        tracing::debug!("Clipboard contents verified, skipping re-set");
    }

    // Final buffer (reduced from 50ms to 20ms)
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

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
                // Reduced from 150ms to 75ms
                tokio::time::sleep(std::time::Duration::from_millis(75)).await;
            }
        }

        // Simulate Cmd+V via spawn_blocking (uses thread::sleep + CGEvent calls)
        let paste_ok = tokio::task::spawn_blocking(simulate_paste)
            .await
            .unwrap_or(false);
        if paste_ok {
            tracing::info!("Cmd+V paste simulated successfully");
        } else {
            tracing::error!("Failed to simulate Cmd+V paste");
        }
        // Transcribed text stays on clipboard so user can manually Cmd+V elsewhere
    } else {
        tracing::info!("auto_paste disabled, text is on clipboard only");
    }

    tracing::info!("inject_text_with_pid total elapsed: {:?}", fn_start.elapsed());
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

        // Small buffer after modifiers released (reduced from 50ms to 25ms)
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
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
