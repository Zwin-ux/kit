---
title: RunDelta
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [kit-core-run-contract]
tags: [core, tui]
status: shipped
---

# RunDelta

Incremental update from a run task to the UI (and any subscriber).

## Variants (contract)

| Variant | Payload | Control Room effect | Detail effect |
|---------|---------|---------------------|---------------|
| `State(RunState)` | new state | update STATE column | header state |
| `Output(String)` | chunk | ignore (or activity pulse later) | append stream |
| `Worktree(PathBuf)` | path | ignore | header worktree line |
| `Gate(GateOutcome)` | outcome | GATE PASS/FAIL + FAIL annotation | gate pane |

## Transport

```
engine  --mpsc (RunId, RunDelta)-->  TUI event loop  -->  AppEvent::RunUpdate
```

Channel closed → TUI disables that select arm (idle CPU invariant).

## Missing deltas (gaps)

| Need | Today | v1.0 plan |
|------|-------|-----------|
| Live diff stream | only at receipt / fixture `RunRow.diff` | `RunDelta::Diff` **or** load receipt on terminal |
| Elapsed wall time from engine | TUI clock ticks from Running | optional `StartedAt` meta |
| Process PID / handle | none | internal registry, not necessarily on wire |

Contract changes require Claude.
