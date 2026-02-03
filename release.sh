#!/bin/bash
set -e

# Build the app
./build_app.sh

# Create release zip
cd .build
zip -r Speech.app.zip Speech.app
cd ..

echo ""
echo "=== Release Package Created ==="
echo "File: .build/Speech.app.zip"
echo ""
echo "To create a GitHub release:"
echo "1. Go to https://github.com/NOGIT007/speech/releases/new"
echo "2. Create a new tag (e.g., v1.2.1)"
echo "3. Upload .build/Speech.app.zip as a release asset"
echo "4. Publish the release"
echo ""
echo "Or use gh CLI:"
echo "  gh release create v1.2.1 .build/Speech.app.zip --title 'v1.2.1' --notes 'Release notes'"
