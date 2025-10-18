# 변경 이력 (Changelog)

## v1.0.4 (최신)

### ✨ 새로운 기능
- **설정 초기화 버튼** 추가 - 런처에서 모든 설정을 기본값으로 초기화 가능
- **Windows 레지스트리 기반 설정 저장**
  - 포트, 언어, 자동 시작 설정
  - 타겟 모드 및 선택값
  - 오버레이 모든 설정 (색상, 레이아웃, 애니메이션 등)
- **포트 충돌 감지 및 상세 오류 메시지**
  - 포트가 이미 사용 중인 경우 감지
  - 권한 부족 시 상세한 안내 메시지

### 🔧 개선사항
- **OBS 브라우저 소스 캐싱 문제 해결**
  - `no-cache` HTTP 헤더 추가
  - 더 이상 OBS에서 수동 캐시 초기화 불필요
- **자동 프로세스 모드 전환**
  - 창 리스트에서 클릭 시 자동으로 "프로세스" 모드로 변경
  - "사용 안 함" 상태에서도 즉시 활성화
- **UI 파일 실행 파일 내장**
  - HTML/CSS 파일이 exe에 임베드
  - Portable 버전이 진짜 단일 파일로 동작

### 🐛 버그 수정
- 설정 파일(JSON) 생성 제거 - 모든 설정이 레지스트리에 저장
- PowerShell 5.x 호환성 개선 (삼항 연산자 제거)

---

## v1.0.0-1.0.3

### 🎉 Tauri 마이그레이션
- Python/PyInstaller → Rust/Tauri 완전 재작성
- 파일 크기 90% 감소 (~80MB → ~8MB)
- 메모리 사용량 70% 감소 (~100MB → ~30MB)
- 바이러스 오탐 대폭 감소

### 주요 기능
- 실시간 키보드 입력 모니터링
- 웹 기반 컨트롤 패널 및 오버레이
- 창 타겟팅 (제목, 프로세스, HWND, 클래스명, 전체)
- OBS Browser Source 지원
- 커스터마이즈 가능한 오버레이
- 다국어 지원 (한국어/English)

### 크로스 플랫폼 지원
- Windows (MSI, NSIS, Portable)
- macOS (Intel, Apple Silicon)
- Linux (AppImage, Deb)

---

## 개발자 변경사항

### 빌드 시스템
- **GitHub Actions 개선**
  - 각 플랫폼 빌드를 별도 job으로 분리
  - 실패한 빌드가 있어도 성공한 것만 릴리스
  - 빌드 결과 체크리스트 자동 생성
  - 성공한 파일만 첨부

### 아키텍처
- **설정 저장 방식 변경**
  - 파일 시스템 → Windows 레지스트리
  - 레지스트리 위치: `HKEY_CURRENT_USER\Software\KeyViewer`
- **HTTP 캐시 헤더**
  - 모든 정적 리소스에 `Cache-Control: no-cache` 추가
  - OBS 브라우저 소스 호환성 향상

### 코드 개선
- Rust `u16` ↔ `u32` 변환 (레지스트리 호환)
- 사용하지 않는 import 제거 (`Html`)
- 포트 바인딩 전 가용성 체크

---

## 알려진 이슈

### Windows
- 관리자 권한 필요 (전역 키보드 후킹)
- 1024 미만 포트는 권한 문제 가능성

### macOS
- 접근성 권한 필요
- Gatekeeper 경고 (코드 서명 필요)

### Linux
- X11 필요 (Wayland 제한적 지원)

---

**전체 변경사항**: [GitHub Releases](https://github.com/YOUR_USERNAME/keyviewer/releases)

