#!/bin/bash
set -e

echo "Building Speech v1.2.1..."
./build_app.sh

echo ""
echo "Quitting existing Speech app..."
osascript -e 'quit app "Speech"' 2>/dev/null || true
sleep 1

echo "Installing to /Applications..."
cp -R .build/Speech.app /Applications/

echo "Launching Speech..."
open /Applications/Speech.app

echo ""
echo "✓ Speech v1.2.1 installed and running"
echo "Test: Hold ⌥Space in a terminal to test auto-insert"
