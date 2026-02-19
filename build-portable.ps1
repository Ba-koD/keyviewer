Write-Host "Running tauri build..." -ForegroundColor Yellow
cargo tauri build
exit $LASTEXITCODE
