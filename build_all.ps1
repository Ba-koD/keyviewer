$ErrorActionPreference = "Stop"

function Test-Exe([string]$name) {
	try { Get-Command $name -ErrorAction Stop | Out-Null; return $true } catch { return $false }
}

# Ensure this session allows script execution
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force

$venvPython = ".\.venv\Scripts\python.exe"

Write-Host "[Build] Ensuring Python & venv..." -ForegroundColor Cyan
if (-Not (Test-Path $venvPython)) {
	# Ensure Python exists
	if (-Not (Test-Exe "python") -and -Not (Test-Exe "py")) {
		if (Test-Exe "winget") {
			Write-Host "[Build] Installing Python 3.11 via winget (silent)" -ForegroundColor Cyan
			winget install -e --id Python.Python.3.11 --accept-package-agreements --accept-source-agreements --silent | Out-Null
			Start-Sleep -Seconds 3
		} else {
			throw "Python is not installed and 'winget' is not available. Please install Python 3.11+ and re-run."
		}
	}
	# Create venv
	if (-Not (Test-Path $venvPython)) {
		try { Write-Host "[Build] Creating venv via 'py -3'" -ForegroundColor Cyan; py -3 -m venv .venv }
		catch { Write-Host "[Build] 'py' not available. Trying 'python'" -ForegroundColor Yellow; python -m venv .venv }
	}
}

if (-Not (Test-Path $venvPython)) { throw "Could not create venv. Ensure Python 3.11+ is installed and available." }

Write-Host "[Build] Installing dependencies" -ForegroundColor Cyan
& $venvPython -m pip install --upgrade pip
& $venvPython -m pip install -r requirements.txt

# Clean previous outputs
if (Test-Path .\build) { Remove-Item -Recurse -Force .\build }
if (Test-Path .\dist) { Remove-Item -Recurse -Force .\dist }

# Read version from version.txt file
$versionFile = "version.txt"
if (Test-Path $versionFile) {
    $version = Get-Content $versionFile -Raw | ForEach-Object { $_.Trim() }
    Write-Host "[Build] Version loaded from $versionFile : $version" -ForegroundColor Green
} else {
    $version = "1.0.0"
    Write-Host "[Build] Version file not found, using default: $version" -ForegroundColor Yellow
    Write-Host "[Build] Create version.txt file to set custom version" -ForegroundColor Yellow
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "    KeyQueueViewer Full Build Script" -ForegroundColor Cyan
Write-Host "    (PyInstaller Only - Modular Build)" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build main program (onedir)
Write-Host "[Step 1] Building main program... (onedir)" -ForegroundColor Green
Write-Host "This step builds with PyInstaller onedir to create a folder-based executable." -ForegroundColor Yellow
Write-Host ""

try {
    Write-Host "[Build] Running onedir.ps1..." -ForegroundColor Cyan
    & ".\onedir.ps1"
    if ($LASTEXITCODE -ne 0) {
        throw "onedir build failed with exit code $LASTEXITCODE"
    }
    Write-Host "[Step 1] Complete!" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] onedir build failed: $_" -ForegroundColor Red
    Write-Host "Please check onedir.ps1 for detailed error information." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "===============================================" -ForegroundColor Cyan

# Step 2: Build installer (onefile)
Write-Host "[Step 2] Building installer... (onefile)" -ForegroundColor Green
Write-Host "This step creates a single installer executable using PyInstaller." -ForegroundColor Yellow
Write-Host ""

try {
    Write-Host "[Build] Running installer.ps1..." -ForegroundColor Cyan
    & ".\installer.ps1"
    if ($LASTEXITCODE -ne 0) {
        throw "Installer build failed with exit code $LASTEXITCODE"
    }
    Write-Host "[Step 2] Complete!" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Installer build failed: $_" -ForegroundColor Red
    Write-Host "Please check installer.ps1 for detailed error information." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "===============================================" -ForegroundColor Cyan

# Step 3: Build portable version (onefile)
Write-Host "[Step 3] Building portable version... (onefile)" -ForegroundColor Green
Write-Host "This step creates a portable executable using PyInstaller (may trigger Windows Defender)." -ForegroundColor Yellow
Write-Host ""

try {
    Write-Host "[Build] Running portable.ps1..." -ForegroundColor Cyan
    & ".\portable.ps1"
    if ($LASTEXITCODE -ne 0) {
        throw "Portable version build failed with exit code $LASTEXITCODE"
    }
    Write-Host "[Step 3] Complete!" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Portable version build failed: $_" -ForegroundColor Red
    Write-Host "Please check portable.ps1 for detailed error information." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "===============================================" -ForegroundColor Cyan

# Step 4: Create onedir zip package
Write-Host "[Step 4] Creating onedir zip package..." -ForegroundColor Green
$zip_name = "KBQV-v$version.zip"
$zip_path = "dist\$zip_name"
$source_folder = "dist\KBQV-v$version"

# 소스 폴더 확인
if (-not (Test-Path $source_folder)) {
    Write-Host "[ERROR] Source folder not found: $source_folder" -ForegroundColor Red
    exit 1
}

# 폴더 내용 확인
$files = Get-ChildItem $source_folder -Recurse | Where-Object { -not $_.PSIsContainer }
if ($files.Count -eq 0) {
    Write-Host "[ERROR] Source folder is empty: $source_folder" -ForegroundColor Red
    exit 1
}

Write-Host "Found $($files.Count) files in source folder" -ForegroundColor Cyan

if (Test-Path $zip_path) {
    Remove-Item $zip_path -Force
}

# 폴더 자체를 압축 (폴더 구조 유지)
Compress-Archive -Path $source_folder -DestinationPath $zip_path
Write-Host "Created: $zip_path" -ForegroundColor Green

# Step 5: Check build results
Write-Host "[Step 5] Checking build results..." -ForegroundColor Green

$distPath = ".\dist"
if (Test-Path $distPath) {
    Write-Host "Generated files:" -ForegroundColor Cyan
    Get-ChildItem -Path $distPath -Recurse | ForEach-Object {
        if ($_.PSIsContainer) {
            Write-Host "  [FOLDER] $($_.Name)" -ForegroundColor White
        } else {
            Write-Host "  [FILE] $($_.Name)" -ForegroundColor White
        }
    }
} else {
    Write-Host "[WARNING] dist folder not found." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "BUILD COMPLETE!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Built files location: dist/" -ForegroundColor Yellow
Write-Host "Files created:" -ForegroundColor Yellow
Write-Host "   * KBQV-Installer-$version.exe (Installer - onefile)" -ForegroundColor White
Write-Host "   * KBQV-Portable-$version.exe (Portable - onefile)" -ForegroundColor White
Write-Host "   * KBQV-v$version.zip (Main Program - onedir)" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "   1. Test the installer: dist\KBQV-Installer-$version.exe" -ForegroundColor White
Write-Host "   2. Test the portable: dist\KBQV-Portable-$version.exe" -ForegroundColor White
Write-Host "   3. Extract and test: dist\KBQV-v$version.zip" -ForegroundColor White
Write-Host "   4. Push to GitHub for automated release" -ForegroundColor White
Write-Host ""
Write-Host "===============================================" -ForegroundColor Cyan 