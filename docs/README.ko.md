# KeyQueueViewer 가이드 (한국어)

이 문서는 KeyQueueViewer(키 입력 모니터링 도구)를 설치/설정/빌드하는 완전 가이드입니다.

## 기능 요약
- 실시간 키 입력 모니터링
- 웹 기반 인터페이스로 키 로그 확인
- 시스템 트레이 통합
- Windows 시작 프로그램 관리
- 자동 설치기로 쉬운 배포

## 빌드 시스템

이 프로젝트는 **PyInstaller 기반 모듈러 빌드 시스템**을 사용하여 다양한 배포 옵션을 제공합니다:

- **메인 프로그램**: `PyInstaller onedir`로 빌드 (폴더 기반 실행 파일)
- **설치기**: `PyInstaller onefile`로 빌드 (단일 실행 파일 배포)
- **휴대용 버전**: `PyInstaller onefile`로 빌드 (별도 설치 없이 실행)

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

### 전체 빌드 실행 (모든 버전)
```powershell
./build_all.ps1
```

### 개별 빌드 실행
```powershell
# 메인 프로그램 (onedir)
./onedir.ps1

# 설치기 (onefile)
./installer.ps1

# 휴대용 버전 (onefile)
./portable.ps1
```

### 빌드 과정
1. **메인 프로그램 빌드**: PyInstaller onedir로 폴더 기반 실행 파일 생성
2. **설치기 빌드**: PyInstaller onefile로 단일 실행 파일 생성
3. **휴대용 버전 빌드**: PyInstaller onefile로 별도 설치 없이 실행 가능한 버전 생성
4. **패키지 생성**: 메인 프로그램을 ZIP 파일로 압축

### 빌드 결과
```
dist/
├── KBQV-v1.0.4/                    # 메인 프로그램 (onedir)
├── KBQV-Installer-1.0.4.exe       # 설치기 (onefile)
├── KBQV-Portable-1.0.4.exe        # 휴대용 버전 (onefile)
└── KBQV-v1.0.4.zip               # 메인 프로그램 압축 파일
```

## 3) 설치 과정

### 설치기 사용 (권장)
1. **설치기 실행**: `KBQV-Installer-1.0.4.exe` 실행
2. **경로 선택**: 설치 디렉토리 선택 (기본: `C:\Program Files\KeyQueueViewer`)
3. **바로가기 옵션**: 바탕화면 및 시작 메뉴 바로가기 생성 여부 선택
4. **설치 완료**: 프로그램이 자동으로 등록되고 바로가기 생성

### 휴대용 버전 사용
1. **직접 실행**: `KBQV-Portable-1.0.4.exe`를 원하는 위치에 복사하여 실행
2. **별도 설치 불필요**: 레지스트리나 시스템 폴더에 파일을 복사하지 않음
3. **이동 가능**: USB나 다른 컴퓨터로 쉽게 이동 가능

### 압축 파일 사용
1. **압축 해제**: `KBQV-v1.0.4.zip`을 원하는 위치에 압축 해제
2. **실행**: 압축 해제된 폴더 내의 `KBQV-v1.0.4.exe` 실행

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
├── build_all.ps1             # 전체 빌드 스크립트
├── onedir.ps1                # 메인 프로그램 빌드 스크립트
├── installer.ps1             # 설치기 빌드 스크립트
├── portable.ps1              # 휴대용 버전 빌드 스크립트
├── requirements.txt           # 의존성 관리
└── version.txt               # 버전 정보
```

## 빌드 옵션

### onedir (폴더 기반)
- **장점**: 빠른 시작, 디버깅 용이, 파일 수정 가능
- **단점**: 여러 파일로 구성, 배포 시 주의 필요
- **용도**: 개발, 테스트, 사용자 정의

### onefile (단일 파일)
- **장점**: 단일 실행 파일, 배포 간편, 이동 용이
- **단점**: 시작 시간 느림, Windows Defender 오탐 가능성
- **용도**: 최종 배포, 사용자 설치

## 보안 기능

- **PyInstaller 빌드**: 안정적이고 검증된 빌드 시스템
- **모듈러 접근법**: 용도에 맞는 최적화된 빌드 옵션 제공
- **자체 포함**: 각 빌드에 필요한 모든 파일 포함
- **레지스트리 통합**: 설치기 사용 시 적절한 Windows 프로그램 등록

## 문제 해결

### 빌드 문제
- Python 3.11+ 설치 확인
- PowerShell을 관리자 권한으로 실행
- 가상환경 설정 확인
- PyInstaller 설치 확인: `pip install PyInstaller`

### 설치 문제
- 설치기를 관리자 권한으로 실행
- 대상 디렉토리가 쓰기 가능한지 확인
- Windows Defender 제외 설정 확인 (필요 시)

### 실행 문제
- 정렬/방향이 어긋남: OBS 사용자 지정 CSS의 `margin: 0;` 적용 + 소스 새로고침
- 포트 충돌: 런처가 감지 시 기존 Control을 자동 오픈
- 키가 고정됨: 관리자 권한 실행, 보안 프로그램 확인

## 주의사항

- **"모든 창" 모드**: 민감한 입력 상황에서 오버레이가 표시될 수 있으므로 주의 (2초 지연 확인 포함)
- **관리자 권한**: 전역 키 후킹 시 권장
- **Windows Defender**: onefile 빌드는 오탐 가능성이 있으므로 필요 시 제외 설정

## 지원

문제나 질문이 있으면 다음을 확인하세요:
- GitHub Issues
- 프로젝트 문서
- 빌드 로그 및 오류 메시지
- PowerShell 실행 정책 설정