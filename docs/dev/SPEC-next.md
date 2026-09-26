# Spec: What must be done for Kit to be a daily-driver tool

**Date:** 2026-08-16  
**Status:** specification (not a promise that all of this ships in one PR)  
**Product:** Kit 1.0 Control Room — `1.0.0-alpha.1`  
**North star:** a widely used, high-quality developer tool. One verb. One install story. A loop people trust. Same class as `gh` / `lazygit` / `just`.

Execution board: [`tasks/todo.md`](../../tasks/todo.md)  
Implementation order: [`tasks/plan.md`](../../tasks/plan.md)  
Ground truth (Aug 13, stale on kit.toml / dogfood): [`design-package/00-HONEST-STATE.md`](design-package/00-HONEST-STATE.md)  
QA evidence: [`dogfood-notes/2026-08-16-qa.md`](dogfood-notes/2026-08-16-qa.md)

---

## Assumptions (correct these if wrong)

1. The product is the **Control Room + gate + receipt**, not personas, not a skill marketplace.
2. Personas (product / design / eng / qa) stay as shipped: ENG default, TUI-local briefs. No expansion this horizon.
3. Frozen contracts stay Claude-only: `kit-core` run/config/gate, `kit-agents` trait, `kit-tui/src/event.rs`.
4. GitHub About / topics / latest release are **human-only**. Agents cannot do them.
5. Commit / PR only when you ask. No self-merge.
6. PTY, npm installer, Board pull-queue, vibe mode, mascot, F6, `app.rs` split-for-size are **not** this horizon unless a kill criterion forces a tiny slice.
7. First user is you on this Windows machine. Second user is a clone-from-GitHub stranger.

---

## Objective

**User:** a developer already running Codex / Claude / Grok / Ollama, drowning in terminals, who does not trust “done.”

**Job:** dispatch many agents, watch them in one place, refuse unproven work.

**Success (this horizon — “alpha you would open tomorrow”):**

- The command named `kit` on a machine that followed the README is the Rust Control Room.
- Kit-on-Kit has a non-vacuous dry-run receipt **and** one live-agent receipt.
- A human can `--demo`, see FAIL, dispatch, kill or retry, and find the receipt.
- Docs and GitHub describe 1.0, not the cut 0.1 skill launcher.
- Idle Control Room stays under 1% of one core (measured). Reduced motion works.

**Success (1.0.0 — later, PRD §10):** clean-machine install on three OSes by a stranger; cold start &lt;100ms in CI; eight **live** concurrent runs; documented receipts; 60s gate-catch demo. Do not tag 1.0.0 until those are true.

---

## Commands (how we prove anything)

```text
cargo fmt --all --check
cargo clippy -p kitctl -p kitctl-tui --all-targets -- -D warnings
cargo test --workspace
cargo test -p kitctl-tui
.\kit.cmd --version
.\kit.cmd doctor
.\kit.cmd doctor --json
.\kit.cmd --demo
.\kit.cmd run --dry-run --task "smoke" --json
.\kit.cmd run --agent <ready> --task "…"
.\kit.cmd receipt list --limit 5
.\kit.cmd receipt show <id>
. .\scripts\use-rust-kit.ps1    # session PATH → Rust kit
```

Gate (this repo): [`kit.toml`](../../kit.toml) — fmt + clippy -D warnings + `cargo test --workspace`, 15m.

---

## Already done (do not redo)

| ID | What | Evidence |
|----|------|----------|
| D1 | Rust workspace + Control Room / Detail / Dispatch / Board | snapshots, `cargo test -p kitctl-tui` (75) |
| D2 | Filter `f`/`F`, help `?`, kill/retry wired | #13 + later TUI work |
| D3 | Root `kit.toml` + engine tests on `bare_git_fixture()` | `workspace_kit_toml_declares_a_real_gate` |
| D4 | Session A Kit-on-Kit dry-run | receipt `01M06A2PXBBH43ZFF3GJ9VQW94`, `gateVacuous: false`, PASS |
| D5 | Repo shims `kit.cmd` / `kit.ps1` → Rust binary | `.\kit.cmd --version` → `1.0.0-alpha.1` |
| D6 | Doctor path + PATH collision warning | `pathCollisions` in JSON |
| D7 | `kit --demo` is a TUI launch (was `unknown command`) | `demo_flag_is_tui_not_unknown_command` |
| D8 | Idle CPU measured | 0.31% of one core empty room; 0.31% demo |
| D9 | Motion-off + RUNNING does not dirty | `motion_off_running_row_does_not_dirty_on_tick` |
| D10 | Personas on Dispatch (ENG default) | TUI-local; do not expand |
| D11 | README 1.0-first; design package in-repo | uncommitted |

