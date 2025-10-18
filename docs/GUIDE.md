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
1. [Releases](https://github.com/YOUR_USERNAME/keyviewer/releases)에서 다운로드
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

### 필수 요구사항
- Rust 1.70+
- Tauri CLI

### Windows
```powershell
winget install Rustlang.Rust.GNU
winget install Microsoft.VisualStudio.2022.BuildTools
cargo install tauri-cli
```

### macOS
```bash
# Xcode Command Line Tools
xcode-select --install

# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo install tauri-cli
```

### Linux (Ubuntu/Debian)
```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 시스템 의존성
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libx11-dev \
    libxdo-dev

cargo install tauri-cli
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

**문의사항**: [GitHub Issues](https://github.com/YOUR_USERNAME/keyviewer/issues)

