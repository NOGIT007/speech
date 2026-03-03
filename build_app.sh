#!/bin/bash
set -e

APP_NAME="Speech"
VERSION="3.5.0"

echo "Building $APP_NAME v$VERSION with Tauri..."

# Build the Tauri app (handles both frontend + Rust backend)
bun run tauri build

# Find the built .app bundle
APP_BUNDLE="src-tauri/target/release/bundle/macos/$APP_NAME.app"

if [ ! -d "$APP_BUNDLE" ]; then
    echo "Error: Expected bundle not found at $APP_BUNDLE"
    exit 1
fi

# Copy to .build for backwards compatibility
mkdir -p .build
rm -rf ".build/$APP_NAME.app"
cp -R "$APP_BUNDLE" ".build/$APP_NAME.app"

# Kill running instance (if any), install to /Applications, and launch
echo "Installing $APP_NAME v$VERSION..."
pkill -x "$APP_NAME" 2>/dev/null && sleep 1 || true
rm -rf "/Applications/$APP_NAME.app"
cp -R "$APP_BUNDLE" "/Applications/$APP_NAME.app"

# Clear stale TCC entries (ad-hoc signing changes code requirement each build)
BUNDLE_ID="com.kennetkusk.speech"
tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null || true
tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null || true

open "/Applications/$APP_NAME.app"

echo ""
echo "$APP_NAME v$VERSION installed and launched."
