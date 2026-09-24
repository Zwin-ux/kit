# Releasing Kit to npm

`@mzwin/kit` 1.x is a Node launcher (`npm/kit/bin/kit.js`) plus one binary package per platform (`npm/platforms.json`). npm installs only the platform package that matches the host's `os`, `cpu` and `libc`.

| Package | Contents |
|---------|----------|
| `@mzwin/kit` | launcher, `platforms.json`, README |
| `@mzwin/kit-win32-x64` | `bin/kit.exe` (static CRT) |
| `@mzwin/kit-darwin-arm64` / `-darwin-x64` | `bin/kit` |
| `@mzwin/kit-linux-x64-gnu` / `-linux-arm64-gnu` | `bin/kit` (glibc 2.35+) |

## Check locally

```bash
cargo build --release -p kit-cli
node scripts/npm-smoke.mjs
```

The smoke test packs the launcher and this host's platform package, installs both tarballs into a temp project, and runs `kit` through the npm shim. CI runs it on Linux, macOS and Windows (`rust.yml`, job `npm`).

## Publish

1. Set `version` in `[workspace.package]` of `Cargo.toml`. It is the only version source.
2. Push the tag `v<version>`. `release-npm.yml` builds all five targets, checks that the tag matches the version, and publishes the platform packages first, then the launcher.
3. To try the pipeline without publishing: run **Release npm** from the Actions tab with `publish` unchecked.

Dist-tags: `1.0.0-alpha.N` goes to `alpha`, `-beta.N` to `beta`, `-rc.N` to `rc`. Only a plain version (`1.0.0`) moves `latest`. Until then, `npm i -g @mzwin/kit` installs 0.1.x.

A rerun skips packages that are already published at that version.

## Requirements (owner)

- Repo secret `NPM_TOKEN`: an npm automation token with publish rights on the `@mzwin` scope.
- Provenance needs a public repo and the `id-token: write` permission (set in the workflow).
