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
    @Published var profiles: [ModelProfile] = [] {
        didSet { saveProfiles() }
    }
    @Published var activeProfileIndex: Int = 0 {
        didSet { UserDefaults.standard.set(activeProfileIndex, forKey: "activeProfileIndex") }
    }
    @Published var switchHotkeyConfig: HotkeyConfig {
        didSet {
            switchHotkeyConfig.save(prefix: "switch")
            NotificationCenter.default.post(name: .switchHotkeyConfigChanged, object: nil)
        }
    }
    @AppStorage("switchHotkeyEnabled") var switchHotkeyEnabled: Bool = false {
        didSet {
            NotificationCenter.default.post(name: .switchHotkeyConfigChanged, object: nil)
        }
    }
    private var recordingStartTask: Task<Void, Never>?

    var activeProfile: ModelProfile? {
        guard !profiles.isEmpty else { return nil }
        let index = min(activeProfileIndex, profiles.count - 1)
        return profiles[index]
    }

    enum ModelStatus: Equatable {
        case notDownloaded
        case downloading(progress: Double)
        case ready
        case error(String)
    }

    private init() {
        self.hotkeyConfig = HotkeyConfig.load()
        self.switchHotkeyConfig = HotkeyConfig.load(prefix: "switch")
        self.profiles = AppState.loadProfiles()
        self.activeProfileIndex = UserDefaults.standard.integer(forKey: "activeProfileIndex")
        migrateModelSelection()
        migrateProfiles()
    }

    /// Migrate users who had "medium.en" selected to the multilingual "medium" model
    private func migrateModelSelection() {
        if let stored = UserDefaults.standard.string(forKey: "selectedModel"), stored == "medium.en" {
            UserDefaults.standard.set(WhisperModel.medium.rawValue, forKey: "selectedModel")
            selectedModel = .medium
        }
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
        guard !isRecording, recordingStartTask == nil else { return }

        // Save which app has focus BEFORE any async gap (so we can paste back to it)
        TextInjector.shared.saveFocusedApp()
        errorMessage = nil

        recordingStartTask = Task {
            do {
                try await AudioRecorder.shared.startRecording()
                // Only show UI after mic is actually capturing
                self.isRecording = true
                RecordingOverlayController.shared.show()
            } catch {
                self.errorMessage = "Failed to start recording: \(error.localizedDescription)"
            }
            self.recordingStartTask = nil
        }
    }

    func cancelRecording() {
        recordingStartTask?.cancel()
        recordingStartTask = nil

        if isRecording {
            isRecording = false
            Task {
                let url = try? await AudioRecorder.shared.stopRecording()
                if let url { try? FileManager.default.removeItem(at: url) }
            }
        }

        RecordingOverlayController.shared.hide()
        TextInjector.shared.clearPreviousApp()
    }

    func stopRecordingAndTranscribe() {
        if let startTask = recordingStartTask {
            // Mic still starting — wait for it, then stop
            Task {
                await startTask.value
                if self.isRecording {
                    self.performStopAndTranscribe()
                }
            }
            return
        }

        guard isRecording else { return }
        performStopAndTranscribe()
    }

    private func performStopAndTranscribe() {
        isRecording = false
        isTranscribing = true

        // Show processing state in overlay
        RecordingOverlayController.shared.showProcessing()

        Task {
            do {
                let audioURL = try await AudioRecorder.shared.stopRecording()
                var transcription = try await WhisperService.shared.transcribe(audioURL: audioURL, language: selectedLanguage)

                if UserDefaults.standard.object(forKey: "removeFillerWords") as? Bool ?? true {
                    transcription = TextCleaner.clean(transcription)
                }

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

                // Inject text at cursor position (this also hides overlay)
                if !transcription.isEmpty {
                    await TextInjector.shared.injectText(transcription)
                }

                // Show "Ready" overlay briefly, then hide
                await MainActor.run {
                    RecordingOverlayController.shared.showReadyThenHide()
                }

                // Clean up audio file
                try? FileManager.default.removeItem(at: audioURL)

            } catch {
                await MainActor.run {
                    self.isTranscribing = false
                    self.errorMessage = "Transcription failed: \(error.localizedDescription)"
                    RecordingOverlayController.shared.hide()
                }
            }
        }
    }
}

