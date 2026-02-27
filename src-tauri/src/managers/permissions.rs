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

        // Also clean up old bundle ID if it differs (migration from com.speech.app)
        let old_id = "com.speech.app";
        if app_bundle_id != old_id {
            let _ = std::process::Command::new("/usr/bin/tccutil")
                .args(["reset", "Accessibility", old_id])
                .output();
            let _ = std::process::Command::new("/usr/bin/tccutil")
                .args(["reset", "ListenEvent", old_id])
                .output();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app_bundle_id;
    }
}
