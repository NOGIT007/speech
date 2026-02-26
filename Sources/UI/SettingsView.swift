import SwiftUI
import ServiceManagement

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @AppStorage("launchAtLogin") private var launchAtLogin = false
    @AppStorage("autoPaste") private var autoPaste = true
    @AppStorage("removeFillerWords") private var removeFillerWords = true
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
        .frame(width: 450, height: 560)
    }

    private var generalTab: some View {
        Form {
            Section {
                Toggle("Launch at login", isOn: $launchAtLogin)
                    .onChange(of: launchAtLogin) { newValue in
                        setLaunchAtLogin(newValue)
                    }
                Toggle("Auto-paste text", isOn: $autoPaste)
                Toggle("Remove filler words", isOn: $removeFillerWords)
            }

            Section("Dictation Hotkey") {
                HotkeyRecorderView(config: $appState.hotkeyConfig)
                Text("Hold to record, release to transcribe")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Section {
                ForEach(Array(appState.profiles.enumerated()), id: \.element.id) { index, profile in
                    profileCard(index: index)
                }

                Button(action: {
                    let profile = ModelProfile(
                        name: "New",
                        model: appState.selectedModel,
                        language: .english
                    )
                    appState.profiles.append(profile)
                }) {
                    Label("Add Profile", systemImage: "plus")
                }
                .buttonStyle(.borderless)
            } header: {
                Text("Profiles")
            } footer: {
                Text("Preset model + language combinations")
            }

            if appState.profiles.count >= 2 {
                Section("Profile Switching") {
                    Toggle("Enable quick switch", isOn: $appState.switchHotkeyEnabled)

                    if appState.switchHotkeyEnabled {
                        HotkeyRecorderView(
                            config: $appState.switchHotkeyConfig,
                            label: "Switch shortcut"
                        )

                        if appState.switchHotkeyConfig == appState.hotkeyConfig {
                            Label("Conflicts with dictation hotkey", systemImage: "exclamationmark.triangle.fill")
                                .foregroundColor(.orange)
                                .font(.caption)
                        }
                    }

                    Text("Tap to cycle between profiles")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

        }
        .formStyle(.grouped)
        .padding()
    }

    private var sortedLanguages: [TranscriptionLanguage] {
        let auto = TranscriptionLanguage.allCases.filter { $0 == .auto }
        let rest = TranscriptionLanguage.allCases.filter { $0 != .auto }
            .sorted { $0.displayName < $1.displayName }
        return auto + rest
    }

    private func profileCard(index: Int) -> some View {
        let safeNameBinding = Binding<String>(
            get: { index < appState.profiles.count ? appState.profiles[index].name : "" },
            set: { if index < appState.profiles.count { appState.profiles[index].name = $0 } }
        )
        let safeModelBinding = Binding<WhisperModel>(
            get: { index < appState.profiles.count ? appState.profiles[index].model : .largeV3Turbo },
            set: { if index < appState.profiles.count { appState.profiles[index].model = $0 } }
        )
        let safeLanguageBinding = Binding<TranscriptionLanguage>(
            get: { index < appState.profiles.count ? appState.profiles[index].language : .english },
            set: { if index < appState.profiles.count { appState.profiles[index].language = $0 } }
        )

        return VStack(alignment: .leading, spacing: 6) {
            HStack {
                Button(action: {
                    appState.activeProfileIndex = index
                    appState.applyActiveProfile()
                }) {
                    Image(systemName: index == appState.activeProfileIndex
                        ? "checkmark.circle.fill" : "circle")
                        .foregroundColor(index == appState.activeProfileIndex ? .green : .secondary)
                        .font(.system(size: 14))
                }
                .buttonStyle(.borderless)

                SelectAllTextField(text: safeNameBinding, placeholder: "Name")
                    .frame(maxWidth: 120)

                Spacer()

                if appState.profiles.count > 1 {
                    Button(action: {
                        let wasActive = appState.activeProfileIndex
                        if wasActive == index {
                            appState.activeProfileIndex = max(0, index - 1)
                        } else if wasActive > index {
                            appState.activeProfileIndex = wasActive - 1
                        }
                        appState.profiles.remove(at: index)
                    }) {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                            .font(.system(size: 12))
                    }
                    .buttonStyle(.borderless)
                }
            }

            HStack {
                Text("Model")
                Spacer()
                SearchablePicker(
                    items: WhisperModel.allCases.map { ($0.shortName, $0) },
                    selection: safeModelBinding
                )
                .frame(width: 160)
            }

            HStack {
                Text("Language")
                Spacer()
                SearchablePicker(
                    items: sortedLanguages.map { ($0.displayName, $0) },
                    selection: safeLanguageBinding
                )
                .frame(width: 160)
            }

            if index < appState.profiles.count - 1 {
                Divider()
            }
        }
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

                Text("⭐ Recommended for English. For other languages, Large v3 is best.")
                    .font(.caption)
                    .foregroundColor(.secondary)
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

                if !accessibilityPermission || !inputMonitoringPermission {
                    Button("Reset & Re-grant Permissions") {
                        resetPermissions()
                    }
                    .foregroundColor(.red)
                    Text("Use this if permissions appear stuck. Clears stale entries and restarts the app so you can re-grant them.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
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

    private func resetPermissions() {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/tccutil")
        process.arguments = ["reset", "Accessibility", "com.speech.app"]
        try? process.run()
        process.waitUntilExit()

        let process2 = Process()
        process2.executableURL = URL(fileURLWithPath: "/usr/bin/tccutil")
        process2.arguments = ["reset", "ListenEvent", "com.speech.app"]
        try? process2.run()
        process2.waitUntilExit()

        // Relaunch the app
        let appURL = Bundle.main.bundleURL
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        task.arguments = [appURL.path]
        try? task.run()

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            NSApp.terminate(nil)
        }
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

struct SelectAllTextField: NSViewRepresentable {
    @Binding var text: String
    var placeholder: String

    func makeNSView(context: Context) -> NSTextField {
        let field = NSTextField()
        field.placeholderString = placeholder
        field.stringValue = text
        field.bezelStyle = .roundedBezel
        field.delegate = context.coordinator
        return field
    }

    func updateNSView(_ field: NSTextField, context: Context) {
        if field.stringValue != text {
            field.stringValue = text
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    class Coordinator: NSObject, NSTextFieldDelegate {
        var parent: SelectAllTextField
        init(_ parent: SelectAllTextField) { self.parent = parent }

        func controlTextDidChange(_ obj: Notification) {
            guard let field = obj.object as? NSTextField else { return }
            parent.text = field.stringValue
        }

        func controlTextDidBeginEditing(_ obj: Notification) {
            guard let field = obj.object as? NSTextField,
                  let editor = field.currentEditor() else { return }
            editor.selectAll(nil)
        }
    }
}

struct SearchablePicker<T: Hashable>: View {
    let items: [(title: String, value: T)]
    @Binding var selection: T
    @State private var isOpen = false
    @State private var search = ""

    private var selectedTitle: String {
        items.first(where: { $0.value == selection })?.title ?? ""
    }

    private var filteredItems: [(title: String, value: T)] {
        if search.isEmpty { return items }
        return items.filter { $0.title.localizedCaseInsensitiveContains(search) }
    }

    var body: some View {
        Button(action: { isOpen.toggle() }) {
            HStack(spacing: 4) {
                Text(selectedTitle)
                    .lineLimit(1)
                Spacer(minLength: 2)
                Image(systemName: "chevron.up.chevron.down")
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(Color(nsColor: .controlBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 5)
                    .stroke(Color(nsColor: .separatorColor), lineWidth: 0.5)
            )
        }
        .buttonStyle(.plain)
        .popover(isPresented: $isOpen, arrowEdge: .bottom) {
            VStack(spacing: 0) {
                TextField("Filter…", text: $search)
                    .textFieldStyle(.roundedBorder)
                    .padding(8)

                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(filteredItems.enumerated()), id: \.element.value) { idx, item in
                                Button(action: {
                                    selection = item.value
                                    search = ""
                                    isOpen = false
                                }) {
                                    HStack {
                                        if item.value == selection {
                                            Image(systemName: "checkmark")
                                                .font(.system(size: 10, weight: .bold))
                                                .frame(width: 14)
                                        } else {
                                            Spacer().frame(width: 14)
                                        }
                                        Text(item.title)
                                            .lineLimit(1)
                                        Spacer()
                                    }
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 5)
                                    .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                                .background(
                                    item.value == selection
                                        ? Color.accentColor.opacity(0.15)
                                        : Color.clear
                                )
                                .id(item.value)
                            }
                        }
                    }
                    .frame(maxHeight: 200)
                    .onAppear {
                        proxy.scrollTo(selection, anchor: .center)
                    }
                }
            }
            .frame(width: 180)
        }
        .onChange(of: isOpen) { open in
            if !open { search = "" }
        }
    }
}
