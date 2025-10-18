# Local Build Test Script for KeyQueueViewer
# This script allows testing builds locally before pushing to GitHub Actions

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet('windows', 'linux', 'all')]
    [string]$Platform = 'all',
    
    [Parameter(Mandatory=$false)]
    [switch]$Clean
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyQueueViewer - Local Build Test    " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Docker is installed (for Linux builds)
$dockerInstalled = $false
try {
    $dockerVersion = docker --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        $dockerInstalled = $true
        Write-Host "✓ Docker found: $dockerVersion" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠ Docker not found - Linux builds will be skipped" -ForegroundColor Yellow
}

Write-Host ""

# Function to test Windows build
function Test-WindowsBuild {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Testing Windows Portable Build       " -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    
    Write-Host "Running build-portable.ps1..." -ForegroundColor Yellow
    $startTime = Get-Date
    
    & .\build-portable.ps1
    
    if ($LASTEXITCODE -eq 0) {
        $duration = (Get-Date) - $startTime
        Write-Host ""
        Write-Host "✓ Windows build completed successfully in $($duration.TotalSeconds) seconds" -ForegroundColor Green
        
        # Check if output exists
        if (Test-Path "dist\KBQV-Portable-*.zip") {
            Write-Host "✓ Portable ZIP found in dist\" -ForegroundColor Green
            Get-ChildItem "dist\KBQV-Portable-*.zip" | ForEach-Object {
                $sizeMB = [math]::Round($_.Length / 1MB, 2)
                Write-Host "  - $($_.Name) ($sizeMB MB)" -ForegroundColor White
            }
        } else {
            Write-Host "⚠ Warning: Portable ZIP not found in dist\" -ForegroundColor Yellow
        }
    } else {
        Write-Host ""
        Write-Host "✗ Windows build failed!" -ForegroundColor Red
        return $false
    }
    
    Write-Host ""
    return $true
}

# Function to test Linux build
function Test-LinuxBuild {
    if (-not $dockerInstalled) {
        Write-Host "⚠ Skipping Linux build - Docker not installed" -ForegroundColor Yellow
        Write-Host "   Install Docker Desktop to enable Linux build testing" -ForegroundColor Gray
        Write-Host "   https://www.docker.com/products/docker-desktop" -ForegroundColor Gray
        return $true
    }
    
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Testing Linux Build (Docker)         " -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    
    Write-Host "Building Docker image..." -ForegroundColor Yellow
    docker build -f Dockerfile.linux -t keyviewer-linux-build .
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Docker image build failed!" -ForegroundColor Red
        return $false
    }
    
    Write-Host ""
    Write-Host "Running build in Docker container..." -ForegroundColor Yellow
    Write-Host "This may take several minutes on first run..." -ForegroundColor Gray
    
    $startTime = Get-Date
    
    # Run build in container
    docker run --rm `
        -v "${PWD}:/app" `
        -v "keyviewer-cargo-cache:/root/.cargo/registry" `
        -v "keyviewer-target-cache:/app/src-tauri/target" `
        keyviewer-linux-build `
        bash -c "cd /app && cargo tauri build"
    
    if ($LASTEXITCODE -eq 0) {
        $duration = (Get-Date) - $startTime
        Write-Host ""
        Write-Host "✓ Linux build completed successfully in $($duration.TotalSeconds) seconds" -ForegroundColor Green
        
        # Check for build artifacts
        $bundlePath = "src-tauri\target\release\bundle"
        if (Test-Path $bundlePath) {
            Write-Host "✓ Build artifacts found:" -ForegroundColor Green
            
            if (Test-Path "$bundlePath\deb") {
                Get-ChildItem "$bundlePath\deb\*.deb" -ErrorAction SilentlyContinue | ForEach-Object {
                    $sizeMB = [math]::Round($_.Length / 1MB, 2)
                    Write-Host "  - DEB: $($_.Name) ($sizeMB MB)" -ForegroundColor White
                }
            }
            
            if (Test-Path "$bundlePath\appimage") {
                Get-ChildItem "$bundlePath\appimage\*.AppImage" -ErrorAction SilentlyContinue | ForEach-Object {
                    $sizeMB = [math]::Round($_.Length / 1MB, 2)
                    Write-Host "  - AppImage: $($_.Name) ($sizeMB MB)" -ForegroundColor White
                }
            }
        }
    } else {
        Write-Host ""
        Write-Host "✗ Linux build failed!" -ForegroundColor Red
        return $false
    }
    
    Write-Host ""
    return $true
}

# Clean previous builds if requested
if ($Clean) {
    Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
    
    if (Test-Path "dist") {
        Remove-Item -Path "dist" -Recurse -Force
        Write-Host "✓ Cleaned dist/" -ForegroundColor Green
    }
    
    if (Test-Path "src-tauri\target") {
        Remove-Item -Path "src-tauri\target" -Recurse -Force
        Write-Host "✓ Cleaned src-tauri/target/" -ForegroundColor Green
    }
    
    if ($dockerInstalled) {
        Write-Host "Cleaning Docker volumes..." -ForegroundColor Yellow
        docker volume rm keyviewer-cargo-cache keyviewer-target-cache 2>$null
        Write-Host "✓ Cleaned Docker volumes" -ForegroundColor Green
    }
    
    Write-Host ""
}

# Run tests based on platform parameter
$allSuccess = $true

if ($Platform -eq 'windows' -or $Platform -eq 'all') {
    $result = Test-WindowsBuild
    $allSuccess = $allSuccess -and $result
}

if ($Platform -eq 'linux' -or $Platform -eq 'all') {
    $result = Test-LinuxBuild
    $allSuccess = $allSuccess -and $result
}

# Summary
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build Test Summary                    " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($allSuccess) {
    Write-Host "✓ All builds completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Review the build artifacts" -ForegroundColor Gray
    Write-Host "  2. Test the portable executable" -ForegroundColor Gray
    Write-Host "  3. Push to GitHub to trigger Actions" -ForegroundColor Gray
} else {
    Write-Host "✗ Some builds failed - please check the errors above" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting tips:" -ForegroundColor Yellow
    Write-Host "  1. Check Rust installation: cargo --version" -ForegroundColor Gray
    Write-Host "  2. Check Tauri CLI: cargo tauri --version" -ForegroundColor Gray
    Write-Host "  3. Review build logs above for specific errors" -ForegroundColor Gray
    Write-Host "  4. Try running with -Clean to start fresh" -ForegroundColor Gray
}

Write-Host ""
Write-Host "✓ Done!" -ForegroundColor Green
Write-Host ""

# Exit with appropriate code
if ($allSuccess) {
    exit 0
} else {
    exit 1
}

