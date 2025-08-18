# KeyQueueViewer 가이드 (한국어)

이 문서는 KeyQueueViewer(키 입력 모니터링 도구)를 설치/설정/빌드하는 완전 가이드입니다.

## 기능 요약
- 실시간 키 입력 모니터링
- 웹 기반 인터페이스로 키 로그 확인
- 시스템 트레이 통합
- Windows 시작 프로그램 관리
- 자동 설치기로 쉬운 배포

## 빌드 시스템

이 프로젝트는 **하이브리드 빌드 접근법**을 사용하여 보안과 편의성을 모두 최적화합니다:

- **메인 프로그램**: `cx_Freeze`로 빌드 (Windows Defender 오탐 감소)
- **설치기**: `PyInstaller onefile`로 빌드 (단일 실행 파일 배포)

## 요구 사항
- Windows 10/11
- Python 3.11+
- PowerShell (빌드용)
- 가상환경 (권장)

## 1) 개발 모드 실행
PowerShell에서 실행:
```powershell
cd <프로젝트 경로>
Set-ExecutionPolicy Bypass -Scope Process -Force
./run.ps1
```
- 서버 실행 후:
  - Control: `http://127.0.0.1:포트/control`
  - Overlay: `http://127.0.0.1:포트/overlay` (기본 8000)

## 2) 빌드 및 설치 (권장)

### 하이브리드 빌드 실행
```powershell
./build_hybrid.ps1
```

### 빌드 과정
1. **메인 프로그램 빌드**: cx_Freeze로 안전하게 빌드
2. **설치기 빌드**: PyInstaller onefile로 단일 실행 파일 생성
3. **패키지 생성**: 설치기 + 메인 프로그램을 자동으로 패키징

### 빌드 결과
```
dist/
├── KeyQueueViewer_Main/          # 메인 프로그램 (cx_Freeze)
├── KeyQueueViewer_Installer.exe  # 설치기 (PyInstaller onefile)
└── KeyQueueViewer_Installer_Complete/  # 배포용 패키지
    ├── KeyQueueViewer_Installer.exe
    ├── main_program/
    └── README.txt
```

## 3) 설치 과정

1. **설치기 실행**: `KeyQueueViewer_Installer.exe` 실행
2. **경로 선택**: 설치 디렉토리 선택 (기본: `C:\Program Files\KeyQueueViewer`)
3. **바로가기 옵션**: 바탕화면 및 시작 메뉴 바로가기 생성 여부 선택
4. **설치 완료**: 프로그램이 자동으로 등록되고 바로가기 생성

## 4) OBS 설정
1. 브라우저 소스 추가
2. URL: `http://127.0.0.1:포트/overlay`
3. 사용자 지정 CSS 빈칸으로 설정:
```css
body { background-color: rgba(0,0,0,0); margin: 0; overflow: hidden; }
```

## 5) 프로젝트 구조
```
keyviewer/
├── app/
│   └── launcher.py           # 메인 애플리케이션
├── installer.py              # 자동 설치기
├── setup_main.py             # cx_Freeze 설정
├── build_hybrid.ps1          # 빌드 스크립트
├── requirements.txt           # 의존성 관리
└── version.txt               # 버전 정보
```

## 보안 기능

- **cx_Freeze 빌드**: Windows Defender 오탐 감소
- **하이브리드 접근법**: 보안(cx_Freeze)과 편의성(PyInstaller) 결합
- **자체 포함**: 설치기에 모든 필요한 파일 포함
- **레지스트리 통합**: 적절한 Windows 프로그램 등록

## 문제 해결

### 빌드 문제
- Python 3.11+ 설치 확인
- PowerShell을 관리자 권한으로 실행
- 가상환경 설정 확인

### 설치 문제
- 설치기를 관리자 권한으로 실행
- 대상 디렉토리가 쓰기 가능한지 확인
- Windows Defender 제외 설정 확인

### 실행 문제
- 정렬/방향이 어긋남: OBS 사용자 지정 CSS의 `margin: 0;` 적용 + 소스 새로고침
- 포트 충돌: 런처가 감지 시 기존 Control을 자동 오픈
- 키가 고정됨: 관리자 권한 실행, 보안 프로그램 확인

## 주의사항

- **"모든 창" 모드**: 민감한 입력 상황에서 오버레이가 표시될 수 있으므로 주의 (2초 지연 확인 포함)
- **관리자 권한**: 전역 키 후킹 시 권장
- **Windows Defender**: cx_Freeze 빌드로 오탐 감소

## 지원

문제나 질문이 있으면 다음을 확인하세요:
- GitHub Issues
- 프로젝트 문서
- 빌드 로그 및 오류 메시지