---

## Work that should be done

Each item: **why**, **acceptance**, **owner**, **depends**.  
Owner: **You** = human. **Power** = Grok/`kit-tui`. **Factory** = Codex/engine/CI. **CEO** = Claude/contracts/merge.

### Horizon 0 — Identity and land (blocks “widely used”)

| ID | Work | Why | Acceptance | Owner | Depends |
|----|------|-----|------------|-------|---------|
| I1 | GitHub About + topics + pin on v0.1.4 | Strangers meet 0.1 first | About = Control Room one-liner; topics rust/cli/tui; 0.1.4 release notes “not 1.0” | You | — |
| I2 | Commit + PR uncommitted branch | 9 days of product work is only on disk | PR from `feat/1.0-dogfood-kit-toml`; CI green; no self-merge | Power + You | — |
| I3 | Session B: one live `kit run --agent <ready>` | Dry-run is not the product | Receipt id; worktree + gate + stream; note in dogfood 2026-08-16 | Power | I2 optional |
| I4 | Session C: TUI `--demo` then live dispatch + `k` or `r` | The 30-second loop | Notes: FAIL visible; fan-out appeared; kill or retry once | You + Power | I3 |
| I5 | Session D notes complete | Dogfood kill criterion | Table in `DOGFOOD.md` filled; “open tomorrow?” answered | You | I4 |
| I6 | Honest PATH story in README/CURRENT | Dual `kit` is the #1 adoption bug | README 30s leads with `cargo build` + shim; never bare `kit` without warning | Power | — |
| I7 | Update `00-HONEST-STATE.md` + `DOGFOOD.md` checkboxes | Those files still say no kit.toml / no Session A | One honest addendum dated 2026-08-16 | Power | — |

### Horizon 1 — Proof gaps the PRD treats as kill criteria

| ID | Work | Why | Acceptance | Owner | Depends |
|----|------|-----|------------|-------|---------|
| P1 | Isolate `CARGO_TARGET_DIR` (and similar) in run/gate worktrees | Kit-on-Kit gate compiled HEAD into the agent target dir; workspace tests then lied | Gate/run child env unsets or sets a worktree-local `CARGO_TARGET_DIR`; test that parent target is not written | Factory | I2 |
| P2 | Cold start in CI is TUI-relevant, not only `--version` | M0 kill is &lt;100ms first paint | CI job records `kit --version` **and** a bounded startup of `kit doctor` or a headless first-frame; number published | Factory | I2 |
| P3 | Idle CPU harness (even Windows-only script) | Measured once by hand; CI still blind | Script or test doc in `docs/dev/`; fail if empty-room sample ≥1% of one core over 15s | Factory | — |
| P4 | Doctor: `ready` ≠ “binary exists” | `authenticated: true` if the CLI is on PATH is a false ready | Doctor distinguishes `installed` vs `ready`; JSON fields named; **ask CEO** if this touches a frozen shape | CEO + Factory | — |
| P5 | Live kill once | Kill is proven only on dry-run handles | One live RUNNING job `k`’d; receipt `killed`; note | Power | I3 |
| P6 | JSON honesty when gate is vacuous | Dry-run without kit.toml still `state: pass` + `gateVacuous: true` | Document the pair; TUI already says UNCONFIGURED; do **not** change `state` without CEO | CEO | — |
| P7 | Adapter mock-binary tests | Live adapters have no fake-CLI integration test | One mock on PATH; spawn + stream + exit; no network | Factory | — |

### Horizon 2 — Stop paying for the cut product

| ID | Work | Why | Acceptance | Owner | Depends |
|----|------|-----|------------|-------|---------|
| N1 | Path-filter or skip Node CI on Rust-only PRs | 6 Node jobs + failing keep-alive on every PR | `.github` only runs Node jobs when `packages/` changes; keep-alive disabled or isolated | Factory | I2 |
| N2 | CHANGELOG 1.0 section | CHANGELOG is entirely 0.1 | `1.0.0-alpha.1` entry: Control Room, gate, receipt; 0.1 collapsed | Power | I2 |
| N3 | Decide npm `@mzwin/kit` bin name | `npm i -g` owns `kit` | You pick: deprecate npm bin, rename to `@mzwin/kit-legacy`, or stop publishing 0.1 | You | I1 |
| N4 | Linguist / `packages/` dominance | GitHub says TypeScript | `linguist-vendored` or archive `packages/` after N1; do not delete without You | You + Factory | N1 |

