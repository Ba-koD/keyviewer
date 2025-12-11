# Tauri Build Script for KeyQueueViewer
# This script builds the Tauri application for Windows

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyQueueViewer - Tauri Build Script  " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed!" -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

$rustVersion = cargo --version
Write-Host "✓ Rust found: $rustVersion" -ForegroundColor Green

# Check if Tauri CLI is installed
Write-Host "`nChecking Tauri CLI..." -ForegroundColor Yellow
if (!(Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    Write-Host "Tauri CLI not found. Installing..." -ForegroundColor Yellow
    cargo install tauri-cli
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to install Tauri CLI!" -ForegroundColor Red
        exit 1
    }
}
Write-Host "✓ Tauri CLI found" -ForegroundColor Green

# Format code before build
Write-Host "`nFormatting code..." -ForegroundColor Yellow
cd src-tauri
cargo fmt
cd ..
Write-Host "✓ Code formatted" -ForegroundColor Green

# Read version from version.txt
Write-Host "`nReading version..." -ForegroundColor Yellow
$version = Get-Content "version.txt" -Raw | ForEach-Object { $_.Trim() }
Write-Host "✓ Version: $version" -ForegroundColor Green

# Build the application
Write-Host "`nBuilding Tauri application..." -ForegroundColor Yellow
Write-Host "This may take a few minutes on first build..." -ForegroundColor Gray

cargo tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    exit 1
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Build completed successfully!         " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

# List generated files
Write-Host "`nGenerated files:" -ForegroundColor Yellow
Write-Host ""

$bundlePath = "src-tauri\target\release\bundle"

if (Test-Path "$bundlePath\msi") {
    Write-Host "MSI Installers:" -ForegroundColor Cyan
    Get-ChildItem "$bundlePath\msi\*.msi" | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  - $($_.Name) ($size MB)" -ForegroundColor White
    }
}

if (Test-Path "$bundlePath\nsis") {
    Write-Host "`nNSIS Installers:" -ForegroundColor Cyan
    Get-ChildItem "$bundlePath\nsis\*.exe" | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  - $($_.Name) ($size MB)" -ForegroundColor White
    }
}

Write-Host "`nAll files are located in:" -ForegroundColor Yellow
Write-Host "  $bundlePath" -ForegroundColor White

Write-Host "`n✓ Done!" -ForegroundColor Green

