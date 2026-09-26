# Releasing Kit

Kit 1.x ships one Rust binary (`kit`, `kit.exe`) through four channels. One tag push feeds the first three.

| Channel | Command for users | Source |
|---------|-------------------|--------|
| npm (primary) | `npm install -g @mzwin/kit@alpha` | `release-npm.yml`, job `publish` |
| Shell installer | `curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh \| sh -s -- --prerelease` | GitHub Release, job `github-release` |
| PowerShell installer | `& ([scriptblock]::Create((irm https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.ps1))) -Prerelease` | GitHub Release, job `github-release` |

Until 1.0.0 there is no stable 1.x release, so the installer lines need `--prerelease` / `-Prerelease` (or a version). After 1.0.0 the plain forms work: `curl -fsSL …/install.sh | sh` and `irm …/install.ps1 | iex`.

**No 1.x GitHub Release exists yet.** `v1.0.0-alpha.1` went to npm before `github-release` existed. The installers work from the first tag pushed after this job lands. Do not backfill alpha.1 with a manual run: a manual run uses the workflow and the code of the branch it runs on, so its archives would not be the published alpha.1 binaries.
| Cargo | `cargo install --git https://github.com/Zwin-ux/kit kitctl --locked` | the git repo (not crates.io, see below) |

Targets: Windows x64 (static CRT), macOS arm64/x64, Linux x64/arm64 with glibc 2.17 or newer. There is no musl build. On musl (Alpine) use the cargo line.

## Version and tag

1. Set `version` in `[workspace.package]` of `Cargo.toml`. It is the only version source.
2. Add the changes to `CHANGELOG.md`. The release notes link to it.
3. Push the tag `v<version>`.

`release-npm.yml` then builds all five targets, runs the release binaries on each OS (`verify`), and after that runs two jobs in parallel:

- `publish`: npm packages, then `npm-check` (below).
- `github-release`: archives, `SHA256SUMS` and the GitHub Release (below).

Both jobs stop if the tag does not match the Cargo version.

To try the pipeline without publishing: run **Release npm** from the Actions tab with `publish` unchecked. The run packs the npm packages (`npm publish --dry-run`) and the release archives (artifact `release-archives`), but publishes nothing and creates no release.

## npm

`@mzwin/kit` 1.x is a Node launcher (`npm/kit/bin/kit.js`) plus one binary package per platform (`npm/platforms.json`). npm installs only the platform package that matches the host's `os`, `cpu` and `libc`.

| Package | Contents |
|---------|----------|
| `@mzwin/kit` | launcher, `platforms.json`, README |
| `@mzwin/kit-win32-x64` | `bin/kit.exe` (static CRT) |
| `@mzwin/kit-darwin-arm64` / `-darwin-x64` | `bin/kit` |
| `@mzwin/kit-linux-x64-gnu` / `-linux-arm64-gnu` | `bin/kit` (glibc 2.17+) |

Check locally:

```bash
cargo build --release -p kitctl
node scripts/npm-smoke.mjs
```

The smoke test packs the launcher and this host's platform package, installs both tarballs into a temp project, and runs `kit` through the npm shim. CI runs it on Linux, macOS and Windows (`rust.yml`, job `npm`).

The job publishes the platform packages first, then the launcher.

**Registry delay.** A new package name can return 404 from `registry.npmjs.org` for about 5 minutes after `npm publish` succeeds. During the 1.0.0-alpha.1 publish, users in that window got the launcher without a platform binary, and the workflow was green. The job now waits until every platform package document returns 200 and `npm view <pkg>@<version> version --prefer-online` prints the version. Then it publishes the launcher and waits for it the same way. Each wait stops after 15 minutes and fails the run.

Job `npm-check` runs after `publish` (not in a dry run). On Ubuntu, macOS 14 and Windows it installs `@mzwin/kit@<version>` from the registry with an empty npm cache, then checks that `kit --version` prints `kit <version>` and that `kit doctor --json` has `ok: true` and `data.install == "npm"`. On Windows it also runs `kit.cmd`. If this job fails, the packages are live but broken: deprecate the version (`npm deprecate`) and publish a fix.

Dist-tags: `1.0.0-alpha.N` goes to `alpha`, `-beta.N` to `beta`, `-rc.N` to `rc`. Only a plain version (`1.0.0`) moves `latest`. Until then, `npm i -g @mzwin/kit` installs 0.1.x.

A rerun skips a package only when `npm view <pkg>@<version> version` prints exactly that version.

## GitHub Release and installers

Job `github-release` makes one archive per target:

| File | Contents |
|------|----------|
| `kit-<version>-<target>.tar.gz` | `kit-<version>-<target>/kit`, `LICENSE`, `README.md` |
| `kit-<version>-x86_64-pc-windows-msvc.zip` | `kit-<version>-x86_64-pc-windows-msvc/kit.exe`, `LICENSE`, `README.md` |
| `SHA256SUMS` | `sha256sum` output for all five archives |

It creates the release `v<version>` (marked prerelease when the version has a `-`). The tag must already exist on GitHub (`--verify-tag`), so a manual run with `publish` checked works only from a pushed tag. If the release already exists, a rerun replaces its assets (`gh release upload --clobber`). The job needs `contents: write`; all other jobs keep `contents: read`.

`scripts/install.sh` (Linux, macOS) and `scripts/install.ps1` (Windows x64):

