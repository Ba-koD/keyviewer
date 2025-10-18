# KeyViewer

⌨️ **실시간 키 입력 모니터링 도구 - Rust + Tauri로 재탄생**

## ✨ 주요 특징

- 🎯 **실시간 키 입력 표시** - 특정 창/프로그램 타겟팅 가능
- 🌐 **웹 기반 오버레이** - OBS Browser Source 완벽 호환
- 🎨 **커스터마이징** - 색상, 레이아웃, 애니메이션 자유롭게 설정
- 🚀 **빠른 성능** - Rust 네이티브 바이너리, 낮은 메모리 사용
- 🔒 **낮은 오탐률** - 바이러스 백신 오탐 대폭 감소
- 💻 **크로스 플랫폼** - Windows, macOS, Linux 지원

## 📦 다운로드

[**Releases**](https://github.com/YOUR_USERNAME/keyviewer/releases)에서 최신 버전 다운로드

### Windows
- `KBQV-Portable-*.zip` - 설치 불필요 (권장)
- `KeyQueueViewer_*_x64_en-US.msi` - MSI 설치 프로그램
- `KeyQueueViewer_*_x64-setup.exe` - NSIS 설치 프로그램

### macOS
- `KeyQueueViewer_*_x64.dmg` - Intel Mac
- `KeyQueueViewer_*_aarch64.dmg` - Apple Silicon (M1/M2/M3)

### Linux
- `*.AppImage` - 모든 배포판
- `*.deb` - Debian/Ubuntu

## 🚀 빠른 시작

1. 앱 실행 → 포트 설정 → **서버 시작**
2. 브라우저에서 `http://localhost:8000/control` 접속
3. 타겟 창 설정 (모드 선택 후 창 클릭)
4. OBS에서 Browser Source 추가: `http://localhost:8000/overlay`

## 📚 문서

- **[사용자 & 개발 가이드](docs/GUIDE.md)** - 설치, 사용법, 빌드 방법
- **[변경 이력](CHANGELOG.md)** - 최신 업데이트 및 변경사항

## 🛠️ 빌드

```bash
# 개발 모드
cd src-tauri
cargo tauri dev

# 프로덕션 빌드
cargo tauri build
```

## 🔧 기술 스택

- **Backend**: Rust + Tauri 2.0
- **Web Server**: Axum + WebSocket
- **Keyboard Hook**: rdev
- **Frontend**: HTML/CSS/JavaScript

## 📊 성능 비교 (Python → Rust)

| 항목 | 이전 | 현재 | 개선율 |
|------|------|------|--------|
| 파일 크기 | ~80MB | ~8MB | **90% ↓** |
| 메모리 | ~100MB | ~30MB | **70% ↓** |
| 시작 시간 | ~2-3초 | ~0.5초 | **80% ↓** |
| 바이러스 오탐 | 높음 | 거의 없음 | **대폭 개선** |

## 📞 문의 및 지원

- **버그 리포트**: [Issues](https://github.com/YOUR_USERNAME/keyviewer/issues)
- **기능 요청**: [Discussions](https://github.com/YOUR_USERNAME/keyviewer/discussions)

## 📝 라이선스

MIT License - 자유롭게 사용 가능

---

**Made with ❤️ using Rust and Tauri**
