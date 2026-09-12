# Kit 1.0 — Honest State (Ground Truth)

**Reviewed 2026-08-13** against code, tests, doctor, dry-run, and GitHub.  
Docs and wiki were treated as claims, not facts.

**Addendum 2026-08-16:** root `kit.toml` exists. Session A receipt `01M06A2PXBBH43ZFF3GJ9VQW94` is `gateVacuous: false`, PASS. Idle CPU measured ~0.31% of one core. `kit --demo` launches the TUI. Bare `kit` on this Windows PATH is still npm 0.1. Full remaining work: [`docs/dev/SPEC-next.md`](../SPEC-next.md). Do not treat the Aug 13 “no kit.toml / never dogfooded” lines as current.

## Summary

- Credible **alpha**. Not a 1.0. Not dogfooded.
- The Rust control room exists and the offline path works.
- The product still has **not run itself**.
- Version is `1.0.0-alpha.1`.
- Last product commit was 2026-08-07.
- Horizon 0 (dogfood) is unchecked.
- Tagging 1.0.0 now would be a lie.

## Proven Tonight

| Check                        | Result                                      | Catch                                      |
|-----------------------------|---------------------------------------------|--------------------------------------------|
| `cargo test --workspace`    | 88 pass (agents 7, cli 13, core 4, gate 6, tui 58) | No live-adapter spawn tests               |
| `kit doctor`                | codex / claude / grok / ollama all ready   | probe sets `authenticated: true` whenever the binary exists |
| `kit run --dry-run --json`  | ok, worktree cleaned, `schemaVersion: 1`   | state is `pass` while `gateVacuous` is true |
| Rust CI on last merge (#13) | ubuntu / macos / windows + startup budget green | startup measures `kit --version`, not TUI first paint |
| Git                         | origin/main = 0567ca2 (#13 filter+dispatch) | local checkout is the already-merged feature branch |

## Claimed vs Proven

| Layer                        | Docs say                          | Reality                                              |
|-----------------------------|-----------------------------------|------------------------------------------------------|
| Control Room + Detail + Dispatch + Board | Real 1.0 craft                   | Real. Filter `f` landed in #13. Board is still a prefill list. |
| Kill / retry / max-8        | Wired + proven                    | Wired. Proven only on dry-run supervisor (12 jobs). Never killed a live Codex. |
| Gate / Guardian             | Real; fixture suite green         | Firewall ~50 allow/deny strings in one test. PRD still talks about an 855-case suite. |
| Vacuous gate                | UNCONFIGURED, never PASS          | TUI label is honest. JSON dry-run still reports `state: pass` + `gatePassed: true`. |
| Dogfood Kit-on-Kit          | H0 next; checklist in DOGFOOD.md  | Zero notes. No `kit.toml` in this repo. Product does not gate itself. |
| M0 idle CPU < 1%            | Kill criterion                    | Not measured anywhere in CI.                         |
| PTY attach                  | Stub, 1.0.1                       | Honest. Attached screen exists; no PTY.              |
| Install / 1.0.0             | npm platform binaries + curl      | Install is `git clone && cargo build`. npm `@mzwin/kit` is still the 0.1 skill launcher. |

## The Real Problems

1. **Two products in one repo**  
   README opens as a Rust control room, then documents `kit unify`, `kit pack`, `kit tui workbench`, and `pnpm install`. CHANGELOG is entirely 0.1. Node CI still runs 6 jobs on every PR. Keep-alive still tries to promote the skill catalog.

2. **Docs lie by staleness**  
   wiki status-matrix (Aug 1) still says kill incomplete and 8-concurrent planned. tasks/todo.md still lists filter as unchecked after #13 merged. docs/dev/AGENTS.md still says F4 in progress. CURRENT.md (Aug 7) never mentions filter.

3. **Dispatch is not a path browser**  
   Repos are short labels seeded from cwd plus a hardcoded sibling list. Two functions duplicate that list and disagree. Filesystem probes run on every `d` keypress. Filter only binds lowercase `f`.

4. **Proof gaps the PRD treats as kill criteria**  
   Cold start CI is `kit --version`. Idle CPU has no harness. Eight concurrent runs are dry-runs, not live agents. Live adapters have no mock-binary integration test. Doctor always reports authenticated if installed (by design, but also a false ready).

## Structure That Will Hurt Later

- `crates/kit-tui/src/app.rs` — 2033 lines. Reducer, dispatch seeding, filter, kill/retry, demo fixture, and tests in one file.
- `crates/kit-gate/src/lib.rs` — 1139 lines. Whole gate + firewall in one module.

## What Is Actually Next (from the review)

| Priority | Work                                      | Why |
|----------|-------------------------------------------|-----|
| **P0**   | Dogfood session A–C from `docs/dev/DOGFOOD.md` | Without it, Kit is a demo TUI plus a CLI. The PRD's first user is you. |
| **P0**   | Add `kit.toml` to this repo               | Gate inference is the wedge. Shipping without using it is the 0.1 failure mode again. |
| **P1**   | Make README / CURRENT / todo / wiki match HEAD | Agents will implement against stale checklists. |
| **P1**   | Retire or isolate 0.1 Node CI + keep-alive | Paying 6 Node jobs and a failing catalog bot for a cut product. |
| Later    | F6 hit-testing, PTY, npm installer        | Correctly deferred. Do not start these instead of dogfood. |

---

**This document is ground truth. Treat all other docs as claims until proven.**
