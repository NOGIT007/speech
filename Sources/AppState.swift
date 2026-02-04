import SwiftUI
import Combine
import HotKey
import Carbon.HIToolbox

@MainActor
class AppState: ObservableObject {
    static let shared = AppState()

    @Published var isRecording = false
    @Published var isTranscribing = false
    @Published var lastTranscription: String?
    @Published var transcriptionHistory: [TranscriptionItem] = []
    @Published var errorMessage: String?
    @Published var modelStatus: ModelStatus = .notDownloaded
    @AppStorage("selectedModel") var selectedModel: WhisperModel = .small
    @Published var loadedModel: WhisperModel?
    @AppStorage("selectedLanguage") var selectedLanguage: TranscriptionLanguage = .english
    @Published var hotkeyConfig: HotkeyConfig {
        didSet {
            hotkeyConfig.save()
            NotificationCenter.default.post(name: .hotkeyConfigChanged, object: nil)
        }
    }

    enum ModelStatus: Equatable {
        case notDownloaded
        case downloading(progress: Double)
        case ready
        case error(String)
    }

    private init() {
        self.hotkeyConfig = HotkeyConfig.load()
    }

    func showPermissionAlert(for permission: PermissionsManager.PermissionType) {
        let alert = NSAlert()
        alert.alertStyle = .warning

        switch permission {
        case .microphone:
            alert.messageText = "Microphone Access Required"
            alert.informativeText = "Speech needs microphone access to record your voice. Please grant access in System Settings."
            alert.addButton(withTitle: "Open Settings")
            alert.addButton(withTitle: "Later")

            if alert.runModal() == .alertFirstButtonReturn {
                PermissionsManager().openMicrophoneSettings()
            }

        case .accessibility:
            alert.messageText = "Accessibility Access Required"
            alert.informativeText = "Speech needs accessibility access to inject text into applications. Please grant access in System Settings."
            alert.addButton(withTitle: "Open Settings")
            alert.addButton(withTitle: "Later")

            if alert.runModal() == .alertFirstButtonReturn {
                PermissionsManager().openAccessibilitySettings()
            }

        case .inputMonitoring:
            alert.messageText = "Input Monitoring Required"
            alert.informativeText = "Speech needs input monitoring to detect global hotkeys."
            alert.addButton(withTitle: "OK")
            alert.runModal()
        }
    }

    func startRecording() {
        guard !isRecording else { return }

        // Save which app has focus BEFORE we start (so we can paste back to it)
        TextInjector.shared.saveFocusedApp()

        isRecording = true
        errorMessage = nil

        // Show visual overlay
        RecordingOverlayController.shared.show()

        Task {
            do {
                try await AudioRecorder.shared.startRecording()
            } catch {
                await MainActor.run {
                    self.isRecording = false
                    RecordingOverlayController.shared.hide()
                    self.errorMessage = "Failed to start recording: \(error.localizedDescription)"
                }
            }
        }
    }

    func stopRecordingAndTranscribe() {
        guard isRecording else { return }
        isRecording = false
        isTranscribing = true

        // Hide recording overlay, show transcribing
        RecordingOverlayController.shared.hide()

        Task {
            do {
                let audioURL = try await AudioRecorder.shared.stopRecording()
                let transcription = try await WhisperService.shared.transcribe(audioURL: audioURL, language: selectedLanguage)

                await MainActor.run {
                    self.lastTranscription = transcription
                    self.isTranscribing = false
                    // Add to history (keep last 5)
                    if !transcription.isEmpty {
                        self.transcriptionHistory.insert(
                            TranscriptionItem(text: transcription, timestamp: Date()),
                            at: 0
                        )
                        if self.transcriptionHistory.count > 5 {
                            self.transcriptionHistory.removeLast()
                        }
                    }
                }

                // Inject text at cursor position
                if !transcription.isEmpty {
                    await TextInjector.shared.injectText(transcription)
                }

                // Clean up audio file
                try? FileManager.default.removeItem(at: audioURL)

            } catch {
                await MainActor.run {
                    self.isTranscribing = false
                    self.errorMessage = "Transcription failed: \(error.localizedDescription)"
                }
            }
        }
    }
}

enum WhisperModel: String, CaseIterable, Identifiable {
    case tiny = "tiny"
    case base = "base"
    case small = "small"
    case mediumEn = "medium.en"

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .tiny: return "Tiny (75 MB) - Fastest"
        case .base: return "Base (142 MB) - Balanced"
        case .small: return "Small (466 MB) - Accurate"
        case .mediumEn: return "Medium English (1.5 GB) - High Accuracy"
        }
    }

    var fileName: String {
        "ggml-\(rawValue).bin"
    }

    var downloadURL: URL {
        URL(string: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/\(fileName)")!
    }

    var fileSize: Int64 {
        switch self {
        case .tiny: return 75_000_000
        case .base: return 142_000_000
        case .small: return 466_000_000
        case .mediumEn: return 1_500_000_000
        }
    }
}

