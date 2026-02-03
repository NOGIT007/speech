import SwiftUI

struct MenuBarView: View {
    @EnvironmentObject var appState: AppState
    @StateObject private var updateManager = UpdateManager.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Status header
            statusHeader

            Divider()
                .padding(.vertical, 4)

            // Transcription history
            if !appState.transcriptionHistory.isEmpty {
                historySection

                Divider()
                    .padding(.vertical, 4)
            }

            // Error message
            if let error = appState.errorMessage {
                errorSection(error)
                Divider()
                    .padding(.vertical, 4)
            }

            // Actions
            if #available(macOS 14.0, *) {
                SettingsLink {
                    Text("Settings...")
                }
                .keyboardShortcut(",", modifiers: .command)
            } else {
                Button("Settings...") {
                    NSApp.activate(ignoringOtherApps: true)
                    NSApp.sendAction(Selector(("showSettingsWindow:")), to: nil, from: nil)
                }
                .keyboardShortcut(",", modifiers: .command)
            }

            Divider()
                .padding(.vertical, 4)

            // Action buttons
            HStack(spacing: 8) {
                Button(action: relaunchApp) {
                    Label("Relaunch", systemImage: "arrow.clockwise")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)

                Button(action: { NSApplication.shared.terminate(nil) }) {
                    Label("Quit", systemImage: "xmark.circle")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .keyboardShortcut("q", modifiers: .command)
            }
            .padding(.horizontal, 12)

            // Update section
            updateSection
        }
        .padding(.vertical, 8)
        .frame(width: 320)
    }

    @ViewBuilder
    private var updateSection: some View {
        VStack(spacing: 4) {
            if updateManager.isDownloading {
                ProgressView(value: updateManager.downloadProgress)
                    .padding(.horizontal, 12)
                Text("Downloading update...")
                    .font(.system(size: 10))
                    .foregroundColor(.secondary)
            } else if updateManager.updateAvailable, let version = updateManager.latestVersion {
                Button(action: { Task { await updateManager.downloadAndInstall() } }) {
                    Label("Update to v\(version)", systemImage: "arrow.down.circle")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .padding(.horizontal, 12)
            } else {
                HStack(spacing: 4) {
                    Text("Speech v\(appVersion)")
                        .font(.system(size: 10))
                        .foregroundColor(.secondary)

                    if updateManager.isChecking {
                        ProgressView()
                            .scaleEffect(0.5)
                        Text("Checking...")
                            .font(.system(size: 9))
                            .foregroundColor(.secondary)
                    } else if updateManager.latestVersion != nil && !updateManager.updateAvailable {
                        Image(systemName: "checkmark.circle.fill")
                            .font(.system(size: 9))
                            .foregroundColor(.green)
                        Text("Up to date")
                            .font(.system(size: 9))
                            .foregroundColor(.secondary)
                    } else {
                        Button(action: { Task { await updateManager.checkForUpdates() } }) {
                            HStack(spacing: 2) {
                                Image(systemName: "arrow.clockwise")
                                    .font(.system(size: 9))
                                Text("Update")
                                    .font(.system(size: 9))
                            }
                        }
                        .buttonStyle(.plain)
                        .foregroundColor(.secondary)
                    }
                }
            }

            if let error = updateManager.errorMessage {
                Text(error)
                    .font(.system(size: 9))
                    .foregroundColor(.red)
                    .lineLimit(2)
                    .padding(.horizontal, 12)
            }
        }
        .padding(.top, 8)
    }

    private var appVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
    }

    // MARK: - Status Header

    private var statusHeader: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                statusIndicator
                VStack(alignment: .leading, spacing: 2) {
                    Text(statusText)
                        .font(.system(size: 13, weight: .semibold))
                    Text("Hold \(appState.hotkeyConfig.displayString) to dictate")
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                }
                Spacer()
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
    }

    @ViewBuilder
    private var statusIndicator: some View {
        Circle()
            .fill(statusColor)
            .frame(width: 10, height: 10)
            .shadow(color: statusColor.opacity(0.5), radius: 3)
    }

    private var statusText: String {
        if appState.isRecording {
            return "Recording..."
        } else if appState.isTranscribing {
            return "Transcribing..."
        } else {
            switch appState.modelStatus {
            case .notDownloaded:
                return "Model not downloaded"
            case .downloading(let progress):
                return "Downloading \(Int(progress * 100))%"
            case .ready:
                return "Ready"
            case .error:
                return "Error"
            }
        }
    }

    private var statusColor: Color {
        if appState.isRecording {
            return .red
        } else if appState.isTranscribing {
            return .orange
        } else {
            switch appState.modelStatus {
            case .ready: return .green
            case .downloading: return .yellow
            default: return .gray
            }
        }
    }

    private func relaunchApp() {
        let bundlePath = Bundle.main.bundlePath
        let task = Process()
        task.launchPath = "/usr/bin/open"
        task.arguments = ["-n", bundlePath]
        try? task.run()

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            NSApplication.shared.terminate(nil)
        }
    }

    // MARK: - History Section

    private var historySection: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Recent")
                .font(.system(size: 11, weight: .medium))
                .foregroundColor(.secondary)
                .padding(.horizontal, 12)
                .padding(.bottom, 2)

            ForEach(appState.transcriptionHistory) { item in
                TranscriptionRow(item: item)
            }
        }
    }

    // MARK: - Error Section

    private func errorSection(_ error: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundColor(.red)
                .font(.system(size: 12))
            Text(error)
                .font(.system(size: 11))
                .foregroundColor(.red)
                .lineLimit(2)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
    }
}

// MARK: - Transcription Row

struct TranscriptionRow: View {
    let item: TranscriptionItem
    @State private var isHovered = false

    var body: some View {
        Button(action: { copyToClipboard() }) {
            HStack(spacing: 8) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(item.preview)
                        .font(.system(size: 12))
                        .lineLimit(2)
                        .foregroundColor(.primary)
                    Text(item.timeAgo)
                        .font(.system(size: 10))
                        .foregroundColor(.secondary)
                }
                Spacer()
                Image(systemName: "doc.on.doc")
                    .font(.system(size: 11))
                    .foregroundColor(.secondary)
                    .opacity(isHovered ? 1 : 0.5)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(isHovered ? Color.primary.opacity(0.1) : Color.clear)
            .cornerRadius(4)
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
        }
    }

    private func copyToClipboard() {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(item.text, forType: .string)
    }
}
