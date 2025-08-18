# KeyQueueViewer onedir 빌드 스크립트
# onedir만 빌드합니다 (cx_Freeze 사용, 압축 안함)

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "📁 KeyQueueViewer onedir 빌드 시작" -ForegroundColor Yellow
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
Write-Host "📦 Step 1: onedir 빌드 (cx_Freeze)" -ForegroundColor Yellow
Write-Host "===============================================" -ForegroundColor Cyan

try {
    # cx_Freeze로 onedir 빌드
    Write-Host "🔨 onedir 빌드 중..." -ForegroundColor Blue
    python setup_main.py build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ onedir 빌드 성공!" -ForegroundColor Green
    } else {
        throw "onedir 빌드 실패"
    }
    
} catch {
    Write-Host "❌ onedir 빌드 실패: $_" -ForegroundColor Red
    Write-Host "💡 해결 방법:" -ForegroundColor Yellow
    Write-Host "   1. cx_Freeze 설치: pip install cx_Freeze" -ForegroundColor White
    Write-Host "   2. 가상환경 활성화 확인" -ForegroundColor White
    Write-Host "   3. Python 경로 확인" -ForegroundColor White
    Write-Host "   4. setup_main.py 파일 존재 확인" -ForegroundColor White
    exit 1
}

# 빌드 결과 확인
$build_folder = "KBQV-v$version"
$build_path = "dist\$build_folder"
if (Test-Path $build_path) {
    # 폴더 내용 확인
    $files = Get-ChildItem $build_path -Recurse | Measure-Object
    $folder_size = (Get-ChildItem $build_path -Recurse | Measure-Object -Property Length -Sum).Sum
    $folder_size_mb = [math]::Round($folder_size / 1MB, 2)
    
    Write-Host "📁 onedir 폴더: $build_path" -ForegroundColor Green
    Write-Host "📊 폴더 크기: $folder_size_mb MB" -ForegroundColor Green
    Write-Host "📋 파일 개수: $($files.Count)개" -ForegroundColor Green
    
    # 주요 파일들 확인
    Write-Host "🔍 주요 파일들:" -ForegroundColor Blue
    Get-ChildItem $build_path -Name | ForEach-Object {
        Write-Host "   📄 $_" -ForegroundColor White
    }
    
} else {
    Write-Host "❌ onedir 폴더를 찾을 수 없습니다!" -ForegroundColor Red
    exit 1
}

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "🎉 ONEDIR 빌드 완료!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "📁 빌드된 파일 위치: dist/" -ForegroundColor Blue
Write-Host "📋 생성된 폴더:" -ForegroundColor Blue
Write-Host "   📁 $build_folder (onedir - 압축 안함)" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "💡 다음 단계:" -ForegroundColor Yellow
Write-Host "   1. onedir 테스트: dist\$build_folder\KBQV-v$version.exe" -ForegroundColor White
Write-Host "   2. GitHub에 푸시하여 자동 릴리즈" -ForegroundColor White
Write-Host "===============================================" -ForegroundColor Cyan 