// MARK: - Hotkey Configuration

struct HotkeyConfig: Equatable {
    var keyCode: UInt32
    var modifiers: NSEvent.ModifierFlags

    static let `default` = HotkeyConfig(
        keyCode: UInt32(kVK_Space),
        modifiers: .option
    )

    var displayString: String {
        var parts: [String] = []
        if modifiers.contains(.control) { parts.append("⌃") }
        if modifiers.contains(.option) { parts.append("⌥") }
        if modifiers.contains(.shift) { parts.append("⇧") }
        if modifiers.contains(.command) { parts.append("⌘") }
        parts.append(keyName)
        return parts.joined()
    }

    var keyName: String {
        switch Int(keyCode) {
        case kVK_Space: return "Space"
        case kVK_Return: return "↩"
        case kVK_Tab: return "⇥"
        case kVK_Delete: return "⌫"
        case kVK_Escape: return "⎋"
        case kVK_F1: return "F1"
        case kVK_F2: return "F2"
        case kVK_F3: return "F3"
        case kVK_F4: return "F4"
        case kVK_F5: return "F5"
        case kVK_F6: return "F6"
        case kVK_F7: return "F7"
        case kVK_F8: return "F8"
        case kVK_F9: return "F9"
        case kVK_F10: return "F10"
        case kVK_F11: return "F11"
        case kVK_F12: return "F12"
        default:
            if let chars = keyCodeToString(keyCode) {
                return chars.uppercased()
            }
            return "Key \(keyCode)"
        }
    }

    private func keyCodeToString(_ keyCode: UInt32) -> String? {
        let source = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
        guard let layoutData = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
            return nil
        }
        let dataRef = unsafeBitCast(layoutData, to: CFData.self)
        let keyboardLayout = unsafeBitCast(CFDataGetBytePtr(dataRef), to: UnsafePointer<UCKeyboardLayout>.self)

        var deadKeyState: UInt32 = 0
        var chars = [UniChar](repeating: 0, count: 4)
        var length: Int = 0

        let status = UCKeyTranslate(
            keyboardLayout,
            UInt16(keyCode),
            UInt16(kUCKeyActionDown),
            0,
            UInt32(LMGetKbdType()),
            UInt32(kUCKeyTranslateNoDeadKeysBit),
            &deadKeyState,
            chars.count,
            &length,
            &chars
        )

        guard status == noErr, length > 0 else { return nil }
        return String(utf16CodeUnits: chars, count: length)
    }

    func save() {
        UserDefaults.standard.set(keyCode, forKey: "hotkeyKeyCode")
        UserDefaults.standard.set(modifiers.rawValue, forKey: "hotkeyModifiers")
    }

    static func load() -> HotkeyConfig {
        let keyCode = UserDefaults.standard.object(forKey: "hotkeyKeyCode") as? UInt32 ?? UInt32(kVK_Space)
        let modifiersRaw = UserDefaults.standard.object(forKey: "hotkeyModifiers") as? UInt ?? NSEvent.ModifierFlags.option.rawValue
        return HotkeyConfig(keyCode: keyCode, modifiers: NSEvent.ModifierFlags(rawValue: modifiersRaw))
    }

    var hotKeyKey: Key? {
        Key(carbonKeyCode: keyCode)
    }

    var hotKeyModifiers: NSEvent.ModifierFlags {
        modifiers
    }
}

extension Notification.Name {
    static let hotkeyConfigChanged = Notification.Name("hotkeyConfigChanged")
}

// MARK: - Transcription History

struct TranscriptionItem: Identifiable {
    let id = UUID()
    let text: String
    let timestamp: Date

    var timeAgo: String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: timestamp, relativeTo: Date())
    }

    var preview: String {
        if text.count <= 50 {
            return text
        }
        return String(text.prefix(47)) + "..."
    }
}

// MARK: - Transcription Language

enum TranscriptionLanguage: String, CaseIterable, Identifiable {
    case auto = "auto"
    case english = "en"
    case spanish = "es"
    case french = "fr"
    case german = "de"
    case italian = "it"
    case portuguese = "pt"
    case dutch = "nl"
    case polish = "pl"
    case russian = "ru"
    case japanese = "ja"
    case chinese = "zh"
    case korean = "ko"

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .auto: return "Auto-detect"
        case .english: return "English"
        case .spanish: return "Spanish"
        case .french: return "French"
        case .german: return "German"
        case .italian: return "Italian"
        case .portuguese: return "Portuguese"
        case .dutch: return "Dutch"
        case .polish: return "Polish"
        case .russian: return "Russian"
        case .japanese: return "Japanese"
        case .chinese: return "Chinese"
        case .korean: return "Korean"
        }
    }
}
