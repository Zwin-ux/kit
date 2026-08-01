# Kit 1.0 — Build Assignment

Companion to `PRD-1.0.md`. Who builds what, where the seams are, and how work merges.

## Roster (verified on this machine)

| Agent | Version | Headless invocation |
|---|---|---|
| **Claude Code** | 2.1.220 | orchestrator — runs interactively |
| **Codex** | 0.145.0 | `codex exec -C <dir> "<task>"` · `codex review` · `--json` · `-o <file>` |
| **Grok Build** | 0.2.117 | `grok -p "<task>" --cwd <dir>` · native `--worktree=<name>` · `--always-approve` |
| **Ollama** | 0.32.5 | local, non-critical work only |

Toolchain: rustc/cargo 1.97.1, node 24.18.0, gh 2.88.1, pnpm at `AppData\Roaming\npm\pnpm.cmd`.

---

## The rule that makes parallel agent work survivable

**Contracts first, then fan out.**

Two agents editing the same Rust workspace will collide on types, not on files. So Claude writes five type-only files *before any agent starts* — roughly 400 lines, no implementation:

| File | Freezes |
|---|---|
| `kit-core/src/run.rs` | `Run`, `RunState`, `Receipt` (serde) |
| `kit-core/src/config.rs` | `kit.toml` schema |
| `kit-agents/src/lib.rs` | `Agent` trait — the adapter seam |
| `kit-gate/src/lib.rs` | `Gate` trait, `GateOutcome` |
| `kit-tui/src/event.rs` | `AppEvent` enum + reducer signature |

Once these compile, Grok and Codex work in parallel with zero coordination. **Changing one of these five files is a Claude-only operation** — an agent that wants a contract changed files an issue, it does not edit.

---

## Ownership map

### Backend — the engine

| # | Corner | Owner | Why |
|---|---|---|---|
| B1 | `kit-core` — Run model, worktree lifecycle, receipts, store | **Codex** | Mechanical port of existing TS logic; spec visible on both sides |
| B2 | `kit-agents` — 4 adapters (codex/claude/grok/ollama) | **Codex** | Repetitive impls against a frozen trait |
| B2-pty | PTY supervision + bounded streaming | **Grok** | Terminal-native Rust; the hardest cross-platform surface |
| B3 | `kit-gate` — Guardian JS→Rust + firewall + fixtures | **Codex** | 700 lines of JS with a passing fixture suite as the oracle |
| B4 | `kit-cli` — headless commands, `--json` parity | **Codex** | Contract already defined by PR #7 |

### Frontend — the surface

| # | Corner | Owner | Why |
|---|---|---|---|
| F1 | Event loop, frame clock, `tokio::select!` merge | **Grok** | The architectural spine — Rust-idiomatic async |
| F2 | Control Room screen + live table | **Grok** | |
| F3 | Run detail — stream, diff, gate log, attach/detach | **Grok** | |
| F4 | Dispatch + Board screens | **Grok** | |
| F5 | Theme, motion, mascot port, reduced-motion | **Grok** | |
| F6 | Layout + hit-testing from one rect source | **Grok** | |

**The entire TUI goes to one agent deliberately.** Kit 0.1 is 4/10 largely because its interaction language is inconsistent — three competing front doors, uneven feedback. Splitting the surface across agents reproduces exactly that failure. One owner, one voice.

### Cross-cutting

| # | Corner | Owner |
|---|---|---|
| X1 | 3-OS CI from M0 (Windows included) | **Codex** |
| X2 | Distribution — npm platform packages, cargo, curl installer | **Codex** (port the trenchwire pattern) |
| X3 | `insta` snapshot harness | **Codex** |
| X4 | Docs + demo recording | **Claude**, Ollama drafts |
| X5 | The five contracts, integration, every merge review | **Claude** |

---

## Per-agent briefs

### Grok Build — owns the surface

**Reference implementation:** `C:\Users\mzwin\Documents\fennec\crates\fennec-tui`. It already runs ratatui 0.30 + crossterm 0.29 with a single `AppEvent::AnimationTick` and 28 insta snapshots. Follow that architecture; do not invent a new one.

