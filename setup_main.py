import sys
from cx_Freeze import setup, Executable
import os

# 버전 정보 읽기
def get_version():
    try:
        with open("version.txt", "r", encoding="utf-8") as f:
            return f.read().strip()
    except:
        return "1.0.0"

version = get_version()
build_folder = f"KBQV-v{version}"

# Windows에서 GUI 애플리케이션으로 실행
base = None
if sys.platform == "win32":
    base = "Win32GUI"

# 필요한 모듈들 포함
build_exe_options = {
    "packages": [
        "fastapi", "uvicorn", "websockets", "keyboard", "win32api", "win32con", 
        "win32gui", "psutil", "pystray", "PIL", "asyncio", "uvicorn.logging",
        "uvicorn.config", "uvicorn.protocols", "uvicorn.lifespan", "uvicorn.loops",
        "uvicorn.middleware", "uvicorn.server", "uvicorn.workers", "http", "http.client",
        "http.server", "http.cookies", "http.cookiejar", "urllib", "urllib.request",
        "urllib.parse", "urllib.error", "urllib.robotparser", "json", "zipfile",
        "tempfile", "shutil", "os", "sys", "threading", "time", "datetime"
    ],
    "excludes": [],
    "include_files": [
        ("web/", "web/"),
        ("version.txt", "version.txt")
    ],
    "build_exe": f"dist/{build_folder}",
    "optimize": 2,
    "include_msvcr": True,
    "zip_include_packages": "*",
    "zip_exclude_packages": []
}

executables = [
    Executable(
        "app/launcher.py",
        base=base,
        target_name=f"KBQV-v{version}.exe",
        icon="web/favicon.ico"
    )
]

metadata = {
    "name": "KeyQueueViewer",
    "version": version,
    "description": "Keyboard Queue Viewer - Real-time keyboard input monitoring tool",
    "author": "KeyQueueViewer",
    "author_email": "support@keyqueueviewer.com",
    "url": "https://github.com/your-username/keyviewer",
    "options": {"build_exe": build_exe_options}
}

setup(
    name=metadata["name"],
    version=metadata["version"],
    description=metadata["description"],
    author=metadata["author"],
    author_email=metadata["author_email"],
    url=metadata["url"],
    options=metadata["options"],
    executables=executables
) 