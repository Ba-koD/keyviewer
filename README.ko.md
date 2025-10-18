# KeyViewer - 로컬 테스트 가이드

## 🎯 빠른 시작

```powershell
# 1. icon.png 생성 (필수 - 한 번만)
.\convert-icon.ps1

# 2. Windows 빌드 테스트
.\build-portable.ps1

# 3. 결과 확인
dir dist\
```

## 📋 주요 명령어

| 명령어 | 용도 |
|--------|------|
| `.\convert-icon.ps1` | icon.png 생성 (Linux/macOS 빌드용) |
| `.\build-portable.ps1` | Windows 포터블 빌드 |
| `.\test-local.ps1` | Windows 전체 테스트 |
| `.\docker-test.ps1 -Platform linux` | Linux 빌드 (Docker) |
| `.\docker-test.ps1 -Shell` | Docker 컨테이너 디버깅 |

## 🐛 GitHub Actions 에러 해결

### "no such command: tauri" (Windows)

**문제**: GitHub Actions에서 Tauri CLI를 찾을 수 없음

**로컬 재현**:
```powershell
# 로컬에서는 자동으로 설치됨
.\build-portable.ps1
```

**GitHub Actions 수정**: `.github/workflows/tauri-build.yml`에 Tauri CLI 설치 단계 추가됨

### "icon.png not found" (Linux/macOS)

**문제**: icon.png 파일이 없음

**해결**:
```powershell
.\convert-icon.ps1
```

### Linux/macOS 빌드 에러

**Docker로 재현**:
```powershell
# 동일 환경에서 테스트
.\docker-test.ps1 -Platform linux

# 디버깅 모드
.\docker-test.ps1 -Platform linux -Shell
```

## 📚 자세한 가이드

전체 가이드는 [docs/LOCAL-TESTING.md](docs/LOCAL-TESTING.md)를 참고하세요.

## 💡 개발 팁

```powershell
# 빠른 개발 모드 (빌드 없이 테스트)
cd src-tauri
cargo tauri dev

# 코드 체크만 (빌드 안 함)
cargo check
cargo clippy
```

## 🔧 필수 도구

- **Windows 빌드**: [Rust](https://rustup.rs/)
- **Linux 빌드**: [Docker Desktop](https://www.docker.com/products/docker-desktop)

## ⏱️ 예상 시간

- Windows (처음): 5-10분
- Windows (캐시): 2-5분  
- Linux (처음): 15-25분
- Linux (캐시): 5-10분

