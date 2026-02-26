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

        // Position at center of screen containing cursor
        if let screen = NSScreen.screens.first(where: { NSMouseInRect(NSEvent.mouseLocation, $0.frame, false) }) ?? NSScreen.main {
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
    @ObservedObject private var appState = AppState.shared
    private var autoPaste: Bool {
        UserDefaults.standard.bool(forKey: "autoPaste")
    }

    var body: some View {
        VStack(spacing: 14) {
            switch mode {
            case .recording:
                AudioWaveformView(audioLevel: appState.audioLevel)
                    .frame(height: 85)

                Text("Recording...")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(.white.opacity(0.9))

                Text("Release to transcribe · Esc to cancel")
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

                Text(autoPaste ? "Will auto-paste when ready" : "Will copy to clipboard")
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

                Text(autoPaste ? "Pasted!" : "Press ⌘V to paste")
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
    let audioLevel: Float
    let barCount = 5

    var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<barCount, id: \.self) { index in
                WaveformBar(audioLevel: audioLevel, barIndex: index, barCount: barCount)
            }
        }
    }
}

struct WaveformBar: View {
    let audioLevel: Float
    let barIndex: Int
    let barCount: Int

    private var barHeight: CGFloat {
        let minHeight: CGFloat = 8
        let maxHeight: CGFloat = 80
        // Center bars are taller, edges shorter for visual interest
        let center = Float(barCount - 1) / 2.0
        let centerDistance = abs(Float(barIndex) - center) / max(center, 1)
        let variation = 1.0 - centerDistance * 0.3
        let level = CGFloat(audioLevel) * CGFloat(variation)
        return minHeight + (maxHeight - minHeight) * level
    }

    var body: some View {
        RoundedRectangle(cornerRadius: 3)
            .fill(
                LinearGradient(
                    colors: [Color.blue, Color.cyan],
                    startPoint: .bottom,
                    endPoint: .top
                )
            )
            .frame(width: 12, height: barHeight)
            .animation(.easeOut(duration: 0.08), value: audioLevel)
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
        let autoPaste = UserDefaults.standard.bool(forKey: "autoPaste")
        let delay: UInt64 = autoPaste ? 500_000_000 : 1_000_000_000
        Task {
            try? await Task.sleep(nanoseconds: delay)
            await MainActor.run {
                self.hide()
            }
        }
    }

    func hide() {
        window?.orderOut(nil)
    }
}
