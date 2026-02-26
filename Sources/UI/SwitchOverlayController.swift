import SwiftUI
import AppKit

@MainActor
class SwitchOverlayController {
    static let shared = SwitchOverlayController()

    private var window: NSWindow?
    private var hideTask: Task<Void, Never>?

    private init() {}

    func show(profileName: String, model: WhisperModel, language: TranscriptionLanguage) {
        hideTask?.cancel()

        let view = SwitchOverlayView(
            profileName: profileName,
            modelName: model.shortName,
            languageName: language.displayName
        )

        if window == nil {
            let w = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 260, height: 80),
                styleMask: [.borderless],
                backing: .buffered,
                defer: false
            )
            w.isOpaque = false
            w.backgroundColor = .clear
            w.level = .floating
            w.collectionBehavior = [.canJoinAllSpaces, .stationary]
            w.ignoresMouseEvents = true
            window = w
        }

        window?.contentView = NSHostingView(rootView: view)

        if let screen = NSScreen.screens.first(where: { NSMouseInRect(NSEvent.mouseLocation, $0.frame, false) }) ?? NSScreen.main {
            let screenFrame = screen.frame
            let x = screenFrame.midX - 130
            let y = screenFrame.midY - 40
            window?.setFrameOrigin(NSPoint(x: x, y: y))
        }

        window?.orderFront(nil)

        hideTask = Task {
            try? await Task.sleep(nanoseconds: 1_500_000_000)
            guard !Task.isCancelled else { return }
            self.window?.orderOut(nil)
        }
    }
}

private struct SwitchOverlayView: View {
    let profileName: String
    let modelName: String
    let languageName: String

    var body: some View {
        VStack(spacing: 6) {
            Text(profileName)
                .font(.system(size: 18, weight: .semibold))
                .foregroundColor(.white)
            Text("\(modelName) · \(languageName)")
                .font(.system(size: 13))
                .foregroundColor(.white.opacity(0.7))
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 16)
        .background(
            RoundedRectangle(cornerRadius: 14)
                .fill(Color.black.opacity(0.6))
                .shadow(color: .black.opacity(0.2), radius: 15, x: 0, y: 8)
        )
    }
}
