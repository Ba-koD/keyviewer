# 로컬 테스트 가이드

GitHub Actions 에러를 로컬에서 재현하고 디버깅하는 방법입니다.

## 🚀 빠른 시작

```powershell
# 1. icon.png 생성 (한 번만)
.\convert-icon.ps1

# 2. Windows 빌드 테스트
.\build-portable.ps1

# 3. Linux 빌드 테스트 (Docker 필요)
.\docker-test.ps1 -Platform linux
```

## 📋 카테고리별 가이드

### 1. Windows 빌드 (로컬)

```powershell
.\build-portable.ps1
```

**결과**: `dist\KBQV-Portable-{version}.zip`

**주의**: Tauri CLI가 없으면 자동 설치됩니다 (5-10분 소요)

### 2. Linux 빌드 (Docker)

```powershell
# 빌드 테스트
.\docker-test.ps1 -Platform linux

# 디버깅 (컨테이너 쉘)
.\docker-test.ps1 -Platform linux -Shell
```

**결과**: DEB, AppImage 패키지

### 3. macOS 빌드

**중요**: 실제 macOS 빌드는 실제 macOS 하드웨어가 필요합니다. Docker는 컴파일 체크만 가능합니다.

```powershell
.\docker-test.ps1 -Platform macos-check  # 컴파일만 체크
```

## 🐛 GitHub Actions 에러 재현하기

### "no such command: tauri" 에러

**증상**:
```
error: no such command: `tauri`
Build failed!
```

**원인**: Tauri CLI가 설치되지 않음

**해결**: `build-portable.ps1`이 자동으로 설치하므로 그냥 실행하면 됩니다.
```powershell
.\build-portable.ps1  # 자동으로 Tauri CLI 설치됨
```

### "icon.png not found" 에러

**해결**:
```powershell
.\convert-icon.ps1  # icon.png 생성
```

### Linux/macOS 빌드 에러

**해결**:
```powershell
# Docker 컨테이너에서 디버깅
.\docker-test.ps1 -Platform linux -Shell

# 컨테이너 안에서:
cargo tauri build --verbose  # 자세한 에러 로그
```

## ⚡ 고급 옵션

```powershell
# 캐시 정리
.\docker-test.ps1 -Clean

# Docker 이미지 재빌드
.\docker-test.ps1 -Rebuild

# 모든 플랫폼 테스트
.\docker-test.ps1 -Platform all
```

## 📊 참고 정보

### 빌드 시간
- Windows (처음): 5-10분
- Windows (캐시): 2-5분
- Linux (처음): 15-25분
- Linux (캐시): 5-10분

### 필수 도구
- **Windows 빌드**: Rust, PowerShell
- **Linux 빌드**: Docker Desktop

### 주요 명령어
```powershell
.\convert-icon.ps1              # icon.png 생성
.\build-portable.ps1            # Windows 빌드
.\docker-test.ps1 -Platform linux  # Linux 빌드
.\docker-test.ps1 -Shell        # 디버깅 쉘
.\docker-test.ps1 -Clean        # 캐시 정리
```

