# Kit — current state

**Date:** 2026-11-01
**Released:** `2.0.0` (2026-09-27) on npm (`@mzwin/kit`, `latest`), on crates.io (`kitctl` and its four libraries; the `crates.io` job was re-run on 2026-10-26 after the account email was verified) and as a GitHub Release with installers.
**Trunk:** `2.1.0`, Kit's own agent (A0.1 and the first A0.2 slice). `2.0.1` was never tagged; its fixes ship in 2.1.0. What `v2.1.0` carries is its section in [`CHANGELOG.md`](../../CHANGELOG.md). It adds two crates to crates.io, `kitctl-providers` and `kitctl-loop`.

## Current plan

The kickoff spec of 2026-10-26 sets the M0 milestone (foundation fixes, then `v2.0.1`) and the owner decisions K1–K15:
[`docs/superpowers/specs/2026-10-26-kit-kickoff.md`](../superpowers/specs/2026-10-26-kit-kickoff.md).
The umbrella design is [`docs/superpowers/specs/2026-09-24-kit-v2-design.md`](../superpowers/specs/2026-09-24-kit-v2-design.md).
The bar every change is held to: [`docs/QUALITY.md`](../QUALITY.md).
Next: [`docs/PRD.md`](../PRD.md) (approved 2026-10-31): Kit's own agent loop, the session TUI, harness v2 and Claude Code–compatible mods, milestones A0.0–A1.0 (= Kit 3.0).

## Workspace

- Five crates in `crates/`, all one version: `kitctl` (`kit-cli`, the `kit` binary), `kitctl-core`, `kitctl-agents`, `kitctl-gate`, `kitctl-tui`.
- `kit.toml` at the root is Kit's own gate: fmt, clippy `-D warnings`, workspace tests, plus `[gate.scope]`.
- The legacy TypeScript 0.1 tree (`packages/`, `apps/`, `catalog/`, the Node `CI` and keep-alive workflows, the root pnpm files) was removed in A0.0 (K10, PRD D8). It is kept at tag `legacy-0.1-final`.

## How to run and test

```sh
cargo run -p kitctl -- doctor
cargo run -p kitctl -- --demo
cargo run -p kitctl -- run --dry-run --task smoke --json

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Windows, a checkout in a deep folder can hit the 260-character path limit in `link.exe` (`LNK1104: cannot open file … build_script_build-….exe`). Build with a short target folder: `CARGO_TARGET_DIR=C:/kit-target cargo test --workspace` in Git Bash, or `$env:CARGO_TARGET_DIR = 'C:\kit-target'; cargo test --workspace` in PowerShell. Machines that set `NoDefaultCurrentDirectoryInExePath=1` (a Windows hardening) make `cmd` refuse a bare script name from the current folder: a `kit.toml` check that runs one needs `.\name.cmd`.

Releasing: [`RELEASING.md`](RELEASING.md).
