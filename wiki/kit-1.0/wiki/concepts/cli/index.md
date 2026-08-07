---
title: CLI
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [current, prd-1.0]
tags: [cli]
status: partial
---

# CLI

Binary: `kit` (`crates/kit-cli`).

## Commands (1.0 target)

| Command | Purpose | Status |
|---------|---------|--------|
| `kit` / `kit tui` | Control Room | shipped |
| `kit --demo` | Fixture room | shipped |
| `kit run` | One-shot engine | shipped |
| `kit doctor` | Probes | shipped |
| `kit doctor --json` | Machine probes | planned |
| `kit receipt show` | Read receipt | planned |
| `kit receipt list` | List runs | planned |
| `kit board` | Headless board ops | planned optional |
| 0.1 parity (`ready`, `unify`, …) | Legacy | cut or separate package |

## Global flags

| Flag | Meaning |
|------|---------|
| `--version` / `-V` | version |
| `--help` | help |
| `--json` | where data is produced |

## Env

| Var | Meaning |
|-----|---------|
| `KIT_HOME` | data root (default `~/.kit`) |
| `KIT_DEMO` | demo fixture TUI |
| `KIT_MOTION` / `NO_COLOR` | reduced motion |
| `KIT_FULL_AUTO` | agent approval bypass |
| `KIT_SKILLS_DIR` | skills pack |
| `KIT_OLLAMA_MODEL` | ollama model |

## Error UX

Every error must name the fix (PRD DoD).  
Exit codes: 0 pass, 1 gate fail, 2 error — already partial on `kit run`.
