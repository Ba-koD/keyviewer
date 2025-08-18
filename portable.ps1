# KeyQueueViewer Portable 빌드 스크립트
# Portable만 빌드합니다 (PyInstaller 사용)

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "💻 KeyQueueViewer Portable 빌드 시작" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

# 버전 정보 읽기
$version = Get-Content "version.txt" -Raw
$version = $version.Trim()
Write-Host "📋 빌드 버전: $version" -ForegroundColor Green

# 가상환경 활성화 확인
if (Test-Path ".venv\Scripts\Activate.ps1") {
    Write-Host "🐍 가상환경 활성화 중..." -ForegroundColor Blue
    & ".venv\Scripts\Activate.ps1"
} else {
    Write-Host "⚠️  가상환경을 찾을 수 없습니다. 시스템 Python을 사용합니다." -ForegroundColor Yellow
}

# 기존 dist 폴더 정리
if (Test-Path "dist") {
    Write-Host "🧹 기존 dist 폴더 정리 중..." -ForegroundColor Blue
    Remove-Item "dist" -Recurse -Force
}

# dist 폴더 생성
New-Item -ItemType Directory -Path "dist" -Force | Out-Null

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "📦 Step 1: Portable 버전 빌드 (PyInstaller)" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

try {
    # PyInstaller로 Portable 버전 빌드
    Write-Host "🔨 Portable 버전 빌드 중..." -ForegroundColor Blue
    pyinstaller --onefile --windowed --name "KBQV-Portable-$version" app/launcher.py
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Portable 버전 빌드 성공!" -ForegroundColor Green
    } else {
        throw "Portable 버전 빌드 실패"
    }
    
} catch {
    Write-Host "❌ Portable 버전 빌드 실패: $_" -ForegroundColor Red
    Write-Host "💡 해결 방법:" -ForegroundColor Yellow
    Write-Host "   1. PyInstaller 설치: pip install pyinstaller" -ForegroundColor White
    Write-Host "   2. 가상환경 활성화 확인" -ForegroundColor White
    Write-Host "   3. Python 경로 확인" -ForegroundColor White
    Write-Host "   4. app/launcher.py 파일 존재 확인" -ForegroundColor White
    exit 1
}

# 빌드 결과 확인
$portable_path = "dist\KBQV-Portable-$version.exe"
if (Test-Path $portable_path) {
    $file_size = (Get-Item $portable_path).Length
    $file_size_mb = [math]::Round($file_size / 1MB, 2)
    Write-Host "📁 Portable 파일: $portable_path" -ForegroundColor Green
    Write-Host "📊 파일 크기: $file_size_mb MB" -ForegroundColor Green
} else {
    Write-Host "❌ Portable 파일을 찾을 수 없습니다!" -ForegroundColor Red
    exit 1
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "🎉 PORTABLE 빌드 완료!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "📁 빌드된 파일 위치: dist/" -ForegroundColor Blue
Write-Host "📋 생성된 파일:" -ForegroundColor Blue
Write-Host "   💻 KBQV-Portable-$version.exe (Portable - onefile)" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "💡 다음 단계:" -ForegroundColor Yellow
Write-Host "   1. Portable 테스트: dist\KBQV-Portable-$version.exe" -ForegroundColor White
Write-Host "   2. GitHub에 푸시하여 자동 릴리즈" -ForegroundColor White
Write-Host "===============================================" -ForegroundColor Cyan 