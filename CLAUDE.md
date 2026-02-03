# Speech

macOS menu bar app for speech-to-text using local Whisper models.

## Architecture

```
Sources/
├── SpeechApp.swift          # App entry, MenuBarExtra
├── AppDelegate.swift        # Lifecycle, permissions
├── AppState.swift           # Shared state, models
├── Hotkeys/                 # Global hotkey handling
├── Recording/               # Audio capture (WAV)
├── Transcription/           # WhisperService
├── Injection/               # Paste text to active app
└── UI/                      # SwiftUI views
```

**Dependencies:** SwiftWhisper, HotKey

## Commands

```bash
# Development
swift build
swift run Speech

# Production build
./build_app.sh
cp -R .build/Speech.app /Applications/
```

## Critical Rules

- Target: macOS 13+ (use `#available` for newer APIs)
- Menu bar app: `LSUIElement = true` (no dock icon)
- Models stored in `~/Library/Application Support/Speech/Models/`
- Hotkey is hold-to-record, release-to-transcribe

## Permissions Required

- Microphone (audio recording)
- Accessibility (text injection)

## Releasing

**Current version:** 1.0.0

When releasing to GitHub:

1. Bump version in `build_app.sh` (VERSION variable)
2. Commit changes
3. Tag: `git tag -a vX.Y.Z -m "vX.Y.Z - Description"`
4. Push: `git push origin main --tags`

**Versioning:** Semantic versioning (MAJOR.MINOR.PATCH)
