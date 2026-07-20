# Repo-root shim so `.\kit.ps1` / `kit` works without a global install.
$ErrorActionPreference = "Stop"
$KitRoot = $PSScriptRoot
$KitBin = Join-Path $KitRoot "packages\cli\dist\bin.js"

if (-not (Test-Path $KitBin)) {
  Write-Error "kit: CLI not built. Run: pnpm build"
  exit 1
}

& node $KitBin @args
exit $LASTEXITCODE
