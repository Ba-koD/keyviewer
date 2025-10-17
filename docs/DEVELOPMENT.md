# 개발 & 빌드 가이드

## 📋 목차
- [개발 환경 설정](#개발-환경-설정)
- [로컬 빌드](#로컬-빌드)
- [멀티 플랫폼 빌드](#멀티-플랫폼-빌드)
- [프로젝트 구조](#프로젝트-구조)
- [기술 스택](#기술-스택)

---

## 🛠️ 개발 환경 설정

### 필수 요구사항

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (18+ - 선택사항, UI 개발용)

#### Windows
```powershell
# Rust 설치
winget install Rustlang.Rust.GNU

# Visual Studio Build Tools 필요
winget install Microsoft.VisualStudio.2022.BuildTools
```

#### macOS
```bash
# Homebrew 설치 (없는 경우)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Xcode Command Line Tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)
```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 필수 시스템 라이브러리
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libx11-dev
```

### Tauri CLI 설치

```bash
cargo install tauri-cli
```

---

## 🚀 로컬 빌드

### Windows

#### 개발 모드 실행
```powershell
cd src-tauri
cargo tauri dev
```

#### 프로덕션 빌드
```powershell
# 방법 1: Tauri CLI
cd src-tauri
cargo tauri build

# 방법 2: PowerShell 스크립트
.\build-tauri.ps1
```

#### Portable 버전 빌드
```powershell
.\build-portable.ps1
```

#### 빌드 결과물
- **실행 파일**: `src-tauri/target/release/keyviewer.exe`
- **MSI 설치 파일**: `src-tauri/target/release/bundle/msi/KeyQueueViewer_*.msi`
- **NSIS 설치 파일**: `src-tauri/target/release/bundle/nsis/KeyQueueViewer_*-setup.exe`
- **Portable ZIP**: `dist/KBQV-Portable-*.zip`

### macOS

#### 개발 모드 실행
```bash
cd src-tauri
cargo tauri dev
```

#### 프로덕션 빌드
```bash
# 방법 1: Tauri CLI
cd src-tauri
cargo tauri build

# 방법 2: Shell 스크립트
chmod +x build-tauri.sh
./build-tauri.sh
```

#### ARM64 (Apple Silicon) 빌드
```bash
# M1/M2/M3 Mac용
rustup target add aarch64-apple-darwin
cd src-tauri
cargo tauri build --target aarch64-apple-darwin
```

#### x86_64 (Intel) 빌드
```bash
# Intel Mac용
rustup target add x86_64-apple-darwin
cd src-tauri
cargo tauri build --target x86_64-apple-darwin
```

#### 빌드 결과물
- **DMG**: `src-tauri/target/[target]/release/bundle/dmg/*.dmg`
- **APP**: `src-tauri/target/[target]/release/bundle/macos/*.app`

### Linux

#### 개발 모드 실행
```bash
cd src-tauri
cargo tauri dev
```

#### 프로덕션 빌드
```bash
# 방법 1: Tauri CLI
cd src-tauri
cargo tauri build

# 방법 2: Shell 스크립트
chmod +x build-tauri.sh
./build-tauri.sh
```

#### 빌드 결과물
- **DEB 패키지**: `src-tauri/target/release/bundle/deb/*.deb`
- **AppImage**: `src-tauri/target/release/bundle/appimage/*.AppImage`

---

## 🌍 멀티 플랫폼 빌드

### ⚠️ 중요: 크로스 컴파일 제한

**각 플랫폼은 해당 OS에서만 빌드 가능합니다!**
- Windows → Windows만
- macOS → macOS만 (Intel과 ARM 둘 다 가능)
- Linux → Linux만

**해결 방법: GitHub Actions 자동 빌드 (권장!) ⭐**

### 방법 1: GitHub Actions (가장 쉬움!) ⭐

#### 모든 플랫폼 자동 빌드

```bash
# 1. 변경사항 커밋
git add .
git commit -m "Build all platforms"

# 2. 푸시
git push origin master

# 3. 완료!
# GitHub Actions가 자동으로 모든 플랫폼 빌드
```

#### 자동 빌드되는 플랫폼
- ✅ **Windows (x64)** → MSI, NSIS, Portable ZIP
- ✅ **macOS (Intel x64)** → DMG, APP
- ✅ **macOS (Apple Silicon ARM64)** → DMG, APP
- ✅ **Linux (x64)** → DEB, AppImage

#### 결과물 확인
1. GitHub 저장소 → **Actions** 탭에서 진행상황 확인
2. 완료 후 **Releases**에 자동 업로드!

#### 장점
- ✅ 한 번의 푸시로 모든 플랫폼 빌드
- ✅ 각 OS 환경 자동 설정
- ✅ 자동으로 Release에 업로드
- ✅ 로컬 환경 설정 불필요
- ✅ 무료 (공개 저장소)

### 방법 2: 수동 빌드 (고급)

각 플랫폼에서 개별적으로 빌드한 후 수동으로 모으기

#### 1단계: 각 OS에서 빌드
```bash
# Windows에서
.\build-tauri.ps1

# macOS에서
./build-tauri.sh

# Linux에서
./build-tauri.sh
```

#### 2단계: 빌드 파일 정리
```powershell
# Windows에서 (모든 빌드 파일을 수집한 후)
.\organize-builds.ps1
```

---

## 📁 프로젝트 구조

```
keyviewer/
├── src-tauri/              # Rust 백엔드
│   ├── src/
│   │   ├── main.rs         # 메인 앱 (Tauri + 서버)
│   │   ├── keyboard.rs     # 키보드 후킹
│   │   ├── server.rs       # Axum 웹 서버
│   │   ├── state.rs        # 앱 상태 관리
│   │   ├── settings.rs     # 설정 관리
│   │   └── window_info.rs  # 윈도우 정보 (Windows API)
│   ├── icons/              # 앱 아이콘
│   ├── Cargo.toml          # Rust 의존성
│   └── tauri.conf.json     # Tauri 설정
├── ui/                     # 웹 UI (HTML/CSS/JS)
│   ├── index.html          # GUI 런처
│   ├── launcher.css        # 런처 스타일
│   ├── control.html        # 컨트롤 패널
│   ├── control.css         # 컨트롤 스타일
│   ├── overlay.html        # 오버레이
│   └── overlay.css         # 오버레이 스타일
├── .github/workflows/      # GitHub Actions
│   └── tauri-build.yml     # 자동 빌드/릴리즈
├── docs/                   # 문서
│   ├── DEVELOPMENT.md      # 개발 가이드 (이 파일)
│   ├── USER-GUIDE.md       # 사용자 가이드
│   └── MIGRATION.md        # 마이그레이션 가이드
├── build-tauri.ps1         # Windows 빌드 스크립트
├── build-tauri.sh          # macOS/Linux 빌드 스크립트
├── build-portable.ps1      # Portable 빌드 스크립트
├── organize-builds.ps1     # 빌드 파일 정리 스크립트
├── version.txt             # 버전 정보
└── README.md               # 프로젝트 README
```

---

## 🔧 기술 스택

### Backend (Rust)
- **Tauri 2.0**: Desktop application framework
- **Axum**: Web server for control panel and overlay
- **rdev**: Global keyboard hooking
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **parking_lot**: Fast synchronization primitives
- **windows**: Windows API bindings

### Frontend (Web)
- **HTML5/CSS3/JavaScript**: No framework, pure web technologies
- **Fetch API**: HTTP requests to backend
- **WebSocket**: Real-time key event streaming

### Build & CI/CD
- **Cargo**: Rust package manager
- **Tauri CLI**: Build and bundle tool
- **GitHub Actions**: Automated multi-platform builds
- **PowerShell/Bash**: Build automation scripts

---

## 🐛 디버깅

### 개발자 도구 열기

앱 실행 후 `F12` 또는:
- Windows/Linux: `Ctrl + Shift + I`
- macOS: `Cmd + Option + I`

### 로그 확인

```bash
# Rust 로그 (개발 모드)
RUST_LOG=debug cargo tauri dev

# 서버 로그
# 브라우저 콘솔에서 확인 (F12)
```

### 일반적인 문제 해결

#### 1. 빌드 실패
```bash
# Rust 업데이트
rustup update

# 의존성 클린
cd src-tauri
cargo clean
cargo build
```

#### 2. Tauri API 로드 실패
- `F12`를 눌러 콘솔 확인
- `window.__TAURI__`가 정의되어 있는지 확인
- `tauri.conf.json`에서 `withGlobalTauri: true` 확인

#### 3. 포트 충돌
- 기본 포트: 8000
- GUI 런처에서 다른 포트 설정 가능
- 또는 다른 프로세스 종료:
  ```powershell
  # Windows
  Get-Process -Name "*keyviewer*" | Stop-Process -Force
  ```

---

## 📚 추가 리소스

- [Tauri Documentation](https://tauri.app/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [rdev Documentation](https://docs.rs/rdev/)

---

**문제가 있나요?** [Issues](https://github.com/YOUR_USERNAME/keyviewer/issues)에 보고해주세요!

