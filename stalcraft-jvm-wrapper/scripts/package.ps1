# Stalcraft JVM Wrapper — release zip (EXBO wrapper.zip parity)
$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Tauri = Join-Path $Root "src-tauri"
$Stage = Join-Path $Root "release"
$Zip = Join-Path $Root "wrapper.zip"

Push-Location $Tauri
npm run build --prefix $Root 2>$null | Out-Null
cargo build --release --bins
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
Pop-Location

if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
New-Item -ItemType Directory -Path $Stage | Out-Null
Copy-Item (Join-Path $Tauri "target\release\stalcraft-jvm-wrapper.exe") $Stage
Copy-Item (Join-Path $Tauri "target\release\service.exe") $Stage
Copy-Item (Join-Path $Root "examples") (Join-Path $Stage "examples") -Recurse

if (Test-Path $Zip) { Remove-Item $Zip -Force }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -Force

Write-Host "OK: $Zip"
Get-ChildItem $Stage | Format-Table Name, Length
