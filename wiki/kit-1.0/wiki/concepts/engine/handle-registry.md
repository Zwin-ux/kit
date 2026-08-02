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
RunId -> RunHandle {
  agent: Box<dyn AgentHandle>,
  cancel: CancellationToken,
  started: Instant,
  job: DispatchJob,
}
```

Owned by engine supervisor task in `kit-cli` (or future kit-engine).

## Operations

| Op | Behavior |
|----|----------|
| register | on spawn |
| kill | `agent.kill()` + cancel; state Killed; receipt written |
| get | for attach (PTY) later |
| drop | on terminal state |

## Action wiring

- `Action::KillSelected` → send `EngineCommand::Kill(id)` on channel
- `Action::RetrySelected` → kill if running optional; enqueue new job with failure context

## Concurrency

- Mutex or actor pattern
- Max concurrent runs (default 8) — queue excess as Queued

## Acceptance

- [ ] k stops live codex/claude within 2s
- [ ] Receipt shows Killed
- [ ] Worktree kept if dirty
- [ ] No zombie processes on Windows/macOS/Linux CI
