#!/usr/bin/env bash
set -euo pipefail

# English comments only.
# Remote macOS build/dev helper.
# Defaults can be overridden via environment variables.

MAC_HOST="${MAC_HOST:-ssh.rnen.kr}"
MAC_PORT="${MAC_PORT:-2222}"
MAC_USER="${MAC_USER:-rudgh}"
MAC_DIR="${MAC_DIR:-/Users/rudgh/work/keyviewer}"
MAC_PASS="${MAC_PASS:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

load_dotenv() {
  # Load .env without expanding ~ or variables. English comments only.
  local env_file="${ROOT_DIR}/.env"
  if [ -f "$env_file" ]; then
    while IFS= read -r line; do
      # Skip comments and empty lines
      case "$line" in
        ''|'#'*) continue ;;
      esac
      # Split on first '=' only
      local key="${line%%=*}"
      local val="${line#*=}"
      key="${key## }"; key="${key%% }"
      # Preserve raw value; if not quoted, wrap in double quotes to avoid tilde expansion
      case "$key" in
        MAC_HOST) MAC_HOST="${val%$'\r'}" ;;
        MAC_PORT) MAC_PORT="${val%$'\r'}" ;;
        MAC_USER) MAC_USER="${val%$'\r'}" ;;
        MAC_DIR)  MAC_DIR="${val%$'\r'}" ;;
        MAC_PASS) MAC_PASS="${val%$'\r'}" ;;
      esac
    done < "$env_file"
  fi
}

ssh_cmd() {
  # Use sshpass if password is provided; otherwise use key-based auth.
  if [ -n "${MAC_PASS}" ] && command -v sshpass >/dev/null 2>&1; then
    sshpass -p "${MAC_PASS}" ssh -p "${MAC_PORT}" "$@"
  else
    ssh -p "${MAC_PORT}" "$@"
  fi
}

rsync_over_ssh() {
  if [ -n "${MAC_PASS}" ] && command -v sshpass >/dev/null 2>&1; then
    rsync -e "sshpass -p ${MAC_PASS} ssh -p ${MAC_PORT}" "$@"
  else
    rsync -e "ssh -p ${MAC_PORT}" "$@"
  fi
}

ssh_remote() {
  ssh_cmd "${MAC_USER}@${MAC_HOST}" "$@"
}

usage() {
  cat <<EOF
Usage: $(basename "$0") <setup|sync|dev|build> [extra args]

Actions:
  setup   Install toolchain on remote mac (idempotent)
  sync    Rsync project to remote directory
  dev     Run 'cargo tauri dev' on remote
  build   Run 'cargo tauri build --bundles app,dmg' on remote

Environment overrides:
  MAC_HOST, MAC_PORT, MAC_USER, MAC_DIR

Examples:
  MAC_DIR=/Users/${MAC_USER}/work/keyviewer $(basename "$0") sync
  $(basename "$0") build
EOF
}

ensure_rsync() {
  if ! command -v rsync >/dev/null 2>&1; then
    echo "rsync not found. Install rsync on this machine (e.g. 'sudo apt install rsync')." >&2
    exit 1
  fi
}

ssh_remote_heredoc() {
  # $1..$n: command for remote shell (bash -s is implied). Reads script body from stdin.
  if [ -n "${MAC_PASS}" ] && command -v sshpass >/dev/null 2>&1; then
    sshpass -p "${MAC_PASS}" ssh -p "${MAC_PORT}" "${MAC_USER}@${MAC_HOST}" env MAC_DIR="${MAC_DIR}" bash -s
  else
    ssh -p "${MAC_PORT}" "${MAC_USER}@${MAC_HOST}" env MAC_DIR="${MAC_DIR}" bash -s
  fi
}

do_setup() {
  # This tries best-effort non-interactive setup. Some steps (Xcode CLT) may require manual action.
  ssh_remote_heredoc <<'__KV_REMOTE__'
set -euo pipefail
echo 'Remote: checking toolchain'

# Try to install Command Line Tools if missing (best-effort)
if ! xcode-select -p >/dev/null 2>&1; then
  echo 'Attempting to install Xcode Command Line Tools (best-effort)'
  label=$(softwareupdate -l 2>/dev/null | grep -F 'Command Line Tools' | tail -n1 | cut -d'*' -f2 | sed -e 's/^ *//' || true)
  if [ -n "${label:-}" ]; then
    sudo softwareupdate -i "${label}" -v || true
  fi
  sudo xcodebuild -license accept || true
fi

if ! command -v brew >/dev/null 2>&1; then
  echo 'Homebrew missing. Please install manually if needed.'
fi

# Resolve MAC_DIR (~ expansion and default)
if [ -z "${MAC_DIR:-}" ]; then
  MAC_DIR="$HOME/keyviewer"
fi
MAC_DIR_EXPANDED="${MAC_DIR/#\~/$HOME}"
# If absolute path looks like Linux home, rewrite to current macOS HOME
case "$MAC_DIR_EXPANDED" in
  /home/*)
    MAC_DIR_EXPANDED="$HOME${MAC_DIR_EXPANDED#*/$USER}"
    ;;
