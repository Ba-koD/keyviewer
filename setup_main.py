from cx_Freeze import setup, Executable
import sys
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

# 빌드 옵션 설정
build_exe_options = {
    "packages": [
        "os", "sys", "threading", "time", "webbrowser", "socket", 
        "json", "pathlib", "typing", "tkinter", "tkinter.ttk", 
        "tkinter.messagebox", "winreg", "uvicorn", "ctypes", 
        "logging", "pystray", "PIL", "PIL.Image", "PIL.ImageDraw",
        "win32gui", "win32process", "win32con", "websockets",
        "websockets.legacy.client", "websockets.legacy.server"
    ],
    "excludes": [
        "tkinter.test", "unittest", "test", "distutils", 
        "setuptools", "pkg_resources", "email", "html", "http",
        "xml", "pydoc", "doctest", "argparse", "getopt"
    ],
    "include_files": [
        ("web/", "web/"),  # 웹 파일들 포함
        ("version.txt", "version.txt")  # 버전 파일 포함
    ],
    "optimize": 2,
    "build_exe": f"dist/{build_folder}"
}

# 기본 설정
base = None
if sys.platform == "win32":
    base = "Win32GUI"  # 콘솔 창 숨김

# 실행 파일 설정
executables = [
    Executable(
        "app/launcher.py",
        base=base,
        target_name=f"KBQV-v{version}.exe",
        icon="web/favicon.ico",
        shortcut_name="KeyQueueViewer",
        shortcut_dir="DesktopFolder"
    )
]

# 메타데이터
metadata = {
    "name": "KeyQueueViewer",
    "version": version,
    "description": "Key Input Monitoring Tool with Web Interface",
    "author": "KeyQueueViewer",
    "author_email": "support@keyqueueviewer.com",
    "url": "https://github.com/Ba-koD/keyviewer",
    "license": "MIT"
}

# 설정 실행
setup(
    name=metadata["name"],
    version=metadata["version"],
    description=metadata["description"],
    author=metadata["author"],
    author_email=metadata["author_email"],
    url=metadata["url"],
    license=metadata["license"],
    options={"build_exe": build_exe_options},
    executables=executables
) 