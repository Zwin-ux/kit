# Install the kit binary on Windows x64 from a GitHub Release.
#   irm https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.ps1 | iex
#   & .\install.ps1 -Version 1.0.0-alpha.1
# Env: KIT_VERSION, KIT_INSTALL_DIR (default %LOCALAPPDATA%\kit\bin),
#      KIT_DOWNLOAD_BASE (for testing and mirrors; needs a version).
# The archive must match its SHA-256 line in SHA256SUMS. PATH is not edited.
param([string]$Version, [switch]$Prerelease, [string]$InstallDir, [string]$DownloadBase)

# Everything runs inside this function: `irm | iex` runs in the caller's
# scope, so script-level preferences would leak into the user's session.
function Install-Kit {
    param(
        [string]$Version = $env:KIT_VERSION,
        [switch]$Prerelease,
        [string]$InstallDir = $env:KIT_INSTALL_DIR,
        [string]$DownloadBase = $env:KIT_DOWNLOAD_BASE
    )
    $ErrorActionPreference = 'Stop'
    Set-StrictMode -Version Latest
    $oldProgress = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'
    try {
        $defaultBase = 'https://github.com/Zwin-ux/kit/releases/download'
        if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'kit\bin' }
        if (-not $DownloadBase) { $DownloadBase = $defaultBase }
        if ($DownloadBase -notmatch '^https://' -and $DownloadBase -notmatch '^http://(127\.0\.0\.1|localhost):') {
            throw "refusing non-HTTPS KIT_DOWNLOAD_BASE: $DownloadBase"
        }
        if (-not $Version -and $DownloadBase -ne $defaultBase) {
            throw 'set KIT_VERSION or -Version when KIT_DOWNLOAD_BASE is set'
        }
        $arch = $env:PROCESSOR_ARCHITECTURE
        if ($arch -eq 'ARM64') {
            Write-Host 'kit-install: ARM64 detected; the x64 build runs under emulation'
        } elseif ($arch -ne 'AMD64') {
            throw "unsupported platform Windows/$arch; try: cargo install --git https://github.com/Zwin-ux/kit kit-cli --locked"
        }
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

        if (-not $Version) {
            $api = 'https://api.github.com/repos/Zwin-ux/kit/releases/latest'
            if ($Prerelease) { $api = 'https://api.github.com/repos/Zwin-ux/kit/releases?per_page=30' }
            try {
                $response = Invoke-RestMethod -Uri $api
            } catch {
                throw 'could not find the latest release; use -Version'
            }
            # PS 5.1 returns a JSON array as one object: unroll it twice.
            # 0.x releases are the old Node app and have no archives: skip them.
            $tags = @($response | ForEach-Object { $_ } | ForEach-Object { $_.tag_name } |
                Where-Object { $_ -and $_ -notmatch '^v?0\.' })
            if ($tags.Count -eq 0) {
                if ($Prerelease) { throw 'no 1.x release found; use -Version' }
                throw 'no stable 1.x release yet; use -Prerelease or -Version'
            }
            $Version = $tags[0]
        }
        $Version = $Version -replace '^v', ''
        if ($Version -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') { throw "invalid version: $Version" }

        $tempDir = Join-Path ([IO.Path]::GetTempPath()) ('kit-install-' + [Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $tempDir | Out-Null
        try {
            $archive = "kit-$Version-x86_64-pc-windows-msvc.zip"
            $baseUrl = $DownloadBase.TrimEnd('/') + "/v$Version"
            $archivePath = Join-Path $tempDir $archive
            $sumsPath = Join-Path $tempDir 'SHA256SUMS'
            Write-Host "kit-install: downloading $baseUrl/$archive"
            try {
                Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/$archive" -OutFile $archivePath
            } catch {
                throw "could not download $archive; check that release v$Version exists: $($_.Exception.Message)"
            }
            Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/SHA256SUMS" -OutFile $sumsPath

            $expected = $null
            foreach ($line in [IO.File]::ReadAllLines($sumsPath)) {
                $parts = $line.Trim() -split '\s+'
                if ($parts.Count -ge 2 -and $parts[1].TrimStart('*') -ceq $archive) {
                    $expected = $parts[0]
                    break
                }
            }
            if (-not $expected) { throw "SHA256SUMS has no entry for $archive" }
            $actual = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash.ToLowerInvariant()
            if ($actual -ine $expected) {
                Write-Host "kit-install: expected SHA256: $expected"
                Write-Host "kit-install: actual SHA256: $actual"
                throw 'checksum mismatch; refusing to install'
            }

            Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force
            $binary = Join-Path $tempDir "kit-$Version-x86_64-pc-windows-msvc\kit.exe"
            if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw "archive has no kit.exe at $binary" }
            New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
            $staged = Join-Path $InstallDir 'kit.exe.new'
            $destination = Join-Path $InstallDir 'kit.exe'
            Copy-Item -LiteralPath $binary -Destination $staged -Force
            try {
                Move-Item -LiteralPath $staged -Destination $destination -Force
            } catch {
                Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue
                throw "could not replace $destination; close kit if it is running and rerun: $($_.Exception.Message)"
            }
            Write-Host "kit-install: installed kit $Version to $destination"
            try {
                $kitVersion = & $destination --version
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "kit-install: $kitVersion"
                } else {
                    Write-Host 'kit-install: warning: kit --version failed'
                }
            } catch {
                Write-Host 'kit-install: warning: kit --version failed'
            }
            $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
            $installed = $InstallDir.TrimEnd('\')
            $pathEntries = @($userPath, $env:Path) -split ';'
            if (-not @($pathEntries | Where-Object { $_.TrimEnd('\') -ieq $installed }).Count) {
                Write-Host 'kit-install: warning: add kit to your User PATH with:'
                Write-Host ('[Environment]::SetEnvironmentVariable(''Path'', "' + $InstallDir + ';" + [Environment]::GetEnvironmentVariable(''Path'',''User''), ''User'')')
                Write-Host 'kit-install: open a new terminal afterwards'
            }
        } finally {
            Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    } finally {
        $ProgressPreference = $oldProgress
    }
}

$forward = @{}
if ($Version) { $forward.Version = $Version }
if ($Prerelease) { $forward.Prerelease = $true }
if ($InstallDir) { $forward.InstallDir = $InstallDir }
if ($DownloadBase) { $forward.DownloadBase = $DownloadBase }
try {
    Install-Kit @forward
} catch {
    Write-Host "kit-install: error: $($_.Exception.Message)" -ForegroundColor Red
    # Run as a file: exit 1. Run through iex: $PSCommandPath is empty, and
    # `exit` would close the user's shell, so only return.
    if ($PSCommandPath) { exit 1 }
}
