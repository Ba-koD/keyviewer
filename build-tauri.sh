#!/bin/bash

# Tauri Build Script for KeyQueueViewer (Linux/macOS)
# This script builds the Tauri application

echo "========================================"
echo "  KeyQueueViewer - Tauri Build Script  "
echo "========================================"
echo ""

# Check if Rust is installed
echo "Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed!"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(cargo --version)
echo "✓ Rust found: $RUST_VERSION"

# Check if Tauri CLI is installed
echo ""
echo "Checking Tauri CLI..."
if ! command -v cargo-tauri &> /dev/null; then
    echo "Tauri CLI not found. Installing..."
    cargo install tauri-cli
    if [ $? -ne 0 ]; then
        echo "Error: Failed to install Tauri CLI!"
        exit 1
    fi
fi
echo "✓ Tauri CLI found"

# Format code before build
echo ""
echo "Formatting code..."
cd src-tauri
cargo fmt
cd ..
echo "✓ Code formatted"

# Read version from version.txt
echo ""
echo "Reading version..."
VERSION=$(cat version.txt | tr -d '\n\r')
echo "✓ Version: $VERSION"

# Build the application
echo ""
echo "Building Tauri application..."
echo "This may take a few minutes on first build..."

cargo tauri build

if [ $? -ne 0 ]; then
    echo ""
    echo "Build failed!"
    exit 1
fi

echo ""
echo "========================================"
echo "  Build completed successfully!        "
echo "========================================"

# List generated files
echo ""
echo "Generated files:"
echo ""

BUNDLE_PATH="src-tauri/target/release/bundle"

if [ "$(uname)" == "Darwin" ]; then
    # macOS
    if [ -d "$BUNDLE_PATH/dmg" ]; then
        echo "DMG Files:"
        find "$BUNDLE_PATH/dmg" -name "*.dmg" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo "  - $(basename "$1") ($SIZE)"' _ {} \;
    fi
    if [ -d "$BUNDLE_PATH/macos" ]; then
        echo ""
        echo "App Bundles:"
        find "$BUNDLE_PATH/macos" -name "*.app" -maxdepth 1 -exec bash -c 'SIZE=$(du -sh "$1" | cut -f1); echo "  - $(basename "$1") ($SIZE)"' _ {} \;
    fi
elif [ "$(uname)" == "Linux" ]; then
    # Linux
    if [ -d "$BUNDLE_PATH/deb" ]; then
        echo "DEB Packages:"
        find "$BUNDLE_PATH/deb" -name "*.deb" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo "  - $(basename "$1") ($SIZE)"' _ {} \;
    fi
    if [ -d "$BUNDLE_PATH/appimage" ]; then
        echo ""
        echo "AppImages:"
        find "$BUNDLE_PATH/appimage" -name "*.AppImage" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo "  - $(basename "$1") ($SIZE)"' _ {} \;
    fi
fi

echo ""
echo "All files are located in:"
echo "  $BUNDLE_PATH"

echo ""
echo "✓ Done!"

