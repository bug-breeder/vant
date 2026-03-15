#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="$HOME/Library/Input Methods"
APP_NAME="VantIME.app"

echo "==> Installing VantIME..."

# Kill existing VantIME process
if pgrep -x VantIME > /dev/null 2>&1; then
    echo "==> Killing existing VantIME process..."
    killall VantIME 2>/dev/null || true
    sleep 1
fi

# Build release
echo "==> Building release..."
cd "$PROJECT_DIR"
make build-release

# Find the built app
BUILD_DIR="$PROJECT_DIR/vant-macos/build/Release"
if [ ! -d "$BUILD_DIR/$APP_NAME" ]; then
    # Try DerivedData path
    BUILD_DIR=$(find ~/Library/Developer/Xcode/DerivedData -path "*/Build/Products/Release/$APP_NAME" -type d 2>/dev/null | head -1)
    BUILD_DIR=$(dirname "$BUILD_DIR" 2>/dev/null || true)
fi

if [ -z "$BUILD_DIR" ] || [ ! -d "$BUILD_DIR/$APP_NAME" ]; then
    echo "ERROR: Could not find built $APP_NAME"
    echo "Build the project first with 'make build-release'"
    exit 1
fi

# Install
mkdir -p "$INSTALL_DIR"
if [ -d "$INSTALL_DIR/$APP_NAME" ]; then
    echo "==> Removing old installation..."
    rm -rf "$INSTALL_DIR/$APP_NAME"
fi

echo "==> Copying $APP_NAME to $INSTALL_DIR..."
cp -R "$BUILD_DIR/$APP_NAME" "$INSTALL_DIR/"

echo ""
echo "==> VantIME installed successfully!"
echo "==> To activate: Log out and back in, then go to"
echo "    System Settings > Keyboard > Input Sources > + > search 'Vant'"
