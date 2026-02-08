import Foundation
import WhisperKit

actor WhisperService {
    static let shared = WhisperService()

    private var whisperKit: WhisperKit?
    private var isInitialized = false

    enum WhisperError: LocalizedError {
        case notInitialized
        case transcriptionFailed

        var errorDescription: String? {
            switch self {
            case .notInitialized: return "Whisper not initialized"
            case .transcriptionFailed: return "Transcription failed"
            }
        }
    }

    private init() {}

    func initialize() async {
        let model = await MainActor.run { AppState.shared.selectedModel }

        await MainActor.run {
            AppState.shared.modelStatus = .downloading(progress: 0)
        }

        do {
            // Clean up legacy ggml model files from SwiftWhisper
            cleanupLegacyModels()

            // Download model (WhisperKit handles caching internally)
            let modelFolder = try await WhisperKit.download(
                variant: model.whisperKitVariant,
                progressCallback: { progress in
                    Task { @MainActor in
                        AppState.shared.modelStatus = .downloading(progress: progress.fractionCompleted)
                    }
                }
            )

            // Initialize WhisperKit with the downloaded model
            whisperKit = try await WhisperKit(
                model: model.whisperKitVariant,
                modelFolder: modelFolder.path,
                verbose: false,
                logLevel: .error,
                download: false
            )

            isInitialized = true
            await MainActor.run {
                AppState.shared.modelStatus = .ready
                AppState.shared.loadedModel = model
            }
        } catch {
            await MainActor.run {
                AppState.shared.modelStatus = .error("Failed to load model: \(error.localizedDescription)")
            }
        }
    }

    func transcribe(audioURL: URL, language: TranscriptionLanguage = .english) async throws -> String {
        guard isInitialized, let whisperKit = whisperKit else {
            throw WhisperError.notInitialized
        }

        var options = DecodingOptions()
        if language != .auto {
            options.language = language.rawValue
        }

        let results = try await whisperKit.transcribe(
            audioPath: audioURL.path,
            decodeOptions: options
        )

        let transcription = results.map { $0.text }.joined(separator: " ")
        return transcription.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    func cleanup() {
        whisperKit = nil
        isInitialized = false
    }

    /// Remove legacy ggml .bin model files from SwiftWhisper (one-time migration)
    private func cleanupLegacyModels() {
        let key = "didCleanupLegacyModels"
        guard !UserDefaults.standard.bool(forKey: key) else { return }

        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let modelsDir = appSupport.appendingPathComponent("Speech/Models", isDirectory: true)

        if let files = try? FileManager.default.contentsOfDirectory(at: modelsDir, includingPropertiesForKeys: nil) {
            for file in files where file.lastPathComponent.hasPrefix("ggml-") && file.pathExtension == "bin" {
                try? FileManager.default.removeItem(at: file)
            }
        }

        UserDefaults.standard.set(true, forKey: key)
    }
}
