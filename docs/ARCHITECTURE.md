# KeyViewer — 아키텍처 상세 문서

> **⚠️ 이 문서는 AI 에이전트가 프로젝트를 이해하고 올바르게 수정하기 위한 핵심 참조 문서입니다.**  
> **코드 변경 시 이 문서의 해당 섹션도 반드시 동기화해야 합니다. (규칙: §10 참조)**

---

## 목차
1. [프로젝트 개요](#1-프로젝트-개요)
2. [디렉터리 구조 상세](#2-디렉터리-구조-상세)
3. [프론트엔드 아키텍처](#3-프론트엔드-아키텍처)
4. [API 전체 명세](#4-api-전체-명세)
5. [데이터 구조 상세](#5-데이터-구조-상세)
6. [설정 저장소 아키텍처](#6-설정-저장소-아키텍처)
7. [오버레이 렌더링 시스템](#7-오버레이-렌더링-시스템)
8. [클라우드/OAuth 시스템](#8-클라우드oauth-시스템)
9. [빌드/CI/CD 파이프라인](#9-빌드cicd-파이프라인)
10. [문서 변경 규칙](#10-문서-변경-규칙)

---

## 1. 프로젝트 개요

### 기본 정보
| 항목 | 값 |
|------|-----|
| 프로젝트명 | KeyViewer (KeyQueueViewer) |
| 바이너리명 | `KBQV` |
| 식별자 | `com.keyviewer` |
| 버전 관리 | `version.txt` + `src-tauri/Cargo.toml` (현재 1.1.0) |
| 프레임워크 | Tauri 2.0 (Rust 백엔드 + Web UI) |
| 지원 OS | Windows, macOS, Linux |
| 라이선스 | MIT |

### 목적
키보드 입력을 실시간으로 모니터링하여 OBS 등 스트리밍 소프트웨어에 오버레이로 표시하는 데스크톱 애플리케이션.

### 핵심 아키텍처 흐름

```
┌─────────────────────────────────────────────────────────────────────┐
│                         실행 순서                                    │
│                                                                     │
│  1. main.rs 시작                                                    │
│     ├── Windows: 관리자 권한 체크 → 자동 재실행                      │
│     ├── macOS: Accessibility/InputMonitoring/ScreenRecording 체크    │
│     └── Linux: 그대로 진행                                           │
│                                                                     │
│  2. 키보드 후킹 스레드 시작 (keyboard.rs)                            │
│     ├── Windows: GetAsyncKeyState() 폴링 (16ms, ~60fps)             │
│     ├── macOS: CGEventTap (HID 레이어)                               │
│     └── Linux: rdev::listen() + mpsc 채널                            │
│                                                                     │
│  3. Tauri 런처 창 표시 (index.html)                                  │
│     └── 사용자가 서버 시작 버튼 클릭                                  │
│                                                                     │
│  4. HTTP/WS 서버 기동 (server.rs, axum)                              │
│     ├── /control → 브라우저 설정 패널                                 │
│     ├── /overlay → OBS 오버레이                                       │
│     └── /ws → WebSocket (실시간 키 상태)                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 데이터 흐름

```
키보드 입력
    │
    ▼
keyboard.rs (플랫폼별 감지)
    │  ┌─ 타겟 창 체크 (target_config 모드 필터링)
    │  └─ 키 → 레이블 변환 (VK코드/키코드 → 표시 문자열)
    │
    ▼
state.rs (AppState)
    │  ┌─ key_labels: HashMap<u32, String> — VK코드→레이블
    │  ├─ label_counts: HashMap<String, u32> — 레퍼런스 카운팅
    │  └─ label_order: VecDeque<String> — 눌린 순서 유지
    │
    ▼
watch::Sender<Vec<String>> — 변경 알림
    │
    ▼
server.rs → WebSocket 브로드캐스트
    │
    ▼
overlay.html (실시간 키 표시)
```

### 통신 채널

| 경로 | 방식 | 프로토콜 | 설명 |
|------|------|----------|------|
| 런처(index.html) ↔ Rust | Tauri IPC | `__TAURI__.core.invoke()` | 서버 제어, 설정, 권한 체크 |
| 브라우저(control.html) ↔ 서버 | HTTP REST | GET/POST JSON | 설정 CRUD, 창 목록, 포커스 |
| 오버레이(overlay.html) ↔ 서버 | WebSocket | `/ws` | 실시간 키 상태 푸시 |
| 브라우저 → Cloudflare Worker | HTTP | OAuth 흐름 | 클라우드 인증 프록시 |
| 오버레이/컨트롤 간 | localStorage | `storage` 이벤트 | 키 이미지/스타일 동기화 |

---

## 2. 디렉터리 구조 상세

### 루트 파일

| 파일 | 역할 |
|------|------|
| `version.txt` | 버전 문자열 (1.1.0), CI/CD에서 참조 |
| `README.md` | 사용자 대상 설치/실행 가이드 |
| `CHANGELOG.md` | 버전별 변경 이력 |
| `build-portable.ps1` | Windows 포터블 빌드 (`cargo tauri build`) |
| `check-all.ps1` / `.sh` | 품질 검사: fmt → clippy → test |
| `convert-icon.ps1` | ICO → PNG 변환 (.NET System.Drawing) |
| `.githooks/pre-push` | 로컬 Git pre-push 훅. push 대상 ref가 있을 때 `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` → `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` → `cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features` 실행 |
| `docker-compose.yml` | 로컬 크로스빌드 (linux-build, macos-check) |
| `Dockerfile.linux` | Ubuntu 22.04 기반 Linux 빌드 이미지 |
| `Dockerfile.macos-cross` | macOS 크로스 컴파일 체크용 이미지 |
| `prev-control.html` | 이전 버전 control 참고용 (비활성, 개발 히스토리) |
| `prev-overlay.html` | 이전 버전 overlay 참고용 (비활성, 개발 히스토리) |

### `src-tauri/` — Rust 백엔드

| 파일 | 줄 수(약) | 역할 |
|------|-----------|------|
| `main.rs` | ~1023 | 앱 진입점, IPC 커맨드 18개, 시스템 트레이, 권한 체크, 싱글 인스턴스 |
| `server.rs` | ~812 | axum HTTP/WS 서버. 라우터, 정적 파일 임베드, WS 핸들러, OBS 파일 생성 |
| `keyboard.rs` | ~922 | 플랫폼별 키보드 입력 감지. Windows 폴링/macOS CGEventTap/Linux rdev |
| `settings.rs` | ~588 | 영속 설정. Windows 레지스트리/macOS UserDefaults/JSON 파일 |
| `state.rs` | ~524 | 인메모리 상태. AppState, OverlayConfig, KeyStyleConfig 등 구조체 |
| `window_info.rs` | ~331 | 창 열거. 포그라운드 창 추적, 플랫폼별 창 목록 API |
| `Cargo.toml` | | 의존성, 빌드 프로필, 플랫폼별 크레이트 |
| `tauri.conf.json` | | Tauri 앱 설정 (창 크기, 번들, 리소스) |
| `app.manifest` | | Windows 관리자 권한 선언 + DPI 인식 |
| `build.rs` | 2줄 | `tauri_build::build()` 호출 |

### `ui/` — 웹 프론트엔드

| 파일 | 역할 | 로드 방식 |
|------|------|-----------|
| `index.html` | 런처 (Tauri 네이티브 창) | Tauri `frontendDist` |
| `control.html` | 웹 컨트롤 패널 (설정 대시보드) | HTTP `/control` |
| `overlay.html` | OBS 오버레이 (키 표시) | HTTP `/overlay` |
| `permissions.html` | macOS 권한 안내 | Tauri 내부 |
| `obs-local-template.html` | OBS 로컬 파일 구형 템플릿 참고용 (현재 직접 사용 안 함) | 저장소 참고 파일 |
| `control.css` | 컨트롤 패널 스타일 | HTTP `/static/control.css` |
| `overlay.css` | 오버레이 스타일 | HTTP `/static/overlay.css` |
| `launcher.css` | 런처 스타일 (밝은 테마) | Tauri 내부 |
| `chip.css` | 키 칩 공유 스타일 | 오버레이+컨트롤 양쪽 사용 |
| `js/utils.js` | 색상/그라디언트/DOM 유틸 | HTTP `/js/utils.js` |
| `js/gradient-editor.js` | 그라디언트 에디터 컴포넌트 | HTTP `/js/gradient-editor.js` |
| `js/chip-preview.js` | 칩 미리보기 렌더러 | HTTP `/js/chip-preview.js` |
| `js/cloud-auth.js` | OAuth 모듈 (GitHub/Google) | HTTP `/js/cloud-auth.js` |

> **중요**: 모든 UI 파일은 `server.rs`에서 `include_str!()`로 컴파일 타임에 바이너리에 내장됩니다. 새 UI 파일 추가 시 `server.rs`에 라우트와 임베드를 반드시 추가해야 합니다.

### `worker/` — Cloudflare Worker

| 파일 | 역할 |
|------|------|
| `index.js` | OAuth 프록시 (GitHub + Google). HMAC state 서명, Origin 화이트리스트 |

### `.github/workflows/` — CI/CD

| 파일 | 역할 |
|------|------|
| `test.yml` | CI: fmt + clippy + test + 3개 OS 포터블 빌드 (push/PR 트리거) |
| `tauri-build.yml` | CD: test 성공 후 GitHub Release 생성 + 아티팩트 업로드 |

---

## 3. 프론트엔드 아키텍처

### 3.1 설계 원칙
- **프레임워크 없음**: 순수 HTML/CSS/JS (바닐라). 빌드 도구 없음
- **번역**: `data-i18n` 어트리뷰트 기반 한/영 이중언어
- **테마**: 다크 테마가 기본 (CSS 변수)
- **JS 로딩**: `<script>` 태그 (ES Module 아님)

### 3.2 CSS 변수 시스템

**컨트롤/오버레이 (다크 테마)**:
```css
--bg: #0b0c10       /* 최심 배경 */
--panel: #12141a    /* 패널 배경 */
--panel2: #181b22   /* 밝은 패널 */
--text: #eaeef5     /* 기본 텍스트 */
--muted: #9aa4b2    /* 보조 텍스트 */
--primary: #4f8cff  /* 강조색 (파란색) */
--border: #232734   /* 테두리 */
```

**런처 (밝은 테마)**: `launcher.css`에서 별도 정의

**오버레이 칩 변수**:
```css
--chip-gap       /* 칩 간격 */
--chip-pad-v/h   /* 칩 패딩 (세로/가로) */
--chip-radius    /* 칩 모서리 */
--chip-font      /* 폰트 크기 */
--chip-font-weight
--fade-in/out    /* 애니메이션 시간 */
--cols, --rows   /* 그리드 레이아웃 */
--align          /* 정렬 (start/center/end) */
```

### 3.3 페이지별 역할

#### index.html (런처)
**통신**: Tauri IPC (`__TAURI__.core.invoke`)

| 기능 | IPC 커맨드 |
|------|-----------|
| 서버 시작/중지 | `start_server` / `stop_server` |
| 서버 상태 확인 | `get_server_status` |
| 설정 로드/저장 | `get_launcher_settings` / `save_port_setting` / `save_language_setting` |
| 시작프로그램 등록 | `set_run_on_startup` |
| 트레이 최소화 | `minimize_to_tray` |
| 콘솔 표시 | `set_console_visible` (Windows) |
| 설정 초기화 | `reset_settings` |
| macOS 권한 | `check_macos_permissions` / `open_macos_permission_settings` |
| 외부 URL 열기 | `open_url` |
| 포트 충돌 해결 | `get_port_processes` / `kill_process` |

#### control.html (웹 컨트롤 패널)
**통신**: HTTP REST API (fetch)

**주요 섹션**:
1. **헤더**: 클라우드 로그인 (GitHub/Google), 프리셋 Import/Export
2. **타겟 창 설정**: 모드 선택 (disabled/title/process/hwnd/class/all), 창 목록 테이블
3. **오버레이 모드 선택**: Queue 모드 / Key Viewer 모드
4. **키 스타일 설정** (통합 모달):
   - 탭 1: 배경 (투명/단색/그라디언트/이미지)
   - 탭 2: 스타일 그룹 (우선순위 기반, 개별키/그룹키/전체키)
5. **오버레이 설정 모달**: 타이밍, 레이아웃, 이미지
6. **키 이미지 설정 모달**: 글로벌/개별 키 이미지, 크롭 에디터
7. **Import/Export 모달**: Gist/Google Drive/JSON

#### overlay.html (오버레이)
**통신**: WebSocket (`/ws`) + HTTP REST (초기 설정 로드)

**기능**:
- WebSocket으로 실시간 키 상태 수신
- `boot_id` 검증으로 서버 재시작 감지 → 자동 리로드
- Queue 모드: flex 레이아웃, fade in/out 애니메이션
- Key Viewer 모드: 캔버스 기반 절대 위치, opacity + glow 효과
- localStorage에서 키 이미지/스타일 읽기 (탭 간 `storage` 이벤트 동기화)
- `?config=1` URL 또는 `C` 키로 인라인 설정 패널 토글

#### permissions.html (macOS 권한)
**통신**: Tauri IPC

- Accessibility, Input Monitoring, Screen Recording 3개 권한 체크
- 2초 간격 폴링으로 실시간 상태 표시
- 모든 권한 부여 시 Continue 버튼 활성화

#### obs-local-template.html (레거시 참고 파일)
**상태**: 현재 `/obs-local-file` 라우트에서 직접 사용하지 않는 구형 참고 템플릿.

- 최신 OBS 로컬 파일 다운로드는 `overlay.html`을 기반으로 서버에서 동적으로 생성
- 저장 시점의 overlay / key images / key style 스냅샷을 HTML에 주입
- 절대 URL(`http://127.0.0.1:{port}` / `ws://127.0.0.1:{port}/ws`)과 인라인 CSS를 사용

### 3.4 JS 모듈 상세

#### js/utils.js
| 함수 | 역할 |
|------|------|
| `isValidHex(hex)` | hex 색상 검증 |
| `isLightColor(color)` | 텍스트 대비 판단 |
| `hexToRgba(hex, opacity)` | hex → RGBA 변환 |
| `generateGradientCSS(stops, angle)` | CSS 그라디언트 문자열 생성 |
| `createDefaultGradientStops()` | 기본 그라디언트 (검정→회) |
| `getGroupBackgroundStyle(group)` | 그룹 배경 스타일 해석 (이미지/그라디언트/단색) |
| `applyChipStyles(el, styles)` | 칩 요소에 스타일 일괄 적용 |
| `syncColorInputs(picker, text, onChange)` | 색상 picker↔텍스트 양방향 동기화 |
| `getElementValue(id, fallback)` | 안전한 요소 값 읽기 |
| `setElementValue(id, value)` | 안전한 요소 값 쓰기 |

#### js/gradient-editor.js
`GradientEditor` 클래스:
- 그라디언트 스톱 (색상 + 위치%) 관리
- 각도 슬라이더 (0-360°)
- 실시간 미리보기

#### js/chip-preview.js
`ChipPreview` 클래스:
- 칩 배열 렌더링
- 스타일 그룹별 미리보기
- 크기 변형 (normal/medium/wide)

#### js/cloud-auth.js
`CloudAuth` IIFE 모듈:
- OAuth 프록시 URL: `https://keyviewer-oauth.rudghrnt.workers.dev`
- GitHub Gist + Google Drive 두 프로바이더
- 쿠키 관리: `kv_github_token`, `kv_google_auth` (1년 유효)
- URL fragment(`#`)에서 토큰 수신 (보안)
- Google Drive 파일 CRUD (`GoogleDrive` 내부 객체)
- 모달 UI 생성/표시

---

## 4. API 전체 명세

### 4.1 HTTP REST API (`server.rs`)

#### 페이지 라우트

| 경로 | 메서드 | 응답 | 캐시 | 설명 |
|------|--------|------|------|------|
| `/` | GET | 302 → `/control` | - | 루트 리다이렉트 |
| `/overlay` | GET | HTML (boot_id 치환) | no-cache | 오버레이. `{BOOT_ID}` → 실제 타임스탬프 |
| `/control` | GET | HTML | no-cache | 컨트롤 패널 |
| `/static/{name}.css` | GET | CSS | no-cache | 스타일시트 (control/overlay/launcher/chip) |
| `/static/favicon.ico` | GET | ICO | Cache 1h | 파비콘 |
| `/js/{name}.js` | GET | JS | no-cache | JS 모듈 (utils/gradient-editor/chip-preview/cloud-auth) |
| `/obs-local-file` | GET | HTML (generated) | no-cache | OBS 로컬 파일. `overlay.html` 기반 + CSS 인라인 + 포트 바인딩 + 현재 설정 스냅샷 |

#### REST API

| 경로 | 메서드 | 요청 | 응답 | 설명 |
|------|--------|------|------|------|
| `/api/windows` | GET | - | `[{hwnd, title, process, class}]` | 모든 가시 창 목록 |
| `/api/foreground` | GET | - | `{hwnd, title, process, class}` | 현재 포그라운드 창 |
| `/api/target` | GET | - | `{mode, value}` | 현재 타겟 설정 |
| `/api/target` | POST | `{mode, value}` | `{status:"ok"}` | 타겟 설정 변경 + 저장 |
| `/api/config` | GET | - | `{port}` | 서버 포트 |
| `/api/config` | POST | `{port}` | `{status:"ok"}` | 포트 변경 + 저장 |
| `/api/overlay-config` | GET | - | `OverlayConfig` (20+ 필드) | 오버레이 스타일링 전체 |
| `/api/overlay-config` | POST | `OverlayConfig` (부분) | `{status:"ok"}` | 오버레이 스타일링 업데이트 |
| `/api/launcher-language` | GET | - | `{language: "ko"\|"en"}` | UI 언어 |
| `/api/focus` | POST | `{hwnd}` | `{status:"ok"}` | HWND로 창 포커스 (Windows) |
| `/api/key-images` | GET | - | `KeyImagesConfig` | 키 커스텀 이미지 전체 |
| `/api/key-images` | POST | `KeyImagesConfig` | `{status:"ok"}` | 키 이미지 업데이트 + 저장 |
| `/api/key-style` | GET | - | `KeyStyleConfig` | 키 스타일 그룹 전체 |
| `/api/key-style` | POST | `KeyStyleConfig` | `{status:"ok"}` | 키 스타일 업데이트 + 저장 |

#### WebSocket (`/ws`)

**연결**: `ws://localhost:{port}/ws`

**서버 → 클라이언트 메시지**:
```json
// 초기 메시지 (연결 즉시)
{"type": "hello", "boot_id": 1711234567890}

// 키 상태 변경 시 (watch 채널을 통한 이벤트 기반)
{"type": "keys", "keys": ["A", "CTRL", "SHIFT"]}

// 서버 종료 시
{"type": "shutdown"}
```

**클라이언트 동작**:
- `hello`의 `boot_id`를 이전 값과 비교 → 불일치 시 페이지 리로드 (서버 재시작 감지)
- `keys` 메시지로 오버레이 업데이트
- `shutdown` 또는 연결 끊김 시 재연결 루프 (1초 간격)

### 4.2 Cloudflare Worker API (`worker/index.js`)

**배포 URL**: `https://keyviewer-oauth.rudghrnt.workers.dev`

| 경로 | 메서드 | 파라미터 | 설명 |
|------|--------|----------|------|
| `/auth` | GET | `?port=8000&path=/control` | GitHub OAuth 시작 (레거시 호환) |
| `/auth/github` | GET | `?port=8000&path=/control` | GitHub OAuth 시작 |
| `/auth/google` | GET | `?port=8000&path=/control` | Google OAuth 시작 |
| `/callback` | GET | `?code=...&state=...` | GitHub 콜백 (레거시) |
| `/callback/github` | GET | `?code=...&state=...` | GitHub 콜백 |
| `/callback/google` | GET | `?code=...&state=...` | Google 콜백 |
| `/refresh/google` | POST | `{refresh_token: "..."}` | Google 토큰 갱신 |

**OAuth 흐름**:
```
클라이언트 → Worker(/auth/github?port=8000&path=/control)
    → GitHub OAuth 페이지 (HMAC-SHA256 서명된 state)
    → GitHub → Worker(/callback/github?code=...&state=...)
    → state 서명 검증 + 만료 체크 (10분)
    → port/path 입력 검증
    → http://localhost:{port}{path}#github_token={encodeURIComponent(token)}
    → 클라이언트가 URL fragment에서 토큰 수신
```

**보안**:
- State: HMAC-SHA256 서명 + 10분 만료 (CSRF 방지)
- Origin: `localhost`/`127.0.0.1`만 허용 (CORS)
- 입력: port 숫자 검증(1-65535), path 화이트리스트(`/[a-zA-Z0-9/_.-]*`)
- 토큰 전달: URL fragment(`#`) 사용 → Referer/서버로그에 미노출
- HMAC 비교: constant-time (타이밍 공격 방지)
- XSS 방지: `JSON.stringify()`로 스크립트 내 값 이스케이핑

### 4.3 Tauri IPC 커맨드 (`main.rs`)

| 커맨드 | 파라미터 | 반환 | 설명 |
|--------|----------|------|------|
| `get_launcher_settings` | - | `LauncherSettings` | 저장된 런처 설정 로드 |
| `save_port_setting` | `port: u16` | `Result<(), String>` | 포트 저장 (레지스트리/UserDefaults) |
| `save_language_setting` | `language: String` | `Result<(), String>` | 언어 저장 |
| `get_server_status` | - | `ServerStatus` | `{running, port}` |
| `start_server` | `port: u16` | `Result<(), String>` | HTTP/WS 서버 시작 |
| `stop_server` | - | `Result<(), String>` | 서버 중지 |
| `minimize_to_tray` | - | - | 창 숨기고 트레이 아이콘 생성 |
| `set_run_on_startup` | `enabled: bool` | `Result<(), String>` | Windows Run 레지스트리 등록 |
| `set_console_visible` | `visible: bool` | - | Windows 디버그 콘솔 표시/숨김 |
| `reset_settings` | - | `Result<(), String>` | 모든 설정 초기화 |
| `check_macos_permissions` | - | `MacOSPermissions` | 3개 권한 상태 |
| `open_macos_permission_settings` | `permission: String` | - | macOS 시스템 설정 열기 |
| `open_url` | `url: String` | `Result<(), String>` | 외부 브라우저로 URL 열기 |
| `get_port_processes` | `port: u16` | `Vec<PortProcessInfo>` | 포트 사용 프로세스 목록 |
| `kill_process` | `pid: u32` | `Result<(), String>` | 프로세스 강제 종료 |

---

## 5. 데이터 구조 상세

### 5.1 핵심 상태 (`state.rs`)

```rust
pub struct AppState {
    // === 키보드 상태 ===
    pub key_labels: HashMap<u32, String>,           // VK코드/키코드 → 표시 레이블
    pub label_counts: HashMap<String, u32>,         // 레이블 → 참조 카운트
    pub label_order: VecDeque<String>,              // 최초 누름 순서 유지

    // === 설정 ===
    pub target_config: TargetConfig,                // 타겟 창 필터링 규칙
    pub app_config: AppConfig,                      // 서버 + UI 설정 전체

    // === 앱 런타임 ===
    pub language: String,                           // "ko" | "en"
    pub server_alive: bool,                         // 서버 상태
    pub event_tx: Option<watch::Sender<Vec<String>>>, // WS 브로드캐스트 채널
    pub cache_buster: u64,                          // 부트 타임스탬프 (캐시 무효화)
}
```

### 5.2 참조 카운팅 (키 추적)

여러 물리 키가 같은 레이블을 공유할 수 있음 (예: LSHIFT/RSHIFT → "SHIFT"):

```
add_key(VK_LSHIFT, "SHIFT"):
  key_labels[0xA0] = "SHIFT"
  label_counts["SHIFT"] = 1  (신규)
  label_order.push_back("SHIFT")

add_key(VK_RSHIFT, "SHIFT"):
  key_labels[0xA1] = "SHIFT"
  label_counts["SHIFT"] = 2  (증가)
  → label_order에는 추가하지 않음 (이미 존재)

remove_key(VK_LSHIFT):
  label_counts["SHIFT"] = 1  (감소)
  → 아직 1 이상이므로 label_order에서 제거하지 않음

remove_key(VK_RSHIFT):
  label_counts["SHIFT"] = 0  (제거)
  → label_order에서 "SHIFT" 제거
```

### 5.3 설정 구조체

#### TargetConfig
```rust
pub struct TargetConfig {
    pub mode: String,           // "disabled" | "title" | "process" | "hwnd" | "class" | "all"
    pub value: Option<String>,  // 매칭 값 (mode에 따라)
}
```

#### AppConfig
```rust
pub struct AppConfig {
    pub port: u16,
    pub overlay_config: OverlayConfig,
    pub key_images: KeyImagesConfig,
    pub key_style: KeyStyleConfig,
}
```

#### OverlayConfig (레이아웃/타이밍)
```rust
pub struct OverlayConfig {
    // 타이밍
    pub fade_in_ms: u32,        // 페이드인 밀리초
    pub fade_out_ms: u32,       // 페이드아웃 밀리초

    // 칩 스타일링
    pub chip_bg: String,        // 배경색 hex
    pub chip_fg: String,        // 글자색 hex
    pub chip_gap: u32,          // 간격 px
    pub chip_pad_v: u32,        // 세로 패딩
    pub chip_pad_h: u32,        // 가로 패딩
    pub chip_radius: u32,       // 모서리 반경
    pub chip_font_px: u32,      // 폰트 크기
    pub chip_font_weight: u32,  // 폰트 두께

    // 레이아웃
    pub cols: u32,              // 열 수
    pub rows: u32,              // 행 수 (0 = 무제한)
    pub single_line: bool,      // 단일 줄 모드
    pub align: String,          // "left" | "center" | "right"
    pub direction: String,      // "ltr" | "rtl"

    // 그라디언트
    pub color_mode: String,     // "solid" | "gradient"
    pub grad_color1: String,    // 그라디언트 시작 색
    pub grad_color2: String,    // 그라디언트 끝 색
    pub grad_dir: String,       // CSS 방향 문자열 ("to bottom" 등)

    // 표시 모드
    pub overlay_mode: String,   // "queue" | "keyviewer"
}
```

#### KeyImagesConfig (레거시 이미지)
```rust
pub struct KeyImagesConfig {
    pub individual: HashMap<String, KeyImageEntry>,   // 키별 이미지 (base64)
    pub groups: Vec<KeyImageGroup>,                   // 그룹 이미지 (키 목록 + 이미지)
    pub all: Option<KeyImageEntry>,                   // 전체 키 기본 이미지
}

pub struct KeyImageEntry {
    pub image: String,          // base64 데이터 URI
    pub hide_text: bool,        // 텍스트 숨김 여부
}
```

#### KeyStyleConfig (새 스타일 시스템)
```rust
pub struct KeyStyleConfig {
    pub background: BackgroundConfig,       // 오버레이 전체 배경
    pub font: FontConfig,                   // 기본 폰트 설정
    pub style_groups: Vec<StyleGroup>,      // 우선순위 기반 스타일 그룹
    pub raw_style_groups_js: Option<String>, // JS 직렬화 스타일 그룹
    pub keyviewer_canvas: Option<KeyViewerCanvas>, // key viewer 모드 캔버스
}

pub struct BackgroundConfig {
    pub transparent: bool,      // OBS용 투명
    pub mode: String,           // "solid" | "gradient" | "image"
    pub color: String,          // 단색 hex
    pub gradient_stops: Vec<GradientStop>,
    pub gradient_angle: u32,
    pub image: Option<String>,  // base64 이미지
}

pub struct StyleGroup {
    pub name: String,           // 그룹 이름
    pub group_type: String,     // "individual" | "group" | "all"
    pub keys: Vec<String>,      // 적용 대상 키 목록
    pub chip_pad_v: Option<u32>,
    pub chip_pad_h: Option<u32>,
    pub chip_radius: Option<u32>,
    pub font_family: Option<String>,
    pub font_size: Option<u32>,
    pub font_weight: Option<u32>,
    pub font_color_mode: Option<String>,  // "solid" | "gradient"
    pub font_color: Option<String>,
    pub font_gradient_stops: Option<Vec<GradientStop>>,
    pub font_gradient_angle: Option<u32>,
    pub bg_mode: Option<String>,          // "solid" | "gradient" | "image"
    pub bg_color: Option<String>,
    pub bg_gradient_stops: Option<Vec<GradientStop>>,
    pub bg_gradient_angle: Option<u32>,
    pub bg_image: Option<String>,
}

pub struct KeyViewerCanvas {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<KeyViewerCell>,
}

pub struct KeyViewerCell {
    pub label: String,          // 키 레이블
    pub x: f64,                 // X % (0-100)
    pub y: f64,                 // Y % (0-100)
    pub w: f64,                 // W % (0-100)
    pub h: f64,                 // H % (0-100)
    // + 개별 스타일 오버라이드 (Optional)
}
```

#### WindowInfo
```rust
pub struct WindowInfo {
    pub hwnd: String,           // 창 핸들 (문자열, 디버그 형식)
    pub title: String,          // 창 제목
    pub process: String,        // 프로세스/앱 이름
    pub class: String,          // 창 클래스 (Windows만, macOS/Linux는 빈 문자열)
}
```

---

## 6. 설정 저장소 아키텍처

### 6.1 저장 위치

| 데이터 | Windows | macOS | Linux |
|--------|---------|-------|-------|
| 포트, 언어, 시작프로그램 | 레지스트리 `HKCU\Software\KeyViewer` | `NSUserDefaults` | `NSUserDefaults` 미지원 → 파일 |
| 타겟 설정 (mode/value) | 레지스트리 | `NSUserDefaults` | 파일 |
| 오버레이 설정 (20+ 파라미터) | 레지스트리 | `NSUserDefaults` | 파일 |
| 키 이미지 (base64) | JSON 파일 `%APPDATA%\KeyViewer\key_images.json` | JSON 파일 `~/Library/Application Support/KeyViewer/key_images.json` | JSON 파일 `~/.config/keyviewer/key_images.json` |
| 키 스타일 그룹 | JSON 파일 `%APPDATA%\KeyViewer\key_style.json` | JSON 파일 같은 경로 | JSON 파일 같은 경로 |

### 6.2 레지스트리 키 목록 (Windows)

```
HKEY_CURRENT_USER\Software\KeyViewer
├── Port              (DWORD)
├── Language          (SZ)
├── RunOnStartup      (DWORD)
├── TargetMode        (SZ)
├── TargetValue       (SZ)
├── FadeInMs          (DWORD)
├── FadeOutMs         (DWORD)
├── ChipBg            (SZ)
├── ChipFg            (SZ)
├── ChipGap           (DWORD)
├── ChipPadV          (DWORD)
├── ChipPadH          (DWORD)
├── ChipRadius        (DWORD)
├── ChipFontSize      (DWORD)
├── ChipFontWeight    (DWORD)
├── Cols              (DWORD)
├── Rows              (DWORD)
├── SingleLine        (DWORD)
├── Align             (SZ)
├── Direction         (SZ)
├── ColorMode         (SZ)
├── GradientColors    (SZ, JSON)
├── GradientDirection (DWORD)
└── DisplayMode       (SZ)
```

### 6.3 설정 로드 순서

```
main.rs 시작
├── LauncherSettings::load()      → 포트, 언어, 시작프로그램
├── load_target_config()          → 타겟 모드/값
├── load_overlay_config()         → 오버레이 20+ 파라미터
├── load_key_images_config()      → JSON 파일에서 키 이미지
└── load_key_style_config()       → JSON 파일에서 키 스타일
    └── AppState 구성 완료
```

### 6.4 settings.rs 함수 매핑

| 함수 | 읽기/쓰기 | 데이터 |
|------|-----------|--------|
| `LauncherSettings::load()` | R | 포트, 언어, 시작프로그램 |
| `LauncherSettings::save()` | W | 〃 |
| `save_target_config()` / `load_target_config()` | W/R | 타겟 모드/값 |
| `save_overlay_config()` / `load_overlay_config()` | W/R | 오버레이 전체 설정 |
| `save_key_images_config()` / `load_key_images_config()` | W/R | 키 이미지 JSON |
| `save_key_style_config()` / `load_key_style_config()` | W/R | 키 스타일 JSON |
| `set_windows_startup()` | W | Windows Run 레지스트리 |
| `reset_all_settings()` | W | 모든 설정 삭제 |
| `get_config_dir()` | R | 플랫폼별 설정 디렉터리 경로 |

---

## 7. 오버레이 렌더링 시스템

### 7.1 Queue 모드 (기본)

```
┌────────────────────────────────┐
│  [CTRL] [SHIFT] [A] [S] [D]   │  ← flex-wrap 레이아웃
│  cols=8, rows=1                │
│  direction: LTR, align: start │
└────────────────────────────────┘
```

**렌더링 파이프라인**:
1. WS 메시지 수신 `{keys: ["CTRL", "SHIFT", "A"]}`
2. 현재 칩 목록과 비교 → 추가/제거 계산
3. 새 키: `.chip` 요소 생성 → fade-in 애니메이션
4. 제거된 키: `.hide` 클래스 추가 → fade-out 후 DOM 제거
5. 스타일 그룹 해석: 우선순위 배열 순회하며 가장 먼저 매치되는 스타일 적용

**스타일 해석 우선순위**:
```
styleGroups[0] → individual key match?  → 적용
styleGroups[1] → group keys match?      → 적용
...
styleGroups[n] → "all" type?            → 폴백 적용
(매치 없음) → OverlayConfig 기본값 사용
```

### 7.2 Key Viewer 모드

```
┌────────────────────────────┐
│  [ESC]        [F1][F2]     │
│  [1][2][3]    [Q][W][E]    │  ← 절대 위치, 캔버스 %
│  [SHIFT]  [SPACE] [CTRL]   │
└────────────────────────────┘
```

**렌더링 파이프라인**:
1. 캔버스 크기 (width × height) 기준으로 래퍼 비율 계산
2. 종횡비 유지 스케일링 (CSS transform)
3. 각 셀: 절대 위치 (x%, y%, w%, h%)
4. WS 메시지 수신 → 눌린 키: opacity 100% + glow 효과, 안 눌린 키: opacity 30%

**Key Viewer 레이아웃 에디터** (control.html):
- 더블클릭: 새 키 추가
- 드래그: 키 이동
- 엣지 드래그: 키 리사이즈
- 🎤 녹음 모드: 키 입력으로 여러 키 한번에 배치
- 스냅 투 그리드 옵션

### 7.3 OBS 통합

**브라우저 소스 설정**:
- URL: `http://localhost:{port}/overlay`
- 크기: 오버레이 설정의 cols × rows에 맞게
- CSS: `body { background: transparent; }`

**캐시 무효화**:
- 모든 HTTP 응답에 `no-cache` 헤더
- `boot_id` 타임스탬프로 서버 재시작 감지
- OBS 브라우저 소스는 새로고침 시 WS 재연결

**로컬 파일 소스** (대안):
- `/obs-local-file` 엔드포인트에서 `overlay.html` 기반 다운로드 HTML 생성
- CSS 인라인, 포트 바인딩, 현재 overlay/key-style/key-images 스냅샷 포함
- WS 재연결 및 서버 설정 재동기화 로직 내장

---

## 8. 클라우드/OAuth 시스템

### 8.1 아키텍처

```
┌──────────┐     OAuth 시작     ┌─────────────────────┐
│ control  │ ─────────────────→ │ Cloudflare Worker   │
│ .html    │                    │ (OAuth 프록시)       │
│          │                    │                     │
│          │     302 redirect   │ GitHub/Google       │
│          │ ←───────────────── │ OAuth URL +         │
│          │                    │ HMAC signed state   │
└──────────┘                    └─────────────────────┘
      │                                │
      │  사용자 로그인                   │
      ▼                                ▼
┌──────────┐     callback       ┌─────────────────────┐
│ GitHub / │ ─────────────────→ │ Worker /callback/*  │
│ Google   │     ?code=...      │                     │
│ OAuth    │     &state=...     │ state 서명 검증     │
└──────────┘                    │ code → token 교환   │
                                │ redirect →          │
                                │ localhost#token=... │
                                └─────────────────────┘
                                       │
                              fragment에서 토큰 수신
                                       │
                                       ▼
                                ┌──────────────┐
                                │ control.html │
                                │ 쿠키에 저장   │
                                └──────────────┘
```

### 8.2 환경변수

| 변수 | 용도 | 설정 위치 |
|------|------|-----------|
| `GITHUB_CLIENT_ID` | GitHub OAuth 앱 ID | Cloudflare Dashboard / `wrangler secret` |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth 시크릿 | Cloudflare Dashboard / `wrangler secret` |
| `GOOGLE_CLIENT_ID` | Google OAuth 앱 ID | Cloudflare Dashboard / `wrangler secret` |
| `GOOGLE_CLIENT_SECRET` | Google OAuth 시크릿 | Cloudflare Dashboard / `wrangler secret` |
| `STATE_SECRET` | HMAC-SHA256 서명 키 (32자+ 랜덤) | `wrangler secret put STATE_SECRET` |

### 8.3 Google Drive 파일 구조

```
Google Drive/
└── KeyViewer/                    (앱 폴더, drive.file 스코프)
    ├── keyviewer-config.json     (전체 설정 프리셋)
    └── ... (사용자 저장 프리셋)
```

### 8.4 GitHub Gist 구조

```
Gist (비공개)/
└── keyviewer-preset.json         (전체 설정 프리셋)
    ├── overlay_config: {...}
    ├── key_images: {...}
    └── key_style: {...}
```

### 8.5 클라이언트 토큰 저장

| 쿠키 | 내용 | 유효기간 |
|------|------|----------|
| `kv_github_token` | GitHub access_token (문자열) | 1년 |
| `kv_google_auth` | JSON `{access_token, refresh_token, email, picture}` | 1년 |

**토큰 수신**: URL fragment(`#`)에서 파싱 → 쿠키 저장 → `history.replaceState`로 URL 정리

---

## 9. 빌드/CI/CD 파이프라인

### 9.1 Cargo 별칭

```toml
# .cargo/config.toml
kv      = "run --manifest-path src-tauri/Cargo.toml --bin KBQV --target-dir dist"
kb      = "build --manifest-path src-tauri/Cargo.toml --bin KBQV --target-dir dist"
kfmt    = "fmt --manifest-path src-tauri/Cargo.toml --all -- --check"
kclippy = "clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings"
ktest   = "test --manifest-path src-tauri/Cargo.toml --all-targets --all-features"
```

### 9.2 빌드 프로필

| 프로필 | LTO | codegen-units | opt-level | strip | 용도 |
|--------|-----|---------------|-----------|-------|------|
| `release` | full | 1 | `z` (크기 최적화) | true | 배포 빌드 |
| `ci-release` | thin | 16 | 2 | false | CI 속도 최적화 |

### 9.3 크레이트 의존성

**공통**:
| 크레이트 | 버전 | 용도 |
|----------|------|------|
| `tauri` | 2 | 데스크톱 프레임워크 + 트레이 |
| `serde` / `serde_json` | 1 | 직렬화 |
| `tokio` | 1 (full) | 비동기 런타임 |
| `axum` | 0.7 | 웹 프레임워크 |
| `tower-http` | 0.6 | CORS 미들웨어 |
| `parking_lot` | 0.12 | 확장 Mutex/RwLock |
| `once_cell` | 1.19 | 정적 초기화 |
| `rdev` | 0.5 | 크로스 플랫폼 입력 이벤트 |
| `open` | 5.0 | URL/파일 열기 |
| `single-instance` | 0.3 | 다중 실행 방지 |

**Windows 전용**:
| 크레이트 | 용도 |
|----------|------|
| `winreg` | 레지스트리 접근 |
| `windows` | Win32 API (8개 피처) |

**macOS 전용**:
| 크레이트 | 용도 |
|----------|------|
| `cocoa` | macOS AppKit |
| `core-foundation` | CF 타입 |
| `core-graphics` | CG API (이벤트 탭) |
| `objc` | Objective-C 런타임 |

**Linux 전용**:
| 크레이트 | 용도 |
|----------|------|
| `x11` | X11 윈도 시스템 |

### 9.4 CI 워크플로우 (test.yml)

**트리거**: main/master 브랜치 push/PR (특정 경로 필터)

```
prepare (ubuntu)
    └── 버전 추출 (Cargo.toml)

fmt-clippy-test-linux (ubuntu)
    ├── cargo kfmt
    ├── cargo kclippy
    └── cargo ktest

build-windows-portable (windows)
    └── cargo kb --release → .exe + .zip

build-linux-portable (ubuntu)
    └── cargo kb --release → binary + .tar.gz

build-macos-portable (macos)
    └── cargo kb --release → binary + .zip (아키텍처별)
```

### 9.5 CD 워크플로우 (tauri-build.yml)

**트리거**: test.yml 성공 후 자동 또는 수동

```
release
    ├── 코드 체크아웃 (full history)
    ├── 버전 읽기
    ├── test.yml 아티팩트 다운로드
    ├── 빌드 성공 체크리스트 생성
    ├── git tag 생성: v{version}
    └── GitHub Release 생성 + 아티팩트 업로드
```

### 9.6 Docker 환경

| 서비스 | 이미지 | 목적 |
|--------|--------|------|
| `linux-build` | `Dockerfile.linux` (Ubuntu 22.04) | Linux 앱 빌드 (DEB, AppImage) |
| `macos-check` | `Dockerfile.macos-cross` (Ubuntu 22.04) | macOS 컴파일 체크 (빌드 불가) |

### 9.7 Cloudflare Worker 배포

```bash
cd worker
wrangler deploy
# → https://keyviewer-oauth.rudghrnt.workers.dev
```

---

## 10. 문서 변경 규칙

### 10.1 규칙 개요

> **코드를 수정할 때, 변경사항이 이 문서 또는 `.github/copilot-instructions.md`의 내용과 불일치를 발생시킨다면 해당 문서도 반드시 함께 수정해야 합니다.**

### 10.2 변경 매핑 테이블

| 변경 유형 | 업데이트 대상 |
|-----------|--------------|
| API 엔드포인트 추가/수정/삭제 | `copilot-instructions.md` §6 + 이 파일 §4 |
| Tauri IPC 커맨드 추가/수정 | `copilot-instructions.md` §6 + 이 파일 §4.3 |
| 새 파일/모듈 추가 | `copilot-instructions.md` §2 + 이 파일 §2 |
| 데이터 구조(state) 변경 | `copilot-instructions.md` §7 + 이 파일 §5 |
| 설정 저장 방식 변경 | `copilot-instructions.md` §5.4 + 이 파일 §6 |
| 오버레이 모드/렌더링 변경 | `copilot-instructions.md` §8 + 이 파일 §7 |
| Worker/OAuth 변경 | `copilot-instructions.md` §5.3, §6 + 이 파일 §8 |
| UI 파일(html/css/js) 추가 | `copilot-instructions.md` §2 + `server.rs` 라우트 + 이 파일 §2, §3 |
| 빌드/CI 변경 | `copilot-instructions.md` §9 + 이 파일 §9 |
| 디렉터리 구조 변경 | `copilot-instructions.md` §2 + 이 파일 §2 |
| 크레이트 의존성 추가/제거 | 이 파일 §9.3 |
| CSS 변수 변경 | 이 파일 §3.2 |
| 설정 레지스트리 키 변경 | 이 파일 §6.2 |
| 프리셋/클라우드 구조 변경 | 이 파일 §8.3, §8.4 |

### 10.3 검증 체크리스트

변경 완료 후 다음을 확인하세요:

- [ ] 새 API 엔드포인트가 §4에 기록되었는가?
- [ ] 새 IPC 커맨드가 §4.3에 기록되었는가?
- [ ] 새 파일이 §2에 기록되었는가?
- [ ] 데이터 구조 변경이 §5에 반영되었는가?
- [ ] `copilot-instructions.md`의 대응 섹션도 업데이트되었는가?

**문서를 업데이트하지 않으면 이후 에이전트가 잘못된 정보를 참조하여 오류를 발생시킵니다.**
