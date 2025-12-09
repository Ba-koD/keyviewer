#!/usr/bin/env bash
set -euo pipefail

echo "========================================"
echo "  Simulating GitHub Actions Linux Build"
echo "========================================"
echo
echo "Environment Info:"
rustc --version
cargo --version
cargo tauri --version
echo
echo "Pre-flight icon check"
if [ -f /app/src-tauri/icons/icon.png ]; then
  # Ensure square PNG for AppImage
  mogrify -resize 512x512! -alpha on -background none /app/src-tauri/icons/icon.png || true
else
  echo "icon.png missing"
fi

echo "Building Tauri app..."
cd /app/src-tauri
cargo tauri build --verbose

# If AppImage exists but host can't open from container, copy to a shared dist folder too
mkdir -p /app/dist/linux || true
if compgen -G "/app/src-tauri/target/release/bundle/appimage/*.AppImage" > /dev/null; then
  cp -f /app/src-tauri/target/release/bundle/appimage/*.AppImage /app/dist/linux/ || true
fi
if compgen -G "/app/src-tauri/target/release/bundle/deb/*.deb" > /dev/null; then
  cp -f /app/src-tauri/target/release/bundle/deb/*.deb /app/dist/linux/ || true
fi

