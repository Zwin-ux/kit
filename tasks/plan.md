## Native release stages — 2026-10-02

Approved sequence: private alpha for the owner, public beta, then stable. The user explicitly approved the plan and swarm execution. Prepare and verify the next alpha with UI, native execution and release-audit lanes; coordinate source ownership and independently review each change. Keep publishing/tags separate from candidate work, preserve global settings and the approved role/UI scope, and retain actual acceptance gates. Reuse exporter/install/launch paths; keep full-Kit release automation separate from native-Mod distribution.

Completed: [release stages](../docs/dev/NATIVE-RELEASES.md), [owner guide](../docs/dev/NATIVE-ALPHA.md), explicit native 2.0.0-alpha.1 metadata, corrected version-only CLI readiness with a red/green regression, and local folder/archive/checksums. 229-file Rust/Node parity, clean extracted-loader discovery and shared checks pass. Swarm completed four native role tasks, three writer worktrees and read-only exact-artifact Security review on immutable alpha.1. Fixture startup corrected: 38 native tests passed, then independent engine ref0 diagnostics found a host composition defect; later runner attempts naturally rollout-off. Next: reviewed UI corrections, repeatable alpha.2 package, allowed owner native review and the visible draft/capture/review matrix. Public beta requires exact committed candidate CI and all promotion evidence.

## Actual-use continuation — 2026-10-02

Acceptance is real native Backend invocation → isolated artifact → independent checks → immutable captured answer → generated exact-result review → read-only native Security → discoverable Security result. A first same-session run exposed a foreground-pruning defect: Security completed but Kit never saw its identity. Completed: passive native-start observer, denial/reset/unrelated guards, exact packaged alpha.3 and alpha.4 loops in fresh disposable repositories, and retained failed-run evidence. Both answers remain discoverable; 22 artifact assertions and native exit pass. Alpha.4 fixes the stale README label, with runtime bytes identical to alpha.3. Written Security report coverage remains inspectable and completion is never automatic approval. Native visible composer/focus/paint remains a separate gate; preserve the host computer-use denial.

## Native alpha.2 continuation — 2026-10-02

Three independent lanes checked UI contracts, packaging and release gates. Fixed structured close-denial timer handling and parallel Rust scratch-directory collisions; both have failing-before/passing-after regressions. Existing native suite now passes 39 fixtures on naturally enabled Claude 2.1.287. Shared `--host` check passes with 34 lifecycle/four native unit/four native CLI checks, archive regression, strict validation/types/loader, formatting and Clippy. Full workspace still has one unrelated inherited-NO_COLOR TUI failure after 411 passes.

Completed: immutable alpha.2 archive, clean extraction/checksums, actual extracted-loader discovery and 229-file Rust/Node/source parity. Next: owner native visual/draft/result-review acceptance. Terminal automation was denied; preserve host restrictions. Candidate CI remains local/uncommitted and public release gates remain open. See current checkpoint in NATIVE-ENVIRONMENT.md; historical rollout refusals below are earlier observations.

## Native lifecycle repairs — 2026-10-02

Source repairs complete: twenty lifecycle regressions pass. Preserve the four approved roles and sixteen skills; do not edit kit-tui or global Claude settings.

1. Draft preparation: use native append to add only an explicit agent mention or fixed-result request, never replay a captured draft. Reject observed existing agent mentions. Preserve transaction locks/session invalidation; warn about a fill already pending during a session change rather than attempting a destructive rollback.
2. Registration: await successful command registration before enabling Kit UI or starting its clock. On refusal, yield command/render hooks to their downstream owner, continue host startup, and leave no Kit clock. Session reset invalidates pending startup.
3. Status: assign each refresh a latest-request token and reject older shared responses/errors and previous-session responses; use the completion's own snapshot for attribution. Cancel pane opens still waiting across a reset; denied motion settings default off and preserve startup.

Verification: real source in Node host-double tests (red then green), installed-version TypeScript and strict native validation; real local loader/package checks and relevant Rust embedding checks. Official hooks test runner is still rollout-disabled: keep its failure separate from host-double evidence. Rebuild a fresh local payload after fixes and update the actual evidence.

## Active extension — four skills per role and reproducible environment (2026-10-02)

Approved roster: Frontend, Backend, Security, Product. Bundle exactly four skills for each, with full upstream references, pinned commits, licenses and recorded Claude packaging adaptations. Animate UI is Kit-authored integration guidance over official docs, not an upstream skill or copied component library. Preserve read-only Security and native worktree isolation. Set up project-scoped dependencies and shared local/cloud checks; create a Codex Cloud environment if the authorized account can select Zwin-ux/kit. Never claim native painting or model execution from loader checks. Preserve the old dirty Kit checkout and unrelated TUI ownership.

Sequence: vendor isolated role folders; update declarations/catalog/provenance; verify exactly sixteen preloads and complete packaged resources; set up tools and cloud; rerun relevant checks and independent review. No new runtime dependencies.

# Native Claude continuation preparation — 2026-10-02

Current execution plan: [Kit native continuation](../docs/superpowers/plans/2026-10-02-kit-native.md). Isolated branch `codex/kit-native-prep-20261002`, based on PR #37 at `479d53f0`. Source repairs and independent follow-up review are complete. Native visual/execution acceptance remains outstanding.

Priority: canonical macOS fixtures → safe drafts → command/status ownership → measured native UI → readiness/CI → native acceptance. The user approved yesterday's cloud lineup: Frontend, Backend, Security, Product. Preserve its existing native agent IDs and skill mapping; the earlier local lineup is superseded. Today's local baseline and reproduction commands are in the linked plan. Keep the older checkpoint below as history, not evidence that today's native QA passed.

---

# Current native Claude work (2026-10-02)

The active native Mods plan and checkpoint are in [CLAUDE-NATIVE.md](../docs/dev/CLAUDE-NATIVE.md). Implement four real specialist categories inside Claude's host, with explicit invocation, session-local status and result review. The parent-reviewed 22×16 fox grid and four-bucket layout are approved. Cloud starts from Mason checkpoint 8244c43. Implement (1) fixed-cell fox and responsive native strip/pane, (2) explicit drafts and separate task/attempt identities with session reset, (3) offline UI/behavior, type and packaging validation. Native terminal paint, live execution and full Rust gates remain separate evidence requirements. The older Control Room plan below is retained as history, not a restriction on this explicitly requested feature.

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
