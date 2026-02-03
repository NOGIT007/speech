import AppKit
import AVFoundation

class AppDelegate: NSObject, NSApplicationDelegate {
    private var hotkeyManager: HotkeyManager?
    private var permissionsManager: PermissionsManager?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Initialize permissions manager
        permissionsManager = PermissionsManager()

        // Check and request permissions
        Task {
            await checkPermissions()
            await setupHotkey()
            await setupWhisper()
        }
    }

    private func checkPermissions() async {
        guard let manager = permissionsManager else { return }

        // Check microphone permission
        if !manager.hasMicrophonePermission {
            let granted = await manager.requestMicrophonePermission()
            if !granted {
                await MainActor.run {
                    AppState.shared.showPermissionAlert(for: .microphone)
                }
            }
        }

        // Check accessibility permission
        if !manager.hasAccessibilityPermission {
            await MainActor.run {
                AppState.shared.showPermissionAlert(for: .accessibility)
            }
        }
    }

    private func setupHotkey() async {
        await MainActor.run {
            hotkeyManager = HotkeyManager()
            hotkeyManager?.registerHotkey()
        }
    }

    private func setupWhisper() async {
        // Download model if needed, then initialize
        await WhisperService.shared.initialize()
    }

    func applicationWillTerminate(_ notification: Notification) {
        hotkeyManager?.unregisterHotkey()
    }
}

// MARK: - Permissions Manager

class PermissionsManager {
    enum PermissionType {
        case microphone
        case accessibility
        case inputMonitoring
    }

    var hasMicrophonePermission: Bool {
        AVCaptureDevice.authorizationStatus(for: .audio) == .authorized
    }

    var hasAccessibilityPermission: Bool {
        AXIsProcessTrusted()
    }

    var hasInputMonitoringPermission: Bool {
        // Input monitoring is implicitly checked via CGEvent tap creation
        let eventTap = CGEvent.tapCreate(
            tap: .cghidEventTap,
            place: .headInsertEventTap,
            options: .listenOnly,
            eventsOfInterest: CGEventMask(1 << CGEventType.keyDown.rawValue),
            callback: { _, _, event, _ in Unmanaged.passUnretained(event) },
            userInfo: nil
        )
        let hasPermission = eventTap != nil
        return hasPermission
    }

    func requestMicrophonePermission() async -> Bool {
        await withCheckedContinuation { continuation in
            AVCaptureDevice.requestAccess(for: .audio) { granted in
                continuation.resume(returning: granted)
            }
        }
    }

    func openAccessibilitySettings() {
        let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
        NSWorkspace.shared.open(url)
    }

    func openMicrophoneSettings() {
        let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")!
        NSWorkspace.shared.open(url)
    }

    func openInputMonitoringSettings() {
        let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")!
        NSWorkspace.shared.open(url)
    }
}
