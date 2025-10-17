# Organize Build Files
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Organizing Build Files" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

# Read version
$VERSION = Get-Content version.txt -Raw
$VERSION = $VERSION.Trim()
Write-Host "Version: $VERSION`n" -ForegroundColor Yellow

# Create dist directory structure
$DIST_DIR = "dist\v$VERSION"
$WINDOWS_DIR = "$DIST_DIR\windows"
$MACOS_DIR = "$DIST_DIR\macos"
$LINUX_DIR = "$DIST_DIR\linux"
$PORTABLE_DIR = "$DIST_DIR\portable"

Write-Host "Creating directory structure..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $WINDOWS_DIR -Force | Out-Null
New-Item -ItemType Directory -Path $MACOS_DIR -Force | Out-Null
New-Item -ItemType Directory -Path $LINUX_DIR -Force | Out-Null
New-Item -ItemType Directory -Path $PORTABLE_DIR -Force | Out-Null

# Copy Windows builds
Write-Host "`nOrganizing Windows builds..." -ForegroundColor Yellow
if (Test-Path "src-tauri\target\release\bundle\msi") {
    Copy-Item "src-tauri\target\release\bundle\msi\*.msi" $WINDOWS_DIR -ErrorAction SilentlyContinue
    Write-Host "  ✓ MSI installer copied" -ForegroundColor Green
}
if (Test-Path "src-tauri\target\release\bundle\nsis") {
    Copy-Item "src-tauri\target\release\bundle\nsis\*.exe" $WINDOWS_DIR -ErrorAction SilentlyContinue
    Write-Host "  ✓ NSIS installer copied" -ForegroundColor Green
}

# Copy macOS builds (if exist)
Write-Host "Organizing macOS builds..." -ForegroundColor Yellow
if (Test-Path "src-tauri\target\x86_64-apple-darwin\release\bundle\dmg") {
    Copy-Item "src-tauri\target\x86_64-apple-darwin\release\bundle\dmg\*.dmg" "$MACOS_DIR\" -ErrorAction SilentlyContinue
    Write-Host "  ✓ macOS x86_64 DMG copied" -ForegroundColor Green
}
if (Test-Path "src-tauri\target\aarch64-apple-darwin\release\bundle\dmg") {
    Copy-Item "src-tauri\target\aarch64-apple-darwin\release\bundle\dmg\*.dmg" "$MACOS_DIR\" -ErrorAction SilentlyContinue
    Write-Host "  ✓ macOS ARM64 DMG copied" -ForegroundColor Green
}

# Copy Linux builds (if exist)
Write-Host "Organizing Linux builds..." -ForegroundColor Yellow
if (Test-Path "src-tauri\target\x86_64-unknown-linux-gnu\release\bundle\deb") {
    Copy-Item "src-tauri\target\x86_64-unknown-linux-gnu\release\bundle\deb\*.deb" "$LINUX_DIR\" -ErrorAction SilentlyContinue
    Write-Host "  ✓ Debian package copied" -ForegroundColor Green
}
if (Test-Path "src-tauri\target\x86_64-unknown-linux-gnu\release\bundle\appimage") {
    Copy-Item "src-tauri\target\x86_64-unknown-linux-gnu\release\bundle\appimage\*.AppImage" "$LINUX_DIR\" -ErrorAction SilentlyContinue
    Write-Host "  ✓ AppImage copied" -ForegroundColor Green
}

# Copy portable (if exists)
Write-Host "Organizing Portable build..." -ForegroundColor Yellow
if (Test-Path "dist\KBQV-Portable-$VERSION.zip") {
    Copy-Item "dist\KBQV-Portable-$VERSION.zip" $PORTABLE_DIR -ErrorAction SilentlyContinue
    Write-Host "  ✓ Portable ZIP copied" -ForegroundColor Green
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Organization Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Build files organized in:" -ForegroundColor Yellow
Write-Host "  $DIST_DIR" -ForegroundColor White
Write-Host ""

# List all files
Get-ChildItem -Path $DIST_DIR -Recurse -File | ForEach-Object {
    $relativePath = $_.FullName.Replace((Get-Location).Path + "\", "")
    Write-Host "  $relativePath" -ForegroundColor Cyan
}

Write-Host ""

