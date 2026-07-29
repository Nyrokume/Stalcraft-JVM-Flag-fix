# Stalcraft JVM Wrapper — portable release zip (any PC)
# Layout: unpack anywhere; keep both .exe + examples/ side by side.
param(
    [switch]$AllowDevService
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Tauri = Join-Path $Root "src-tauri"
$Stage = Join-Path $Root "release"
$Zip = Join-Path $Root "wrapper.zip"
$Sums = Join-Path $Root "SHA256SUMS.txt"
$VendorService = Join-Path $Root "vendor\service.exe"

if (-not (Test-Path $VendorService) -and -not $AllowDevService) {
    Write-Error @"
vendor\service.exe is required for a portable release (known-good EXBO runtime).
Restore vendor\service.exe or pass -AllowDevService for local/dev packaging only.
"@
    exit 1
}

Push-Location $Tauri
# Vite may write warnings to stderr; don't treat them as terminating errors.
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"
npm run build --prefix $Root | Out-Host
$buildExit = $LASTEXITCODE
$ErrorActionPreference = $prevEap
if ($buildExit -ne 0) { Pop-Location; exit $buildExit }
cargo build --release --bin stalcraft-jvm-wrapper
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
Pop-Location

if (-not (Test-Path $VendorService)) {
    Push-Location $Tauri
    cargo build --release --bin service
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
    Pop-Location
}

if (-not (Test-Path $Stage)) {
    New-Item -ItemType Directory -Path $Stage | Out-Null
}

# Clean staged leftovers so portable zip is self-contained
Get-ChildItem $Stage -Force | Where-Object {
    $_.Name -notin @('stalcraft-jvm-wrapper.exe', 'service.exe', 'examples', 'PORTABLE.txt', 'configs', 'logs')
} | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

Copy-Item (Join-Path $Tauri "target\release\stalcraft-jvm-wrapper.exe") $Stage -Force
if (Test-Path $VendorService) {
    Copy-Item $VendorService $Stage -Force
    Write-Host "service.exe: vendored (known-good EXBO runtime build)"
} else {
    Copy-Item (Join-Path $Tauri "target\release\service.exe") $Stage -Force
    Write-Host "service.exe: cargo build (dev only)"
}

$GuiExe = Join-Path $Stage "stalcraft-jvm-wrapper.exe"
$SvcExe = Join-Path $Stage "service.exe"
if (-not (Test-Path $GuiExe) -or -not (Test-Path $SvcExe)) {
    Write-Error "Stage incomplete: both stalcraft-jvm-wrapper.exe and service.exe are required in $Stage"
    exit 1
}
$svcLen = (Get-Item $SvcExe).Length
if ($svcLen -eq 0) {
    Write-Error "service.exe is empty (0 bytes) - refusing to package broken IFEO debugger"
    exit 1
}
if ((Test-Path $VendorService) -and $svcLen -ne (Get-Item $VendorService).Length) {
    Write-Warning "Staged service.exe size ($svcLen) differs from vendor ($((Get-Item $VendorService).Length))"
}

$examplesDest = Join-Path $Stage "examples"
if (Test-Path $examplesDest) {
    Remove-Item $examplesDest -Recurse -Force -ErrorAction SilentlyContinue
}
Copy-Item (Join-Path $Root "examples") $examplesDest -Recurse -Force

$portableTxt = @"
STALZONE JVM Wrapper — works on ANY Windows PC
===============================================
Rule: keep both .exe + examples\ in one folder, then INSTALL once (UAC).
IFEO stores an absolute path to THAT service.exe — flags inject on every launch.

Recommended folders (pick one):
  EXBO:  %AppData%\Roaming\EXBO\jvm_wrapper\
  Steam: …\steamapps\common\STALCRAFT\jvm_wrapper\
  EGS:   …\Epic Games\STALCRAFT\jvm_wrapper\
  VK:    …\VK Play\STALCRAFT\jvm_wrapper\

Must-haves:
  - Folder name for EXBO must be jvm_wrapper (NOT ""STALZONE JVM Wrapper"")
  - stalcraft-jvm-wrapper.exe + service.exe side by side
  - After INSTALL: VERIFY → pick JVM preset → fully quit launcher → start game
  - Find Game (PID) only detects the client; inject is IFEO-at-create every time
  - Moved the folder? Open GUI → REPAIR
  - Never run service.exe manually

Setup.exe default install path is %AppData%\EXBO\jvm_wrapper (change in wizard if Steam/EGS/VK).
"@
Set-Content -Path (Join-Path $Stage "PORTABLE.txt") -Value $portableTxt -Encoding UTF8

# Optional empty dirs for first run clarity
foreach ($dir in @('configs', 'logs')) {
    $p = Join-Path $Stage $dir
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p | Out-Null }
}

if (Test-Path $Zip) {
    Remove-Item $Zip -Force
}
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -Force

$lines = @()
foreach ($f in @($Zip)) {
    if (Test-Path $f) {
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $f).Hash.ToLowerInvariant()
        $lines += "$hash  $(Split-Path $f -Leaf)"
    }
}
$lines | Set-Content -Path $Sums -Encoding ascii

Write-Host "OK: $Zip"
Write-Host "OK: $Sums"
Write-Host "Unpack zip anywhere - keep both .exe side by side, then INSTALL."
Get-ChildItem $Stage | Format-Table Name, Length
