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

**Current version:** 1.2.1

### Release Workflow

1. **Bump version** in `build_app.sh` (VERSION variable)
2. **Commit** changes with message: `vX.Y.Z: Brief description`
3. **Push** to main: `git push origin master`
4. **Create tag**: `git tag -a vX.Y.Z -m "vX.Y.Z - Description"`
5. **Push tag**: `git push origin vX.Y.Z`
6. **Create GitHub release**:
   ```bash
   gh release create vX.Y.Z --title "vX.Y.Z - Title" --notes "Release notes here"
   ```

❗ **Important**: Users with the app installed check for updates via GitHub releases. Without a release, they won't see the update.

**Versioning:** Semantic versioning (MAJOR.MINOR.PATCH)
