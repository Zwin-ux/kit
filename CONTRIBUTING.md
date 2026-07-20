# Contributing to Kit

## Setup

```sh
pnpm install
pnpm build
pnpm test
```

## Run

```sh
pnpm tui                 # pixel TUI (kit-idle animation on splash)
pnpm kit -- --help
pnpm kit -- init --pack essentials
pnpm kit -- pack list
pnpm kit -- paths
pnpm kit -- link --to claude-code          # dry-run
```

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

- Terminal only. No web app.
- Pure black silhouette mascot in the TUI.
- Small commits. Update docs when behavior changes.
- Prefer Simplified Technical English in docs.

## Mascot / kit-idle

The README GIF (`assets/pixel/kit-idle.gif`) and the TUI splash use the same six frames (`kit-frame-1.png` … `6.png`).  
Ink cannot play GIF files portably; `MascotPlayer` cycles those frames live.
