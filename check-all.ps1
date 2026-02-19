$ErrorActionPreference = "Stop"

Write-Host "[1/3] cargo kfmt" -ForegroundColor Cyan
cargo kfmt

Write-Host "[2/3] cargo kclippy" -ForegroundColor Cyan
cargo kclippy

Write-Host "[3/3] cargo ktest" -ForegroundColor Cyan
cargo ktest

Write-Host "All checks passed." -ForegroundColor Green
