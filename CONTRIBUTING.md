# Contributing to Kit

## Setup

```sh
pnpm install
pnpm build
pnpm test
```

## Run the CLI (Windows)

From the repo root after `pnpm build`:

```powershell
.\kit.cmd --help
.\kit.cmd whoami
.\kit.cmd explore packs
.\kit.cmd login
```

Or:

```powershell
pnpm kit -- whoami
node packages\cli\dist\bin.js whoami
```

There is no global `kit` command until you install/link the CLI yourself.

## Run

```sh
pnpm tui                 # pixel TUI (kit-idle animation on splash)
pnpm kit -- --help
pnpm kit -- init --pack essentials
pnpm kit -- pack list
pnpm kit -- explore packs
pnpm kit -- paths
pnpm kit -- link --to claude-code          # dry-run
pnpm kit -- test --all-packs
pnpm kit -- doctor
```

## CI

`.github/workflows/ci.yml` runs build + unit tests + pack tests + doctor on `main`.  
It is additive and does not deploy. See `.github/README.md`.

## Layout

| Path | Job |
|------|-----|
| `packages/core` | Skill/pack engine (offline) |
| `packages/cli` | Command line |
| `packages/tui` | Pixel-art TUI |
| `packages/shared` | Shared types |
| `skills/` | Skill catalog |
| `packs/` | Starter packs |
| `assets/pixel/` | Mascot frames + `kit-idle.gif` |

## Rules

- Product surface is the CLI and pixel TUI.
- Pure black silhouette mascot in the TUI.
- Small commits. Update docs when behavior changes.
- Prefer Simplified Technical English in docs.

## Mascot / kit-idle

The README GIF (`assets/pixel/kit-idle.gif`) and the TUI splash use the same six frames (`kit-frame-1.png` … `6.png`).  
Ink cannot play GIF files portably; `MascotPlayer` cycles those frames live.
