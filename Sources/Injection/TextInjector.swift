import AppKit
import UserNotifications

@MainActor
class TextInjector {
    static let shared = TextInjector()

    private var previousApp: NSRunningApplication?

    private init() {
        // Request notification permission
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
    }

    func saveFocusedApp() {
        previousApp = NSWorkspace.shared.frontmostApplication
    }

    func clearPreviousApp() {
        previousApp = nil
    }

    func injectText(_ text: String) async {
        let autoPaste = UserDefaults.standard.bool(forKey: "autoPaste")

        // Save current clipboard contents for later restoration
        let savedClipboard = NSPasteboard.general.string(forType: .string)

        // Set clipboard BEFORE focus restore
        copyToClipboard(text)

        // Wait for clipboard to propagate system-wide
        try? await Task.sleep(nanoseconds: 100_000_000) // 100ms

        // Restore focus to the previous app
        if let app = previousApp {
            app.activate(options: [.activateIgnoringOtherApps])

            // Wait for app to become active
            var attempts = 0
            while NSWorkspace.shared.frontmostApplication?.processIdentifier != app.processIdentifier && attempts < 20 {
                try? await Task.sleep(nanoseconds: 25_000_000) // 25ms
                attempts += 1
            }
        }

        // Buffer after focus restoration
        try? await Task.sleep(nanoseconds: 100_000_000) // 100ms

        // Set clipboard AGAIN after focus restore (ensures clipboard is fresh)
        copyToClipboard(text)

        // Final buffer
        try? await Task.sleep(nanoseconds: 50_000_000) // 50ms

        if autoPaste {
            // Wait for hotkey modifiers to be released
            await waitForModifierRelease()
            // Simulate ⌘V via CGEvent
            if !simulatePaste() {
                showPasteNotification(preview: String(text.prefix(50)))
            } else {
                // Restore previous clipboard contents after paste completes
                try? await Task.sleep(nanoseconds: 100_000_000) // 100ms
                if let savedClipboard {
                    copyToClipboard(savedClipboard)
                } else {
                    NSPasteboard.general.clearContents()
                }
            }
        } else {
            // Manual mode: show notification for user to press ⌘V
            showPasteNotification(preview: String(text.prefix(50)))
        }

        previousApp = nil
    }

    // MARK: - Auto-Paste

    /// Wait until all physical modifier keys are released (critical for hold-to-record hotkeys)
    private func waitForModifierRelease() async {
        let relevantModifiers: NSEvent.ModifierFlags = [.shift, .control, .option, .command]
        let timeout: UInt64 = 1_000_000_000 // 1 second
        let start = DispatchTime.now().uptimeNanoseconds

        while !NSEvent.modifierFlags.intersection(relevantModifiers).isEmpty {
            let elapsed = DispatchTime.now().uptimeNanoseconds - start
            if elapsed >= timeout { break }
            try? await Task.sleep(nanoseconds: 10_000_000) // 10ms
        }

        // Small buffer after modifiers released
        try? await Task.sleep(nanoseconds: 50_000_000) // 50ms
    }

    /// Simulate ⌘V using CGEvent — works with Input Monitoring permission
    private func simulatePaste() -> Bool {
        let vKeyCode: CGKeyCode = 0x09 // Virtual key code for 'V'

        guard let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: vKeyCode, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: nil, virtualKey: vKeyCode, keyDown: false) else {
            return false
        }

        keyDown.flags = .maskCommand
        keyUp.flags = .maskCommand

        keyDown.post(tap: .cghidEventTap)
        keyUp.post(tap: .cghidEventTap)

        return true
    }

    // MARK: - Clipboard & Notification

    private func copyToClipboard(_ text: String) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.writeObjects([text as NSString])
    }

    private func showPasteNotification(preview: String) {
        let content = UNMutableNotificationContent()
        content.title = "Speech Ready"
        content.body = "Press ⌘V to paste: \(preview)\(preview.count >= 50 ? "..." : "")"
        content.sound = .default

        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )

        UNUserNotificationCenter.current().add(request)
    }
}
