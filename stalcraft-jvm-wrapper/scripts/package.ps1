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

if (-not (Test-Path $Stage)) { New-Item -ItemType Directory -Path $Stage | Out-Null }
Copy-Item (Join-Path $Tauri "target\release\stalcraft-jvm-wrapper.exe") $Stage -Force
Copy-Item (Join-Path $Tauri "target\release\service.exe") $Stage -Force
$examplesDest = Join-Path $Stage "examples"
if (Test-Path $examplesDest) { Remove-Item $examplesDest -Recurse -Force -ErrorAction SilentlyContinue }
Copy-Item (Join-Path $Root "examples") $examplesDest -Recurse -Force

if (Test-Path $Zip) { Remove-Item $Zip -Force }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -Force

Write-Host "OK: $Zip"
Get-ChildItem $Stage | Format-Table Name, Length
