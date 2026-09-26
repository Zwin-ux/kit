# kit

Kit sets your coding agents up for one job, then proves what they do.

```bash
npm install -g @mzwin/kit
kit setup
```

`kit setup` finds Claude Code, Codex and Grok on your machine, asks which job you want them set up for, shows every skill, rule, MCP server and hook it will install (each pinned to an exact version, with its licence), and installs only after you say yes.

## Commands

| Command | Result |
|---------|--------|
| `kit setup` | Pick agents and a kit; install after a yes |
| `kit show [kit]` | The starter kits, or what one kit installs |
| `kit add <kit>` / `kit remove <kit>` | Install or undo a kit exactly |
| `kit list` | Installed kits and anything changed by hand |
| `kit search` / `kit sync` / `kit new` | Find kits, install a repo's `kit.lock`, start your own |
| `kit init` | Write `kit.toml`: the checks every run must pass |
| `kit run "task"` | One agent in its own git worktree, then the checks, then a receipt |
| `kit land <id>` | Put a proven run's changes on a new branch |
| `kit` | The Control Room: every run in one table (`kit --demo` for sample runs) |
| `kit doctor` | Which agents are ready, and whether your kits are intact |

## Platforms

Prebuilt binaries: Windows x64, macOS arm64 and x64, Linux x64 and arm64 (glibc 2.17+). npm installs only the platform package for your machine (for example `@mzwin/kit-win32-x64`). With `--omit=optional` the binary is not installed and `kit` stops with an error.

Other platforms: `cargo install kitctl --locked` (the crate is `kitctl`; the command is `kit`). Set `KIT_BINARY_PATH` to use a binary you built.

## Versions

`2.x` is the native binary with kits. `1.0.0-alpha.1` was a preview of it. `0.1.x` was the old Node app.

MIT · [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)