1. Find the target. `install.sh` stops on musl and on other systems, and prints the cargo line.
2. Find the version: `--version <v>` / `-Version <v>` or `KIT_VERSION`. Without one, the newest stable release from the GitHub API; `--prerelease` / `-Prerelease` also accepts prereleases. Releases `v0.x` (the old Node app, no archives) are skipped.
3. Download the archive and `SHA256SUMS` over HTTPS only.
4. Compare the SHA-256 of the archive with its line in `SHA256SUMS`. On a mismatch or a missing line, stop and install nothing. `install.sh` needs `sha256sum` or `shasum`, and stops if neither exists.
5. Copy `kit` to `KIT_INSTALL_DIR` (default `~/.local/bin`; on Windows `%LOCALAPPDATA%\kit\bin`). No sudo. The file is replaced in one step.
6. If that directory is not on `PATH`, print the line to add. They do not edit shell profiles or the Windows PATH.

| Env | Effect |
|-----|--------|
| `KIT_VERSION` | Version to install (same as `--version`) |
| `KIT_INSTALL_DIR` | Install directory |
| `KIT_DOWNLOAD_BASE` | Download root for tests and mirrors. Files come from `<base>/v<version>/<file>`. Needs a version. Must be `https://`, or `http://127.0.0.1:<port>` / `http://localhost:<port>` for local tests |

Test locally with a fake release.

Linux or macOS (`t` is your target, for example `x86_64-unknown-linux-gnu` or `aarch64-apple-darwin`):

```bash
cargo build --release -p kitctl
v=1.0.0-alpha.1; t=x86_64-unknown-linux-gnu
mkdir -p /tmp/rel/v$v /tmp/stage/kit-$v-$t
cp target/release/kit LICENSE README.md /tmp/stage/kit-$v-$t/
tar -C /tmp/stage -czf /tmp/rel/v$v/kit-$v-$t.tar.gz kit-$v-$t
(cd /tmp/rel/v$v && sha256sum kit-* > SHA256SUMS)   # macOS: shasum -a 256
python3 -m http.server 8765 --bind 127.0.0.1 --directory /tmp/rel &
KIT_DOWNLOAD_BASE=http://127.0.0.1:8765 KIT_INSTALL_DIR=/tmp/kitbin sh scripts/install.sh --version $v
```

Windows (PowerShell):

```powershell
cargo build --release -p kitctl
$v = '1.0.0-alpha.1'; $n = "kit-$v-x86_64-pc-windows-msvc"
$stage = "$env:TEMP\kitrel\stage\$n"; $rel = "$env:TEMP\kitrel\srv\v$v"
New-Item -ItemType Directory -Force $stage, $rel | Out-Null
Copy-Item target\release\kit.exe, LICENSE, README.md $stage
Compress-Archive -Path $stage -DestinationPath "$rel\$n.zip" -Force
"$((Get-FileHash "$rel\$n.zip").Hash.ToLower())  $n.zip" | Set-Content -Encoding ascii "$rel\SHA256SUMS"
Start-Process python -ArgumentList '-m','http.server','8765','--bind','127.0.0.1','--directory',"$env:TEMP\kitrel\srv"; Start-Sleep 2
.\scripts\install.ps1 -Version $v -DownloadBase http://127.0.0.1:8765 -InstallDir $env:TEMP\kitbin
```

To check the refusal, append one byte to the archive and run the installer again. It must print `refusing to install` and exit non-zero.

CI: `installers.yml` runs shellcheck, then both installers against a fake release on `127.0.0.1` (Ubuntu, macOS, Windows PowerShell 5.1 and 7), including the corrupted-archive refusal.

## crates.io

Nothing is published to crates.io. Status on 2026-09-23:

| Name | Status |
|------|--------|
| `kit-cli` | **Taken.** Another project (`dayemsiddiqui/kit`, "CLI for scaffolding Kit web applications", 0.1.67). It also installs a binary called `kit`. |
| `kit` | Taken (`weshardee/kit`, 0.0.2, 2020). |
| `kit-core`, `kit-agents`, `kit-gate`, `kit-tui` | Free. |
| `kitctl`, `kitctl-core`, `kitctl-agents`, `kitctl-gate`, `kitctl-tui`, `kit-control-room` | Free. |

Result: `cargo install kit-cli`, as PRD §6 says, installs the other project's `kit` today. Do not tell users to run it.

`cargo install <name>` from crates.io also needs every path dependency on crates.io. `kit-cli` depends on `kit-core`, `kit-agents`, `kit-gate` and `kit-tui`, so all five crates must be published, and in dependency order.

Decision (2026-09-26): the packages are renamed to the free `kitctl` prefix. The binary crate is `kitctl`, the libraries are `kitctl-core`, `kitctl-agents`, `kitctl-gate` and `kitctl-tui`, and each library keeps its Rust crate name (`kit_core`, …) through `[lib] name`, so no code changed. The binary is still `kit`.

- Today: `cargo install --git https://github.com/Zwin-ux/kit kitctl --locked`.
- After a crates.io publish (not done; needs the owner): `cargo install kitctl`. Publish in dependency order: `kitctl-core`, then `kitctl-agents` and `kitctl-gate`, then `kitctl-tui`, then `kitctl`. The library crates are implementation detail with no semver promise of their own; their docs say so.

## Requirements (owner)

- Repo secret `NPM_TOKEN`: an npm automation token with publish rights on the `@mzwin` scope.
- Provenance needs a public repo and the `id-token: write` permission (set in the workflow).
- The GitHub Release uses the built-in `GITHUB_TOKEN`. No secret is needed.
