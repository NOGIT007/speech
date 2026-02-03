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

    func injectText(_ text: String, autoPaste: Bool = true) async {
        // Always copy to clipboard first
        copyToClipboard(text)

        // Restore focus to the previous app
        if let app = previousApp {
            app.activate(options: [.activateIgnoringOtherApps])

            // Wait for app to become active
            var attempts = 0
            while NSWorkspace.shared.frontmostApplication?.processIdentifier != app.processIdentifier && attempts < 20 {
                try? await Task.sleep(nanoseconds: 25_000_000) // 25ms
                attempts += 1
            }

            // Additional delay for focus stability and modifier key release
            try? await Task.sleep(nanoseconds: 500_000_000) // 500ms
        }

        // Auto-paste using Cmd+V
        if autoPaste && AXIsProcessTrusted() {
            let success = simulatePaste()
            if !success {
                showPasteNotification(preview: String(text.prefix(50)))
            }
        } else {
            showPasteNotification(preview: String(text.prefix(50)))
        }

        previousApp = nil
    }

    private func simulatePaste() -> Bool {
        // Try AppleScript up to 3 times with small delays
        for _ in 1...3 {
            if pasteViaAppleScript() {
                return true
            }
            usleep(100_000) // 100ms between retries
        }
        // Final fallback to CGEvent
        return pasteViaCGEvent()
    }

    private func pasteViaAppleScript() -> Bool {
        let script = """
        tell application "System Events"
            keystroke "v" using command down
        end tell
        """

        guard let appleScript = NSAppleScript(source: script) else { return false }
        var error: NSDictionary?
        appleScript.executeAndReturnError(&error)
        return error == nil
    }

    private func pasteViaCGEvent() -> Bool {
        let source = CGEventSource(stateID: .hidSystemState)

        guard let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 9, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: source, virtualKey: 9, keyDown: false) else {
            return false
        }

        keyDown.flags = .maskCommand
        keyUp.flags = .maskCommand

        keyDown.post(tap: .cgSessionEventTap)
        usleep(50000) // 50ms
        keyUp.post(tap: .cgSessionEventTap)

        return true
    }

    private func copyToClipboard(_ text: String) {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(text, forType: .string)
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
