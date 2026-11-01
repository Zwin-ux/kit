# Contributing to Kit

Thanks for helping. Kit is meant to stay small enough to read in an afternoon, so the best contributions
are small, tested and easy to review.

## Setup (three commands)

```bash
git clone https://github.com/Zwin-ux/kit.git && cd kit
cargo test --workspace
cargo run -p kitctl -- --demo
```

You need a stable Rust toolchain (`rustup`), and git. On Windows, a checkout in a deep folder can hit the
260-character path limit in the linker: set `CARGO_TARGET_DIR` to a short folder
(PowerShell: `$env:CARGO_TARGET_DIR = 'C:\kit-target'`).

## Before you open a PR

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same on Ubuntu, macOS and Windows. The bar is [`docs/QUALITY.md`](docs/QUALITY.md):

- Tests at the right layer: unit tests in the crate, CLI behaviour in `crates/kit-cli/tests/`, screens as
  insta snapshots in `crates/kit-tui`.
- A screen change includes its updated `.snap` files, and you read every changed line.
- A change to `--json`, `KIT.toml`, `kit.toml`, `kit.lock` or receipts needs a `CHANGELOG.md` entry under
  `## Unreleased` and a test that pins the new shape.
- No stub copy, no "coming soon", no key in a footer that does nothing.

## Sign your commits (DCO)

Kit uses the [Developer Certificate of Origin](https://developercertificate.org/), not a CLA. Add a
`Signed-off-by` line to each commit to certify you wrote it or have the right to submit it:

```bash
git commit -s -m "fix(gate): …"
```

## Where things live

| Path | What |
|------|------|
| `crates/kit-cli` | The `kit` binary: commands, kits, wiring |
| `crates/kit-core` | Run, config, gate and receipt types |
| `crates/kit-gate` | The gate (your repo's checks) |
| `crates/kit-agents` | Adapters that run Claude Code, Codex and Grok |
| `crates/kit-tui` | The Control Room and its screens |
| `crates/kit-cli/kits` | The starter kits |
| `docs/PRD.md` | The product plan and milestones |
| `docs/dev/` | Architecture, design, releasing |

The plan for Kit's own agent (new crates `kit-loop`, `kit-ext`, `kit-providers`, `kit-sandbox`) is in
[`docs/PRD.md`](docs/PRD.md) §8. Pick a milestone item from the issues labelled `good first issue`, or open
an issue before starting something large.

## Conduct and security

- [Code of Conduct](CODE_OF_CONDUCT.md)
- Security problems: see [SECURITY.md](SECURITY.md). Please don't open a public issue.
- How decisions and merges work: [GOVERNANCE.md](GOVERNANCE.md)
