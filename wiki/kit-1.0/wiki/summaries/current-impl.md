---
title: Summary — CURRENT implementation
type: summary
created: 2026-08-01
updated: 2026-08-01
sources: [current]
tags: [implementation, source]
---

# Summary — CURRENT implementation

Source: `raw/notes/current.md` (`docs/dev/CURRENT.md`)

## Shipped (real)

- Control Room TUI (table, sort, stable select, elapsed, FAIL annotation)
- Run Detail (stream / gate / diff panes, attach **stub**)
- Dispatch + Board UI
- Event loop + single clock
- kit-gate Guardian port + fixtures
- M1 engine: worktree, dry-run or live adapters, gate, receipt under `~/.kit/runs/`
- Adapters: codex / claude / grok / ollama with skills injection
- `kit` / `kit run` / `kit doctor` entrypoints
- agent-skills pack under `.agents/skills`

## Partial / stub

- Attach (screen only, no PTY)
- Kill / retry (Action seams, no process registry)
- Theme / mascot / mouse hit-testing
- Board as true pull-queue (UI queue + prefill only)
- Vacuous gate still marks `passed: true` with log honesty (policy open)

## Not shipped

- Headless JSON parity with 0.1 CLI (B4)
- npm platform binary packages + curl installer (M5)
- Full 8-concurrent frame proof harness
- Documented third-party receipt consumers

## Implication

v1.0 quality is **blocked less by screens** and more by **control plane completeness**: kill, attach, concurrency proof, install path, and gate policy for empty configs.
