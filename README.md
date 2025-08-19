# KeyQueueViewer

Key Input Monitoring Tool with Web Interface

## Quick Start

### Development Mode
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\run.ps1
```

### Build All Versions
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
.\build_all.ps1
```

### Individual Builds
```powershell
# Main program (onedir)
.\onedir.ps1

# Installer (onefile)
.\installer.ps1

# Portable version (onefile)
.\portable.ps1
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
- Multiple build options (onedir, onefile, portable)

## Build Output

After running `build_all.ps1`, you'll get:
```
dist/
├── KBQV-v1.0.4/                    # Main program (onedir)
├── KBQV-Installer-1.0.4.exe       # Installer (onefile)
├── KBQV-Portable-1.0.4.exe        # Portable version (onefile)
└── KBQV-v1.0.4.zip               # Main program compressed file
```

## Requirements

- Windows 10/11
- Python 3.11+
- PowerShell
- PyInstaller