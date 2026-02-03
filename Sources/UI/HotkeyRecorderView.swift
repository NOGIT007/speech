import SwiftUI
import Carbon.HIToolbox

struct HotkeyRecorderView: View {
    @Binding var config: HotkeyConfig
    @State private var isRecording = false
    @State private var eventMonitor: Any?

    var body: some View {
        HStack {
            Text("Dictation shortcut")
            Spacer()

            Button(action: { startRecording() }) {
                Text(isRecording ? "Press shortcut..." : config.displayString)
                    .frame(minWidth: 100)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
            }
            .buttonStyle(.bordered)
            .foregroundColor(isRecording ? .accentColor : .primary)

            if config != .default {
                Button(action: { config = .default }) {
                    Image(systemName: "arrow.counterclockwise")
                }
                .buttonStyle(.borderless)
                .help("Reset to default (⌥Space)")
            }
        }
    }

    private func startRecording() {
        isRecording = true

        eventMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            let modifiers = event.modifierFlags.intersection([.control, .option, .shift, .command])

            // Escape cancels
            if event.keyCode == UInt16(kVK_Escape) {
                stopRecording()
                return nil
            }

            // Require at least one modifier
            guard !modifiers.isEmpty else {
                return nil
            }

            // Don't allow just modifier keys
            let modifierKeyCodes: [Int] = [
                kVK_Shift, kVK_RightShift,
                kVK_Control, kVK_RightControl,
                kVK_Option, kVK_RightOption,
                kVK_Command, kVK_RightCommand
            ]
            if modifierKeyCodes.contains(Int(event.keyCode)) {
                return nil
            }

            config = HotkeyConfig(keyCode: UInt32(event.keyCode), modifiers: modifiers)
            stopRecording()
            return nil
        }
    }

    private func stopRecording() {
        isRecording = false
        if let monitor = eventMonitor {
            NSEvent.removeMonitor(monitor)
            eventMonitor = nil
        }
    }
}
