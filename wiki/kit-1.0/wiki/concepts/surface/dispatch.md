---
title: Dispatch
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current, b2-agent-adapters]
tags: [tui]
status: partial
---

# Dispatch

Fan-out form: **repos × agents × one task** → N isolated runs.

## Why it matters

The fan-out **is** the feature: one task, many repos/agents, many worktrees, many gates.

## Form fields

| Field | UI | Notes |
|-------|-----|-------|
| Repos | multi-toggle list | defaults include kit / guardian / trenchwire labels |
| Agents | multi-toggle | codex, claude, grok, ollama |
| Task | free text (max ~240) | required non-empty |

Focus: Tab cycles Repos → Agents → Task. Space toggles list items.

## Submit algorithm

1. Validate task non-empty; ≥1 repo; ≥1 agent
2. Cap combos at **16** (`DISPATCH_FANOUT_CAP`)
3. For each pair: new `RunId`, insert `Queued` row, build `DispatchJob`
4. Return `Action::DispatchSubmitted { jobs }`
5. CLI loop sends jobs to engine workers
6. Flash: “N run(s) queued — starting engine”

## Engine path (live)

Each job → `engine::execute` with auto dry-run/live → skills inject → adapter → gate → receipt.

## Keys

| Key | Action |
|-----|--------|
| Esc | Control Room (discard unsubmitted form state stays in memory) |
| Tab / S-Tab | Field focus |
| Space | Toggle |
| Enter | Submit |
| type | Task field only |

## v1.0 gaps

- [ ] Repo picker from real disk paths / recent list (not only labels)
- [ ] Skill multi-select (today: always inject full pack routing)
- [ ] Per-agent bounds overrides
- [ ] Confirm when N&gt;4 (“fan out 8 runs?”)
- [ ] Persist last form
