# Implementation Plan: Daily-driver Kit (from SPEC-next)

**Spec:** [`docs/dev/SPEC-next.md`](../docs/dev/SPEC-next.md)  
**Board:** [`tasks/todo.md`](todo.md)  
**QA evidence:** [`docs/dev/dogfood-notes/2026-08-16-qa.md`](../docs/dev/dogfood-notes/2026-08-16-qa.md)

## Overview

Kit is a credible alpha Control Room. Session A proved a real gate. QA proved idle CPU and fixed `kit --demo`. The remaining work is **identity** (the verb `kit`, GitHub, land the branch), **live proof** (Sessions B–D), then **isolate 0.1** and **small TUI honesty**. Not personas, PTY, or an installer.

## Architecture (do not reopen)

1. Engine owns runs, worktrees, gates, receipts. TUI sends `EngineCommand`, paints `RunDelta`.
2. `kit.toml` on this repo is the product gate. Engine tests use `bare_git_fixture()`.
3. `↑↓` move, `k` kills. One `AnimationTick`. No mascot.
4. Personas stay ENG-default, TUI-local. No expansion.
5. In-process engine. No daemon.

## Phases (do not skip)

### Phase I — Identity and land

I1 GitHub About (human) → I2 commit/PR (when asked) → I6/I7 honest docs → I3 live run → I4 TUI session → I5 notes.

**Checkpoint:** you would open Kit tomorrow; `kit --demo` is the product; a live receipt exists.

### Phase P — Proof gaps

P1 `CARGO_TARGET_DIR` isolation (Factory) after I2.  
P3 idle-CPU script (can start anytime).  
P2 / P4 / P6 need CEO for contracts or CI policy.  
P5 live kill after I3.  
P7 mock adapter after I2.

### Phase N — Stop paying for 0.1

N1 Node CI path-filter after I2 merges. N2 CHANGELOG with I2. N3/N4 human.

### Phase U — TUI excellence (one slice at a time)

U1 FAIL annotation width first. Then U2 footer. Then U3–U6. Not U7 expansion.

### Phase S — Ship

S1 alpha.2 only after Phase I checkpoint. S3–S4 after that.

## Task details

### Task I2: Commit + PR

**Description:** Land the uncommitted 1.0 work (`kit.toml`, shims, doctor, TUI, personas, `--demo` fix, docs) so GitHub matches the machine.

**Acceptance:**
- [ ] User asked to commit
- [ ] PR opened; Rust CI green
- [ ] No self-merge

**Files:** most of the dirty tree. Prefer 2–3 commits (engine isolation / kit.toml+docs / TUI+CLI UX).  
**Scope:** M  
**Depends:** You say go

### Task I3: Session B live run

**Description:** One ready agent (doctor lists all four) against this repo. Short, scoped task.

**Acceptance:**
- [ ] `.\kit.cmd run --agent <ready> --task "…"` receipt id in `docs/dev/dogfood-notes/2026-08-16.md`
- [ ] Gate ran (PASS or named FAIL)

**Verification:** `.\kit.cmd receipt show <id>`  
**Scope:** S  
**Depends:** none (can precede I2)

### Task I4: Session C TUI

**Description:** `.\kit.cmd --demo` then empty/`d` fan-out. Kill or retry once.

**Acceptance:**
- [ ] FAIL annotation seen on demo
- [ ] At least two runs visible from one dispatch **or** one live + demo recorded
- [ ] `k` or `r` used once

**Scope:** S  
**Depends:** I3 preferred

### Task P1: Isolate cargo target in worktrees

**Description:** Gate/run children must not inherit the parent `CARGO_TARGET_DIR`.

**Acceptance:**
- [ ] Child env sets `CARGO_TARGET_DIR` to the worktree (or unsets it)
- [ ] Test: parent target dir mtime/count unchanged across a dry-run gate on a fixture

**Files:** `crates/kit-cli/src/engine/runner.rs`, gate spawn  
**Scope:** S  
**Depends:** I2 or parallel if You allow engine edit now  
**Owner:** Factory

### Task U1: FAIL annotation readable

**Description:** First error line is the wedge. Keep `^ tsc: 3 errors` at 80 and 60.

**Acceptance:**
- [ ] Snapshots at 80×14 and 60×12 show the why, not `^ tsc: 3`
- [ ] `cargo test -p kitctl-tui`

**Files:** `crates/kit-tui/src/ui/control_room.rs`  
**Scope:** S  
**Owner:** Power

### Task N1: Node CI only on `packages/`

**Description:** Stop six Node jobs and keep-alive on Rust-only PRs.

**Acceptance:**
- [ ] Path filters or workflow `if`
- [ ] Keep-alive disabled or `packages/`-only

**Files:** `.github/workflows/*`  
**Scope:** S  
**Owner:** Factory  
**Depends:** I2 merged (or same PR if You want)

## What we will not do in these phases

PTY, installer, vibe UI, F6, `app.rs` split, mascot, marketplace, shrinking `kit.toml`, persona expansion.

## Kill this plan if

- Session B is skipped in favor of installer or mascot
- `kit.toml` is weakened to speed CI
- Bare `kit` is documented as the 1.0 verb while npm 0.1 still owns PATH