esac

# Debug prints to help diagnose path issues (no secrets printed)
echo "Remote: HOME=$HOME"
echo "Remote: MAC_DIR=$MAC_DIR"
echo "Remote: MAC_DIR_EXPANDED=$MAC_DIR_EXPANDED"

if ! command -v rustup >/dev/null 2>&1; then
  echo 'Installing rustup...'
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
  if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
fi

export PATH="$HOME/.cargo/bin:$PATH"
if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
if ! grep -q '\$HOME/.cargo/bin' "$HOME/.zprofile" 2>/dev/null; then
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.zprofile"
fi

rustup show >/dev/null 2>&1 || true
rustup default stable; rustup update
rustup target add aarch64-apple-darwin x86_64-apple-darwin || true

if ! command -v cargo-tauri >/dev/null 2>&1; then
  cargo install --locked tauri-cli --version '^2' || true
fi

mkdir -p -- "$MAC_DIR_EXPANDED"
echo 'Remote setup completed (best-effort). If Xcode CLT is missing, run: xcode-select --install'
__KV_REMOTE__
}

do_sync() {
  ensure_rsync
  rsync_over_ssh -avz --delete \
    --exclude .git --exclude target --exclude src-tauri/target \
    "${ROOT_DIR}/" "${MAC_USER}@${MAC_HOST}:${MAC_DIR}"
}

do_dev() {
  ssh_remote_heredoc <<'__KV_REMOTE__'
set -euo pipefail
if [ -z "${MAC_DIR:-}" ]; then MAC_DIR="$HOME/keyviewer"; fi
MAC_DIR_EXPANDED="${MAC_DIR/#\~/$HOME}"
case "$MAC_DIR_EXPANDED" in
  /home/*) MAC_DIR_EXPANDED="$HOME${MAC_DIR_EXPANDED#*/$USER}" ;;
esac
cd "$MAC_DIR_EXPANDED"
RUST_BACKTRACE=1 cargo tauri dev
__KV_REMOTE__
}

do_build() {
  ssh_remote_heredoc <<'__KV_REMOTE__'
set -euo pipefail
if [ -z "${MAC_DIR:-}" ]; then MAC_DIR="$HOME/keyviewer"; fi
MAC_DIR_EXPANDED="${MAC_DIR/#\~/$HOME}"
case "$MAC_DIR_EXPANDED" in
  /home/*) MAC_DIR_EXPANDED="$HOME${MAC_DIR_EXPANDED#*/$USER}" ;;
esac
cd "$MAC_DIR_EXPANDED"
cargo tauri build --bundles app,dmg

# Copy artifacts into dist/macos for convenience
OUT_DMG_DIR="src-tauri/target/release/bundle/dmg"
OUT_APP_DIR="src-tauri/target/release/bundle/macos"
DEST_DIR="$MAC_DIR_EXPANDED/dist/macos"
mkdir -p "$DEST_DIR"
if compgen -G "$OUT_DMG_DIR/*.dmg" > /dev/null; then
  cp -f "$OUT_DMG_DIR"/*.dmg "$DEST_DIR"/ || true
fi
if compgen -G "$OUT_APP_DIR/*.app" > /dev/null; then
  # Copy .app as a bundle (preserve structure)
  rsync -a "$OUT_APP_DIR"/*.app "$DEST_DIR"/ || true
fi
echo "Copied macOS artifacts to $DEST_DIR"

# Best-effort: if DMG was auto-mounted during bundling, detach to avoid Finder popping up
if [ -d "/Volumes/KeyQueueViewer" ]; then
  hdiutil detach "/Volumes/KeyQueueViewer" -force || true
fi
__KV_REMOTE__
}

load_dotenv

ACTION="${1:-}"
shift || true

case "${ACTION}" in
  setup) do_setup ;;
  sync) do_sync ;;
  dev) do_dev "$@" ;;
  build) do_build "$@" ;;
  *) usage; exit 1 ;;
esac


