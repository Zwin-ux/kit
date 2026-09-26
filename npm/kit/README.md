# kit

Dispatch many coding agents. Watch them in one place. Nothing ships unproven.

Kit runs Codex, Claude, Grok, and Ollama in isolated git worktrees. Each run must pass your gate (format, typecheck, tests) before it is done. Each run writes a receipt.

```bash
npm install -g @mzwin/kit@alpha
kit --demo
```

## Commands

| Command | Result |
|---------|--------|
| `kit` | Open the Control Room |
| `kit --demo` | Control Room with sample runs |
| `kit init` | Write `kit.toml`: a gate for this repo |
| `kit run --task "…"` | One isolated run with a gate |
| `kit run --dry-run --json` | Offline run, JSON result |
| `kit doctor` | Check agents and skills |
| `kit receipt list` | List receipts in `~/.kit/runs/` |

## The gate

In your repo root:

```bash
kit init --check
```

`kit init` reads `Cargo.toml`, `go.mod`, `package.json` or `pyproject.toml` and writes `kit.toml`. It uses only commands that check and do not change files. `--check` runs each command once and keeps the ones that pass. `--print` writes nothing. It does not replace a `kit.toml` unless you add `--force`.

For a Rust repo it writes:

```toml
[gate]
format    = "cargo fmt --all --check"
typecheck = "cargo clippy --workspace --all-targets -- -D warnings"
test      = "cargo test --workspace"
timeout   = "15m"
```

You can also write `kit.toml` by hand. Each command must exit 0 for a run to pass.

A run that fails the gate shows `FAIL` and the first error. Press `r` to retry with the gate output.

## Platforms

Prebuilt binaries: Windows x64, macOS arm64 and x64, Linux x64 and arm64 (glibc 2.17+).

npm installs one platform package (for example `@mzwin/kit-win32-x64`). If you install with `--omit=optional`, the binary is not installed and `kit` stops with an error.

Other platforms: `cargo install --git https://github.com/Zwin-ux/kit kit-cli`.

Set `KIT_BINARY_PATH` to use a binary you built.

## Versions

`1.x` is a native binary. `0.1.x` was a Node app and stays on the `latest` tag until `1.0.0`.

MIT · [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)
