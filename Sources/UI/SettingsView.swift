import SwiftUI
import ServiceManagement

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @AppStorage("launchAtLogin") private var launchAtLogin = false
    @State private var micPermission = false
    @State private var accessibilityPermission = false
    @State private var inputMonitoringPermission = false

    var body: some View {
        TabView {
            generalTab
                .tabItem {
                    Label("General", systemImage: "gear")
                }

            modelTab
                .tabItem {
                    Label("Model", systemImage: "cpu")
                }

            permissionsTab
                .tabItem {
                    Label("Permissions", systemImage: "lock.shield")
                }
        }
        .frame(width: 450, height: 320)
    }

    private var generalTab: some View {
        Form {
            Section {
                Toggle("Launch at login", isOn: $launchAtLogin)
                    .onChange(of: launchAtLogin) { newValue in
                        setLaunchAtLogin(newValue)
                    }
            }

            Section("Hotkey") {
                HotkeyRecorderView(config: $appState.hotkeyConfig)
                Text("Hold the shortcut to record, release to transcribe")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Section("Language") {
                Picker("Transcription language", selection: $appState.selectedLanguage) {
                    ForEach(TranscriptionLanguage.allCases) { lang in
                        Text(lang.displayName).tag(lang)
                    }
                }
            }

            Section("Text Injection") {
                Toggle("Auto-paste after transcription", isOn: $appState.autoPasteEnabled)
                Text("When disabled, text is copied to clipboard only")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .formStyle(.grouped)
        .padding()
    }

    private var modelTab: some View {
        Form {
            Section("Whisper Model") {
                Picker("Model", selection: $appState.selectedModel) {
                    ForEach(WhisperModel.allCases) { model in
                        Text(model.displayName).tag(model)
                    }
                }
                .pickerStyle(.radioGroup)

                modelStatusView
            }

            Section {
                Button(buttonTitle) {
                    Task {
                        await WhisperService.shared.initialize()
                    }
                }
                .disabled(!canDownload)
            }
        }
        .formStyle(.grouped)
        .padding()
    }

    @ViewBuilder
    private var modelStatusView: some View {
        HStack {
            Text("Status:")
            Spacer()

            switch appState.modelStatus {
            case .notDownloaded:
                Text("Not downloaded")
                    .foregroundColor(.secondary)
            case .downloading(let progress):
                ProgressView(value: progress)
                    .frame(width: 100)
                Text("\(Int(progress * 100))%")
            case .ready:
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Ready")
                }
            case .error(let message):
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(.red)
                    Text(message)
                        .foregroundColor(.red)
                }
            }
        }
    }

    private var isDownloading: Bool {
        if case .downloading = appState.modelStatus {
            return true
        }
        return false
    }

    private var canDownload: Bool {
        if isDownloading { return false }
        // Allow download if different model selected or current model not ready
        if appState.loadedModel != appState.selectedModel { return true }
        if appState.modelStatus != .ready { return true }
        return false
    }

    private var buttonTitle: String {
        if appState.loadedModel == appState.selectedModel && appState.modelStatus == .ready {
            return "Model Ready"
        }
        return "Download \(appState.selectedModel.rawValue.capitalized) Model"
    }

    private var permissionsTab: some View {
        Form {
            Section("Required Permissions") {
                permissionRow(
                    title: "Microphone",
                    description: "Required for voice recording",
                    isGranted: micPermission,
                    action: openMicrophoneSettings
                )

                permissionRow(
                    title: "Accessibility",
                    description: "Required for text injection",
                    isGranted: accessibilityPermission,
                    action: openAccessibilitySettings
                )

                permissionRow(
                    title: "Input Monitoring",
                    description: "Required for keyboard simulation",
                    isGranted: inputMonitoringPermission,
                    action: openInputMonitoringSettings
                )
            }

            Section {
                Text("Grant all permissions for Speech to work correctly.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .formStyle(.grouped)
        .padding()
        .onAppear { refreshPermissions() }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { _ in
            refreshPermissions()
        }
    }

    private func refreshPermissions() {
        micPermission = AVCaptureDevice.authorizationStatus(for: .audio) == .authorized
        accessibilityPermission = AXIsProcessTrusted()
        inputMonitoringPermission = PermissionsManager().hasInputMonitoringPermission
    }

    private func permissionRow(
        title: String,
        description: String,
        isGranted: Bool,
        action: @escaping () -> Void
    ) -> some View {
        HStack {
            VStack(alignment: .leading) {
                Text(title)
                    .font(.headline)
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            if isGranted {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.green)
                    .font(.title2)
            } else {
                Button("Grant") {
                    action()
                }
            }
        }
    }

    private func openMicrophoneSettings() {
        PermissionsManager().openMicrophoneSettings()
    }

    private func openAccessibilitySettings() {
        PermissionsManager().openAccessibilitySettings()
    }

    private func openInputMonitoringSettings() {
        PermissionsManager().openInputMonitoringSettings()
    }

    private func setLaunchAtLogin(_ enabled: Bool) {
        if #available(macOS 13.0, *) {
            do {
                if enabled {
                    try SMAppService.mainApp.register()
                } else {
                    try SMAppService.mainApp.unregister()
                }
            } catch {
                print("Failed to set launch at login: \(error)")
            }
        }
    }
}

import AVFoundation
