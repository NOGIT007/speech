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

    func injectText(_ text: String) async {
        // Set text to clipboard
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(text, forType: .string)

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

        // Show notification to paste
        showPasteNotification(preview: String(text.prefix(50)))

        previousApp = nil
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
