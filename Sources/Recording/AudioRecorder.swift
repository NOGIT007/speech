import AVFoundation
import Foundation

actor AudioRecorder {
    static let shared = AudioRecorder()

    private var audioEngine: AVAudioEngine?
    private var audioFile: AVAudioFile?
    private var recordingURL: URL?

    enum AudioRecorderError: LocalizedError {
        case engineNotAvailable
        case noInputNode
        case failedToCreateFile
        case notRecording

        var errorDescription: String? {
            switch self {
            case .engineNotAvailable: return "Audio engine not available"
            case .noInputNode: return "No audio input available"
            case .failedToCreateFile: return "Failed to create audio file"
            case .notRecording: return "Not currently recording"
            }
        }
    }

    private init() {}

    func startRecording() throws {
        let engine = AVAudioEngine()
        self.audioEngine = engine

        let inputNode = engine.inputNode
        let format = inputNode.outputFormat(forBus: 0)

        // Whisper expects 16kHz mono audio
        guard let recordingFormat = AVAudioFormat(
            commonFormat: .pcmFormatFloat32,
            sampleRate: 16000,
            channels: 1,
            interleaved: false
        ) else {
            throw AudioRecorderError.failedToCreateFile
        }

        // Create temp file for recording
        let tempDir = FileManager.default.temporaryDirectory
        let fileName = "speach_recording_\(UUID().uuidString).wav"
        let url = tempDir.appendingPathComponent(fileName)
        self.recordingURL = url

        // Create audio file
        guard let file = try? AVAudioFile(
            forWriting: url,
            settings: [
                AVFormatIDKey: kAudioFormatLinearPCM,
                AVSampleRateKey: 16000,
                AVNumberOfChannelsKey: 1,
                AVLinearPCMBitDepthKey: 16,
                AVLinearPCMIsFloatKey: false,
                AVLinearPCMIsBigEndianKey: false
            ]
        ) else {
            throw AudioRecorderError.failedToCreateFile
        }
        self.audioFile = file

        // Install tap with format conversion
        guard let converter = AVAudioConverter(from: format, to: recordingFormat) else {
            throw AudioRecorderError.failedToCreateFile
        }

        inputNode.installTap(onBus: 0, bufferSize: 4096, format: format) { [weak self] buffer, _ in
            guard let self = self else { return }

            let frameCount = AVAudioFrameCount(
                Double(buffer.frameLength) * (16000.0 / format.sampleRate)
            )

            guard let convertedBuffer = AVAudioPCMBuffer(
                pcmFormat: recordingFormat,
                frameCapacity: frameCount
            ) else { return }

            var error: NSError?
            let inputBlock: AVAudioConverterInputBlock = { _, outStatus in
                outStatus.pointee = .haveData
                return buffer
            }

            converter.convert(to: convertedBuffer, error: &error, withInputFrom: inputBlock)

            if error == nil {
                Task {
                    await self.writeBuffer(convertedBuffer)
                }
            }
        }

        try engine.start()
    }

    private func writeBuffer(_ buffer: AVAudioPCMBuffer) {
        try? audioFile?.write(from: buffer)
    }

    func stopRecording() throws -> URL {
        guard let engine = audioEngine, let url = recordingURL else {
            throw AudioRecorderError.notRecording
        }

        engine.inputNode.removeTap(onBus: 0)
        engine.stop()

        audioFile = nil
        audioEngine = nil

        let finalURL = url
        recordingURL = nil

        return finalURL
    }

    func isRecording() -> Bool {
        audioEngine?.isRunning ?? false
    }
}