### Horizon 3 — TUI excellence (existing screens only)

| ID | Work | Why | Acceptance | Owner | Depends |
|----|------|-----|------------|-------|---------|
| U1 | FAIL first-error line at 80 and 60 cols | Wedge is “see why without opening” | Snapshot: `^ tsc: 3 errors` readable at 80×14 and 60×12 | Power | — |
| U2 | Dispatch footer includes `↑↓` | CR says select; Dispatch omits move | Same footer grammar family | Power | — |
| U3 | Dispatch density | 3 equal columns leave empty wells | Repos column not a vacant lot when one repo; no new features | Power | — |
| U4 | One voice `vendor·role` everywhere | Already mostly done | Board + CR + detail match; no `eng/codex` reversal | Power | — |
| U5 | Empty / too-small / Board copy | First paint must teach the next key | Empty room + empty Board + too-small snapshots | Power | — |
| U6 | F5 light / high-contrast | Partial | `KIT_THEME=high` documented + snapshot; no mascot | Power | — |
| U7 | Personas: no expansion | Product said third axis is 0.1 smell | ENG default stays; no QA-as-Gate replacement this horizon | Power | — |

### Horizon 4 — Ship (after B+C, not instead)

| ID | Work | Why | Acceptance | Owner | Depends |
|----|------|-----|------------|-------|---------|
| S1 | Tag `v1.0.0-alpha.2` | Signal that dogfood happened | Sessions B–D done; I1 done; I2 merged | You + CEO | I5, I1, I2 |
| S2 | 60s demo (gate catch) | PRD §10 | Recording or GIF: FAIL → retry or blocked ship | You | I4 |
| S3 | npm / cargo / curl installer | M5 | Clean-machine install on 3 OSes by someone who is not you | Factory | S1, N3 |
| S4 | Tag `v1.0.0` | Only when PRD §10 is true | Third-party README walk; idle+cold-start numbers; receipt schema published | CEO | S3, P2, P3 |

### Parked (specified so they are not forgotten — do not start)

| ID | Work | When |
|----|------|------|
| Z1 | PTY attach | 1.0.1 |
| Z2 | Board pull-queue | 1.1 |
| Z3 | Recommended vs vibe + mode on receipts | 1.1; CEO (contracts) |
| Z4 | F6 hit-testing from drawn rects | after U* if mouse matters |
| Z5 | `app.rs` / `kit-gate` split | only if a slice is blocked by file size |
| Z6 | Guardian 855-case expansion | not instead of dogfood; ~50 strings is the honest suite |
| Z7 | Dispatch filesystem path browser | 1.1; abs paths already stored |
| Z8 | Marketplace / registry / accounts | cut for 1.0 |
| Z9 | Animated mascot | never as the product |
| Z10 | Live 8-agent concurrency harness | after P5; dry-run 8 already proven |

---

## Tech stack / structure

```text
crates/kit-core     contracts (CEO)
crates/kit-agents   adapters + skills inject (Factory)
crates/kit-gate     gate + firewall (Factory)
crates/kit-tui      Control Room (Power) — event.rs CEO
crates/kit-cli      CLI + engine supervisor (Factory)
kit.toml            this repo's gate
docs/dev/           PRD, CURRENT, specs, dogfood notes
tasks/              plan.md + todo.md (this horizon)
packages/           0.1 Node — isolate, do not feature
```

## Testing strategy

- Reducer + insta for TUI (`cargo test -p kitctl-tui`).
- Engine tests **must** use `bare_git_fixture()`, never this repo’s `kit.toml`.
- Dogfood proof = receipt ids in `docs/dev/dogfood-notes/`.
- Clippy `-D warnings` on `kit-cli` + `kit-tui`.
- Do not shrink `kit.toml` to make a test faster.

## Boundaries

- **Always:** evidence (test, receipt, doctor, CPU sample); ↑↓ move, `k` kills; reduced motion.
- **Ask first:** contract shapes, GitHub Settings, npm unpublish/rename, deleting `packages/`, commit/PR.
- **Never:** PTY/installer/marketplace this horizon; edit `event.rs` as Power; weaken the gate; invent blink/mascot; type bare `kit` for dogfood on this machine.

## Open questions (human)

1. npm `@mzwin/kit`: deprecate the `kit` bin, rename, or leave and warn forever? (N3)
2. Doctor `ready` vs `installed` — change JSON? (P4, CEO)
3. Vacuous JSON `state: pass` — document only or change? (P6, CEO)
4. When to commit I2?

## Checkpoint: “open tomorrow”

All of I1–I7 and P1. Then S1 is allowed. Not before.
