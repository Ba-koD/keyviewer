# Key Queue Viewer Guide (English)

This document is a complete guide to install, configure, and build Key Queue Viewer (variable-length key queue overlay) on Windows + OBS.

## Features
- Show keys only when the selected window is focused
- Keys stack in the order pressed; pop when released (queue)
- Target matching: title (contains), process (exact), HWND, class, or all windows
- Overlay customization: colors (background/text/chip), font/corner/padding, fade in/out, columns/rows, alignment (left/center/right), direction (LTR/RTL)
- Built-in preview + color pickers, foreground/open windows panel with sortable headers
- EXE launcher: start/stop/status, admin hint, toggle console, run at startup

## Requirements
- Windows 10/11
- Admin privileges recommended (global key hook)

## 1) Run (dev)
In PowerShell (admin recommended):
```powershell
cd <project path>
Set-ExecutionPolicy Bypass -Scope Process -Force
./run.ps1
```
After the server starts:
- Control: `http://127.0.0.1:<port>/control`
- Overlay: `http://127.0.0.1:<port>/overlay` (default 8000)

## 2) Target window
- Choose mode: title/process/HWND/class/all
- Value select box auto-fills candidates based on current open windows
- Open windows table: title/process/HWND/class, sortable by header; click title to focus that window

## 3) Overlay settings
- Click “Overlay settings” → adjust colors/layout/alignment/direction/fades
- Preview follows cols×rows; rows=0 wraps unlimited
- Colors: hex inputs + color-picker buttons (transparent background option)
- Saving applies instantly to overlay (WebSocket broadcast)

## 4) OBS setup
1. Add Browser Source
2. URL: `http://127.0.0.1:<port>/overlay`
3. Remove custom CSS:
```css
body { background-color: rgba(0,0,0,0); margin: 0; overflow: hidden; }
```

Delete Above Settings

## 5) Build EXE
- Build script:
```powershell
./build_exe.ps1
```
- Includes elevation, execution policy bypass, Python/venv ensure, deps install, clean build, icon registration
- Output: `dist/KeyQueueViewer/KeyQueueViewer.exe`


## Troubleshooting
- Alignment/direction mismatch: ensure `margin: 0` in OBS custom CSS, then refresh source
- Port in use: launcher auto-opens existing Control
- Stuck keys: run as admin; check security/anti-cheat

## Note
- “All windows” mode can reveal overlay during password entry; we add a 2-second confirm dialog.