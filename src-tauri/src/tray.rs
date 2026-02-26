use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Icon states matching the 3 phases of the app
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayIconState {
    Idle,
    Recording,
    Processing,
}

/// Set up the system tray icon with click handler to toggle the menu bar panel.
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // Build a right-click context menu with Settings and Quit
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Speech", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("no default window icon set in tauri.conf.json");

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Speech")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::commands::settings::open_settings(app).await {
                        tracing::error!("Failed to open settings: {}", e);
                    }
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_panel(app);
            }
        })
        .build(app)?;

    Ok(())
}

/// Toggle the main menu bar panel window visibility.
/// Positions it below the tray icon area.
fn toggle_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // Position the window near the menu bar area
            // The exact tray icon position isn't easily accessible,
            // so we position it at the mouse cursor location (near the tray click)
            if let Ok(pos) = window.cursor_position() {
                let width = 320.0;
                let x = (pos.x - width / 2.0).max(0.0);
                let y = 0.0; // Top of screen, below menu bar
                let _ = window.set_position(tauri::Position::Logical(
                    tauri::LogicalPosition::new(x, y),
                ));
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
