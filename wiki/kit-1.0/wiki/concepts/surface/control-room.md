---
title: Control Room
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [tui]
status: shipped
---

# Control Room

Default surface (`kit`). Live table of all runs.

## Layout

```
KIT / CONTROL ROOM                    N RUNNING  M GATED
┌─────────────────────────────────────────────────────┐
│ REPO   AGENT   TASK              STATE     GATE     │
│ > kit  codex   port guard.js     RUN 2m    --       │
│   …                                                 │
│                          ^ tsc: 3 errors            │
└─────────────────────────────────────────────────────┘
 [d]ispatch  [b]oard  [enter] open  [g]ate log  [k]ill  [r]etry
```

## Columns

| Column | Source |
|--------|--------|
| REPO | `RunRow.repo` (short name or path leaf) |
| AGENT | agent label |
| TASK | truncated task |
| STATE | QUEUED / RUN t / GATING t / DONE / KILLED / ERROR |
| GATE | PASS / FAIL / -- |

## Sorting

1. State rank: Running → Gating → Queued → Fail → Error → Pass → Killed  
2. Then age (`seq` / creation order)

## Selection

- Stored as `selected_id: Option<RunId>`
- ↑↓ move in **display order**
- Re-sort does not change selected run

## FAIL annotation

If selected/failed row has `gate_summary()`, show indented line:  
`^ <first error summary>`

## Keys

| Key | Action |
|-----|--------|
| ↑↓ | Move selection |
| Enter | Open Run Detail (Stream) |
| g | Open Run Detail (Gate) |
| d | Open Dispatch |
| b | Open Board |
| k | KillSelected (engine) |
| r | RetrySelected (engine) |
| q | Quit |

## Flash

Unwired or engine messages: short banner ~2s via clock (no extra timer).

## Acceptance (done for table)

- [x] Sort + stable select + elapsed + FAIL line + snapshots  
- [ ] Live kill works  
- [ ] Live retry works  
- [ ] 8 concurrent without scroll thrash / interleave
