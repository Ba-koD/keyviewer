# KeyViewer (KeyQueueViewer) — Copilot 지시 문서

> **⚠️ 필수 규칙: 이 프로젝트의 코드를 수정할 때, 변경사항이 이 문서 또는 `docs/ARCHITECTURE.md`의 내용과 불일치가 발생한다면 해당 문서도 반드시 함께 수정해야 합니다.**  
> **파일 추가/삭제, API 엔드포인트 변경, 새 기능 추가, 설정 구조 변경 시 관련 문서 섹션을 즉시 업데이트하세요.**

---

## 1. 프로젝트 개요

- **이름**: KeyViewer (바이너리명: `KBQV`, 식별자: `com.keyviewer`)
- **버전**: `version.txt` 및 `src-tauri/Cargo.toml`에서 관리 (현재 1.1.0)
- **유형**: Tauri 2.0 데스크톱 앱 (Rust 백엔드 + 웹 UI 프론트엔드)
- **목적**: 키보드 입력을 실시간 모니터링하여 OBS 등 스트리밍 소프트웨어에 오버레이로 표시
- **지원 OS**: Windows, macOS, Linux
- **라이선스**: MIT

---

## 2. 디렉터리 구조 및 역할

```
keyviewer/
├── .cargo/config.toml       # Cargo 별칭 (kv, kb, kfmt, kclippy, ktest)
├── .github/
│   ├── copilot-instructions.md  # ← 이 파일 (AI 에이전트 지시)
│   └── workflows/
│       ├── test.yml             # CI: fmt, clippy, test + 포터블 빌드
│       └── tauri-build.yml      # CD: GitHub Release 생성
├── docs/
│   ├── ARCHITECTURE.md      # 아키텍처 상세 문서 (필수 참조)
│   ├── GUIDE.md             # 사용자 가이드
│   ├── LOCAL-TESTING.md     # 로컬 빌드/테스트 가이드
│   ├── MACOS-SIGNING.md     # macOS 코드 서명 가이드
│   └── REMOTE-MAC.md        # 원격 macOS 빌드 가이드
├── src-tauri/               # Rust 백엔드 (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json      # Tauri 앱 설정
│   ├── app.manifest         # Windows 관리자 권한선언
│   └── src/
│       ├── main.rs          # 앱 진입점, IPC 커맨드, 트레이, 권한
│       ├── server.rs        # HTTP/WebSocket 서버 (axum)
│       ├── keyboard.rs      # 플랫폼별 키보드 입력 감지
│       ├── settings.rs      # 영속 설정 저장 (레지스트리/UserDefaults/파일)
│       ├── state.rs         # 인메모리 상태 & 데이터 구조
│       └── window_info.rs   # 창 목록 열거 & 포그라운드 창 추적
├── ui/                      # 웹 프론트엔드 (빌드 시 바이너리에 임베드)
│   ├── index.html           # 런처 (데스크톱 Tauri 창)
│   ├── control.html         # 웹 컨트롤 패널 (브라우저 접속)
│   ├── overlay.html         # 오버레이 (OBS 브라우저 소스)
│   ├── permissions.html     # macOS 권한 안내 페이지
│   ├── obs-local-template.html  # OBS 로컬 파일 소스용 독립 HTML
│   ├── control.css / overlay.css / launcher.css / chip.css
│   └── js/
│       ├── utils.js             # 공용 유틸리티 (색상, 그라디언트, DOM)
│       ├── gradient-editor.js   # 그라디언트 에디터 컴포넌트
│       ├── chip-preview.js      # 칩 미리보기 렌더러
│       └── cloud-auth.js        # OAuth 모듈 (GitHub/Google)
├── worker/                  # Cloudflare Worker (OAuth 프록시)
│   └── index.js
├── prev-control.html        # 이전 버전 control 참고용 (비활성)
├── prev-overlay.html        # 이전 버전 overlay 참고용 (비활성)
├── build-portable.ps1       # Windows 포터블 빌드
├── check-all.ps1 / .sh     # 품질 검사 (fmt + clippy + test)
├── convert-icon.ps1         # ICO → PNG 변환
├── docker-compose.yml       # 로컬 크로스빌드 환경
├── Dockerfile.linux         # Linux 빌드 이미지
└── Dockerfile.macos-cross   # macOS 크로스 컴파일 체크용 이미지
```

