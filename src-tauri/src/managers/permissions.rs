use serde::{Deserialize, Serialize};

/// Permission status for the three required macOS permissions.
/// Matches AppDelegate.swift:62-115 (PermissionsManager).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub microphone: bool,
    pub accessibility: bool,
    pub input_monitoring: bool,
}

/// Check all permission statuses.
pub fn check_permissions() -> PermissionStatus {
    PermissionStatus {
        microphone: check_microphone(),
        accessibility: check_accessibility(),
        input_monitoring: check_input_monitoring(),
    }
}

/// Check microphone permission via AVCaptureDevice authorizationStatus.
/// Matches AppDelegate.swift:70-71.
fn check_microphone() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc::runtime::{Class, Object};
        use objc::*;
        unsafe {
            // AVCaptureDevice.authorizationStatus(for: .audio)
            // AVAuthorizationStatus.authorized == 3
            let class = Class::get("AVCaptureDevice").unwrap();
            // AVMediaTypeAudio = "soun"
            let media_type: *mut Object = msg_send![
                Class::get("NSString").unwrap(),
                stringWithUTF8String: b"soun\0".as_ptr()
            ];
            let status: i64 = msg_send![class, authorizationStatusForMediaType: media_type];
            status == 3 // .authorized
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Check accessibility permission via AXIsProcessTrusted().
/// Matches AppDelegate.swift:74-75.
fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Check input monitoring permission via CGEvent tap creation test.
/// Matches AppDelegate.swift:78-89.
fn check_input_monitoring() -> bool {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::{CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType};
        // Try to create a passive event tap - succeeds only if input monitoring is granted
        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown],
            |_proxy, _event_type, event| Some(event.clone()),
        );
        tap.is_ok()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Open the accessibility permission prompt dialog.
/// Matches AppDelegate.swift:100-104.
pub fn prompt_accessibility() {
    #[cfg(target_os = "macos")]
    {
        use core_foundation::base::TCFType;
        use core_foundation::boolean::CFBoolean;
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::string::CFString;

        extern "C" {
            fn AXIsProcessTrustedWithOptions(options: core_foundation::base::CFTypeRef) -> bool;
        }

        let key = unsafe {
            extern "C" {
                static kAXTrustedCheckOptionPrompt: core_foundation::string::CFStringRef;
            }
            CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt)
        };

        let dict = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
        unsafe {
            AXIsProcessTrustedWithOptions(dict.as_CFTypeRef());
        }
    }
}

/// Open System Settings to the Microphone privacy pane.
/// Matches AppDelegate.swift:106-108.
pub fn open_microphone_settings() {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}

/// Open System Settings to the Input Monitoring privacy pane.
/// Matches AppDelegate.swift:111-113.
pub fn open_input_monitoring_settings() {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent";
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}

/// Reset TCC permissions and relaunch the app.
/// Matches SettingsView.swift:363-386.
pub fn reset_permissions_and_relaunch(app_bundle_id: &str) {
    #[cfg(target_os = "macos")]
    {
        // Reset Accessibility
        let _ = std::process::Command::new("/usr/bin/tccutil")
            .args(["reset", "Accessibility", app_bundle_id])
            .output();

        // Reset Input Monitoring (ListenEvent)
        let _ = std::process::Command::new("/usr/bin/tccutil")
            .args(["reset", "ListenEvent", app_bundle_id])
            .output();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app_bundle_id;
    }
}
