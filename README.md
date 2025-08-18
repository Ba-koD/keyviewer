# KeyQueueViewer

Key Input Monitoring Tool with Web Interface

## Quick Start

### Development Mode
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\run.ps1
```

### Build and Install
```powershell
.\build_hybrid.ps1
```

## Documentation

- **한국어 가이드**: [docs/README.ko.md](docs/README.ko.md)
- **English Guide**: [docs/README.en.md](docs/README.en.md)

## Features

- Real-time key input monitoring
- Web-based interface for viewing key logs
- System tray integration
- Windows startup management
- Auto-installer for easy deployment

## Build System

This project uses a **hybrid build approach**:
- **Main Program**: `cx_Freeze` (reduces Windows Defender false positives)
- **Installer**: `PyInstaller onefile` (single executable distribution)