# KeyViewer

Rust + Tauri 기반 키 입력 오버레이 도구입니다.

## 실행 (소스 코드)

### 공통
```bash
cd src-tauri
cargo run --bin keyviewer
```

### OS별 권장 실행

#### Windows
```powershell
cd src-tauri
cargo run --bin keyviewer
```

#### macOS
```bash
cd src-tauri
cargo run --bin keyviewer
```

#### Linux
```bash
cd src-tauri
cargo run --bin keyviewer
```

## Cargo alias (프로젝트에 추가됨)

루트에서 아래 명령 사용 가능:

```bash
cargo kv
cargo kv-win
cargo kv-mac-intel
cargo kv-mac-arm
cargo kv-linux
```

참고: `kv-win`, `kv-mac-*`, `kv-linux`는 해당 target toolchain 설치가 필요합니다.

## 개발용 기본 명령

```bash
# 타입/컴파일 체크
cd src-tauri
cargo check

# Tauri 개발 모드
cargo tauri dev
```

## 빌드

### GitHub Actions에서 사용하는 스크립트
- `build-portable.ps1`
- `convert-icon.ps1`

### 로컬 수동 빌드
```bash
cd src-tauri
cargo tauri build
```

## 실행 후 접속

- 컨트롤: `http://localhost:8000/control`
- 오버레이: `http://localhost:8000/overlay`

## 이슈

- 버그/요청: `https://github.com/Ba-koD/keyviewer/issues`
