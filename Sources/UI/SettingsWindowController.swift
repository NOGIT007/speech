import AppKit
import SwiftUI

@MainActor
class SettingsWindowController: NSObject, NSWindowDelegate {
    static let shared = SettingsWindowController()

    private var window: NSWindow?

    private override init() {}

    func show() {
        if let window = window {
            // Re-show existing window
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let settingsView = SettingsView()
            .environmentObject(AppState.shared)

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 450, height: 420),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Speech Settings"
        window.contentView = NSHostingView(rootView: settingsView)
        window.center()
        window.isReleasedWhenClosed = false
        window.delegate = self

        self.window = window
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func windowWillClose(_ notification: Notification) {
        window = nil
    }
}
