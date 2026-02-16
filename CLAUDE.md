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

**Dependencies:** WhisperKit, HotKey

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

- Target: macOS 14+ (WhisperKit requirement)
- Menu bar app: `LSUIElement = true` (no dock icon)
- Models stored in WhisperKit's default cache (HuggingFace Hub cache)
- Hotkey is hold-to-record, release-to-transcribe

## Permissions Required

- Microphone (audio recording)
- Accessibility (text injection, auto-paste)
- Input Monitoring (keyboard simulation for auto-paste)

## Releasing

**Current version:** 2.3.1

### Release Workflow

1. **Bump version** in `build_app.sh` (VERSION variable) and `CLAUDE.md`
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

❗ **Important**: Users with the app installed check for updates via GitHub releases. Without a release, they won't see the update.

**Versioning:** Semantic versioning (MAJOR.MINOR.PATCH)
