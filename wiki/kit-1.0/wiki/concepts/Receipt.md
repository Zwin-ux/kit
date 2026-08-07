---
title: Receipt
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, kit-core-run-contract, current]
tags: [core]
status: partial
---

# Receipt

Immutable record that a [[concepts/Run/index|Run]] happened. **Proof or it didn’t happen.**

## On-disk layout

```
$KIT_HOME/runs/<run-id>/
  receipt.json     # schema versioned
  output.log       # full captured agent output
  diff.patch       # optional, if non-empty
  gate.json        # optional mirror of gate outcome
```

## receipt.json fields (contract v1)

| Field | Type | Notes |
|-------|------|-------|
| `version` | u32 | currently `1` |
| `id` | RunId | |
| `spec` | RunSpec | repo, agent, task, bounds, branch |
| `state` | RunState | terminal |
| `started_at` / `ended_at` | time | |
| `diff` | string | unified diff |
| `gate` | GateOutcome? | must be present for honest Pass |
| `output_truncated` | bool | hit output cap |

## Consumers

- Humans: open folder after failure
- Third-party tools: parse JSON without Kit
- Future: `kit receipt show <id> --json`

## Stability rules (v1.0)

1. Never rewrite a receipt after write.
2. Bump `version` on breaking shape changes.
3. Document schema in README or `docs/receipts.md` before 1.0.0.
4. Paths use forward-looking portable JSON (paths as strings).

## Status

Write path shipped. Documented third-party consumer guide **planned**. CLI `receipt` command **planned**.
