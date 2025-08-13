$ErrorActionPreference = "Stop"

function Test-Exe([string]$name) {
	try { Get-Command $name -ErrorAction Stop | Out-Null; return $true } catch { return $false }
}

$venvPython = ".\.venv\Scripts\python.exe"

Write-Host "[KeyViewer] Ensuring Python & venv..." -ForegroundColor Cyan
if (-Not (Test-Path $venvPython)) {
	# Ensure Python exists
	if (-Not (Test-Exe "python") -and -Not (Test-Exe "py")) {
		if (Test-Exe "winget") {
			Write-Host "[KeyViewer] Installing Python 3.11 via winget (silent)" -ForegroundColor Cyan
			winget install -e --id Python.Python.3.11 --accept-package-agreements --accept-source-agreements --silent | Out-Null
			Start-Sleep -Seconds 3
		} else {
			throw "Python is not installed and 'winget' is not available. Please install Python 3.11+ and re-run."
		}
	}
	# Create venv
	if (-Not (Test-Path $venvPython)) {
		try {
			Write-Host "[KeyViewer] Creating venv via 'py -3'" -ForegroundColor Cyan
			py -3 -m venv .venv
		} catch {
			Write-Host "[KeyViewer] 'py' not available. Trying 'python'" -ForegroundColor Yellow
			python -m venv .venv
		}
	}
}

if (-Not (Test-Path $venvPython)) {
	throw "Could not create venv. Ensure Python 3.11+ is installed and available."
}

Write-Host "[KeyViewer] Installing dependencies" -ForegroundColor Cyan
& $venvPython -m pip install --upgrade pip
& $venvPython -m pip install -r requirements.txt

Write-Host "[KeyViewer] Starting server (Admin recommended)" -ForegroundColor Green
& $venvPython -m server.main