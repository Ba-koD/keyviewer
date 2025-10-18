# Docker-based Local Testing for GitHub Actions
# This script simulates the exact build environment used in GitHub Actions

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet('linux', 'macos-check', 'all')]
    [string]$Platform = 'all',
    
    [Parameter(Mandatory=$false)]
    [switch]$Clean,
    
    [Parameter(Mandatory=$false)]
    [switch]$Rebuild,
    
    [Parameter(Mandatory=$false)]
    [switch]$Shell  # Open shell instead of building
)

$ErrorActionPreference = "Continue"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Docker-based GitHub Actions Test     " -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check Docker
try {
    $dockerVersion = docker --version
    if ($LASTEXITCODE -ne 0) { throw }
    Write-Host "✓ Docker: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker is not installed or not running!" -ForegroundColor Red
    Write-Host "  Please install Docker Desktop: https://www.docker.com/products/docker-desktop" -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Docker Compose: $(docker compose version)" -ForegroundColor Green
Write-Host ""

# Read version
$VERSION = (Get-Content version.txt -Raw).Trim()
Write-Host "Version: $VERSION`n" -ForegroundColor Yellow

# Function to generate icon.png if missing
function Ensure-IconPng {
    Write-Host "Checking for icon.png..." -ForegroundColor Yellow
    
    if (-not (Test-Path "src-tauri\icons\icon.png")) {
        Write-Host "icon.png not found, generating from icon.ico..." -ForegroundColor Yellow
        
        if (Test-Path "src-tauri\icons\icon.ico") {
            # Run conversion script
            & .\convert-icon.ps1
            
            if (Test-Path "src-tauri\icons\icon.png") {
                Write-Host "✓ icon.png generated successfully" -ForegroundColor Green
            } else {
                Write-Host "✗ Failed to generate icon.png" -ForegroundColor Red
                Write-Host "  Creating placeholder icon.png..." -ForegroundColor Yellow
                
                # Create a simple 256x256 placeholder PNG using ImageMagick in Docker
                docker run --rm -v "${PWD}:/work" alpine/imagemagick:latest `
                    convert -size 256x256 xc:blue -fill white -gravity center `
                    -pointsize 48 -annotate +0+0 "KV" /work/src-tauri/icons/icon.png
                
                if (Test-Path "src-tauri\icons\icon.png") {
                    Write-Host "✓ Placeholder icon.png created" -ForegroundColor Green
                } else {
                    Write-Host "✗ Could not create icon.png" -ForegroundColor Red
                    return $false
                }
            }
        } else {
            Write-Host "✗ icon.ico not found!" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "✓ icon.png exists" -ForegroundColor Green
    }
    
    Write-Host ""
    return $true
}

# Clean if requested
if ($Clean) {
    Write-Host "Cleaning previous builds and caches..." -ForegroundColor Yellow
    
    # Clean local builds
    if (Test-Path "dist") {
        Remove-Item -Path "dist" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "✓ Cleaned dist/" -ForegroundColor Green
    }
    
    if (Test-Path "src-tauri\target") {
        Remove-Item -Path "src-tauri\target" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "✓ Cleaned src-tauri/target/" -ForegroundColor Green
    }
    
    # Clean Docker volumes
    Write-Host "Cleaning Docker volumes..." -ForegroundColor Yellow
    docker volume rm keyviewer-linux-cargo-cache 2>$null
    docker volume rm keyviewer-linux-target-cache 2>$null
    docker volume rm keyviewer-macos-cargo-cache 2>$null
    Write-Host "✓ Cleaned Docker volumes" -ForegroundColor Green
    
    Write-Host ""
}

# Rebuild images if requested
if ($Rebuild) {
    Write-Host "Rebuilding Docker images..." -ForegroundColor Yellow
    docker compose build --no-cache
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Failed to rebuild Docker images" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Docker images rebuilt`n" -ForegroundColor Green
}

# Ensure icon.png exists before building
if (-not (Ensure-IconPng)) {
    Write-Host "✗ Cannot proceed without icon.png" -ForegroundColor Red
    exit 1
}

# Function to test Linux build
function Test-LinuxBuild {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Linux Build (Ubuntu 22.04 + Docker)  " -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan
    
    if ($Shell) {
        Write-Host "Opening interactive shell in Linux build container..." -ForegroundColor Yellow
        Write-Host "Commands you can run:" -ForegroundColor Gray
        Write-Host "  - cargo tauri build      # Full build" -ForegroundColor Gray
        Write-Host "  - cargo tauri dev        # Dev mode" -ForegroundColor Gray
        Write-Host "  - cargo build            # Rust only" -ForegroundColor Gray
        Write-Host "  - exit                   # Exit shell`n" -ForegroundColor Gray
        
        docker compose run --rm linux-build bash
        return $true
    }
    
    Write-Host "Building Docker image..." -ForegroundColor Yellow
    docker compose build linux-build
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Docker image build failed!" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✓ Docker image built`n" -ForegroundColor Green
    Write-Host "Running Tauri build in container..." -ForegroundColor Yellow
    Write-Host "This simulates GitHub Actions Linux environment`n" -ForegroundColor Gray
    
    $startTime = Get-Date
    
    # Run build matching GitHub Actions workflow
    docker compose run --rm linux-build bash -c @"
set -e
echo '========================================' 
echo '  Simulating GitHub Actions Linux Build'
echo '========================================'
echo ''
echo 'Environment Info:'
rustc --version
cargo --version
cargo tauri --version
echo ''
echo 'Building Tauri app...'
cd /app/src-tauri
cargo tauri build --verbose
"@
    
    $buildSuccess = $LASTEXITCODE -eq 0
    $duration = (Get-Date) - $startTime
    
    if ($buildSuccess) {
        Write-Host "`n✓ Linux build completed in $([math]::Round($duration.TotalSeconds, 1))s" -ForegroundColor Green
        
        # Check artifacts
        Write-Host "`nBuild Artifacts:" -ForegroundColor Yellow
        
        $artifacts = @()
        if (Test-Path "src-tauri\target\release\bundle\deb\*.deb") {
            $debFiles = Get-ChildItem "src-tauri\target\release\bundle\deb\*.deb"
            foreach ($file in $debFiles) {
                $sizeMB = [math]::Round($file.Length / 1MB, 2)
                Write-Host "  ✓ DEB: $($file.Name) ($sizeMB MB)" -ForegroundColor Green
                $artifacts += "deb"
            }
        }
        
        if (Test-Path "src-tauri\target\release\bundle\appimage\*.AppImage") {
            $appimageFiles = Get-ChildItem "src-tauri\target\release\bundle\appimage\*.AppImage"
            foreach ($file in $appimageFiles) {
                $sizeMB = [math]::Round($file.Length / 1MB, 2)
                Write-Host "  ✓ AppImage: $($file.Name) ($sizeMB MB)" -ForegroundColor Green
                $artifacts += "appimage"
            }
        }
        
        if ($artifacts.Count -eq 0) {
            Write-Host "  ⚠ No artifacts found (build may have succeeded but bundles not created)" -ForegroundColor Yellow
        }
        
        return $true
    } else {
        Write-Host "`n✗ Linux build failed after $([math]::Round($duration.TotalSeconds, 1))s" -ForegroundColor Red
        Write-Host "`nTroubleshooting:" -ForegroundColor Yellow
        Write-Host "  1. Check error messages above" -ForegroundColor Gray
        Write-Host "  2. Run with -Shell to debug interactively" -ForegroundColor Gray
        Write-Host "  3. Check src-tauri/Cargo.toml dependencies" -ForegroundColor Gray
        return $false
    }
}

# Function to check macOS compilation
function Test-MacOSCheck {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  macOS Compilation Check              " -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan
    Write-Host "Note: This only checks if code compiles for macOS" -ForegroundColor Yellow
    Write-Host "Actual .app/.dmg building requires real macOS hardware`n" -ForegroundColor Yellow
    
    if ($Shell) {
        Write-Host "Opening interactive shell in macOS check container..." -ForegroundColor Yellow
        docker compose run --rm macos-check bash
        return $true
    }
    
    Write-Host "Building Docker image..." -ForegroundColor Yellow
    docker compose build macos-check
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Docker image build failed!" -ForegroundColor Red
        return $false
    }
    
    Write-Host "✓ Docker image built`n" -ForegroundColor Green
    Write-Host "Checking macOS target compilation..." -ForegroundColor Yellow
    
    $startTime = Get-Date
    
    # Check compilation for macOS targets
    docker compose run --rm macos-check bash -c @"
set -e
echo '========================================' 
echo '  macOS Compilation Check (x86_64)'
echo '========================================'
echo ''
echo 'Environment Info:'
rustc --version
cargo --version
rustup target list --installed
echo ''
echo 'Checking compilation for x86_64-apple-darwin...'
cd /app/src-tauri
cargo check --target x86_64-apple-darwin --verbose
"@
    
    $checkSuccess = $LASTEXITCODE -eq 0
    $duration = (Get-Date) - $startTime
    
    if ($checkSuccess) {
        Write-Host "`n✓ macOS compilation check passed in $([math]::Round($duration.TotalSeconds, 1))s" -ForegroundColor Green
        Write-Host "  Code can be compiled for macOS (actual build needs macOS)" -ForegroundColor Gray
        return $true
    } else {
        Write-Host "`n✗ macOS compilation check failed after $([math]::Round($duration.TotalSeconds, 1))s" -ForegroundColor Red
        Write-Host "  Code has macOS-specific issues" -ForegroundColor Yellow
        return $false
    }
}

# Run tests
$results = @{}

if ($Platform -eq 'linux' -or $Platform -eq 'all') {
    $results['Linux'] = Test-LinuxBuild
    Write-Host ""
}

if ($Platform -eq 'macos-check' -or $Platform -eq 'all') {
    $results['macOS Check'] = Test-MacOSCheck
    Write-Host ""
}

# Summary
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Test Summary                          " -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$allPassed = $true
foreach ($test in $results.Keys) {
    if ($results[$test]) {
        Write-Host "  ✓ $test" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $test" -ForegroundColor Red
        $allPassed = $false
    }
}

Write-Host ""

if ($allPassed) {
    Write-Host "✓ All tests passed!" -ForegroundColor Green
    Write-Host "`nNext steps:" -ForegroundColor Yellow
    Write-Host "  1. Review build artifacts in src-tauri/target/" -ForegroundColor Gray
    Write-Host "  2. Test Windows build with: .\test-local.ps1 -Platform windows" -ForegroundColor Gray
    Write-Host "  3. Push to GitHub to run full CI/CD" -ForegroundColor Gray
} else {
    Write-Host "✗ Some tests failed!" -ForegroundColor Red
    Write-Host "`nDebugging options:" -ForegroundColor Yellow
    Write-Host "  - Run with -Shell to open interactive container" -ForegroundColor Gray
    Write-Host "  - Run with -Clean to clear caches" -ForegroundColor Gray
    Write-Host "  - Run with -Rebuild to rebuild Docker images" -ForegroundColor Gray
    Write-Host "  - Check error messages above" -ForegroundColor Gray
}

Write-Host ""

if ($allPassed) {
    exit 0
} else {
    exit 1
}

