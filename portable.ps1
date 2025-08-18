# KeyQueueViewer Portable Build Script
# Builds only the portable version (PyInstaller)

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "KeyQueueViewer Portable Build Started" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

# Read version information
$version = Get-Content "version.txt" -Raw
$version = $version.Trim()
Write-Host "Build Version: $version" -ForegroundColor Green

# Check and activate virtual environment
if (Test-Path ".venv\Scripts\Activate.ps1") {
    Write-Host "Activating virtual environment..." -ForegroundColor Blue
    & ".venv\Scripts\Activate.ps1"
} else {
    Write-Host "Virtual environment not found. Using system Python." -ForegroundColor Yellow
}

# Clean existing dist folder
if (Test-Path "dist") {
    Write-Host "Cleaning existing dist folder..." -ForegroundColor Blue
    Remove-Item "dist" -Recurse -Force
}

# Create dist folder
New-Item -ItemType Directory -Path "dist" -Force | Out-Null

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Step 1: Portable Version Build (PyInstaller)" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

try {
    # Build Portable version with PyInstaller
    Write-Host "Building Portable version..." -ForegroundColor Blue
    pyinstaller --onefile --windowed --name "KBQV-Portable-$version" app/launcher.py
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Portable version build successful!" -ForegroundColor Green
    } else {
        throw "Portable version build failed"
    }
    
} catch {
    Write-Host "Portable version build failed: $_" -ForegroundColor Red
    Write-Host "Solution:" -ForegroundColor Yellow
    Write-Host "   1. Install PyInstaller: pip install pyinstaller" -ForegroundColor White
    Write-Host "   2. Check virtual environment activation" -ForegroundColor White
    Write-Host "   3. Check Python path" -ForegroundColor White
    Write-Host "   4. Check app/launcher.py file exists" -ForegroundColor White
    exit 1
}

# Check build result
$portable_path = "dist\KBQV-Portable-$version.exe"
if (Test-Path $portable_path) {
    $file_size = (Get-Item $portable_path).Length
    $file_size_mb = [math]::Round($file_size / 1MB, 2)
    Write-Host "Portable file: $portable_path" -ForegroundColor Green
    Write-Host "File size: $file_size_mb MB" -ForegroundColor Green
} else {
    Write-Host "Portable file not found!" -ForegroundColor Red
    exit 1
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "PORTABLE BUILD COMPLETE!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Built files location: dist/" -ForegroundColor Blue
Write-Host "Created file:" -ForegroundColor Blue
Write-Host "   KBQV-Portable-$version.exe (Portable - onefile)" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "   1. Test Portable: dist\KBQV-Portable-$version.exe" -ForegroundColor White
Write-Host "   2. Push to GitHub for automated release" -ForegroundColor White
Write-Host "===============================================" -ForegroundColor Cyan 