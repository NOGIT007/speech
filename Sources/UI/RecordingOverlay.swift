import SwiftUI
import AppKit

class RecordingOverlayWindow: NSWindow {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 200, height: 80),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )

        self.isOpaque = false
        self.backgroundColor = .clear
        self.level = .floating
        self.collectionBehavior = [.canJoinAllSpaces, .stationary]
        self.ignoresMouseEvents = true

        let hostingView = NSHostingView(rootView: RecordingOverlayView())
        self.contentView = hostingView

        // Position at top center of screen
        if let screen = NSScreen.main {
            let screenFrame = screen.visibleFrame
            let x = screenFrame.midX - 100
            let y = screenFrame.maxY - 100
            self.setFrameOrigin(NSPoint(x: x, y: y))
        }
    }
}

struct RecordingOverlayView: View {
    @State private var pulse = false

    var body: some View {
        HStack(spacing: 12) {
            // Pulsing red circle
            Circle()
                .fill(Color.red)
                .frame(width: 16, height: 16)
                .scaleEffect(pulse ? 1.2 : 1.0)
                .animation(
                    .easeInOut(duration: 0.5).repeatForever(autoreverses: true),
                    value: pulse
                )

            Text("Recording...")
                .font(.system(size: 16, weight: .medium))
                .foregroundColor(.white)
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color.black.opacity(0.85))
        )
        .onAppear {
            pulse = true
        }
    }
}

@MainActor
class RecordingOverlayController {
    static let shared = RecordingOverlayController()

    private var window: RecordingOverlayWindow?

    private init() {}

    func show() {
        if window == nil {
            window = RecordingOverlayWindow()
        }
        window?.orderFront(nil)
    }

    func hide() {
        window?.orderOut(nil)
    }
}
