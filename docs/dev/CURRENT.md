# Kit 1.0 — Current architecture state

**Owner judgment:** Grok Build acting as surface architect (session).  
**Date:** 2026-08-01  
**Branch focus:** Control Room is buildable and **runnable** with demo data.

---

## The product in one line

Dispatch many agents. Watch them in one place. Nothing ships unproven.

## What is real today

| Layer | Status | How to prove |
|-------|--------|----------------|
| Rust workspace (5 crates) | Real | `cargo test --workspace` |
| Single event loop + clock | Real | kit-tui tests + idle arm guard |
| Control Room / detail / dispatch / board | Real UI | `cargo run -p kit-cli -- --demo` |
| Gate (Guardian port) | Real engine | kit-gate fixture tests |
| Agent spawn / worktree / PTY | **Not built** | M1 + B2-pty |
| Headless JSON CLI parity (0.1) | **Not ported** | Codex B4 |
| npm 0.1.x skill TUI | Legacy | `packages/`, archive branch |

## Architectural spine

```
kit (kit-cli)
  └── kit_tui::run_configured
        ├── AppEvent merge: terminal | AnimationTick | RunDelta
        ├── Screen: ControlRoom | RunDetail | Dispatch | Board | Attached
        └── (later) kit-agents + kit-gate on run completion
```

**Hard rules**

1. One clock — no timers outside the event loop (`event.rs` contract).
2. Contracts are Claude-only; Grok owns `kit-tui`; Codex owns mechanical engine ports.
3. UI may create **Queued** runs and board items; only the engine may mark **Running**.
4. Failures surface first error line in the table; detail holds the full gate log.

## Critical path (architect priority)

1. ~~Event loop~~ done (M0)
2. ~~Gate port~~ done (M3)
3. ~~Control Room surface~~ done (F2–F4 on PR)
4. **Runnable entry** — `kit` / `kit --demo` (this milestone)
5. **M1 one run E2E** — worktree + stream + receipt (Codex + Grok PTY)
6. Wire dispatch submit → real spawn
7. Ship distribution (M5)

Polish (theme, mascot, mouse hit-test) **after** a real run path. Demo mode unblocks design dogfood without lying about process control.

## How to run

```bash
# Control Room with PRD fixture (no engine required)
cargo run -p kit-cli -- --demo

# Empty room
cargo run -p kit-cli

# Release binary
cargo build -p kit-cli --release
./target/release/kit --demo
```

Env: `KIT_DEMO=1` same as `--demo`.  
Motion off: `KIT_MOTION=off` or `NO_COLOR=1`.

## Agent skills

`.agents/skills` = [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (24).  
Root `AGENTS.md` routes define → plan → build → verify → review → ship.

## PR / merge

Surface work lands via PR; Claude reviews crate boundaries.  
No agent merges its own work (BUILD-ASSIGNMENT).
