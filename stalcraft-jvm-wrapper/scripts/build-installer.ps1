# Build Windows Setup.exe (Inno Setup 6) from staged release\
# Usage: powershell -ExecutionPolicy Bypass -File scripts/build-installer.ps1
# Optional: -SkipPackage  (reuse existing release\)
param(
    [switch]$SkipPackage,
    [switch]$AllowDevService
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Stage = Join-Path $Root "release"
$Iss = Join-Path $Root "installer\stalcraft-jvm-wrapper.iss"
$OutDir = Join-Path $Root "installer\output"
$VendorService = Join-Path $Root "vendor\service.exe"

function Find-ISCC {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe"),
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe"
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    $cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

if (-not $SkipPackage) {
    Write-Host "==> Staging release via package.ps1"
    & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "package.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$GuiExe = Join-Path $Stage "stalcraft-jvm-wrapper.exe"
$SvcExe = Join-Path $Stage "service.exe"
if (-not (Test-Path $GuiExe) -or -not (Test-Path $SvcExe)) {
    Write-Error "Missing staged binaries in $Stage - run package.ps1 first"
    exit 1
}
if ((Get-Item $SvcExe).Length -eq 0) {
    Write-Error "service.exe is empty - refusing to build installer"
    exit 1
}
if (-not (Test-Path $VendorService) -and -not $AllowDevService) {
    Write-Warning "vendor\service.exe missing - installer will ship cargo/dev service.exe (use -AllowDevService to silence)"
}

$Iscc = Find-ISCC
if (-not $Iscc) {
    Write-Error @"
ISCC.exe not found. Install Inno Setup 6:
  winget install --id JRSoftware.InnoSetup -e
Then re-run this script.
"@
    exit 1
}

if (-not (Test-Path $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir | Out-Null
}

Write-Host "==> Compiling installer with $Iscc"
& $Iscc $Iss
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$setup = Get-ChildItem $OutDir -Filter "STALZONE-JVM-Wrapper-Setup-*.exe" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $setup) {
    Write-Error "Setup.exe not found in $OutDir"
    exit 1
}

Write-Host "OK: $($setup.FullName)"
Write-Host "Install to a stable folder, keep both exes side by side, then open the app and click INSTALL (or use the optional IFEO task in Setup)."

$Sums = Join-Path $Root "SHA256SUMS.txt"
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $setup.FullName).Hash.ToLowerInvariant()
$line = "$hash  $($setup.Name)"
if (Test-Path $Sums) {
    $existing = Get-Content $Sums -ErrorAction SilentlyContinue
    $filtered = @($existing | Where-Object { $_ -notmatch [regex]::Escape($setup.Name) })
    ($filtered + $line) | Set-Content -Path $Sums -Encoding ascii
} else {
    Set-Content -Path $Sums -Value $line -Encoding ascii
}
Write-Host "Checksum appended: $Sums"

Get-Item -LiteralPath $setup.FullName | Format-Table Name, Length, LastWriteTime
