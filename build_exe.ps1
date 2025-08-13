$ErrorActionPreference = "Stop"

function Test-Exe([string]$name) {
	try { Get-Command $name -ErrorAction Stop | Out-Null; return $true } catch { return $false }
}

# Elevate to Administrator if not already (useful for winget install)
$curr = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($curr)
$IsAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
	Write-Host "[Build] Elevating to Administrator..." -ForegroundColor Yellow
	$argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
	Start-Process -FilePath "powershell.exe" -ArgumentList $argList -Verb RunAs
	exit
}

# Ensure this session allows script execution
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force

$venvPython = ".\.venv\Scripts\python.exe"

Write-Host "[Build] Ensuring Python & venv..." -ForegroundColor Cyan
if (-Not (Test-Path $venvPython)) {
	# Ensure Python exists
	if (-Not (Test-Exe "python") -and -Not (Test-Exe "py")) {
		if (Test-Exe "winget") {
			Write-Host "[Build] Installing Python 3.11 via winget (silent)" -ForegroundColor Cyan
			winget install -e --id Python.Python.3.11 --accept-package-agreements --accept-source-agreements --silent | Out-Null
			Start-Sleep -Seconds 3
		} else {
			throw "Python is not installed and 'winget' is not available. Please install Python 3.11+ and re-run."
		}
	}
	# Create venv
	if (-Not (Test-Path $venvPython)) {
		try {
			Write-Host "[Build] Creating venv via 'py -3'" -ForegroundColor Cyan
			py -3 -m venv .venv
		} catch {
			Write-Host "[Build] 'py' not available. Trying 'python'" -ForegroundColor Yellow
			python -m venv .venv
		}
	}
}

if (-Not (Test-Path $venvPython)) {
	throw "Could not create venv. Ensure Python 3.11+ is installed and available."
}

Write-Host "[Build] Installing dependencies" -ForegroundColor Cyan
& $venvPython -m pip install --upgrade pip
& $venvPython -m pip install -r requirements.txt

# PyInstaller options
$entry = "app\launcher.py"
$datas = "web;web"
$icon = "web\favicon.ico"

# Clean previous outputs (optional but recommended)
if (Test-Path .\build) { Remove-Item -Recurse -Force .\build }
if (Test-Path .\dist) { Remove-Item -Recurse -Force .\dist }
if (Test-Path .\KeyQueueViewer.spec) { Remove-Item -Force .\KeyQueueViewer.spec }

# Build command as argument list
$argList = @(
	"-m", "PyInstaller",
	"--clean",
	"--noconsole",
	"--name", "KeyQueueViewer",
	"--icon", $icon,
	"--add-data", $datas,
	$entry
)

Write-Host "[Build] Running: $venvPython $($argList -join ' ')" -ForegroundColor Cyan
& $venvPython @argList

Write-Host "[Build] Done. Output in .\dist\KeyQueueViewer" -ForegroundColor Green