---

## 3. 핵심 아키텍처 흐름

### 실행 순서
1. `main.rs`: 앱 시작 → 관리자 권한 체크(Windows) / Accessibility 권한(macOS)
2. `main.rs`: 키보드 후킹 스레드 시작 (`keyboard.rs`)
3. `main.rs`: Tauri 런처 창 (`index.html`) 표시
4. 사용자가 런처에서 서버 시작 → `server.rs`의 HTTP/WS 서버 기동
5. 브라우저에서 `localhost:{port}/control` 접속 → 설정 관리
6. OBS에서 `localhost:{port}/overlay` 접속 → 키 오버레이 표시

### 데이터 흐름
```
키보드 입력 → keyboard.rs (플랫폼별 감지)
    → state.rs (AppState에 키 추가/제거)
    → watch::Sender로 변경 알림
    → server.rs (WebSocket으로 연결된 클라이언트에 브로드캐스트)
    → overlay.html (실시간 키 표시)
```

### 통신 방식
| 경로 | 방식 | 설명 |
|------|------|------|
| 런처 ↔ Rust | Tauri IPC (`__TAURI__.core.invoke`) | 서버 제어, 설정, 권한 |
| 브라우저 ↔ 서버 | HTTP REST API | 설정 CRUD, 창 목록 |
| 오버레이 ↔ 서버 | WebSocket (`/ws`) | 실시간 키 상태 푸시 |
| 브라우저 → Worker | HTTP (OAuth) | 클라우드 인증 프록시 |

---

## 4. Cargo 별칭 (`.cargo/config.toml`)

| 명령 | 설명 |
|------|------|
| `cargo kv` | 개발 실행 (debug) |
| `cargo kb` | 빌드 (dist 디렉터리) |
| `cargo kb --release` | 릴리스 빌드 |
| `cargo kfmt` | 포매팅 검사 |
| `cargo kclippy` | 린트 검사 |
| `cargo ktest` | 테스트 실행 |

모든 명령은 `--manifest-path src-tauri/Cargo.toml --target-dir dist`를 사용합니다.

---

## 5. 개발 규칙

### 5.1 Rust 백엔드
- **플랫폼 분기**: `#[cfg(target_os = "...")]`로 분기. 세 OS(windows/macos/linux) 모두 고려
- **스레드 안전**: `AppState`는 `Arc<RwLock<>>`로 공유. `parking_lot` 사용
- **에러 처리**: Tauri IPC 커맨드는 `Result<T, String>` 반환
- **정적 임베드**: 모든 UI 파일은 `include_str!()`로 바이너리에 임베드
- **새 UI 파일 추가 시**: `server.rs`에 라우트 + `include_str!()` 추가 필요
- **새 Tauri IPC 커맨드 추가 시**: `main.rs`의 `invoke_handler`에 등록 필요

### 5.2 프론트엔드 (UI)
- **프레임워크 없음**: 순수 HTML/CSS/JS (바닐라). 빌드 도구 없음
- **번역**: `data-i18n` 어트리뷰트 기반 한/영 이중언어
- **테마**: 다크 테마 (CSS 변수 기반: `--bg`, `--panel`, `--primary` 등)
- **JS 모듈**: `js/` 하위 파일들은 `<script>` 태그로 로드 (ES Module 아님)
- **CSS**: 파일별 분리. `chip.css`는 오버레이와 컨트롤 양쪽에서 공유
- **OBS 호환**: `no-cache` 헤더 필수. `boot_id`로 캐시 무효화

