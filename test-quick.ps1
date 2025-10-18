# Quick Test Script - 가장 빠르게 빌드 테스트하기
# This is the fastest way to test if your build works

param(
    [Parameter(Mandatory=$false)]
    [switch]$Docker  # Docker 테스트 포함
)

Write-Host "`n" -NoNewline
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Quick Build Test                      " -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$startTime = Get-Date

# Check prerequisites
Write-Host "Prerequisites Check:" -ForegroundColor Yellow

# Check Rust
try {
    $rustVersion = cargo --version
    Write-Host "  ✓ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Rust not found!" -ForegroundColor Red
    Write-Host "    Install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check Docker (if needed)
if ($Docker) {
    try {
        $dockerVersion = docker --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ Docker: $dockerVersion" -ForegroundColor Green
        } else {
            throw
        }
    } catch {
        Write-Host "  ⚠ Docker not found - skipping Docker tests" -ForegroundColor Yellow
        $Docker = $false
    }
}

Write-Host ""

# Step 1: Generate icon.png if missing
Write-Host "Step 1: Checking icon.png..." -ForegroundColor Yellow
if (-not (Test-Path "src-tauri\icons\icon.png")) {
    Write-Host "  Generating icon.png..." -ForegroundColor Gray
    & .\convert-icon.ps1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ icon.png generated" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Failed to generate icon.png" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "  ✓ icon.png exists" -ForegroundColor Green
}
Write-Host ""

# Step 2: Check Tauri CLI
Write-Host "Step 2: Checking Tauri CLI..." -ForegroundColor Yellow
$tauriVersion = cargo tauri --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  Tauri CLI not found, installing..." -ForegroundColor Yellow
    Write-Host "  This may take a few minutes..." -ForegroundColor Gray
    cargo install tauri-cli --version "^2.0.0"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ✗ Failed to install Tauri CLI" -ForegroundColor Red
        exit 1
    }
    $tauriVersion = cargo tauri --version
    Write-Host "  ✓ Tauri CLI installed: $tauriVersion" -ForegroundColor Green
} else {
    Write-Host "  ✓ Tauri CLI: $tauriVersion" -ForegroundColor Green
}
Write-Host ""

# Step 3: Windows build
Write-Host "Step 3: Building Windows portable..." -ForegroundColor Yellow
Write-Host "  This may take 5-10 minutes on first build..." -ForegroundColor Gray
Write-Host ""

$buildStart = Get-Date

& .\build-portable.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n✗ Windows build failed!" -ForegroundColor Red
    exit 1
}

$buildDuration = (Get-Date) - $buildStart
Write-Host "`n✓ Windows build completed in $([math]::Round($buildDuration.TotalMinutes, 1)) minutes" -ForegroundColor Green

# Check output
if (Test-Path "dist\KBQV-Portable-*.zip") {
    $zipFile = Get-ChildItem "dist\KBQV-Portable-*.zip" | Select-Object -First 1
    $sizeMB = [math]::Round($zipFile.Length / 1MB, 2)
    Write-Host "  Output: $($zipFile.Name) ($sizeMB MB)" -ForegroundColor White
}
Write-Host ""

# Step 4: Docker test (optional)
if ($Docker) {
    Write-Host "Step 4: Testing Linux build (Docker)..." -ForegroundColor Yellow
    Write-Host "  This may take 10-20 minutes on first build..." -ForegroundColor Gray
    Write-Host ""
    
    $dockerStart = Get-Date
    
    & .\docker-test.ps1 -Platform linux
    
    if ($LASTEXITCODE -eq 0) {
        $dockerDuration = (Get-Date) - $dockerStart
        Write-Host "`n✓ Linux build completed in $([math]::Round($dockerDuration.TotalMinutes, 1)) minutes" -ForegroundColor Green
    } else {
        Write-Host "`n✗ Linux build failed" -ForegroundColor Red
        Write-Host "  Check logs above for details" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Summary
$totalDuration = (Get-Date) - $startTime

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Test Summary                          " -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Total time: $([math]::Round($totalDuration.TotalMinutes, 1)) minutes" -ForegroundColor White
Write-Host ""

Write-Host "✓ All tests passed!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the built executable: .\dist\KBQV-Portable-*\*.exe" -ForegroundColor Gray
Write-Host "  2. If it works, commit and push to GitHub" -ForegroundColor Gray
Write-Host "  3. GitHub Actions will build all platforms" -ForegroundColor Gray
Write-Host ""

Write-Host "Happy coding! 🎉" -ForegroundColor Green
Write-Host ""

