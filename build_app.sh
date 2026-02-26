#!/bin/bash
set -e

APP_NAME="Speech"
VERSION="3.0.0"

echo "Building $APP_NAME v$VERSION with Tauri..."

# Build the Tauri app (handles both frontend + Rust backend)
bun run tauri build

# Find the built .app bundle
APP_BUNDLE="src-tauri/target/release/bundle/macos/$APP_NAME.app"

if [ -d "$APP_BUNDLE" ]; then
    # Also copy to .build for backwards compatibility
    mkdir -p .build
    rm -rf ".build/$APP_NAME.app"
    cp -R "$APP_BUNDLE" ".build/$APP_NAME.app"

    echo ""
    echo "Built $APP_BUNDLE"
    echo "Also copied to .build/$APP_NAME.app"
    echo ""
    echo "To install: cp -R .build/$APP_NAME.app /Applications/"
    echo "Run: open /Applications/$APP_NAME.app"
else
    echo "Error: Expected bundle not found at $APP_BUNDLE"
    exit 1
fi
