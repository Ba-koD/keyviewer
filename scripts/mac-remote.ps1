#!/usr/bin/env pwsh
# English comments only.
# Remote macOS build/dev helper for PowerShell on Windows.

param(
  [ValidateSet('setup','sync','dev','build')]
  [string]$Action,
  [string[]]$ExtraArgs
)

$MAC_HOST = $env:MAC_HOST; if (-not $MAC_HOST) { $MAC_HOST = 'ssh.rnen.kr' }
$MAC_PORT = $env:MAC_PORT; if (-not $MAC_PORT) { $MAC_PORT = '2222' }
$MAC_USER = $env:MAC_USER; if (-not $MAC_USER) { $MAC_USER = 'rudgh' }
$MAC_DIR  = $env:MAC_DIR;  if (-not $MAC_DIR)  { $MAC_DIR  = '/Users/rudgh/work/keyviewer' }
$MAC_PASS = $env:MAC_PASS

# Load .env if present (simple parser: KEY=VALUE lines)
$envPath = Join-Path (Resolve-Path ..).Path '.env'
if (Test-Path $envPath) {
  Get-Content $envPath | ForEach-Object {
    if ($_ -match '^[A-Za-z_][A-Za-z0-9_]*=') {
      $k,$v = $_ -split '=',2
      if (-not [string]::IsNullOrWhiteSpace($k)) {
        $trimmed = $v.Trim()
        [Environment]::SetEnvironmentVariable($k, $trimmed)
      }
    }
  }
  if ($env:MAC_HOST) { $MAC_HOST = $env:MAC_HOST }
  if ($env:MAC_PORT) { $MAC_PORT = $env:MAC_PORT }
  if ($env:MAC_USER) { $MAC_USER = $env:MAC_USER }
  if ($env:MAC_DIR)  { $MAC_DIR  = $env:MAC_DIR }
  if ($env:MAC_PASS) { $MAC_PASS = $env:MAC_PASS }
}

function Show-Usage {
  @"
Usage: mac-remote.ps1 -Action <setup|sync|dev|build> [-- ExtraArgs]

Env overrides: MAC_HOST, MAC_PORT, MAC_USER, MAC_DIR
Examples:
  .\scripts\mac-remote.ps1 -Action sync
  .\scripts\mac-remote.ps1 -Action build -- --release
"@
}

function Invoke-Remote([string]$Cmd) {
  if ($MAC_PASS -and (Get-Command sshpass -ErrorAction SilentlyContinue)) {
    sshpass -p $MAC_PASS ssh -p $MAC_PORT "${MAC_USER}@${MAC_HOST}" $Cmd
  } else {
    ssh -p $MAC_PORT "${MAC_USER}@${MAC_HOST}" $Cmd
  }
}

function Test-Rsync {
  if (-not (Get-Command rsync -ErrorAction SilentlyContinue)) {
    Write-Error 'rsync not found. Install rsync or use WSL bash version scripts/mac-remote.sh'
    exit 1
  }
}

function Invoke-Setup {
  $remote = @"
set -euo pipefail
if ! command -v brew >/dev/null 2>&1; then
  echo 'Homebrew missing. Please install manually.'
fi
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="\$HOME/.cargo/bin:\$PATH"
fi
export PATH="\$HOME/.cargo/bin:\$PATH"
rustup default stable; rustup update
rustup target add aarch64-apple-darwin x86_64-apple-darwin || true
if ! command -v cargo-tauri >/dev/null 2>&1; then
  cargo install tauri-cli@2 || true
fi
mkdir -p "$MAC_DIR"
echo 'Remote setup completed (best-effort). If Xcode CLT is missing, run: xcode-select --install'
"@
  Invoke-Remote $remote
}

function Invoke-Sync {
  Test-Rsync
  $root = (Resolve-Path ..).Path
  if ($MAC_PASS -and (Get-Command sshpass -ErrorAction SilentlyContinue)) {
    rsync -avz --delete --exclude .git --exclude target --exclude src-tauri/target -e "sshpass -p $MAC_PASS ssh -p $MAC_PORT" "$root/" "${MAC_USER}@${MAC_HOST}:${MAC_DIR}"
  } else {
    rsync -avz --delete --exclude .git --exclude target --exclude src-tauri/target -e "ssh -p $MAC_PORT" "$root/" "${MAC_USER}@${MAC_HOST}:${MAC_DIR}"
  }
}

function Invoke-Dev {
  Invoke-Remote "cd '$MAC_DIR' && RUST_BACKTRACE=1 cargo tauri dev $($ExtraArgs -join ' ')"
}

function Invoke-Build {
  Invoke-Remote "cd '$MAC_DIR' && cargo tauri build --bundles app,dmg $($ExtraArgs -join ' ')"
}

switch ($Action) {
  'setup' { Invoke-Setup }
  'sync'  { Invoke-Sync }
  'dev'   { Invoke-Dev }
  'build' { Invoke-Build }
  Default { Show-Usage; exit 1 }
}


