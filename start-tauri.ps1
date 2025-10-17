# KeyViewer Tauri Quick Start Script
# 이 스크립트는 Tauri 버전을 쉽게 시작하기 위한 것입니다

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  KeyViewer (Tauri) Quick Start" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

# 기존 프로세스 종료
Write-Host "1. Stopping existing processes..." -ForegroundColor Yellow
Stop-Process -Name "KBQV-v1.0.4" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "keyviewer" -Force -ErrorAction SilentlyContinue
Get-Process python -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1
Write-Host "   Done!" -ForegroundColor Green

# Get script directory (project root)
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# Tauri 실행 파일 찾기
Write-Host "`n2. Finding Tauri executable..." -ForegroundColor Yellow
$tauriPath = Join-Path $projectRoot "src-tauri\target\release\keyviewer.exe"

if (!(Test-Path $tauriPath)) {
    Write-Host "   ERROR: keyviewer.exe not found!" -ForegroundColor Red
    Write-Host "`n   Please build first:" -ForegroundColor Yellow
    Write-Host "   .\build-tauri.ps1" -ForegroundColor White
    exit 1
}

Write-Host "   Found: $tauriPath" -ForegroundColor Green

# Tauri 실행
Write-Host "`n3. Starting Tauri app..." -ForegroundColor Yellow
# Set working directory to project root so that it can find ui/ folder
Start-Process -FilePath $tauriPath -WorkingDirectory $projectRoot
Write-Host "   Done!" -ForegroundColor Green

# 서버 시작 대기
Write-Host "`n4. Waiting for server to start..." -ForegroundColor Yellow
$maxRetries = 10
$retryCount = 0
$serverReady = $false

while ($retryCount -lt $maxRetries -and !$serverReady) {
    Start-Sleep -Seconds 1
    $retryCount++
    Write-Host "   Attempt $retryCount/$maxRetries..." -ForegroundColor Gray
    
    # 포트 8000 확인
    $port8000 = netstat -ano | findstr ":8000.*LISTENING"
    if ($port8000) {
        $serverReady = $true
        Write-Host "   Server is ready!" -ForegroundColor Green
    }
}

if (!$serverReady) {
    Write-Host "   WARNING: Server may not have started properly" -ForegroundColor Yellow
    Write-Host "   Continuing anyway..." -ForegroundColor Gray
}

# 브라우저 열기
Write-Host "`n5. Opening browser..." -ForegroundColor Yellow
Start-Process "http://localhost:8000/control"
Write-Host "   Done!" -ForegroundColor Green

# 완료 메시지
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  KeyViewer is now running!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Control Panel: " -NoNewline -ForegroundColor White
Write-Host "http://localhost:8000/control" -ForegroundColor Cyan

Write-Host "Overlay:       " -NoNewline -ForegroundColor White
Write-Host "http://localhost:8000/overlay" -ForegroundColor Cyan

Write-Host "`nPress Ctrl+C to see this info again, or check TAURI-사용법.md" -ForegroundColor Gray
Write-Host ""

