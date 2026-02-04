import SwiftUI
import AppKit

enum OverlayMode {
    case recording
    case processing
    case ready
}

class RecordingOverlayWindow: NSWindow {
    private var hostingView: NSHostingView<RecordingOverlayView>?

    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 380, height: 220),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )

        self.isOpaque = false
        self.backgroundColor = .clear
        self.level = .floating
        self.collectionBehavior = [.canJoinAllSpaces, .stationary]
        self.ignoresMouseEvents = true

        let view = RecordingOverlayView(mode: .recording)
        hostingView = NSHostingView(rootView: view)
        self.contentView = hostingView

        // Position at center of screen
        if let screen = NSScreen.main {
            let screenFrame = screen.frame
            let x = screenFrame.midX - 190
            let y = screenFrame.midY - 110
            self.setFrameOrigin(NSPoint(x: x, y: y))
        }
    }

    func setMode(_ mode: OverlayMode) {
        hostingView?.rootView = RecordingOverlayView(mode: mode)
    }
}

struct RecordingOverlayView: View {
    let mode: OverlayMode

    var body: some View {
        VStack(spacing: 14) {
            switch mode {
            case .recording:
                AudioWaveformView()
                    .frame(height: 85)

                Text("Recording...")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(.white.opacity(0.9))

                Text("Release to transcribe")
                    .font(.system(size: 12, weight: .regular))
                    .foregroundColor(.white.opacity(0.6))

            case .processing:
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    .scaleEffect(2)
                    .frame(height: 85)

                Text("Processing...")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(.white.opacity(0.9))

                Text("Will paste when ready")
                    .font(.system(size: 12, weight: .regular))
                    .foregroundColor(.white.opacity(0.6))

            case .ready:
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 50))
                    .foregroundColor(.green)
                    .frame(height: 85)

                Text("Ready!")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(.white.opacity(0.9))

                Text("Press ⌘V to paste")
                    .font(.system(size: 12, weight: .regular))
                    .foregroundColor(.white.opacity(0.6))
            }
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
        .background(
            RoundedRectangle(cornerRadius: 18)
                .fill(Color.black.opacity(0.6))
                .shadow(color: .black.opacity(0.2), radius: 20, x: 0, y: 10)
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
            .frame(width: 12, height: height)
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
            height = CGFloat.random(in: 35...80)
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
        window?.setMode(.recording)
        window?.orderFront(nil)
    }

    func showProcessing() {
        window?.setMode(.processing)
    }

    func showReadyThenHide() {
        window?.setMode(.ready)
        // Hide after 1 second
        Task {
            try? await Task.sleep(nanoseconds: 1_000_000_000)
            await MainActor.run {
                self.hide()
            }
        }
    }

    func hide() {
        window?.orderOut(nil)
    }
}
