---
title: Run
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, kit-core-run-contract, current]
tags: [core]
status: partial
---

# Run

The **one primitive**. Every screen is a view over Runs.

## Shape (contract)

```
Run {
  id          ULID  — sortable, stable
  repo        Path  — target repository root
  worktree    Path? — isolated git worktree once started
  agent       codex | claude | grok | ollama
  task        string — prompt (skill reference optional later)
  bounds      Bounds
  state       RunState
  gate        GateOutcome?
  started_at / ended_at
}
```

See frozen contract: `raw/refs/kit-core-run-contract.md` → `crates/kit-core/src/run.rs`.

## Sub-pages

- [[concepts/Run/lifecycle|Lifecycle]] — state machine
- [[concepts/Run/bounds|Bounds]] — timeout, output, scope
- [[concepts/Run/worktree|Worktree]] — isolation rules
- [[concepts/Run/delta|RunDelta]] — UI event stream

## Related

- [[concepts/Gate/index|Gate]] decides PASS/FAIL after agent stop
- [[concepts/Receipt|Receipt]] freezes the run to disk
- [[concepts/surface/control-room|Control Room]] lists runs
- [[concepts/engine/pipeline|Engine pipeline]] executes runs

## Status

| Piece | Status |
|-------|--------|
| Types / deltas | shipped (contract) |
| Create via Dispatch / `kit run` | shipped |
| Live adapters | shipped |
| Kill / retry semantics | partial (UI seams only) |
| Concurrent N=8 proof | planned |
