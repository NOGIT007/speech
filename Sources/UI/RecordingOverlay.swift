import SwiftUI
import AppKit

class RecordingOverlayWindow: NSWindow {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 180, height: 100),
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

        // Position at center of screen
        if let screen = NSScreen.main {
            let screenFrame = screen.frame
            let x = screenFrame.midX - 90
            let y = screenFrame.midY - 50
            self.setFrameOrigin(NSPoint(x: x, y: y))
        }
    }
}

struct RecordingOverlayView: View {
    var body: some View {
        VStack(spacing: 12) {
            // Audio waveform oscillator
            AudioWaveformView()
                .frame(height: 40)

            Text("Recording...")
                .font(.system(size: 14, weight: .medium))
                .foregroundColor(.white.opacity(0.9))
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 16)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.black.opacity(0.85))
                .shadow(color: .black.opacity(0.3), radius: 20, x: 0, y: 10)
        )
    }
}

struct AudioWaveformView: View {
    let barCount = 5
    @State private var animating = false

    var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<barCount, id: \.self) { index in
                WaveformBar(delay: Double(index) * 0.1, animating: animating)
            }
        }
        .onAppear {
            animating = true
        }
    }
}

struct WaveformBar: View {
    let delay: Double
    let animating: Bool

    @State private var height: CGFloat = 8

    var body: some View {
        RoundedRectangle(cornerRadius: 3)
            .fill(
                LinearGradient(
                    colors: [Color.blue, Color.cyan],
                    startPoint: .bottom,
                    endPoint: .top
                )
            )
            .frame(width: 6, height: height)
            .onAppear {
                if animating {
                    startAnimation()
                }
            }
            .onChange(of: animating) { newValue in
                if newValue {
                    startAnimation()
                }
            }
    }

    private func startAnimation() {
        withAnimation(
            .easeInOut(duration: 0.4)
            .repeatForever(autoreverses: true)
            .delay(delay)
        ) {
            height = CGFloat.random(in: 20...40)
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
