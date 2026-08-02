---
title: GateOutcome
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [kit-core-gate-contract]
tags: [core]
status: shipped
---

# GateOutcome

Result of evaluating a gate for one run.

## Fields

| Field | Meaning |
|-------|---------|
| `passed` | Overall boolean |
| `checks[]` | Per-command `GateCheck` |
| `scope_violations[]` | Paths outside scope |
| `firewall_blocks[]` | Commands refused |
| `duration` | Total gate time |

## GateCheck

| Field | Meaning |
|-------|---------|
| `label` | format / typecheck / test / extra |
| `command` | exact command |
| `status` | Pass / Fail / Skipped / TimedOut |
| `exit_code` | optional |
| `summary` | **first meaningful error line** (UI gold) |
| `duration` | per check |

## UI mapping

- Control Room GATE: `PASS` / `FAIL` / `--`
- FAIL annotation: `first_failure().summary` → `^ tsc: 3 errors`
- Detail Gate pane: full checklist via `gate_log_lines`

## API helper

`GateOutcome::first_failure()` — first non-passing check.
