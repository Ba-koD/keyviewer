# KeyQueueViewer onedir Build Script
# Builds only the onedir version (cx_Freeze, no compression)

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "KeyQueueViewer onedir Build Started" -ForegroundColor Yellow
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
Write-Host "Step 1: onedir Build (cx_Freeze)" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

try {
    # Build onedir with cx_Freeze
    Write-Host "Building onedir..." -ForegroundColor Blue
    python setup_main.py build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "onedir build successful!" -ForegroundColor Green
    } else {
        throw "onedir build failed"
    }
    
} catch {
    Write-Host "onedir build failed: $_" -ForegroundColor Red
    Write-Host "Solution:" -ForegroundColor Yellow
    Write-Host "   1. Install cx_Freeze: pip install cx_Freeze" -ForegroundColor White
    Write-Host "   2. Check virtual environment activation" -ForegroundColor White
    Write-Host "   3. Check Python path" -ForegroundColor White
    Write-Host "   4. Check setup_main.py file exists" -ForegroundColor White
    exit 1
}

# Check build result
$build_folder = "KBQV-v$version"
$build_path = "dist\$build_folder"
if (Test-Path $build_path) {
    # Check folder contents
    $files = Get-ChildItem $build_path -Recurse | Measure-Object
    $folder_size = (Get-ChildItem $build_path -Recurse | Measure-Object -Property Length -Sum).Sum
    $folder_size_mb = [math]::Round($folder_size / 1MB, 2)
    
    Write-Host "onedir folder: $build_path" -ForegroundColor Green
    Write-Host "Folder size: $folder_size_mb MB" -ForegroundColor Green
    Write-Host "File count: $($files.Count)" -ForegroundColor Green
    
    # Check main files
    Write-Host "Main files:" -ForegroundColor Blue
    Get-ChildItem $build_path -Name | ForEach-Object {
        Write-Host "   $_" -ForegroundColor White
    }
    
} else {
    Write-Host "onedir folder not found!" -ForegroundColor Red
    exit 1
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "ONEDIR BUILD COMPLETE!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Built files location: dist/" -ForegroundColor Blue
Write-Host "Created folder:" -ForegroundColor Blue
Write-Host "   $build_folder (onedir - no compression)" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "   1. Test onedir: dist\$build_folder\KBQV-v$version.exe" -ForegroundColor White
Write-Host "   2. Push to GitHub for automated release" -ForegroundColor White
Write-Host "===============================================" -ForegroundColor Cyan 