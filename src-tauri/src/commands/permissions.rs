use crate::managers::permissions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub microphone: bool,
    pub accessibility: bool,
    pub input_monitoring: bool,
}

/// Check all permission statuses using native macOS APIs (via dlsym for reliability).
#[tauri::command]
pub fn check_permissions() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        let mic = check_microphone();
        let ax = check_accessibility();
        let im = check_input_monitoring();
        tracing::info!(
            "Permission check: microphone={}, accessibility={}, input_monitoring={}",
            mic, ax, im
        );
        PermissionStatus {
            microphone: mic,
            accessibility: ax,
            input_monitoring: im,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        PermissionStatus {
            microphone: false,
            accessibility: false,
            input_monitoring: false,
        }
    }
}

#[cfg(target_os = "macos")]
fn check_accessibility() -> bool {
    // Use dlsym to dynamically load AXIsProcessTrusted (returns Boolean = u8)
    unsafe {
        let handle = libc::dlopen(
            b"/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices\0"
                .as_ptr() as *const _,
            libc::RTLD_LAZY,
        );
        if handle.is_null() {
            tracing::warn!("Failed to load ApplicationServices framework");
            return false;
        }
        let sym = libc::dlsym(handle, b"AXIsProcessTrusted\0".as_ptr() as *const _);
        if sym.is_null() {
            tracing::warn!("Failed to find AXIsProcessTrusted symbol");
            return false;
        }
        let func: unsafe extern "C" fn() -> u8 = std::mem::transmute(sym);
        let result = func();
        tracing::info!("AXIsProcessTrusted() raw = {}", result);
        result != 0
    }
}

#[cfg(target_os = "macos")]
fn check_input_monitoring() -> bool {
    // Use dlsym to dynamically load IOHIDCheckAccess (returns uint32_t, 0 = granted)
    unsafe {
        let handle = libc::dlopen(
            b"/System/Library/Frameworks/IOKit.framework/IOKit\0".as_ptr() as *const _,
            libc::RTLD_LAZY,
        );
        if handle.is_null() {
            tracing::warn!("Failed to load IOKit framework");
            return false;
        }
        let sym = libc::dlsym(handle, b"IOHIDCheckAccess\0".as_ptr() as *const _);
        if sym.is_null() {
            tracing::warn!("Failed to find IOHIDCheckAccess symbol");
            return false;
        }
        let func: unsafe extern "C" fn(u32) -> u32 = std::mem::transmute(sym);
        let result = func(1);
        tracing::info!("IOHIDCheckAccess(1) raw = {} (0=granted)", result);
        result == 0
    }
}

#[cfg(target_os = "macos")]
fn check_microphone() -> bool {
    use objc::runtime::Class;
    match Class::get("AVCaptureDevice") {
        Some(class) => {
            use objc::*;
            let ns_string_class = Class::get("NSString").unwrap();
            unsafe {
                let media_type: *mut objc::runtime::Object =
                    msg_send![ns_string_class, stringWithUTF8String: b"soun\0".as_ptr()];
                let status: i64 =
                    msg_send![class, authorizationStatusForMediaType: media_type];
                tracing::info!("AVCaptureDevice authorizationStatus raw = {} (3=authorized)", status);
                status == 3 // .authorized
            }
        }
        None => {
            tracing::warn!("AVCaptureDevice class not found");
            false
        }
    }
}

/// Reset TCC permissions and relaunch the app.
#[tauri::command]
pub fn reset_permissions(app: tauri::AppHandle) {
    let bundle_id = app.config().identifier.clone();

    permissions::reset_permissions_and_relaunch(&bundle_id);

    #[cfg(target_os = "macos")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let app_bundle = exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent());

            if let Some(app_path) = app_bundle {
                let _ = std::process::Command::new("open").arg(app_path).spawn();
            }
        }

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::process::exit(0);
        });
    }
}
