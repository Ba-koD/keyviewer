#!/bin/bash
# macOS Signed Build Script
# Builds and signs the app with "KeyViewer Signing" certificate

set -e

SIGNING_IDENTITY="KeyViewer Signing"

echo "==================================="
echo "macOS Signed Build"
echo "==================================="
echo ""

# Check certificate exists (without -v to include untrusted self-signed certs)
if ! security find-identity -p codesigning | grep -q "$SIGNING_IDENTITY"; then
    echo "❌ Certificate '$SIGNING_IDENTITY' not found!"
    echo ""
    echo "Run first: ./scripts/macos-cert-setup.sh <password>"
    echo "Or import: ./scripts/macos-cert-import.sh <p12-file> <password>"
    exit 1
fi

echo "✓ Certificate found: $SIGNING_IDENTITY"
echo ""

# Build
echo "[1/3] Building..."
cargo tauri build

# Sign .app
echo ""
echo "[2/3] Signing .app bundle..."
for app in src-tauri/target/release/bundle/macos/*.app; do
    if [ -d "$app" ]; then
        echo "  Signing: $app"
        codesign --force --deep --sign "$SIGNING_IDENTITY" "$app"
    fi
done

# Sign .dmg
echo ""
echo "[3/3] Signing .dmg..."
for dmg in src-tauri/target/release/bundle/dmg/*.dmg; do
    if [ -f "$dmg" ]; then
        echo "  Signing: $dmg"
        codesign --force --sign "$SIGNING_IDENTITY" "$dmg"
    fi
done

# Verify
echo ""
echo "==================================="
echo "Verifying signatures..."
echo "==================================="

for app in src-tauri/target/release/bundle/macos/*.app; do
    if [ -d "$app" ]; then
        echo ""
        echo "Checking: $app"
        codesign --verify --verbose "$app" && echo "✓ Valid signature" || echo "❌ Invalid signature"
    fi
done

echo ""
echo "==================================="
echo "✅ Build complete!"
echo "==================================="
echo ""
echo "Output files:"
ls -lh src-tauri/target/release/bundle/macos/*.app 2>/dev/null || true
ls -lh src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null || true

