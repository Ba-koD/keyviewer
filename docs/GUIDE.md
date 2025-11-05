# KeyViewer 사용 및 개발 가이드

## 📋 목차
- [사용자 가이드](#사용자-가이드)
  - [설치](#설치)
  - [기본 사용법](#기본-사용법)
  - [OBS 통합](#obs-통합)
  - [문제 해결](#문제-해결)
- [개발자 가이드](#개발자-가이드)
  - [환경 설정](#환경-설정)
  - [빌드 방법](#빌드-방법)
  - [프로젝트 구조](#프로젝트-구조)

---

# 사용자 가이드

## 설치

### Windows
1. [Releases](https://github.com/Ba-koD/keyviewer/releases)에서 다운로드
2. **Portable** (권장): ZIP 압축 해제 후 EXE 실행
3. **Installer**: MSI 또는 NSIS 설치 프로그램 실행

### macOS
1. DMG 파일 다운로드 (Intel 또는 Apple Silicon)
2. 앱을 Applications 폴더로 드래그
3. 처음 실행 시: `시스템 환경설정` → `보안` → "확인 없이 열기"

### Linux
```bash
# Debian/Ubuntu
sudo dpkg -i keyqueueviewer_*.deb

# AppImage (모든 배포판)
chmod +x keyqueueviewer_*.AppImage
./keyqueueviewer_*.AppImage
```

## 기본 사용법

### 1. 서버 시작
1. 앱 실행
2. 언어 및 포트 설정 (기본값: 8000)
3. **"서버 시작"** 클릭

### 2. 타겟 설정
1. 브라우저에서 `http://localhost:8000/control` 접속
2. **타겟 모드** 선택:
   - **제목 (포함)**: 창 제목에 특정 텍스트 포함
   - **프로세스 (정확)**: 프로세스 이름 정확히 일치 (예: `notepad.exe`)
   - **모든 창**: 모든 포커스 창에서 감지
3. 창 리스트에서 원하는 창 클릭 (자동 설정)

### 3. 오버레이 커스터마이징
1. 컨트롤 패널에서 **"오버레이 설정"** 클릭
2. 색상, 크기, 레이아웃 조정
3. **"저장"** 클릭

## OBS 통합

### Browser Source 추가
1. OBS → **Sources** → **+** → **Browser**
2. **URL**: `http://localhost:8000/overlay`
3. **Width**: 800, **Height**: 600
4. **Refresh when active**: ✓ (체크)

### 투명 배경 설정
1. 컨트롤 패널 → 오버레이 설정
2. **"투명 배경"** 체크
3. 저장 → OBS에서 자동 반영

## 문제 해결

### 서버가 시작되지 않음
```powershell
# 포트 충돌 확인
netstat -ano | findstr :8000

# 프로세스 종료
taskkill /PID <PID> /F
```

### 키 입력이 감지되지 않음
1. 컨트롤 패널에서 타겟 모드 확인
2. "모든 창" 모드로 테스트
3. Windows: 관리자 권한으로 실행
4. macOS: 접근성 권한 허용

### 설정 초기화
1. 런처에서 **"설정 초기화"** 버튼 클릭
2. 모든 설정이 기본값으로 복원

---

# 개발자 가이드

## 환경 설정

### PowerShell 실행 정책 설정 (Windows)

빌드 스크립트 실행 시 다음 오류가 발생할 수 있습니다:
```
이 시스템에서 스크립트를 실행할 수 없으므로 파일을 로드할 수 없습니다.
```

**해결 방법 1: 영구적으로 해제 (권장)**
```powershell
# PowerShell을 관리자 권한으로 실행 후
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# 확인
Get-ExecutionPolicy
# 출력: RemoteSigned
```

**해결 방법 2: 현재 세션만 임시로 우회**
```powershell
# 현재 터미널 세션에서만 유효
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass

# 이후 빌드 스크립트 실행
.\build-portable.ps1
```

**해결 방법 3: 한 번만 우회 (추천)**
```powershell
# 스크립트 실행 시마다 우회
powershell -ExecutionPolicy Bypass -File .\build-portable.ps1
```

> **보안 참고**: `RemoteSigned` 정책은 로컬에서 작성한 스크립트는 제한 없이 실행하고, 인터넷에서 다운로드한 스크립트는 서명이 필요합니다. 개발 환경에서 안전하게 사용할 수 있습니다.

### 필수 요구사항
- Rust 1.70+
- Cargo (Rust와 함께 자동 설치)
- Tauri CLI
- Visual Studio Build Tools (Windows만 해당)

### Windows 설치 가이드

#### 1단계: Rust 설치

**방법 A: Rustup 공식 인스톨러 (권장)**
```powershell
# 브라우저에서 https://rustup.rs/ 접속하여 다운로드
# 또는 PowerShell에서 직접 다운로드:
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe"

# 설치 중 나오는 선택지에서 1번 (기본 설치) 선택
# 설치 완료 후 PowerShell을 완전히 종료하고 다시 열기!
```

**방법 B: winget 사용 (Windows 11)**
```powershell
winget install Rustlang.Rustup

# ⚠️ 중요: 설치 후 PowerShell을 완전히 닫고 다시 열기!
```

**설치 확인**
```powershell
# PowerShell 재시작 후 확인
cargo --version
rustc --version
```

**❌ "cargo를 찾을 수 없습니다" 오류가 나면:**
```powershell
# 방법 1: PowerShell 완전히 닫고 다시 열기 (가장 흔한 원인)

# 방법 2: 환경 변수 수동으로 로드
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
cargo --version

# 방법 3: Rustup 공식 인스톨러로 재설치 (권장)
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe"
# 설치 후 반드시 PowerShell 재시작!
```

#### 2단계: Visual Studio Build Tools 설치 (필수) ⚠️

> **중요**: Rust는 Windows에서 `link.exe` (MSVC 링커)가 필요합니다. Build Tools를 설치하지 않으면 컴파일이 불가능합니다!

**방법 A: winget 사용 (권장)**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --interactive
```

**설치 창이 뜨면 반드시 선택:**
- ✅ **"C++를 사용한 데스크톱 개발"** (Desktop development with C++)
- ✅ **"MSVC v143 - VS 2022 C++ x64/x86 빌드 도구"** (자동 포함)
- ✅ **"Windows 10/11 SDK"** (자동 포함)

**방법 B: 수동 다운로드**
```powershell
# 다운로드 페이지 열기
Start-Process "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
```

1. "Tools for Visual Studio" 섹션 찾기
2. **"Build Tools for Visual Studio 2022"** 다운로드
3. 실행 후 **"C++를 사용한 데스크톱 개발"** 워크로드 선택
4. 설치 (약 3-5GB, 10-15분 소요)

**설치 확인**
```powershell
# Build Tools 설치 확인
Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
# True가 나와야 함

# link.exe 경로 확인
where.exe link.exe
# C:\Program Files\...\link.exe 출력되어야 함
```

**❌ "linker link.exe not found" 오류가 나면:**
```powershell
# Visual Studio Build Tools가 제대로 설치되지 않은 것
# 위의 방법으로 재설치 후 PowerShell 재시작 필요
```

#### 3단계: Tauri CLI 설치
```powershell
cargo install tauri-cli --version "^2.0.0"

# 설치 확인
cargo tauri --version
```

#### 4단계: 환경 변수 확인 (자동으로 설정됨)
```powershell
# Rust가 PATH에 추가되었는지 확인
$env:PATH -split ';' | Select-String "cargo"

# 출력 예: C:\Users\YourName\.cargo\bin
```

> **⚠️ 중요**: Rust 설치 후 반드시 PowerShell을 **완전히 종료하고 다시 열어야** 합니다. 환경 변수가 업데이트되어야 `cargo` 명령어를 사용할 수 있습니다.

### macOS 설치 가이드

```bash
# 1. Xcode Command Line Tools 설치 (필수)
xcode-select --install

# 2. Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. 설치 확인
cargo --version
rustc --version

# 4. Tauri CLI 설치
cargo install tauri-cli --version "^2.0.0"
```

### Linux (Ubuntu/Debian) 설치 가이드

```bash
# 1. Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. 설치 확인
cargo --version
rustc --version

# 3. 시스템 의존성 설치
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libx11-dev \
    libxdo-dev \
    libxcb1-dev

# 4. Tauri CLI 설치
cargo install tauri-cli --version "^2.0.0"
```

## 빌드 방법

### 개발 모드
```bash
cd src-tauri
cargo tauri dev
```

### 프로덕션 빌드
```bash
# Windows
.\build-portable.ps1

# macOS/Linux
chmod +x build-tauri.sh
./build-tauri.sh
```

### GitHub Actions 자동 빌드 (권장)
```bash
git add .
git commit -m "Build all platforms"
git push origin master
```
→ GitHub Actions가 자동으로 모든 플랫폼 빌드 및 릴리스

## 프로젝트 구조

```
keyviewer/
├── src-tauri/              # Rust 백엔드
│   ├── src/
│   │   ├── main.rs         # Tauri 앱 + 서버
│   │   ├── keyboard.rs     # 키보드 후킹 (rdev)
│   │   ├── server.rs       # HTTP/WebSocket 서버 (Axum)
│   │   ├── state.rs        # 앱 상태 관리
│   │   ├── settings.rs     # 레지스트리 설정
│   │   └── window_info.rs  # 윈도우 정보 (OS API)
│   ├── Cargo.toml          # Rust 의존성
│   └── tauri.conf.json     # Tauri 설정
├── ui/                     # 웹 UI
│   ├── index.html          # 런처
│   ├── control.html        # 컨트롤 패널
│   ├── overlay.html        # 오버레이
│   └── *.css               # 스타일
├── .github/workflows/      # CI/CD
│   └── tauri-build.yml     # 자동 빌드
└── version.txt             # 버전 정보
```

## 기술 스택

### Backend
- **Tauri 2.0**: Desktop framework
- **Axum**: Web server
- **rdev**: Keyboard hooking
- **tokio**: Async runtime
- **serde**: Serialization
- **winreg**: Windows Registry (Windows only)

### Frontend
- **HTML/CSS/JavaScript**: Vanilla (no framework)
- **Fetch API + WebSocket**: 서버 통신

### Build & CI/CD
- **Cargo**: Rust package manager
- **Tauri CLI**: Build tool
- **GitHub Actions**: Multi-platform builds

## 디버깅

### 개발자 도구
앱 실행 후 `F12` 또는 `Ctrl+Shift+I` (Windows/Linux) / `Cmd+Option+I` (macOS)

### 로그 확인
```bash
# Rust 디버그 로그
RUST_LOG=debug cargo tauri dev

# 웹소켓 트래픽 확인
# 브라우저 개발자 도구 → Network → WS
```

### 일반적인 문제

**빌드 실패**
```bash
rustup update
cd src-tauri
cargo clean
cargo build
```

**Tauri API 로드 실패**
- `F12` 콘솔에서 `window.__TAURI__` 확인
- `undefined`면 `tauri.conf.json`에서 `withGlobalTauri: true` 확인

## 기여하기

1. Fork the Project
2. Create Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit Changes (`git commit -m 'Add AmazingFeature'`)
4. Push to Branch (`git push origin feature/AmazingFeature`)
5. Open Pull Request

## 추가 리소스

- [Tauri Docs](https://tauri.app/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Docs](https://docs.rs/axum/)
- [rdev Docs](https://docs.rs/rdev/)

---

**문의사항**: [GitHub Issues](https://github.com/Ba-koD/keyviewer/issues)



Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\docker-test.ps1 -Platform linux

Unblock-File .\docker-test.ps1
.\docker-test.ps1 -Platform linux

Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned