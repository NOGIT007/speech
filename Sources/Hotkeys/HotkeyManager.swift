import AppKit
import HotKey
import Carbon.HIToolbox

@MainActor
class HotkeyManager {
    private var hotKey: HotKey?
    private var isKeyDown = false
    private var eventMonitor: Any?
    private var configObserver: NSObjectProtocol?
    private var currentConfig: HotkeyConfig

    init() {
        self.currentConfig = AppState.shared.hotkeyConfig
    }

    func registerHotkey() {
        setupHotkey(with: currentConfig)

        // Listen for config changes
        configObserver = NotificationCenter.default.addObserver(
            forName: .hotkeyConfigChanged,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in
                self?.handleConfigChange()
            }
        }
    }

    private func handleConfigChange() {
        let newConfig = AppState.shared.hotkeyConfig
        guard newConfig != currentConfig else { return }
        currentConfig = newConfig
        unregisterHotkey()
        setupHotkey(with: newConfig)
    }

    private func setupHotkey(with config: HotkeyConfig) {
        guard let key = config.hotKeyKey else {
            print("Invalid hotkey configuration")
            return
        }

        hotKey = HotKey(key: key, modifiers: config.hotKeyModifiers)

        hotKey?.keyDownHandler = { [weak self] in
            self?.handleKeyDown()
        }

        hotKey?.keyUpHandler = { [weak self] in
            self?.handleKeyUp()
        }

        // Monitor for key releases
        eventMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.flagsChanged, .keyUp]) { [weak self] event in
            Task { @MainActor in
                self?.handleGlobalEvent(event)
            }
        }
    }

    private func handleKeyDown() {
        guard !isKeyDown else { return }
        isKeyDown = true
        AppState.shared.startRecording()
    }

    private func handleKeyUp() {
        guard isKeyDown else { return }
        isKeyDown = false
        AppState.shared.stopRecordingAndTranscribe()
    }

    private func handleGlobalEvent(_ event: NSEvent) {
        // Check if modifier was released
        if event.type == .flagsChanged {
            let requiredModifiers = currentConfig.modifiers
            let currentModifiers = event.modifierFlags.intersection([.control, .option, .shift, .command])

            if !currentModifiers.contains(requiredModifiers) && isKeyDown {
                handleKeyUp()
            }
        }

        // Check if the key was released
        if event.type == .keyUp && event.keyCode == UInt16(currentConfig.keyCode) {
            if isKeyDown {
                handleKeyUp()
            }
        }
    }

    func unregisterHotkey() {
        hotKey = nil

        if let monitor = eventMonitor {
            NSEvent.removeMonitor(monitor)
            eventMonitor = nil
        }
    }

    deinit {
        if let observer = configObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }
}
