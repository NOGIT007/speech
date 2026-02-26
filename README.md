# Speech

A macOS menu bar app for speech-to-text using local Whisper models. All transcription happens on-device — no cloud services, no API keys, complete privacy.

![macOS](https://img.shields.io/badge/macOS-14%2B-blue)
![Rust](https://img.shields.io/badge/Rust-Tauri%202-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Hold-to-record**: Hold the hotkey to record, release to transcribe and auto-paste
- **Local processing**: Whisper models running entirely on your Mac
- **Privacy-first**: No data leaves your device
- **Multiple engines**: Whisper, Moonshine, Parakeet, SenseVoice
- **Model profiles**: Save different model configurations and switch between them
- **Multi-language**: 25+ languages with auto-detection
- **Customizable hotkey**: Default is `⌥Space` (Option + Space)
- **Auto-update**: Checks for updates from GitHub releases

## Tech Stack

- **Tauri 2** — native macOS app shell
- **Rust** — audio recording, transcription, hotkeys, paste injection
- **Svelte 5** + **Tailwind CSS** — UI (menu bar panel, overlays, settings)
- **transcribe-rs** — local Whisper inference

## Installation

### Requirements

- macOS 14.0 or later

### Download & Install

1. Download the latest release from [Releases](https://github.com/kennetkusk/speech/releases)
2. Unzip and drag `Speech.app` to `/Applications`
3. Open `Speech.app`

> **macOS Gatekeeper:** Since the app is not notarized, macOS may block it. Right-click → **Open** → **Open** again, or go to **System Settings → Privacy & Security → Open Anyway**.

### Build from Source

```bash
git clone https://github.com/kennetkusk/speech.git
cd speech
bun install
./build_app.sh
cp -R .build/Speech.app /Applications/
open /Applications/Speech.app
```

## Setup

On first launch, grant these permissions (check status in **Settings → Permissions**):

1. **Microphone** — required for recording
2. **Accessibility** — required for auto-paste and global hotkey
3. **Input Monitoring** — required for keyboard simulation

Then download a model in **Settings → Models**.

## Usage

1. **Hold** `⌥Space` to start recording
2. Speak clearly
3. **Release** to transcribe and auto-paste

> Press **Escape** while recording to cancel.

### Settings

Click the menu bar icon → **Settings** to configure:

- **General**: Hotkey, language, auto-paste, launch at login
- **Models**: Download/manage transcription models
- **Profiles**: Save model configurations for quick switching
- **Permissions**: Check macOS permission status

## Privacy

All audio is processed locally. No data is sent to external servers.

## License

MIT — see [LICENSE](LICENSE).
