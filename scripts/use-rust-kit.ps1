# Prepend this repo's Rust Control Room so `kit` is 1.0, not npm 0.1.
# Usage (this session only):  . .\scripts\use-rust-kit.ps1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Release = Join-Path $Root "target\release"
$Debug = Join-Path $Root "target\debug"
$BinDir = if (Test-Path (Join-Path $Release "kit.exe")) { $Release } elseif (Test-Path (Join-Path $Debug "kit.exe")) { $Debug } else { $null }
if (-not $BinDir) {
    Write-Error "kit.exe not built. Run: cargo build -p kitctl --release"
}
$env:PATH = "$BinDir;$env:PATH"
$kit = Get-Command kit -ErrorAction SilentlyContinue
Write-Host "kit verb -> $($kit.Source)"
& (Join-Path $BinDir "kit.exe") --version
