# Speech

macOS menu bar app for speech-to-text using local models. Built with Tauri 2 + Rust + Svelte 5.

## Architecture

```
src-tauri/src/
├── lib.rs                   # App setup, plugins, command registration
├── state.rs                 # RecordingCoordinator state machine
├── tray.rs                  # System tray setup
├── text_cleaner.rs          # Filler word removal
├── commands/                # Tauri IPC commands
│   ├── audio.rs             # start/stop recording, audio level
│   ├── model.rs             # list/download/delete models
│   ├── permissions.rs       # macOS permission checks
│   ├── profiles.rs          # Model profile CRUD + switching
│   ├── settings.rs          # Settings read/write + side-effects
│   ├── state.rs             # Phase, history, clipboard, relaunch
│   └── update.rs            # Auto-update check + install
└── managers/                # Business logic
    ├── audio.rs             # cpal recording to WAV via hound
    ├── model.rs             # Model registry, downloads, storage
    ├── transcription.rs     # Whisper/Moonshine/Parakeet/SenseVoice
    ├── hotkey.rs            # Global shortcut handling
    ├── paste.rs             # Clipboard + CGEvent Cmd+V injection
    ├── permissions.rs       # AX/Mic/InputMonitoring via objc
    ├── settings.rs          # AppSettings struct
    └── update.rs            # UpdateInfo struct

src/                         # Svelte 5 frontend
├── App.svelte               # Router by webview label
├── components/
│   ├── MenuBarPanel.svelte  # Main tray popover UI
│   └── TranscriptionRow.svelte
├── overlay/
│   ├── RecordingOverlay.svelte  # Waveform during recording
│   └── SwitchOverlay.svelte     # Profile switch toast
├── settings/
│   ├── SettingsWindow.svelte    # Tab container
│   ├── GeneralTab.svelte        # Preferences + hotkeys
│   ├── ProfilesTab.svelte       # Profile CRUD
│   ├── ProfileCard.svelte       # Single profile card
│   └── ModelTab.svelte          # Model browser + download
│   └── PermissionsTab.svelte    # macOS permission status
└── lib/
    └── types.ts             # Shared TypeScript types
```

**Stack:** Tauri 2, Rust, Svelte 5, Tailwind CSS v4, Vite

**Key Rust deps:** cpal, hound, cocoa, objc, core-graphics, arboard, reqwest, tauri-plugin-store, tauri-plugin-global-shortcut, tauri-plugin-updater, tauri-plugin-autostart

## Commands

```bash
# Development
bun run tauri dev

# Production build
bun run tauri build
# or use the wrapper:
./build_app.sh

# Frontend only
bun run dev
bun run build

# Rust checks
cd src-tauri && cargo check
cd src-tauri && cargo test
```

## Critical Rules

- Target: macOS 14+
- Menu bar app: NSApplicationActivationPolicyAccessory (no dock icon)
- Models stored in app data dir under `models/`
- Hotkey is hold-to-record, release-to-transcribe
- Never hold `app.state::<T>()` across await points (borrow lifetime issue)
- Use `$state`, `$props`, `$effect` (Svelte 5 runes, NOT stores)
- **Never use `swift run` or `swift build`** - this is a Tauri app now

## Permissions Required

- Microphone (audio recording)
- Accessibility (text injection, auto-paste)
- Input Monitoring (keyboard simulation for auto-paste)

## Releasing

**Current version:** 3.4.1

### Release Workflow

1. **Bump version** in `build_app.sh`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and `CLAUDE.md`
2. **Build the app**: `./build_app.sh` (must be AFTER version bump)
3. **Commit** changes with message: `vX.Y.Z: Brief description`
4. **Push** to main: `git push origin master`
5. **Create tag**: `git tag -a vX.Y.Z -m "vX.Y.Z - Description"`
6. **Push tag**: `git push origin vX.Y.Z`
7. **Create GitHub release with binary**:
   ```bash
   cd .build && zip -r Speech.app.zip Speech.app
   gh release create vX.Y.Z Speech.app.zip --title "vX.Y.Z - Title" --notes "Release notes"
   ```

**Release notes format:** Every release should include the app icon image centered at the top and a codename (e.g. "Quiet Mic").

**Important**: Users with the app installed check for updates via GitHub releases (tauri-plugin-updater). Without a release, they won't see the update.

**Versioning:** Semantic versioning (MAJOR.MINOR.PATCH)
