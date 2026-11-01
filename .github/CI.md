# GitHub Actions

| Workflow | Runs on | What it proves |
|----------|---------|----------------|
| `rust.yml` | push to `main`, PRs touching Rust, npm or scripts | fmt, clippy `-D warnings` and tests on Ubuntu, macOS and Windows; `cargo package`; `cargo deny`; the line budget; npm launcher smoke on three OSes; cold-start budget on three OSes |
| `installers.yml` | PRs touching the install scripts | `install.sh` / `install.ps1` against a local release |
| `release-npm.yml` | tag `v*`, or manual (dry run unless `publish` is checked) | build, verify, GitHub Release, npm, crates.io, install checks |

## Safety

- `rust.yml` and `installers.yml` have read-only `contents` and use no secrets.
- Only `release-npm.yml` uses secrets (`NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`), and only on a tag push or a
  manual run with `publish` checked. Only the owner pushes release tags.

## Local equivalents

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node scripts/loc-budget.mjs
```

The Kit 0.1 Node CI and keep-alive workflows were removed with the TypeScript tree in A0.0
(tag `legacy-0.1-final`).
