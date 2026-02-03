# Speech to text

A lightweight macOS menu bar app for speech-to-text using local Whisper models. All transcription happens on-device - no cloud services, no API keys, complete privacy.

![macOS](https://img.shields.io/badge/macOS-13%2B-blue)
![Swift](https://img.shields.io/badge/Swift-5.9-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Hold-to-record**: Hold the hotkey to record, release to transcribe
- **Local processing**: Uses OpenAI's Whisper model running entirely on your Mac
- **Privacy-first**: No data leaves your device
- **Multiple models**: Choose between Tiny (fast), Base (balanced), or Small (accurate)
- **Multi-language**: Supports 12+ languages with auto-detection
- **Customizable hotkey**: Default is `⌥Space` (Option + Space)

## Installation

### Requirements

- macOS 13.0 or later
- ~100MB disk space (for models)

### Download & Install

1. Download the latest release from [Releases](https://github.com/NOGIT007/speech/releases)
2. Unzip and drag `Speech.app` to `/Applications`
3. Open `Speech.app` from Applications

### Build from Source

```bash
# Clone the repository
git clone https://github.com/NOGIT007/speech.git
cd speech

# Build the app
./build_app.sh

# Install to Applications
cp -R .build/Speech.app /Applications/

# Launch
open /Applications/Speech.app
```

## Setup

On first launch, Speech will request the following permissions:

### 1. Microphone Access

Required to record your voice. Click **Allow** when prompted.

### 2. Accessibility Access

Required for the global hotkey to work.

Go to **System Settings → Privacy & Security → Accessibility** and enable **Speech.app**.

### 3. Download a Model

Click the Speech menu bar icon and select a model to download:

- **Tiny (39 MB)** - Fastest, good for quick notes
- **Base (74 MB)** - Balanced speed and accuracy
- **Small (244 MB)** - Best accuracy

## Usage

1. Click the menu bar icon to see the current status
2. **Hold** `⌥Space` (Option + Space) to start recording
3. Speak clearly
4. **Release** the hotkey to transcribe
5. Press `⌘V` to paste the transcribed text

The transcribed text is automatically copied to your clipboard and a notification shows a preview.

### Changing the Hotkey

1. Click the menu bar icon
2. Select **Settings...**
3. Click the hotkey field and press your preferred key combination

## Troubleshooting

### Hotkey not working

- Ensure Speech.app has **Accessibility** permission enabled
- Try restarting the app after granting permissions

### Transcription is slow

- Try using a smaller model (Tiny or Base)
- Ensure no other heavy processes are running

### Poor transcription quality

- Speak clearly and at a moderate pace
- Try the Small model for better accuracy
- Select the correct language in Settings if not using English

## Privacy

Speech processes all audio locally using Whisper models. No audio or text is ever sent to external servers. Your voice data stays on your device.

## Tech Stack

- **Swift** + **SwiftUI** for the app
- **[SwiftWhisper](https://github.com/exPHAT/SwiftWhisper)** for local Whisper inference
- **[HotKey](https://github.com/soffes/HotKey)** for global hotkey handling

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
