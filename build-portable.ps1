# Build Portable Version for Tauri
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Building Portable Version" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

# Read version from version.txt
$VERSION = Get-Content version.txt -Raw
$VERSION = $VERSION.Trim()

Write-Host "Version: $VERSION" -ForegroundColor Yellow
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

# Create README for portable
$README_CONTENT = @"
# KeyViewer Portable v$VERSION

This is the portable version of KeyViewer - a single executable with all UI files embedded.

## How to Use

1. Run KBQV-Portable-$VERSION.exe
2. The GUI launcher will open
3. Configure port and language
4. Click "Start Server"
5. Use the web control panel or overlay

## Features

- ✅ Single executable - no installation required
- ✅ All UI files embedded - no external dependencies
- ✅ Portable - run from any location
- ✅ No registry changes
- ✅ Minimal antivirus false positives (built with Rust/Tauri)

## Requirements

- Windows 10/11 (64-bit)
- No installation required
- Can be run from USB drive or any folder

## Technical Details

- Built with Tauri 2.0 and Rust
- All assets embedded in executable
- File size: ~8-10 MB
- No Python or Node.js runtime required

For more information, visit: https://github.com/rudgh46/keyviewer
"@

$README_CONTENT | Out-File -FilePath "$PORTABLE_DIR\README.txt" -Encoding UTF8

# Create ZIP
Write-Host "4. Creating ZIP archive..." -ForegroundColor Yellow
$ZIP_NAME = "KBQV-Portable-$VERSION.zip"
if (Test-Path "dist\$ZIP_NAME") {
    Remove-Item "dist\$ZIP_NAME" -Force
}

Compress-Archive -Path "$PORTABLE_DIR\*" -DestinationPath "dist\$ZIP_NAME" -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Portable package created:" -ForegroundColor Yellow
Write-Host "  dist\$ZIP_NAME" -ForegroundColor White
Write-Host ""

