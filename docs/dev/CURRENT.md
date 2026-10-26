# Kit — current state

**Date:** 2026-10-26
**Released:** `2.0.0` (2026-09-27) on npm (`@mzwin/kit`, `latest`) and as a GitHub Release with installers. crates.io never got it: the release's `crates.io` job failed on the account's unverified email, so `cargo install kitctl` does not work yet.
**Trunk:** the 2.0.x line. What changed since 2.0.0 is under `## Unreleased` in [`CHANGELOG.md`](../../CHANGELOG.md).

## Current plan

The kickoff spec of 2026-10-26 sets the M0 milestone (foundation fixes, then `v2.0.1`) and the owner decisions K1–K15:
[`docs/superpowers/specs/2026-10-26-kit-kickoff.md`](https://github.com/Zwin-ux/kit/blob/docs/kit-kickoff-20261026/docs/superpowers/specs/2026-10-26-kit-kickoff.md) (branch `docs/kit-kickoff-20261026`).
The umbrella design is [`docs/superpowers/specs/2026-09-24-kit-v2-design.md`](../superpowers/specs/2026-09-24-kit-v2-design.md).

## Workspace

- Five crates in `crates/`, all one version: `kitctl` (`kit-cli`, the `kit` binary), `kitctl-core`, `kitctl-agents`, `kitctl-gate`, `kitctl-tui`.
- `kit.toml` at the root is Kit's own gate: fmt, clippy `-D warnings`, workspace tests, plus `[gate.scope]`.
- The legacy TypeScript 0.1 tree (`packages/`, `apps/`, `catalog/`, the Node `CI` workflow) is still on trunk; its removal waits on owner decision K10.

## How to run and test

```sh
cargo run -p kitctl -- doctor
cargo run -p kitctl -- --demo
cargo run -p kitctl -- run --dry-run --task smoke --json

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Releasing: [`RELEASING.md`](RELEASING.md).
