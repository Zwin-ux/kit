# Kit execution board

Spec (all items + parked): [`docs/dev/SPEC-next.md`](../docs/dev/SPEC-next.md)  
Plan (order + task cards): [`tasks/plan.md`](plan.md)

## Done (do not redo)

- [x] Control Room / Detail / Dispatch / Board + filter + help
- [x] `kit.toml` + bare-git engine fixtures
- [x] Session A — receipt `01M06A2PXBBH43ZFF3GJ9VQW94`, `gateVacuous: false`
- [x] Shims + doctor PATH warning + `use-rust-kit.ps1`
- [x] `kit --demo` launches TUI (was unknown command)
- [x] Idle CPU ~0.31% of one core; motion-off RUNNING test
- [x] Personas (ENG default) — **do not expand**
- [x] QA note `docs/dev/dogfood-notes/2026-08-16-qa.md`

## Horizon 0 — Identity and land (now)

- [ ] **I1** GitHub About + topics + v0.1.4 “not 1.0” note *(human)*
- [ ] **I2** Commit + PR when asked (no self-merge)
- [ ] **I3** Session B — live `kit run --agent <ready>` + receipt id
- [ ] **I4** Session C — `.\kit.cmd --demo` then dispatch; `k` or `r` once
- [ ] **I5** Session D — dogfood table filled; “open tomorrow?”
- [x] **I6** README/CURRENT never tell a stranger to type bare `kit` on Windows
- [x] **I7** Honest-state addendum + DOGFOOD Session A checkbox (2026-08-16)

## Horizon 1 — Proof

- [x] **P1** Isolate `CARGO_TARGET_DIR` in run/gate worktrees
- [ ] **P2** CI startup beyond `kit --version` *(Factory)*
- [ ] **P3** Repeatable idle-CPU script
- [ ] **P4** Doctor `installed` vs `ready` *(ask CEO — JSON)*
- [ ] **P5** Live kill once + receipt `killed`
- [ ] **P6** Vacuous JSON `state: pass` — document or CEO change
- [ ] **P7** Mock-binary adapter test

## Horizon 2 — Stop paying for 0.1

- [x] **N1** Path-filter Node CI / disable keep-alive
- [ ] **N2** CHANGELOG `1.0.0-alpha.1` section
- [ ] **N3** npm `@mzwin/kit` bin decision *(human)*
- [ ] **N4** Linguist / `packages/` vendored or archived *(ask first)*

## Horizon 3 — TUI (one slice at a time)

- [x] **U1** FAIL annotation readable at 80 and 60
- [x] **U2** Dispatch footer includes `↑↓`
- [x] **U3** Dispatch column density
- [x] **U4** `vendor·role` voice everywhere
- [x] **U5** Empty / too-small / Board copy
- [x] **U6** `KIT_THEME=high` documented + snapshot
- [x] **U7** Personas frozen (ENG default)

## Horizon 4 — Ship (after Horizon 0 checkpoint)

- [ ] **S1** Tag `v1.0.0-alpha.2`
- [ ] **S2** 60s gate-catch demo
- [ ] **S3** Installer (npm/cargo/curl)
- [ ] **S4** Tag `v1.0.0` only when PRD §10 is true

## Parked

- [ ] Z1 PTY attach (1.0.1)
- [ ] Z2 Board pull-queue (1.1)
- [ ] Z3 Recommended vs vibe (1.1, CEO)
- [ ] Z4 F6 hit-testing
- [ ] Z5 `app.rs` / `kit-gate` split
- [ ] Z6 Guardian 855-case expansion
- [ ] Z7 Dispatch path browser
- [ ] Z8 Marketplace
- [ ] Z9 Mascot-as-product
- [ ] Z10 Live 8-agent harness

## Non-goals

- Skill marketplace / registry / accounts
- Weakening `kit.toml`
- Editing `event.rs` as Power
