# KeyQueueViewer Portable Build Script
# Builds only the portable version (PyInstaller)

$ErrorActionPreference = "Stop"

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "KeyQueueViewer Portable Build Started" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

# Read version information
$version = Get-Content "version.txt" -Raw
$version = $version.Trim()
Write-Host "Build Version: $version" -ForegroundColor Green

# Auto-setup virtual environment
Write-Host "Setting up virtual environment..." -ForegroundColor Blue

# Check if Python is available
$pythonCmd = $null
if (Test-Path ".venv\Scripts\python.exe") {
    $pythonCmd = ".venv\Scripts\python.exe"
    Write-Host "Using existing virtual environment" -ForegroundColor Green
} elseif (Get-Command "python" -ErrorAction SilentlyContinue) {
    $pythonCmd = "python"
    Write-Host "Using system Python" -ForegroundColor Yellow
} elseif (Get-Command "py" -ErrorAction SilentlyContinue) {
    $pythonCmd = "py"
    Write-Host "Using py launcher" -ForegroundColor Yellow
} else {
    Write-Host "Python not found. Attempting to install..." -ForegroundColor Red
    
    # Try to install Python via winget
    if (Get-Command "winget" -ErrorAction SilentlyContinue) {
        Write-Host "Installing Python 3.11 via winget..." -ForegroundColor Cyan
        winget install -e --id Python.Python.3.11 --accept-package-agreements --accept-source-agreements --silent
        Start-Sleep -Seconds 5
        
        # Refresh PATH and try again
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
        
        if (Get-Command "python" -ErrorAction SilentlyContinue) {
            $pythonCmd = "python"
            Write-Host "Python installed successfully" -ForegroundColor Green
        }
    }
    
    if (-not $pythonCmd) {
        throw "Python installation failed. Please install Python 3.11+ manually."
    }
}

# Create virtual environment if it doesn't exist
if (-not (Test-Path ".venv\Scripts\python.exe")) {
    Write-Host "Creating virtual environment..." -ForegroundColor Cyan
    
    try {
        if ($pythonCmd -eq "py") {
            & py -3 -m venv .venv
        } else {
            & $pythonCmd -m venv .venv
        }
        
        if (Test-Path ".venv\Scripts\python.exe") {
            $pythonCmd = ".venv\Scripts\python.exe"
            Write-Host "Virtual environment created successfully" -ForegroundColor Green
        } else {
            throw "Failed to create virtual environment"
        }
    } catch {
        Write-Host "Failed to create virtual environment: $_" -ForegroundColor Red
        Write-Host "Continuing with system Python..." -ForegroundColor Yellow
        if (Get-Command "python" -ErrorAction SilentlyContinue) {
            $pythonCmd = "python"
        }
    }
}

# Install/upgrade pip and install requirements
Write-Host "Installing dependencies..." -ForegroundColor Blue
try {
    & $pythonCmd -m pip install --upgrade pip
    & $pythonCmd -m pip install -r requirements.txt
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install requirements"
    }
    Write-Host "Dependencies installed successfully" -ForegroundColor Green
} catch {
    Write-Host "Failed to install dependencies: $_" -ForegroundColor Red
    Write-Host "Attempting to install PyInstaller only..." -ForegroundColor Yellow
    
    & $pythonCmd -m pip install PyInstaller
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install PyInstaller"
    }
}

# Clean existing dist folder for this build
$buildName = "KBQV-Portable-$version"
$buildPath = "build\$buildName"
$distPath = "dist\$buildName.exe"

if (Test-Path $buildPath) { 
    Write-Host "Cleaning existing build folder..." -ForegroundColor Blue
    Remove-Item $buildPath -Recurse -Force 
}
if (Test-Path $distPath) { 
    Write-Host "Cleaning existing dist file..." -ForegroundColor Blue
    Remove-Item $distPath -Force 
}

# Create dist folder if it doesn't exist
if (-Not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" -Force | Out-Null
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "Step 1: Portable Version Build (PyInstaller onefile)" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

try {
    # Build Portable version with PyInstaller
    Write-Host "Building Portable version..." -ForegroundColor Blue
    
    & $pythonCmd -m PyInstaller --clean --onefile --noconsole --name $buildName --icon "web/favicon.ico" --add-data "web;web" --distpath "dist" --workpath "build" --collect-all "uvicorn" --collect-all "fastapi" --collect-all "websockets" --hidden-import "keyboard" --hidden-import "win32api" --hidden-import "psutil" --hidden-import "pystray" --hidden-import "PIL" app/launcher.py
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Portable version build successful!" -ForegroundColor Green
    } else {
        throw "Portable version build failed with exit code $LASTEXITCODE"
    }
    
} catch {
    Write-Host "Portable version build failed: $_" -ForegroundColor Red
    Write-Host "Solution:" -ForegroundColor Yellow
    Write-Host "   1. Check if Python is installed and in PATH" -ForegroundColor White
    Write-Host "   2. Check if requirements.txt exists" -ForegroundColor White
    Write-Host "   3. Check if app/launcher.py file exists" -ForegroundColor White
    Write-Host "   4. Check if web/ folder exists with required files" -ForegroundColor White
    Write-Host "   5. Check if web/favicon.ico file exists" -ForegroundColor White
    Write-Host "   6. No antivirus is blocking the build process" -ForegroundColor White
    Write-Host "   7. Try running as Administrator" -ForegroundColor White
    exit 1
}

# Check build result
$portable_path = "dist\KBQV-Portable-$version.exe"
if (Test-Path $portable_path) {
    $file_size = (Get-Item $portable_path).Length
    $file_size_mb = [math]::Round($file_size / 1MB, 2)
    Write-Host "Portable file: $portable_path" -ForegroundColor Green
    Write-Host "File size: $file_size_mb MB" -ForegroundColor Green
    
    # Check if file is executable
    try {
        $fileInfo = Get-Item $portable_path
        Write-Host "File created: $($fileInfo.CreationTime)" -ForegroundColor Green
        Write-Host "File modified: $($fileInfo.LastWriteTime)" -ForegroundColor Green
    } catch {
        Write-Host "WARNING: Could not get file information" -ForegroundColor Yellow
    }
    
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
Write-Host "   2. Run build_all.ps1 for complete build" -ForegroundColor White
Write-Host "===============================================" -ForegroundColor Cyan 