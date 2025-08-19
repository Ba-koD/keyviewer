# KeyQueueViewer PyInstaller Spec File
# This file is used for PyInstaller configuration

import os
import sys

# 버전 정보 읽기
def get_version():
    try:
        with open("version.txt", "r", encoding="utf-8") as f:
            return f.read().strip()
    except:
        return "1.0.0"

version = get_version()
build_folder = f"KBQV-v{version}"

# PyInstaller 설정
# 이 파일은 참고용이며, 실제 빌드는 다음 명령어로 수행합니다:
# pyinstaller --onedir --noconsole --name "KBQV-v{version}" --icon "web/favicon.ico" --add-data "web;web" app/launcher.py

# 필요한 모듈들 (PyInstaller가 자동으로 감지하지만, 명시적으로 포함할 수 있음)
hidden_imports = [
    "fastapi", "uvicorn", "websockets", "keyboard", "win32api", "win32con", 
    "win32gui", "psutil", "pystray", "PIL", "asyncio", "uvicorn.logging",
    "uvicorn.config", "uvicorn.protocols", "uvicorn.lifespan", "uvicorn.loops",
    "uvicorn.middleware", "uvicorn.server", "uvicorn.workers", "http", "http.client",
    "http.server", "http.cookies", "http.cookiejar", "urllib", "urllib.request",
    "urllib.parse", "urllib.error", "urllib.robotparser", "json", "zipfile",
    "tempfile", "shutil", "os", "sys", "threading", "time", "datetime"
]

# 포함할 데이터 파일들
datas = [
    ("web/", "web/"),
    ("version.txt", ".")
]

# 제외할 모듈들
excludes = []

# PyInstaller 명령어 예시
print(f"PyInstaller 명령어 예시:")
print(f"pyinstaller --onedir --noconsole --name 'KBQV-v{version}' --icon 'web/favicon.ico' --add-data 'web;web' app/launcher.py")
print(f"")
print(f"또는 onefile 버전:")
print(f"pyinstaller --onefile --noconsole --name 'KBQV-v{version}' --icon 'web/favicon.ico' --add-data 'web;web' app/launcher.py")
print(f"")
print(f"빌드된 파일은 dist/ 폴더에 생성됩니다.")
print(f"onedir: dist/KBQV-v{version}/ 폴더")
print(f"onefile: dist/KBQV-v{version}.exe 파일") 