enum WhisperModel: String, CaseIterable, Identifiable, Codable {
    case tiny = "tiny"
    case base = "base"
    case small = "small"
    case medium = "medium"
    case largeV3Turbo = "large-v3-turbo"
    case largeV3 = "large-v3"

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .tiny: return "Tiny (~40 MB) - Fastest"
        case .base: return "Base (~80 MB) - Balanced"
        case .small: return "Small (~250 MB) - Accurate"
        case .medium: return "Medium (~800 MB) - High Accuracy"
        case .largeV3Turbo: return "Large v3 Turbo (~1.1 GB) - Fast & Accurate ⭐"
        case .largeV3: return "Large v3 (~1.5 GB) - Best Accuracy"
        }
    }

    var shortName: String {
        switch self {
        case .tiny: return "Tiny"
        case .base: return "Base"
        case .small: return "Small"
        case .medium: return "Medium"
        case .largeV3Turbo: return "Large v3 Turbo"
        case .largeV3: return "Large v3"
        }
    }

    var whisperKitVariant: String {
        switch self {
        case .tiny: return "openai_whisper-tiny"
        case .base: return "openai_whisper-base"
        case .small: return "openai_whisper-small"
        case .medium: return "openai_whisper-medium"
        case .largeV3Turbo: return "openai_whisper-large-v3-v20240930_turbo"
        case .largeV3: return "openai_whisper-large-v3"
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

    func save(prefix: String = "") {
        UserDefaults.standard.set(keyCode, forKey: "\(prefix)hotkeyKeyCode")
        UserDefaults.standard.set(modifiers.rawValue, forKey: "\(prefix)hotkeyModifiers")
    }

    static func load(prefix: String = "") -> HotkeyConfig {
        let defaultKeyCode = prefix.isEmpty ? UInt32(kVK_Space) : UInt32(kVK_Space)
        let defaultModifiers = prefix.isEmpty ? NSEvent.ModifierFlags.option.rawValue : NSEvent.ModifierFlags([.shift, .option]).rawValue
        let keyCode = UserDefaults.standard.object(forKey: "\(prefix)hotkeyKeyCode") as? UInt32 ?? defaultKeyCode
        let modifiersRaw = UserDefaults.standard.object(forKey: "\(prefix)hotkeyModifiers") as? UInt ?? defaultModifiers
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
    static let switchHotkeyConfigChanged = Notification.Name("switchHotkeyConfigChanged")
}

// MARK: - Transcription History

extension AppState {
    func deleteHistoryItem(id: UUID) {
        transcriptionHistory.removeAll { $0.id == id }
    }

    func clearHistory() {
        transcriptionHistory.removeAll()
    }
}

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

// MARK: - Model Profiles

extension AppState {
    private static func loadProfiles() -> [ModelProfile] {
        guard let data = UserDefaults.standard.data(forKey: "modelProfiles"),
              let profiles = try? JSONDecoder().decode([ModelProfile].self, from: data) else {
            return []
        }
        return profiles
    }

    private func saveProfiles() {
        if let data = try? JSONEncoder().encode(profiles) {
            UserDefaults.standard.set(data, forKey: "modelProfiles")
        }
    }

    /// Create a default profile from existing settings on first launch
    fileprivate func migrateProfiles() {
        guard profiles.isEmpty else { return }
        let profile = ModelProfile(
            name: selectedLanguage.displayName,
            model: selectedModel,
            language: selectedLanguage
        )
        profiles = [profile]
    }

    func switchToNextProfile() {
        guard profiles.count >= 2 else { return }
        activeProfileIndex = (activeProfileIndex + 1) % profiles.count
        applyActiveProfile()
    }

    func applyActiveProfile() {
        guard let profile = activeProfile else { return }
        let modelChanged = selectedModel != profile.model
        selectedLanguage = profile.language
        selectedModel = profile.model

        SwitchOverlayController.shared.show(profileName: profile.name, model: profile.model, language: profile.language)

        if modelChanged && loadedModel != profile.model {
            Task {
                await WhisperService.shared.initialize()
            }
        }
    }
}

// MARK: - Model Profile

struct ModelProfile: Codable, Identifiable, Equatable {
    let id: UUID
    var name: String
    var model: WhisperModel
    var language: TranscriptionLanguage

    init(id: UUID = UUID(), name: String, model: WhisperModel, language: TranscriptionLanguage) {
        self.id = id
        self.name = name
        self.model = model
        self.language = language
    }
}

// MARK: - Transcription Language

enum TranscriptionLanguage: String, CaseIterable, Identifiable, Codable {
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
    case danish = "da"
    case norwegian = "no"
    case swedish = "sv"
    case finnish = "fi"

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
        case .danish: return "Danish"
        case .norwegian: return "Norwegian"
        case .swedish: return "Swedish"
        case .finnish: return "Finnish"
        }
    }
}
