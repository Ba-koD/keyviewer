#!/usr/bin/env bash
set -euo pipefail

echo "[1/3] cargo kfmt"
cargo kfmt

echo "[2/3] cargo kclippy"
cargo kclippy

echo "[3/3] cargo ktest"
cargo ktest

echo "All checks passed."
