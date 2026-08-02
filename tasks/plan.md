# Implementation Plan: Kit TUI surface (F4–F6)

Skills active: `planning-and-task-breakdown`, `incremental-implementation`,  
`test-driven-development`, `frontend-ui-engineering` (TUI-adapted).

## Overview

Complete the Grok-owned Control Room surface after F2+F3 (PR #11):

1. **F4 Dispatch** — repos × agents × one task → fan-out into queued runs  
2. **F4 Board** — shared task queue (orchestrator view)  
3. **F5 Theme** — minimal high-contrast theme + reduced-motion already partially honored  
4. **F6** — layout rect registry for hit-testing (later slice; keyboard-first until then)

Engine still missing (Codex M1). UI creates **queued** runs and board items so the product is demoable and testable without agents.

## Architecture decisions

1. **Dispatch is pure reducer** until M1: submit inserts `RunRow`s in `Queued` state; flash notes engine will execute later. No fake "running" without process.
2. **Board is TUI-local** `Vec<BoardTask>` — not a second source of truth for Runs. Claiming a board task can pre-fill Dispatch.
3. **One form focus model** for Dispatch (field index), same as fennec trade form simplicity.
4. **No contract edits.** Repos/agents are string labels matching `AgentKind::label`.
5. **Skills pack** lives in `.agents/skills` (addyosmani/agent-skills); agents must route via `using-agent-skills`.

## Task list

### Phase A: Dispatch (F4a)

- [x] A1: `Screen::Dispatch` + `DispatchForm` state + open via `d`
- [x] A2: Toggle repos/agents, edit task text, focus navigation
- [x] A3: Submit fan-out → N queued runs + return Control Room
- [x] A4: Draw dispatch frame + snapshots

### Checkpoint A
- [x] cargo test -p kit-tui green
- [x] Enter from empty selection no-ops; Esc always returns

### Phase B: Board (F4b)

- [x] B1: `Screen::Board` + board task list + open via `b`
- [x] B2: Add / select / remove queue items (keyboard)
- [x] B3: Enter on item → open Dispatch prefilled
- [x] B4: Board draw + snapshots

### Checkpoint B
- [x] Full nav: ControlRoom ↔ Dispatch ↔ Board ↔ Detail
- [x] Footer key map updated on Control Room

### Phase C: Theme (F5 light)

- [x] C1: `theme.rs` with default + high-contrast styles
- [ ] C2: Apply theme to every widget (partial — module ready)
- [x] C3: Document `KIT_MOTION` / `NO_COLOR` in module docs

### Checkpoint C
- [x] clippy -D warnings, fmt, all kit-tui tests

### Phase D: later (not this session unless time)

- [ ] F6 rect hit-testing
- [ ] Mascot port from assets
- [ ] Wire Submit to real engine when M1 lands

## Risks

| Risk | Mitigation |
|------|------------|
| Form editing complex on Windows keys | Press-only filter; char insert; Backspace |
| Fan-out floods table | Cap selected combos warning if &gt; 16 |
| Scope into engine | Queued only; flash honesty |

## Open questions

- None blocking: fan-out is UI-only until M1.