### 5.3 클라우드/OAuth (`worker/index.js`)
- **Cloudflare Workers** 기반 OAuth 프록시
- **보안**: HMAC-SHA256 서명된 state, Origin 화이트리스트, 입력값 검증
- **토큰 전달**: URL fragment(`#`) 사용 (query string 아님)
- **환경변수**: `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `STATE_SECRET`
- **배포**: `wrangler deploy` → `keyviewer-oauth.rudghrnt.workers.dev`
- **수정 시**: 클라이언트 측 (`cloud-auth.js`, `control.html`)의 토큰 파싱 로직도 동기화 필요

### 5.4 설정 저장소
| 플랫폼 | 위치 | 방식 |
|--------|------|------|
| Windows | `HKEY_CURRENT_USER\Software\KeyViewer` | 레지스트리 |
| macOS | `NSUserDefaults` | UserDefaults (Obj-C) |
| 모든 OS | `~/.config/keyviewer/` (Linux), `%APPDATA%\KeyViewer` (Win), `~/Library/Application Support/KeyViewer` (Mac) | JSON 파일 (`key_images.json`, `key_style.json`) |

---

## 6. API 엔드포인트 목록

### HTTP REST (`server.rs`)

| 경로 | 메서드 | 설명 |
|------|--------|------|
| `/` | GET | `/control`로 리다이렉트 |
| `/overlay` | GET | 오버레이 HTML |
| `/control` | GET | 컨트롤 패널 HTML |
| `/static/{file}` | GET | CSS 파일 |
| `/static/favicon.ico` | GET | 파비콘 |
| `/js/{file}` | GET | JS 모듈 |
| `/ws` | WS | WebSocket (실시간 키 상태) |
| `/api/windows` | GET | 모든 창 목록 |
| `/api/foreground` | GET | 현재 포그라운드 창 |
| `/api/target` | GET/POST | 타겟 창 설정 |
| `/api/config` | GET/POST | 서버 포트 설정 |
| `/api/overlay-config` | GET/POST | 오버레이 스타일링 (20+ 파라미터) |
| `/api/launcher-language` | GET | 현재 UI 언어 |
| `/api/focus` | POST | HWND로 창 포커스 |
| `/api/key-images` | GET/POST | 키 커스텀 이미지 |
| `/api/key-style` | GET/POST | 키 스타일 (배경, 폰트, 그라디언트) |
| `/obs-local-file` | GET | OBS 로컬 파일 소스용 독립 HTML |

### Cloudflare Worker (`worker/index.js`)

| 경로 | 메서드 | 설명 |
|------|--------|------|
| `/auth` | GET | GitHub OAuth (레거시) |
| `/auth/github` | GET | GitHub OAuth |
| `/auth/google` | GET | Google OAuth |
| `/callback` | GET | GitHub 콜백 (레거시) |
| `/callback/github` | GET | GitHub 콜백 |
| `/callback/google` | GET | Google 콜백 |
| `/refresh/google` | POST | Google 토큰 갱신 |

### Tauri IPC 커맨드 (`main.rs`)

| 커맨드 | 설명 |
|--------|------|
| `get_launcher_settings` | 런처 설정 로드 |
| `save_port_setting` | 포트 저장 |
| `save_language_setting` | 언어 저장 |
| `get_server_status` | 서버 상태 확인 |
| `start_server` / `stop_server` | 서버 제어 |
| `minimize_to_tray` | 시스템 트레이로 최소화 |
| `set_run_on_startup` | 시작 시 실행 설정 |
| `set_console_visible` | 디버그 콘솔 표시(Windows) |
| `reset_settings` | 전체 설정 초기화 |
| `check_macos_permissions` | macOS 권한 확인 |
| `open_macos_permission_settings` | macOS 설정 열기 |
| `open_url` | 외부 URL 열기 |
| `get_port_processes` / `kill_process` | 포트 충돌 해결 |

---

## 7. 주요 데이터 구조 (`state.rs`)

```
AppState
├── key_labels: HashMap<u32, String>        # VK코드 → 표시 레이블
├── label_counts: HashMap<String, u32>      # 레이블별 참조 카운트
├── label_order: VecDeque<String>           # 눌린 순서
├── target_config: TargetConfig             # 타겟 창 룰
│   ├── mode: "disabled"|"title"|"process"|"hwnd"|"class"|"all"
│   └── value: Option<String>
├── app_config: AppConfig
│   ├── port: u16
│   ├── overlay_config: OverlayConfig       # 레이아웃/스타일 20+ 파라미터
│   ├── key_images: KeyImagesConfig         # 키별 이미지 (base64)
│   └── key_style: KeyStyleConfig           # 배경/폰트/그라디언트 스타일 그룹
├── language: String                        # "ko", "en"
├── server_alive: bool
├── event_tx: Option<watch::Sender>         # WebSocket 브로드캐스트 채널
└── cache_buster: u64                       # 부트 타임스탬프
```

---

## 8. 오버레이 렌더링 모드

### Queue 모드 (기본)
- 키가 순차적으로 가로 큐에 표시
- Flex 레이아웃, cols/rows로 그리드 제어
- 페이드인/아웃 애니메이션

### Key Viewer 모드
- 자유 배치 캔버스: 키를 원하는 위치에 배치
- 셀별 크기/위치/스타일 오버라이드
- 눌림 상태 시각화 (opacity + glow)
- 종횡비 유지 스케일링

---

## 9. 빌드 & 배포

### 로컬 빌드
```powershell
# Windows (개발)
cargo kv