**Hard boundaries:**
- Never edit `crates/kit-core`, `kit-gate`, or `kit-cli`
- Never own a timer outside the event loop — one clock, no exceptions
- Hit-testing reads the same ratatui rects the renderer drew; no parallel geometry

Grok's native `--worktree=<name>` gives it isolation for free — use it.

### Codex — owns the engine

**Ports have an oracle on both sides.** Guardian's fixture suite is the acceptance test for `kit-gate`; PR #7's E2E harness is the acceptance test for `kit-cli`. A port is done when the existing tests pass against the Rust implementation — not when it looks right.

**Hard boundaries:**
- Never edit `crates/kit-tui`
- Never change the five contract files — file an issue instead
- Every crate ships with its tests in the same PR

### Claude — owns the seams

Contracts, crate boundaries, gate semantics, receipt format, integration, review, ship. Reviews every PR before merge. Does not write screen code or port files — that's the point of delegating.

### Ollama — drafts only

Changelog drafts, doc first passes, scratch. Nothing on the critical path.

---

## M0 dispatch

Sequenced because F1 is the spine — everything else waits on it.

**Step 0 — Claude, before anyone starts**
```
archive the Ink work    → branch archive/ink-tui-final, pushed
land PR #7              → contract + E2E oracle in main
scaffold workspace      → 5 crates, Cargo.toml, the 5 contract files
```

**Step 1 — parallel, once contracts compile**

Grok, the event loop:
```
grok -p "Implement the kit-tui event loop per docs/dev/PRD-1.0.md section 5.2. \
Single tokio::select! merging crossterm EventStream, one animation interval, and \
run updates. Motion is a pure function of tick count. Reference \
C:/Users/mzwin/Documents/fennec/crates/fennec-tui/src/app.rs. Do not edit crates \
outside kit-tui. Add insta snapshots." --cwd C:/Users/mzwin/kit --worktree=m0-eventloop
```

Codex, the gate port:
```
codex exec -C C:/Users/mzwin/kit "Port C:/Users/mzwin/grok-build-guardian/hooks/guard.js \
(546 lines) and hooks/done-gate.js (154 lines) to crates/kit-gate in Rust, against the \
frozen Gate trait. tests/firewall.test.js is the acceptance oracle — every fixture must \
pass. Do not edit crates/kit-tui or the five contract files."
```

Codex, CI:
```
codex exec -C C:/Users/mzwin/kit "Add 3-OS GitHub Actions CI for the Rust workspace: \
ubuntu/macos/windows, fmt + clippy -D warnings + test, fail-fast off. Windows is \
required, not optional."
```

**Step 2 — M0 kill criteria, measured by Claude**

Cold start < 100ms, idle CPU < 1%, snapshots green on all three OSes. Miss these and we stop and reconsider the stack — that is the entire point of a kill criterion.

---

## Merge gate

No agent merges its own work.

1. Agent opens a PR from its worktree
2. `codex review` — independent adversarial pass
3. Claude reviews the diff, focusing on contract boundaries
4. CI green on all three OSes
5. Claude merges

From M3 onward Kit gates its own development — the dogfood loop closes and every agent run goes through `kit-gate` before it can claim done.

---

## gstack skills that apply

Most of the gstack suite is web-shaped and does not fit a terminal product. Honest split:

**Use:**
- `/codex` — independent review and adversarial challenge on every crate merge
- `/review` — pre-landing diff review
- `/freeze` — scope edits to one crate per agent lane; the cleanest guard against parallel-agent collisions
- `/guard` — during PTY and worktree work, where `git worktree remove` and process kills are genuinely destructive
- `/investigate` — root-cause discipline if M0 misses its numbers
- `/ship` — VERSION, CHANGELOG, PR at each milestone
- `/simplify` — post-merge cleanup
- `/retro` — weekly
- `/document-release` — after 1.0

**Skip — these assume a website:**
`/browse`, `/qa`, `/qa-only`, `/design-review`, `/canary`, `/benchmark`, `/setup-deploy`, `/land-and-deploy`, `/setup-browser-cookies`, `/use-railway`, `/supabase`

**Gap:** nothing in gstack measures TUI startup time or frame stability. M0's kill criteria need a small custom bench — Claude writes it, roughly 40 lines using `hyperfine` plus a frame-timing harness.
