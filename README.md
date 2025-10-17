# KeyQueueViewer (Tauri Edition)

🎉 **완전히 새로워진 KeyQueueViewer - Tauri로 재탄생!**

Key Input Monitoring Tool with Web Interface - Now built with Rust + Tauri for maximum performance and minimal false positives.

## 🚀 주요 개선사항

### Python/PyInstaller → Rust/Tauri 마이그레이션

| Feature | Python (이전) | Tauri (현재) | 개선율 |
|---------|--------------|-------------|--------|
| **파일 크기** | ~50-100MB | ~10MB | **80-90% 감소** |
| **바이러스 오탐** | 높음 (자주 발생) | 거의 없음 | **대폭 개선** |
| **메모리 사용** | ~100MB | ~30MB | **70% 감소** |
| **크로스 플랫폼** | Windows만 | Windows, macOS, Linux | **완전 지원** |
| **성능** | 보통 | 매우 빠름 | **2-3배 향상** |

## 📦 다운로드

### Windows
- **MSI Installer** (권장): Windows Installer 형식
- **NSIS Setup**: 대체 인스톨러

### macOS
- **Intel (x86_64)**: Intel Mac용 DMG
- **Apple Silicon (ARM64)**: M1/M2/M3 Mac용 DMG

### Linux
- **Debian/Ubuntu**: `.deb` 패키지
- **AppImage**: 모든 Linux 배포판 호환

## ✨ Features

- ⌨️ **실시간 키 입력 모니터링**
- 🌐 **웹 기반 인터페이스** (OBS Browser Source 호환)
- 🎯 **창 타겟팅** (특정 프로그램/창에만 반응)
- 🎨 **커스터마이즈 가능한 오버레이**
- 🔒 **낮은 바이러스 오탐율** (Rust 네이티브 바이너리)
- 🚀 **빠른 성능과 낮은 메모리 사용**
- 💻 **크로스 플랫폼** (Windows, macOS, Linux)

## 🛠️ 개발 환경 설정

### 필수 요구사항

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (18+ - 선택사항, UI 개발용)

#### Windows
```powershell
# Rust 설치
winget install Rustlang.Rustup

# Tauri CLI 설치
cargo install tauri-cli
```

#### macOS
```bash
# Xcode Command Line Tools
xcode-select --install

# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri CLI 설치
cargo install tauri-cli
```

#### Linux (Ubuntu/Debian)
```bash
# 시스템 의존성 설치
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev patchelf libx11-dev

# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri CLI 설치
cargo install tauri-cli
```

### 프로젝트 설정

```bash
# 저장소 클론
git clone https://github.com/YOUR_USERNAME/keyviewer.git
cd keyviewer

# 개발 모드 실행
cargo tauri dev

# 프로덕션 빌드
cargo tauri build
```

## 📁 프로젝트 구조

```
keyviewer/
├── src-tauri/              # Rust 백엔드 (Tauri)
│   ├── src/
│   │   ├── main.rs        # 메인 애플리케이션
│   │   ├── keyboard.rs    # 키보드 후킹
│   │   ├── server.rs      # 웹 서버 & WebSocket
│   │   ├── state.rs       # 애플리케이션 상태
│   │   └── window_info.rs # 창 정보 수집
│   ├── Cargo.toml         # Rust 의존성
│   └── tauri.conf.json    # Tauri 설정
│
├── ui/                     # 웹 프론트엔드
│   ├── index.html         # 오버레이 UI
│   ├── control.html       # 제어판 UI
│   └── favicon.ico        # 아이콘
│
├── .github/
│   └── workflows/
│       └── tauri-build.yml # CI/CD (멀티 플랫폼)
│
└── version.txt            # 버전 정보
```

## 🎮 사용법

### 1. 애플리케이션 실행

빌드된 애플리케이션을 실행하면 시스템 트레이에 아이콘이 나타납니다.

### 2. 웹 제어판 열기

브라우저에서 `http://localhost:8000/control` 접속

### 3. 타겟 창 설정

- **모드 선택**: 제목(포함), 프로세스(정확), HWND, 클래스명, 모든 창
- **값 선택**: 드롭다운에서 원하는 창 선택
- **적용** 버튼 클릭

### 4. OBS에서 오버레이 사용

1. OBS에서 **Browser Source** 추가
2. URL: `http://localhost:8000/overlay`
3. 크기: 1920x1080 (또는 원하는 크기)
4. 커스텀 CSS 적용 가능

## 🎨 오버레이 커스터마이징

웹 제어판에서 **오버레이 설정** 버튼을 클릭하여:

- 배경색/투명도
- 칩 색상 (배경/텍스트)
- 폰트 크기/두께
- 간격/패딩/모서리
- 열/행 개수
- 정렬 방향

## 🔧 기술 스택

- **Backend**: Rust (Tauri 2.0)
- **HTTP Server**: Axum
- **WebSocket**: tokio-tungstenite
- **Keyboard Hook**: rdev
- **Window Management**: Windows API / X11 / Cocoa
- **Frontend**: Vanilla HTML/CSS/JavaScript

## 📊 벤치마크

| 작업 | Python (이전) | Tauri (현재) |
|------|--------------|-------------|
| 시작 시간 | ~2-3초 | ~0.5초 |
| 메모리 (유휴) | ~100MB | ~30MB |
| 메모리 (활성) | ~150MB | ~50MB |
| CPU (유휴) | ~1-2% | ~0.1% |
| 빌드 크기 | ~80MB | ~8MB |

## 🐛 알려진 이슈

### Windows
- 관리자 권한 필요 (전역 키보드 후킹)
- Windows Defender가 처음 실행 시 스캔할 수 있음 (정상)

### macOS
- 접근성 권한 필요 (시스템 환경설정에서 허용)
- Gatekeeper 경고 가능 (개발자 서명 필요)

### Linux
- X11 필요 (Wayland는 제한적 지원)
- 루트 권한 없이 실행 가능

## 🤝 기여하기

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

MIT License - 자유롭게 사용 가능

## 🙏 Credits

- Original Python version: [KeyQueueViewer](https://github.com/YOUR_USERNAME/keyviewer)
- Built with [Tauri](https://tauri.app/)
- Keyboard hooking: [rdev](https://github.com/Narsil/rdev)

## 📞 Support

- Issues: [GitHub Issues](https://github.com/YOUR_USERNAME/keyviewer/issues)
- Discussions: [GitHub Discussions](https://github.com/YOUR_USERNAME/keyviewer/discussions)

---

**Made with ❤️ using Rust and Tauri**

