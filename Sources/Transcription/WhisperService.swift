import Foundation
import SwiftWhisper

actor WhisperService {
    static let shared = WhisperService()

    private var whisper: Whisper?
    private var isInitialized = false

    enum WhisperError: LocalizedError {
        case notInitialized
        case modelNotFound
        case transcriptionFailed
        case audioLoadFailed

        var errorDescription: String? {
            switch self {
            case .notInitialized: return "Whisper not initialized"
            case .modelNotFound: return "Model file not found"
            case .transcriptionFailed: return "Transcription failed"
            case .audioLoadFailed: return "Failed to load audio file"
            }
        }
    }

    private init() {}

    func initialize() async {
        // Check if model exists, download if needed
        let model = await MainActor.run { AppState.shared.selectedModel }
        let modelPath = getModelPath(for: model)

        if !FileManager.default.fileExists(atPath: modelPath.path) {
            await downloadModel(model)
        }

        await loadModel(from: modelPath)
    }

    private func getModelDirectory() -> URL {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let speachDir = appSupport.appendingPathComponent("Speech/Models", isDirectory: true)

        if !FileManager.default.fileExists(atPath: speachDir.path) {
            try? FileManager.default.createDirectory(at: speachDir, withIntermediateDirectories: true)
        }

        return speachDir
    }

    private func getModelPath(for model: WhisperModel) -> URL {
        getModelDirectory().appendingPathComponent(model.fileName)
    }

    private func downloadModel(_ model: WhisperModel) async {
        await MainActor.run {
            AppState.shared.modelStatus = .downloading(progress: 0)
        }

        let destination = getModelPath(for: model)

        do {
            // Use URLSession with delegate for progress
            let (tempURL, _) = try await downloadWithProgress(from: model.downloadURL) { progress in
                Task { @MainActor in
                    AppState.shared.modelStatus = .downloading(progress: progress)
                }
            }

            // Move to final destination
            if FileManager.default.fileExists(atPath: destination.path) {
                try FileManager.default.removeItem(at: destination)
            }
            try FileManager.default.moveItem(at: tempURL, to: destination)

            await MainActor.run {
                AppState.shared.modelStatus = .ready
            }
        } catch {
            await MainActor.run {
                AppState.shared.modelStatus = .error("Download failed: \(error.localizedDescription)")
            }
        }
    }

    private func downloadWithProgress(
        from url: URL,
        progressHandler: @escaping (Double) -> Void
    ) async throws -> (URL, URLResponse) {
        let delegate = DownloadProgressDelegate(progressHandler: progressHandler)
        let session = URLSession(configuration: .default, delegate: delegate, delegateQueue: nil)

        return try await withCheckedThrowingContinuation { continuation in
            delegate.continuation = continuation

            let task = session.downloadTask(with: url)
            task.resume()
        }
    }

    private func loadModel(from path: URL) async {
        guard FileManager.default.fileExists(atPath: path.path) else {
            await MainActor.run {
                AppState.shared.modelStatus = .error("Model file not found")
            }
            return
        }

        let model = await MainActor.run { AppState.shared.selectedModel }
        whisper = Whisper(fromFileURL: path)
        if whisper != nil {
            isInitialized = true
            await MainActor.run {
                AppState.shared.modelStatus = .ready
                AppState.shared.loadedModel = model
            }
        } else {
            await MainActor.run {
                AppState.shared.modelStatus = .error("Failed to load model")
            }
        }
    }

    func transcribe(audioURL: URL, language: TranscriptionLanguage = .english) async throws -> String {
        guard isInitialized, let whisper = whisper else {
            throw WhisperError.notInitialized
        }

        // Load audio samples from WAV file
        let samples = try loadAudioSamples(from: audioURL)

        // Set language on whisper params
        if let whisperLang = WhisperLanguage(rawValue: language.rawValue) {
            whisper.params.language = whisperLang
        }

        // Transcribe using SwiftWhisper
        let segments = try await whisper.transcribe(audioFrames: samples)

        // Combine all segments
        let transcription = segments.map { $0.text }.joined(separator: " ")

        return transcription.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func loadAudioSamples(from url: URL) throws -> [Float] {
        let data = try Data(contentsOf: url)

        // Parse WAV header to get to PCM data
        // Standard WAV header is 44 bytes
        guard data.count > 44 else {
            throw WhisperError.audioLoadFailed
        }

        let pcmData = data.dropFirst(44)

        // Convert 16-bit PCM to float samples
        var samples = [Float]()
        samples.reserveCapacity(pcmData.count / 2)

        for i in stride(from: 0, to: pcmData.count - 1, by: 2) {
            let index = pcmData.startIndex + i
            let low = UInt16(pcmData[index])
            let high = UInt16(pcmData[index + 1])
            let sample = Int16(bitPattern: low | (high << 8))
            samples.append(Float(sample) / 32768.0)
        }

        return samples
    }

    func cleanup() {
        whisper = nil
        isInitialized = false
    }
}

// MARK: - Download Progress Delegate

private class DownloadProgressDelegate: NSObject, URLSessionDownloadDelegate {
    let progressHandler: (Double) -> Void
    var continuation: CheckedContinuation<(URL, URLResponse), Error>?

    init(progressHandler: @escaping (Double) -> Void) {
        self.progressHandler = progressHandler
    }

    func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didFinishDownloadingTo location: URL
    ) {
        guard let response = downloadTask.response else {
            continuation?.resume(throwing: URLError(.badServerResponse))
            return
        }

        // Copy to temp location before returning (original gets deleted)
        let tempURL = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".bin")

        do {
            try FileManager.default.copyItem(at: location, to: tempURL)
            continuation?.resume(returning: (tempURL, response))
        } catch {
            continuation?.resume(throwing: error)
        }
    }

    func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didWriteData bytesWritten: Int64,
        totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        let progress = Double(totalBytesWritten) / Double(totalBytesExpectedToWrite)
        progressHandler(progress)
    }

    func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        didCompleteWithError error: Error?
    ) {
        if let error = error {
            continuation?.resume(throwing: error)
        }
    }
}
