# Free port 1420, stop stale app binary, start Tauri dev (GUI + Vite HMR).
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Port = 1420

Get-Process -Name 'stalcraft-jvm-wrapper' -ErrorAction SilentlyContinue |
    Stop-Process -Force -ErrorAction SilentlyContinue

Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }

Start-Sleep -Milliseconds 500
Set-Location $Root
npm run tauri -- dev
