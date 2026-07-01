# Commit without IDE-injected Co-authored-by trailers (use Git for Windows directly).
param(
    [Parameter(Mandatory = $true)]
    [string]$Message
)

$ErrorActionPreference = "Stop"
$git = "C:\Program Files\Git\bin\git.exe"
if (-not (Test-Path $git)) { throw "Git not found at $git" }

$root = Split-Path -Parent $PSScriptRoot
Push-Location $root

if ($Message -match 'cursoragent@cursor\.com|Co-authored-by:\s*Cursor') {
    throw "Message must not include Cursor co-author trailers"
}

$status = & $git status --porcelain
if (-not $status) { throw "Nothing to commit" }

& $git add -A
$tree = & $git write-tree
$parent = & $git rev-parse HEAD
$msgFile = [System.IO.Path]::GetTempFileName()
try {
    [System.IO.File]::WriteAllText($msgFile, $Message, [System.Text.UTF8Encoding]::new($false))
    $newCommit = & $git commit-tree $tree -p $parent -F $msgFile
    & $git reset --hard $newCommit
    $body = & $git log -1 --format=%B
    if ($body -match 'cursoragent|Co-authored-by:\s*Cursor') {
        throw "Commit message was altered after commit-tree"
    }
    Write-Host "OK: $($newCommit.Substring(0,7)) $Message"
} finally {
    Remove-Item $msgFile -Force -ErrorAction SilentlyContinue
    Pop-Location
}
