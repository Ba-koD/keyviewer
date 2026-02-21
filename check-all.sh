#!/usr/bin/env bash
set -euo pipefail

echo "[1/3] cargo fmt (auto-format)"
cargo fmt --manifest-path src-tauri/Cargo.toml

echo "[2/3] cargo kclippy"
cargo kclippy

echo "[3/3] cargo ktest"
cargo ktest

echo "All checks passed."