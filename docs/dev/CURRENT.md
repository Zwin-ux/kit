# Kit 1.0 — Current architecture state

**Owner judgment:** Grok Build as surface + integration architect.  
**Date:** 2026-08-01  
**Goal:** production path, not prototype aura.

---

## The product in one line

Dispatch many agents. Watch them in one place. Nothing ships unproven.

## What is real today

| Layer | Status | How to prove |
|-------|--------|----------------|
| Rust workspace (5 crates) | Real | `cargo test --workspace` |
| Single event loop + clock | Real | kit-tui tests + idle arm guard |
| Control Room / detail / dispatch / board | Real UI | `cargo run -p kit-cli -- --demo` |
| Gate (Guardian port) | Real | kit-gate fixtures |
| **M1 dry-run engine** | **Real skeleton** | `kit run --task "…" --json` |
| Worktree + receipt store | Real | `~/.kit/runs/<id>/receipt.json` |
| Agent CLI adapters (codex/…) | Dry-run only | B2 real spawn next |
| PTY attach | Stub screen | B2-pty |
| Headless JSON CLI parity (0.1) | Not ported | B4 |

## Architectural spine

```
kit (kit-cli)
  ├── kit run  ──► engine::execute
  │                  ├── git worktree (isolated)
  │                  ├── dry-run stream → RunDelta
  │                  ├── kit-gate::evaluate (or vacuous)
  │                  └── ~/.kit/runs/<id>/ receipt.json + output.log
  │
  └── kit / kit --demo  ──► kit_tui::run_configured
        ├── AppEvent: terminal | AnimationTick | RunDelta
        ├── Dispatch → EngineRequest channel → same execute()
        └── Screens: ControlRoom | Detail | Dispatch | Board | Attached
```

**Hard rules (production)**

1. One clock — no timers outside the event loop.
2. Contracts are Claude-only.
3. **Only the engine marks Running / Gating / Pass / Fail.** UI may queue.
4. **No PASS without a gate outcome** (vacuous gate is explicit in the log).
5. Clean worktrees are removed; dirty worktrees are kept for forensics.
6. Receipts are on disk before the process claims done.

## Critical path

1. ~~Event loop~~ M0  
2. ~~Gate port~~ M3  
3. ~~Control Room surface~~ F2–F4  
4. ~~Runnable entry~~ `kit` / `--demo`  
5. ~~M1 dry-run E2E skeleton~~ worktree + stream + gate + receipt  
6. **B2 real agent adapters** (codex/claude/grok/ollama spawn)  
7. **B2-pty** attach  
8. Wire kill/retry to process handles  
9. M5 distribution  

## How to run

```bash
# Headless M1 (CI-friendly)
cargo run -p kit-cli -- run --task "smoke" --json

# Control Room (demo fixture, no engine traffic)
cargo run -p kit-cli -- --demo

# Control Room empty — Dispatch (d) starts real dry-run engine jobs
cargo run -p kit-cli

# Release
cargo build -p kit-cli --release
./target/release/kit run --repo . --task "…" --json
```

Env: `KIT_HOME` (default `~/.kit`), `KIT_DEMO=1`, `KIT_MOTION=off`, `NO_COLOR=1`.

## Agent skills

`.agents/skills` = [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills).  
Root `AGENTS.md` routes define → plan → build → verify → review → ship.

## PR / merge

Surface + engine skeleton land via PR; Claude reviews crate boundaries.  
No agent merges its own work (BUILD-ASSIGNMENT).
