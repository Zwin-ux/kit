---
title: Handle Registry
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [engine]
status: planned
---

# Handle Registry

In-memory map enabling **kill** and **retry** from the Control Room.

## Problem

Today each run is a fire-and-forget `tokio::spawn`. The TUI cannot stop a process.

## Design

```
RunId -> Arc<CancelHandle>   // AtomicBool + Notify (no tokio-util)
EngineCommand { Start(job) | Kill { id } | Retry { source_id, job } }
Supervisor: Semaphore(8) + RunRegistry + execute_cancellable
```

Owned by engine supervisor task in `kit-cli` (CEO vetoed kit-engine extract for P1).

## Operations

| Op | Behavior |
|----|----------|
| register | before concurrency wait (so queued kills work) |
| kill | `CancelHandle::cancel` → agent `kill()`; state **Killed**; receipt written |
| timeout | same path as kill; output reason `timeout` (not a new RunState) |
| drop | unregister on terminal |

## Action wiring

- `Action::KillSelected { id }` → `EngineCommand::Kill { id }`
- `Action::RetrySelected { source_id, job }` → fail-only; task includes `## Previous gate failure`

## Concurrency

- `tokio::sync::Semaphore` max **8**
- Excess wait for permit; cancel before acquire → Killed, no worktree

## Acceptance

- [x] EngineCommand channel + registry (Power)
- [x] Receipt shows Killed on cancel
- [x] Fail-only retry with gate context
- [ ] k stops live codex/claude within 2s on 3 OS (dogfood)
- [ ] No zombie processes proven in CI
- [ ] Dispatch 12 → ≤8 concurrent
