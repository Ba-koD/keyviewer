# KeyQueueViewer Guide (English)

This document is a complete guide to install, configure, and build KeyQueueViewer (key input monitoring tool) on Windows.

## Features
- Real-time key input monitoring
- Web-based interface for viewing key logs
- System tray integration
- Windows startup management
- Auto-installer for easy deployment

## Build System

This project uses a **hybrid build approach** to optimize both security and convenience:

- **Main Program**: Built with `cx_Freeze` (reduces Windows Defender false positives)
- **Installer**: Built with `PyInstaller onefile` (single executable distribution)

## Requirements
- Windows 10/11
- Python 3.11+
- PowerShell (for builds)
- Virtual environment (recommended)

## 1) Development Mode
In PowerShell:
```powershell
cd <project path>
Set-ExecutionPolicy Bypass -Scope Process -Force
./run.ps1
```
After the server starts:
- Control: `http://127.0.0.1:<port>/control`
- Overlay: `http://127.0.0.1:<port>/overlay` (default 8000)

## 2) Build and Install (Recommended)

### Run Hybrid Build
```powershell
./build_hybrid.ps1
```

### Build Process
1. **Main Program Build**: Uses cx_Freeze for security
2. **Installer Build**: Uses PyInstaller onefile for convenience
3. **Package Creation**: Automatically creates a self-contained installer package

### Build Output
```
dist/
├── KeyQueueViewer_Main/          # Main program (cx_Freeze)
├── KeyQueueViewer_Installer.exe  # Installer (PyInstaller onefile)
└── KeyQueueViewer_Installer_Complete/  # Distribution package
    ├── KeyQueueViewer_Installer.exe
    ├── main_program/
    └── README.txt
```

## 3) Installation Process

1. **Run Installer**: Execute `KeyQueueViewer_Installer.exe`
2. **Choose Path**: Select installation directory (default: `C:\Program Files\KeyQueueViewer`)
3. **Shortcut Options**: Choose to create desktop and Start Menu shortcuts
4. **Complete**: Program is automatically registered and shortcuts are created

## 4) OBS Setup
1. Add Browser Source
2. URL: `http://127.0.0.1:<port>/overlay`
3. Set custom CSS to:
```css
body { background-color: rgba(0,0,0,0); margin: 0; overflow: hidden; }
```

## 5) Project Structure
```
keyviewer/
├── app/
│   └── launcher.py           # Main application
├── installer.py              # Auto-installer
├── setup_main.py             # cx_Freeze configuration
├── build_hybrid.ps1          # Build script
├── requirements.txt           # Dependencies
└── version.txt               # Version information
```

## Security Features

- **cx_Freeze Build**: Reduces Windows Defender false positives
- **Hybrid Approach**: Combines security (cx_Freeze) with convenience (PyInstaller)
- **Self-contained**: Installer includes all necessary files
- **Registry Integration**: Proper Windows program registration

## Troubleshooting

### Build Issues
- Ensure Python 3.11+ is installed
- Run PowerShell as Administrator if needed
- Check virtual environment setup

### Installation Issues
- Run installer as Administrator
- Ensure target directory is writable
- Check Windows Defender exclusions if needed

### Runtime Issues
- Alignment/direction mismatch: ensure `margin: 0` in OBS custom CSS, then refresh source
- Port in use: launcher auto-opens existing Control
- Stuck keys: run as admin; check security/anti-cheat

## Notes

- **"All windows" mode**: Can reveal overlay during password entry; includes 2-second confirm dialog
- **Admin privileges**: Recommended for global key hook
- **Windows Defender**: cx_Freeze build reduces false positives

## Support

For issues and questions, please check:
- GitHub Issues
- Project documentation
- Build logs and error messages