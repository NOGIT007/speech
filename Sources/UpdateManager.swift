import Foundation
import AppKit

@MainActor
class UpdateManager: ObservableObject {
    static let shared = UpdateManager()

    @Published var isChecking = false
    @Published var isDownloading = false
    @Published var downloadProgress: Double = 0
    @Published var updateAvailable: Bool = false
    @Published var latestVersion: String?
    @Published var errorMessage: String?

    private let githubRepo = "NOGIT007/speech"
    private let currentVersion: String

    private init() {
        currentVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0.0"
    }

    func checkForUpdates() async {
        isChecking = true
        errorMessage = nil
        updateAvailable = false

        defer { isChecking = false }

        guard let url = URL(string: "https://api.github.com/repos/\(githubRepo)/releases/latest") else {
            errorMessage = "Invalid URL"
            return
        }

        do {
            var request = URLRequest(url: url)
            request.setValue("application/vnd.github.v3+json", forHTTPHeaderField: "Accept")

            let (data, response) = try await URLSession.shared.data(for: request)

            guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
                errorMessage = "Failed to check for updates"
                return
            }

            guard let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let tagName = json["tag_name"] as? String else {
                errorMessage = "Invalid response"
                return
            }

            let remoteVersion = tagName.replacingOccurrences(of: "v", with: "")
            latestVersion = remoteVersion

            if isNewerVersion(remoteVersion, than: currentVersion) {
                updateAvailable = true
            }
        } catch {
            errorMessage = "Network error: \(error.localizedDescription)"
        }
    }

    func downloadAndInstall() async {
        isDownloading = true
        downloadProgress = 0
        errorMessage = nil

        defer { isDownloading = false }

        guard let url = URL(string: "https://api.github.com/repos/\(githubRepo)/releases/latest") else {
            errorMessage = "Invalid URL"
            return
        }

        do {
            // Get release info
            var request = URLRequest(url: url)
            request.setValue("application/vnd.github.v3+json", forHTTPHeaderField: "Accept")

            let (data, _) = try await URLSession.shared.data(for: request)

            guard let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let assets = json["assets"] as? [[String: Any]],
                  let zipAsset = assets.first(where: { ($0["name"] as? String)?.hasSuffix(".zip") == true }),
                  let downloadURLString = zipAsset["browser_download_url"] as? String,
                  let downloadURL = URL(string: downloadURLString) else {
                errorMessage = "No downloadable release found. Please download manually from GitHub."
                return
            }

            // Download the zip file
            downloadProgress = 0.1
            let (zipFileURL, _) = try await URLSession.shared.download(from: downloadURL)
            downloadProgress = 0.5

            // Create temp directory for extraction
            let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
            try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)

            // Unzip
            let unzipProcess = Process()
            unzipProcess.executableURL = URL(fileURLWithPath: "/usr/bin/unzip")
            unzipProcess.arguments = ["-o", zipFileURL.path, "-d", tempDir.path]
            try unzipProcess.run()
            unzipProcess.waitUntilExit()
            downloadProgress = 0.7

            // Find the .app in extracted files
            let contents = try FileManager.default.contentsOfDirectory(at: tempDir, includingPropertiesForKeys: nil)
            guard let appURL = contents.first(where: { $0.pathExtension == "app" }) else {
                errorMessage = "No app found in download"
                return
            }

            // Replace current app
            let applicationsURL = URL(fileURLWithPath: "/Applications/Speech.app")

            // Remove old app
            if FileManager.default.fileExists(atPath: applicationsURL.path) {
                try FileManager.default.removeItem(at: applicationsURL)
            }

            // Copy new app
            try FileManager.default.copyItem(at: appURL, to: applicationsURL)
            downloadProgress = 0.9

            // Cleanup
            try? FileManager.default.removeItem(at: tempDir)
            try? FileManager.default.removeItem(at: zipFileURL)

            downloadProgress = 1.0

            // Relaunch
            relaunchApp()

        } catch {
            errorMessage = "Update failed: \(error.localizedDescription)"
        }
    }

    private func isNewerVersion(_ remote: String, than local: String) -> Bool {
        let remoteComponents = remote.split(separator: ".").compactMap { Int($0) }
        let localComponents = local.split(separator: ".").compactMap { Int($0) }

        for i in 0..<max(remoteComponents.count, localComponents.count) {
            let r = i < remoteComponents.count ? remoteComponents[i] : 0
            let l = i < localComponents.count ? localComponents[i] : 0

            if r > l { return true }
            if r < l { return false }
        }
        return false
    }

    private func relaunchApp() {
        let bundlePath = "/Applications/Speech.app"
        let task = Process()
        task.launchPath = "/usr/bin/open"
        task.arguments = ["-n", bundlePath]
        try? task.run()

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            NSApplication.shared.terminate(nil)
        }
    }
}
