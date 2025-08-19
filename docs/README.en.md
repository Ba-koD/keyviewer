# KeyQueueViewer Guide (English)

This document is a complete guide to install, configure, and build KeyQueueViewer (key input monitoring tool) on Windows.

## Features
- Real-time key input monitoring
- Web-based interface for viewing key logs
- System tray integration
- Windows startup management
- Auto-installer for easy deployment

## Build System

This project uses a **PyInstaller-based modular build system** to provide various deployment options:

- **Main Program**: Built with `PyInstaller onedir` (folder-based executable)
- **Installer**: Built with `PyInstaller onefile` (single executable distribution)
- **Portable Version**: Built with `PyInstaller onefile` (executable without installation)

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

### Run Full Build (All Versions)
```powershell
./build_all.ps1
```

### Run Individual Builds
```powershell
# Main program (onedir)
./onedir.ps1

# Installer (onefile)
./installer.ps1

# Portable version (onefile)
./portable.ps1
```

### Build Process
1. **Main Program Build**: Uses PyInstaller onedir to create folder-based executable
2. **Installer Build**: Uses PyInstaller onefile for convenience
3. **Portable Version Build**: Uses PyInstaller onefile to create executable without installation
4. **Package Creation**: Automatically creates ZIP file of main program

### Build Output
```
dist/
├── KBQV-v1.0.4/                    # Main program (onedir)
├── KBQV-Installer-1.0.4.exe       # Installer (onefile)
├── KBQV-Portable-1.0.4.exe        # Portable version (onefile)
└── KBQV-v1.0.4.zip               # Main program compressed file
```

## 3) Installation Process

### Using Installer (Recommended)
1. **Run Installer**: Execute `KBQV-Installer-1.0.4.exe`
2. **Choose Path**: Select installation directory (default: `C:\Program Files\KeyQueueViewer`)
3. **Shortcut Options**: Choose to create desktop and Start Menu shortcuts
4. **Complete**: Program is automatically registered and shortcuts are created

### Using Portable Version
1. **Direct Execution**: Copy `KBQV-Portable-1.0.4.exe` to desired location and run
2. **No Installation Required**: No files copied to registry or system folders
3. **Portable**: Easy to move to USB or other computers

### Using Compressed File
1. **Extract**: Extract `KBQV-v1.0.4.zip` to desired location
2. **Run**: Execute `KBQV-v1.0.4.exe` in the extracted folder

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
├── build_all.ps1             # Full build script
├── onedir.ps1                # Main program build script
├── installer.ps1             # Installer build script
├── portable.ps1              # Portable version build script
├── requirements.txt           # Dependencies
└── version.txt               # Version information
```

## Build Options

### onedir (Folder-based)
- **Advantages**: Fast startup, easy debugging, file modification possible
- **Disadvantages**: Multiple files, deployment requires attention
- **Use case**: Development, testing, customization

### onefile (Single file)
- **Advantages**: Single executable, easy deployment, portable
- **Disadvantages**: Slower startup, potential Windows Defender false positives
- **Use case**: Final distribution, user installation

## Security Features

- **PyInstaller Build**: Stable and proven build system
- **Modular Approach**: Provides optimized build options for different purposes
- **Self-contained**: Each build includes all necessary files
- **Registry Integration**: Proper Windows program registration when using installer

## Troubleshooting

### Build Issues
- Ensure Python 3.11+ is installed
- Run PowerShell as Administrator if needed
- Check virtual environment setup
- Verify PyInstaller installation: `pip install PyInstaller`

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
- **Windows Defender**: onefile builds may trigger false positives; set exclusions if needed

## Support

For issues and questions, please check:
- GitHub Issues
- Project documentation
- Build logs and error messages
- PowerShell execution policy settings