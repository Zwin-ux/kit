# Repo-root shim: Rust Control Room (1.0), not the Node 0.1 skill launcher.
$ErrorActionPreference = "Stop"
$KitRoot = $PSScriptRoot
$KitBin = Join-Path $KitRoot "target\release\kit.exe"
if (-not (Test-Path $KitBin)) {
    $KitBin = Join-Path $KitRoot "target\debug\kit.exe"
}
if (-not (Test-Path $KitBin)) {
    Write-Error "kit: Rust binary not built.`n  cargo build -p kitctl --release`nThen run: kit --demo"
    exit 1
}
& $KitBin @args
exit $LASTEXITCODE
