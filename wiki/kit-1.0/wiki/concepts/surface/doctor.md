---
title: Doctor
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [cli, tui]
status: partial
---

# Doctor

Answers: what’s installed, authenticated, broken, and how to fix it.

## CLI today (`kit doctor`)

Reports:

- binary / control room / gate / run engine ok
- `KIT_HOME`, skills pack path
- Per-agent probe: ready/missing + version + remedy

Does **not** read credentials (principle 4). “Authenticated” for adapters is currently optimistic if binary exists.

## Full concept (v1.0)

| Check | Pass criteria | Remedy example |
|-------|---------------|----------------|
| git | on PATH | install git |
| kit binary | version prints | reinstall |
| agent binaries | probe | install codex/claude/… |
| agent auth | provider-specific non-secret check | `codex login` |
| skills pack | dir exists | clone/install skills |
| kit.toml sample | optional warn | scaffold command |
| disk for KIT_HOME | writable | fix permissions |

## TUI screen

PRD lists Doctor as a screen.  
**Plan:** `Shift-D` or `?` from Control Room; reuse same probe data. Not blocking if CLI doctor is excellent.

## JSON

`kit doctor --json` for CI — **planned** (DoD: every data command has --json).
