# 사용자 가이드 / User Guide

[한국어](#한국어) | [English](#english)

---

## 한국어

### 📋 목차
- [설치](#설치)
- [시작하기](#시작하기)
- [주요 기능](#주요-기능)
- [설정 가이드](#설정-가이드)
- [OBS 통합](#obs-통합)
- [문제 해결](#문제-해결)

---

### 🚀 설치

#### Windows

1. **GitHub Releases**에서 다운로드:
   - **MSI Installer** (권장): `KeyQueueViewer_*_x64_en-US.msi`
   - **NSIS Setup**: `KeyQueueViewer_*_x64-setup.exe`
   - **Portable**: `KBQV-Portable-*.zip` (설치 불필요)

2. 설치 파일 실행 후 마법사 지침을 따릅니다

3. 설치 완료!

#### macOS

1. **GitHub Releases**에서 다운로드:
   - Intel Mac: `KeyQueueViewer_*_x64.dmg`
   - Apple Silicon (M1/M2/M3): `KeyQueueViewer_*_aarch64.dmg`

2. DMG 파일을 열고 Applications 폴더로 드래그

3. 처음 실행 시 "확인되지 않은 개발자" 경고가 나타나면:
   - `시스템 환경설정` → `보안 및 개인 정보 보호`
   - "확인 없이 열기" 클릭

#### Linux

**Debian/Ubuntu:**
```bash
sudo dpkg -i keyqueueviewer_*.deb
```

**AppImage (모든 배포판):**
```bash
chmod +x keyqueueviewer_*.AppImage
./keyqueueviewer_*.AppImage
```

---

### 🎯 시작하기

#### 1. GUI 런처 실행

앱을 실행하면 GUI 런처 창이 나타납니다.

**기본 설정:**
- **언어**: 한국어 / English
- **포트**: 8000 (기본값, 필요시 변경 가능)
- **Windows 시작 시 실행**: 체크박스로 설정

#### 2. 서버 시작

1. 포트 및 언어 설정
2. **"서버 시작"** 버튼 클릭
3. 서버가 시작되면 상태가 "실행 중"으로 표시됩니다

#### 3. 웹 인터페이스 접속

서버가 실행 중일 때:

- **컨트롤 패널**: `http://localhost:8000/control`
  - 타겟 설정, 오버레이 구성 등
  
- **오버레이**: `http://localhost:8000/overlay`
  - OBS/XSplit 등에서 Browser Source로 사용

---

### ⚙️ 주요 기능

#### 1. 타겟 모드 설정

컨트롤 패널 (`/control`)에서 키 입력을 감지할 대상을 설정합니다:

| 모드 | 설명 | 예시 |
|------|------|------|
| **사용 안 함** | 키 입력을 감지하지 않음 | - |
| **제목 (포함)** | 윈도우 제목에 값이 포함되면 감지 | "Notepad" |
| **프로세스 (정확)** | 프로세스 이름과 정확히 일치 | "notepad.exe" |
| **HWND** | 특정 윈도우 핸들 | "12345678" |
| **클래스 명 (정확)** | 윈도우 클래스명과 정확히 일치 | "Notepad" |
| **모든 창** | 포커스된 모든 창에서 감지 | - |

#### 2. 창 클릭으로 빠른 설정

컨트롤 패널의 **"Currently Open Windows"** 테이블에서:

1. 원하는 **모드** 선택 (예: "제목(포함)")
2. 테이블에서 원하는 **창의 행을 클릭**
3. 자동으로 해당 창이 타겟으로 설정됩니다!

#### 3. 오버레이 커스터마이즈

컨트롤 패널에서 **"오버레이 설정"** 버튼 클릭:

**기본 설정:**
- **Fade In/Out**: 애니메이션 속도 (ms)
- **배경색**: 전체 배경 색상
- **투명 배경**: 체크하면 완전 투명
- **칩 배경색**: 각 키 칩의 배경색
- **텍스트 색**: 키 텍스트 색상

**레이아웃:**
- **칩 간격**: 키 칩 사이의 간격 (px)
- **패딩**: 칩 내부 여백 (세로/가로)
- **모서리**: 칩의 둥근 정도 (px)
- **폰트 크기**: 텍스트 크기 (px)
- **폰트 두께**: 100-900 (700 = bold)

**그리드:**
- **열 (Cols)**: 가로로 표시할 키 개수
- **행 (Rows)**: 세로 줄 수 (0 = 무제한)
- **정렬**: 좌측/중앙/우측
- **쌓이는 방향**: LTR (왼→오) / RTL (오→왼)

**미리보기:**
- 설정 변경 시 실시간 미리보기
- "저장" 버튼으로 적용

---

### 📺 OBS 통합

#### 1. Browser Source 추가

1. OBS Studio에서 **Sources** 패널
2. **+** 클릭 → **Browser** 선택
3. 이름 입력 (예: "Key Overlay")
4. **OK** 클릭

#### 2. Browser Source 설정

- **URL**: `http://localhost:8000/overlay`
- **Width**: 800 (또는 원하는 값)
- **Height**: 600 (또는 원하는 값)
- **FPS**: Custom (30)
- **Refresh browser when scene becomes active**: ✓ (체크)
- **Shutdown source when not visible**: ✓ (체크)

#### 3. 오버레이 위치 조정

1. OBS 캔버스에서 오버레이를 원하는 위치로 드래그
2. 크기 조정 (모서리 드래그)
3. 완료!

#### 4. 투명 배경 설정 (선택사항)

오버레이의 배경을 투명하게 하려면:

1. 컨트롤 패널 (`/control`)에서 **"오버레이 설정"** 클릭
2. **"투명 배경"** 체크박스 선택
3. **"저장"** 클릭
4. OBS에서 자동으로 투명하게 표시됩니다

---

### 🔧 설정 가이드

#### 포트 변경

1. GUI 런처 열기
2. **"포트"** 필드에 원하는 포트 입력 (예: 9000)
3. **"포트 저장"** 클릭
4. 서버 재시작

#### 언어 변경

1. GUI 런처에서 언어 선택 (한국어 / English)
2. 자동으로 저장됨
3. 컨트롤 패널도 자동으로 동기화 (5초 이내)

#### Windows 시작 시 자동 실행

1. GUI 런처에서 **"Windows 시작 시 실행"** 체크박스 선택
2. 자동으로 Windows 레지스트리에 등록됨
3. 체크 해제 시 자동 실행 비활성화

#### Portable 버전 설정

Portable 버전은 설정을 레지스트리 대신 `localStorage`에 저장:

- 마지막 Mode와 Value Selection 자동 저장
- 앱 재시작 시 자동 복원
- 파일 시스템에 별도 파일 저장하지 않음

---

### 🐛 문제 해결

#### 1. 서버가 시작되지 않음

**증상**: "서버 시작" 버튼을 눌러도 반응 없음

**해결 방법:**
```powershell
# 다른 프로세스가 포트를 사용 중인지 확인
netstat -ano | findstr :8000

# 프로세스 종료 (PID를 찾은 후)
taskkill /PID <PID> /F

# 또는 다른 포트 사용
```

#### 2. Control 페이지가 열리지 않음

**증상**: `http://localhost:8000/control`에 접속할 수 없음

**해결 방법:**
1. 서버가 실행 중인지 확인 (GUI 런처에서 상태 확인)
2. 방화벽이 차단하는지 확인
3. 브라우저 캐시 삭제 (`Ctrl + Shift + Delete`)
4. 다른 브라우저에서 시도

#### 3. 키 입력이 감지되지 않음

**증상**: 오버레이에 키가 표시되지 않음

**해결 방법:**
1. 컨트롤 패널에서 타겟 모드 확인
2. 올바른 창이 포커스되어 있는지 확인
3. "현재 포커스 창" 섹션에서 현재 창 정보 확인
4. "모든 창" 모드로 테스트

#### 4. Tauri API 로드 실패

**증상**: 콘솔에 "Failed to load Tauri API" 에러

**해결 방법:**
1. `F12`를 눌러 개발자 도구 열기
2. 콘솔에서 `window.__TAURI__` 입력
3. `Object`가 나오면 정상, `undefined`면 문제
4. 앱 재시작 또는 재설치

#### 5. 언어 설정이 동기화되지 않음

**증상**: GUI는 한국어인데 Control 페이지는 영어

**해결 방법:**
1. Control 페이지 새로고침 (`F5`)
2. 5초 대기 (자동 동기화)
3. 그래도 안 되면 서버 재시작

#### 6. 트레이 아이콘이 사라지지 않음

**증상**: 앱을 종료해도 트레이 아이콘이 남아있음

**해결 방법:**
```powershell
# 프로세스 강제 종료
Get-Process -Name "*keyviewer*" | Stop-Process -Force

# 또는
taskkill /IM keyviewer.exe /F
```

---

### 💡 팁과 트릭

#### 1. 빠른 타겟 설정

"Currently Open Windows" 테이블에서 원하는 창의 **행 전체를 클릭**하면 자동으로 타겟이 설정됩니다!

#### 2. 여러 타겟 사용

한 번에 하나의 타겟만 설정할 수 있습니다. 여러 창을 동시에 모니터링하려면 "모든 창" 모드를 사용하세요.

#### 3. OBS에서 성능 최적화

- **FPS**: 30-60 권장
- **Shutdown source when not visible**: 체크 (메모리 절약)
- **Refresh browser when scene becomes active**: 체크 (안정성)

#### 4. 게임과 함께 사용

게임이 전체화면 모드일 때:
1. 창 모드 또는 테두리 없는 창 모드로 변경
2. 또는 "프로세스" 모드 사용 (예: `game.exe`)

---

## English

### 📋 Table of Contents
- [Installation](#installation)
- [Getting Started](#getting-started)
- [Main Features](#main-features)
- [Configuration Guide](#configuration-guide)
- [OBS Integration](#obs-integration)
- [Troubleshooting](#troubleshooting)

---

### 🚀 Installation

#### Windows

1. Download from **GitHub Releases**:
   - **MSI Installer** (recommended): `KeyQueueViewer_*_x64_en-US.msi`
   - **NSIS Setup**: `KeyQueueViewer_*_x64-setup.exe`
   - **Portable**: `KBQV-Portable-*.zip` (no installation required)

2. Run the installer and follow the wizard

3. Done!

#### macOS

1. Download from **GitHub Releases**:
   - Intel Mac: `KeyQueueViewer_*_x64.dmg`
   - Apple Silicon (M1/M2/M3): `KeyQueueViewer_*_aarch64.dmg`

2. Open the DMG file and drag to Applications folder

3. First run: If you see "Unverified Developer" warning:
   - Go to `System Preferences` → `Security & Privacy`
   - Click "Open Anyway"

#### Linux

**Debian/Ubuntu:**
```bash
sudo dpkg -i keyqueueviewer_*.deb
```

**AppImage (all distributions):**
```bash
chmod +x keyqueueviewer_*.AppImage
./keyqueueviewer_*.AppImage
```

---

### 🎯 Getting Started

#### 1. Launch GUI Launcher

When you run the app, the GUI launcher window appears.

**Basic Settings:**
- **Language**: Korean / English
- **Port**: 8000 (default, change if needed)
- **Run on Windows Startup**: Checkbox to enable

#### 2. Start Server

1. Configure port and language
2. Click **"Start Server"** button
3. Status will show "Running" when server starts

#### 3. Access Web Interface

When server is running:

- **Control Panel**: `http://localhost:8000/control`
  - Configure target, overlay settings, etc.
  
- **Overlay**: `http://localhost:8000/overlay`
  - Use as Browser Source in OBS/XSplit

---

### ⚙️ Main Features

#### 1. Target Mode Configuration

Set target for key input detection in Control Panel (`/control`):

| Mode | Description | Example |
|------|-------------|---------|
| **Disabled** | No key detection | - |
| **Title (Contains)** | Matches if window title contains value | "Notepad" |
| **Process (Exact)** | Exact match with process name | "notepad.exe" |
| **HWND** | Specific window handle | "12345678" |
| **Class Name (Exact)** | Exact match with window class | "Notepad" |
| **All Windows** | Detect in all focused windows | - |

#### 2. Quick Setup by Clicking Windows

In Control Panel's **"Currently Open Windows"** table:

1. Select desired **mode** (e.g., "Title (Contains)")
2. **Click any row** in the table
3. That window is automatically set as target!

#### 3. Customize Overlay

Click **"Overlay Settings"** button in Control Panel:

**Basic Settings:**
- **Fade In/Out**: Animation speed (ms)
- **Background Color**: Overall background color
- **Transparent Background**: Check for full transparency
- **Chip Background**: Background color of each key chip
- **Text Color**: Key text color

**Layout:**
- **Chip Gap**: Space between key chips (px)
- **Padding**: Inner spacing (vertical/horizontal)
- **Corner Radius**: Roundness of chips (px)
- **Font Size**: Text size (px)
- **Font Weight**: 100-900 (700 = bold)

**Grid:**
- **Columns (Cols)**: Number of keys horizontally
- **Rows**: Number of vertical lines (0 = unlimited)
- **Alignment**: Left/Center/Right
- **Direction**: LTR (Left→Right) / RTL (Right→Left)

**Preview:**
- Real-time preview when changing settings
- Click "Save" to apply

---

### 📺 OBS Integration

#### 1. Add Browser Source

1. In OBS Studio **Sources** panel
2. Click **+** → Select **Browser**
3. Enter name (e.g., "Key Overlay")
4. Click **OK**

#### 2. Configure Browser Source

- **URL**: `http://localhost:8000/overlay`
- **Width**: 800 (or desired value)
- **Height**: 600 (or desired value)
- **FPS**: Custom (30)
- **Refresh browser when scene becomes active**: ✓ (check)
- **Shutdown source when not visible**: ✓ (check)

#### 3. Position Overlay

1. Drag overlay to desired position in OBS canvas
2. Resize (drag corners)
3. Done!

#### 4. Set Transparent Background (Optional)

To make overlay background transparent:

1. Go to Control Panel (`/control`) and click **"Overlay Settings"**
2. Check **"Transparent Background"** checkbox
3. Click **"Save"**
4. OBS will automatically show it as transparent

---

### 🔧 Configuration Guide

#### Change Port

1. Open GUI Launcher
2. Enter desired port in **"Port"** field (e.g., 9000)
3. Click **"Save Port"**
4. Restart server

#### Change Language

1. Select language in GUI Launcher (Korean / English)
2. Automatically saved
3. Control Panel syncs automatically (within 5 seconds)

#### Run on Windows Startup

1. Check **"Run on Windows Startup"** checkbox in GUI Launcher
2. Automatically registered in Windows Registry
3. Uncheck to disable auto-start

#### Portable Version Settings

Portable version saves settings in `localStorage` instead of Registry:

- Last Mode and Value Selection automatically saved
- Auto-restored on app restart
- No separate file saved to filesystem

---

### 🐛 Troubleshooting

#### 1. Server Won't Start

**Symptom**: No response when clicking "Start Server"

**Solution:**
```powershell
# Check if another process is using the port
netstat -ano | findstr :8000

# Kill process (after finding PID)
taskkill /PID <PID> /F

# Or use a different port
```

#### 2. Control Page Won't Open

**Symptom**: Cannot access `http://localhost:8000/control`

**Solution:**
1. Verify server is running (check status in GUI Launcher)
2. Check if firewall is blocking
3. Clear browser cache (`Ctrl + Shift + Delete`)
4. Try another browser

#### 3. Key Input Not Detected

**Symptom**: No keys showing in overlay

**Solution:**
1. Check target mode in Control Panel
2. Verify correct window is focused
3. Check current window info in "Current Focus Window" section
4. Test with "All Windows" mode

#### 4. Tauri API Load Failed

**Symptom**: "Failed to load Tauri API" error in console

**Solution:**
1. Press `F12` to open Developer Tools
2. Enter `window.__TAURI__` in console
3. Should return `Object`, if `undefined` there's a problem
4. Restart app or reinstall

#### 5. Language Not Syncing

**Symptom**: GUI is in Korean but Control Page is in English

**Solution:**
1. Refresh Control Page (`F5`)
2. Wait 5 seconds (auto-sync)
3. If still not working, restart server

#### 6. Tray Icon Won't Disappear

**Symptom**: Tray icon remains after closing app

**Solution:**
```powershell
# Force kill process
Get-Process -Name "*keyviewer*" | Stop-Process -Force

# Or
taskkill /IM keyviewer.exe /F
```

---

### 💡 Tips and Tricks

#### 1. Quick Target Setup

Click **entire row** in "Currently Open Windows" table to automatically set that window as target!

#### 2. Multiple Targets

Only one target can be set at a time. To monitor multiple windows simultaneously, use "All Windows" mode.

#### 3. Optimize Performance in OBS

- **FPS**: 30-60 recommended
- **Shutdown source when not visible**: Check (saves memory)
- **Refresh browser when scene becomes active**: Check (stability)

#### 4. Using with Games

When game is in fullscreen mode:
1. Change to windowed or borderless window mode
2. Or use "Process" mode (e.g., `game.exe`)

---

**Need Help?** Report issues at [GitHub Issues](https://github.com/YOUR_USERNAME/keyviewer/issues)!