# Windows (릴리스)
cargo tauri build

# 포터블 빌드
.\build-portable.ps1
```

### CI/CD (GitHub Actions)
1. **test.yml**: Push/PR 시 → fmt + clippy + test + 3개 OS 포터블 빌드
2. **tauri-build.yml**: test 성공 후 → GitHub Release 생성 + 아티팩트 업로드

### Docker (로컬 크로스빌드)
```powershell
docker compose run linux-build    # Linux 빌드
docker compose run macos-check    # macOS 컴파일 체크
```

### Cloudflare Worker 배포
```bash
cd worker
wrangler deploy
```

---

## 10. 문서 변경 필수 규칙

**코드 변경 시 아래 시나리오에 해당하면 반드시 문서를 업데이트하세요:**

| 변경 유형 | 업데이트 대상 |
|-----------|--------------|
| API 엔드포인트 추가/수정/삭제 | 이 파일 §6 + `docs/ARCHITECTURE.md` §4 |
| Tauri IPC 커맨드 추가/수정 | 이 파일 §6 + `docs/ARCHITECTURE.md` §4 |
| 새 파일/모듈 추가 | 이 파일 §2 + `docs/ARCHITECTURE.md` §2 |
| 데이터 구조(state) 변경 | 이 파일 §7 + `docs/ARCHITECTURE.md` §5 |
| 설정 저장 방식 변경 | 이 파일 §5.4 + `docs/ARCHITECTURE.md` §6 |
| 오버레이 모드/렌더링 변경 | 이 파일 §8 + `docs/ARCHITECTURE.md` §7 |
| Worker/OAuth 변경 | 이 파일 §5.3, §6 + `docs/ARCHITECTURE.md` §8 |
| UI 파일(html/css/js) 추가 | 이 파일 §2 + server.rs 라우트 확인 + `docs/ARCHITECTURE.md` §3 |
| 빌드/CI 변경 | 이 파일 §9 + `docs/ARCHITECTURE.md` §9 |
| 디렉터리 구조 변경 | 이 파일 §2 + `docs/ARCHITECTURE.md` §2 |

**문서를 업데이트하지 않으면 이후 에이전트가 잘못된 정보를 참조하여 오류를 발생시킵니다.**
