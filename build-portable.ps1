# Build Portable Version for Tauri
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Building Portable Version" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

# Format code before build
Write-Host "0. Formatting code..." -ForegroundColor Yellow
cd src-tauri
cargo fmt
cd ..
Write-Host "Code formatted!" -ForegroundColor Green
Write-Host ""

# Read version from version.txt
$VERSION = Get-Content version.txt -Raw
$VERSION = $VERSION.Trim()

Write-Host "Version: $VERSION" -ForegroundColor Yellow
Write-Host ""

# Check and install Tauri CLI if needed
Write-Host "0. Checking Tauri CLI..." -ForegroundColor Yellow
$tauriVersion = cargo tauri --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri CLI not found, installing..." -ForegroundColor Yellow
    cargo install tauri-cli --version "^2.0.0"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install Tauri CLI!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Tauri CLI installed!" -ForegroundColor Green
} else {
    Write-Host "Tauri CLI found: $tauriVersion" -ForegroundColor Green
}
Write-Host ""

# Ensure icon.png exists
Write-Host "0.5. Checking icon.png..." -ForegroundColor Yellow
if (-not (Test-Path "src-tauri\icons\icon.png")) {
    Write-Host "icon.png not found, generating..." -ForegroundColor Yellow
    & .\convert-icon.ps1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to generate icon.png!" -ForegroundColor Red
        exit 1
    }
}
Write-Host "icon.png ready!" -ForegroundColor Green
Write-Host ""

# Build Tauri release
Write-Host "1. Building Tauri release..." -ForegroundColor Yellow
cd src-tauri
cargo tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
cd ..
Write-Host "Build complete!" -ForegroundColor Green

# Create portable directory
$PORTABLE_DIR = "dist\KBQV-Portable-$VERSION"
Write-Host "`n2. Creating portable directory..." -ForegroundColor Yellow
if (Test-Path $PORTABLE_DIR) {
    Remove-Item -Path $PORTABLE_DIR -Recurse -Force
}
New-Item -ItemType Directory -Path $PORTABLE_DIR -Force | Out-Null

# Copy executable (UI files are already embedded in the exe)
Write-Host "3. Copying executable..." -ForegroundColor Yellow
Copy-Item "src-tauri\target\release\keyviewer.exe" "$PORTABLE_DIR\KBQV-Portable-$VERSION.exe"

# Add administrator manifest to executable
Write-Host "4. Adding administrator manifest..." -ForegroundColor Yellow
$mtPath = "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\mt.exe"
$mtExe = Get-ChildItem -Path $mtPath -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName

if ($mtExe) {
    Write-Host "Found mt.exe at: $mtExe" -ForegroundColor Green
    & $mtExe -manifest "src-tauri\app.manifest" -outputresource:"$PORTABLE_DIR\KBQV-Portable-$VERSION.exe;#1"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Administrator manifest added successfully!" -ForegroundColor Green
    } else {
        Write-Host "Warning: Failed to add manifest (error code: $LASTEXITCODE)" -ForegroundColor Yellow
        Write-Host "The app will still work, but won't automatically request admin privileges" -ForegroundColor Yellow
    }
} else {
    Write-Host "Warning: mt.exe not found. Skipping manifest embedding." -ForegroundColor Yellow
    Write-Host "The app will still work, but you'll need to run it as administrator manually." -ForegroundColor Yellow
    Write-Host "To enable auto-admin prompt, install Windows SDK: https://developer.microsoft.com/windows/downloads/windows-sdk/" -ForegroundColor Cyan
}

# Create README for portable
Write-Host "5. Creating README..." -ForegroundColor Yellow
$README_CONTENT = @"
KeyViewer Portable v$VERSION
========================================

This is the portable version of KeyViewer - a single executable with all UI files embedded.

HOW TO USE
----------
1. Run KBQV-Portable-$VERSION.exe (Administrator privileges will be requested)
2. The GUI launcher will open
3. Configure port and language
4. Click "Start Server"
5. Use the web control panel or overlay

FEATURES
--------
- Single executable - no installation required
- All UI files embedded - no external dependencies
- Portable - run from any location
- No registry changes (except user settings)
- Minimal antivirus false positives (built with Rust/Tauri)

REQUIREMENTS
------------
- Windows 10/11 (64-bit)
- Administrator privileges (required for keyboard hook to work properly)
- No installation required
- Can be run from USB drive or any folder

IMPORTANT NOTES
---------------
ADMINISTRATOR PRIVILEGES REQUIRED:
This program requests administrator privileges to properly capture keyboard 
input across all applications. This is necessary for the global keyboard 
hook functionality.

When you run the program, Windows will show a UAC (User Account Control) 
prompt asking for permission. Click "Yes" to allow the program to run.

TECHNICAL DETAILS
-----------------
- Built with Tauri 2.0 and Rust
- All assets embedded in executable
- File size: ~8-10 MB
- No Python or Node.js runtime required

For more information, visit: https://github.com/Ba-koD/keyviewer
"@

$README_CONTENT | Out-File -FilePath "$PORTABLE_DIR\README.txt" -Encoding ASCII

# Copy EXE to dist root for easy download
Write-Host "6. Copying EXE to dist root..." -ForegroundColor Yellow
Copy-Item "$PORTABLE_DIR\KBQV-Portable-$VERSION.exe" "dist\KBQV-Portable-$VERSION.exe" -Force

# Create ZIP (optional, includes README)
Write-Host "7. Creating ZIP archive (with README)..." -ForegroundColor Yellow
$ZIP_NAME = "KBQV-Portable-$VERSION.zip"
if (Test-Path "dist\$ZIP_NAME") {
    Remove-Item "dist\$ZIP_NAME" -Force
}

Compress-Archive -Path "$PORTABLE_DIR\*" -DestinationPath "dist\$ZIP_NAME" -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Portable files created:" -ForegroundColor Yellow
Write-Host "  EXE only: dist\KBQV-Portable-$VERSION.exe" -ForegroundColor White
Write-Host "  ZIP with README: dist\$ZIP_NAME" -ForegroundColor White
Write-Host ""

