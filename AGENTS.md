# Kit — Agent Context

**Product (1.0):** Kit is the **control room for parallel agent work**.  
Dispatch many agents. Watch them in one place. Nothing ships unproven.

**Not the product:** skill marketplace, registry social, Ink TUI (archived).  
Skills remain local substrate — what you dispatch, not the point.

Full PRD: `docs/dev/PRD-1.0.md`  
Who builds what: `docs/dev/BUILD-ASSIGNMENT.md`  
Active surface tasks: `tasks/plan.md`, `tasks/todo.md`

---

## Multi-agent roles (CEO / Power / Luna)

See **`docs/dev/ORCHESTRATION.md`**.

| Role | Who | Job |
|------|-----|-----|
| **CEO** | Claude | Sequence, kill criteria, contract edits, merge, product policy |
| **Power** | Grok | Heavy implementation, TUI/engine wiring, spawn Luna threads |
| **Factory** | Codex | Mechanical ports, fixtures, CI scaffolding |
| **Luna** | Many short agents | One-shard diagnose/design/verify (parallel token burn) |

**Luna storm:** parallel explore/implement shards; Power synthesizes; CEO decides/merges.  
Workflow sketch: `.grok/workflows/kit-luna-storm.rhai` (local; `.grok` may be gitignored).  
**Quality ladder (long-running):** `docs/dev/WORKFLOW-QUALITY.md` + workflow `kit-quality-cycle`  
  (audit → skeptic → plan → implement → prove). Git copy: `docs/dev/workflows/kit-quality-cycle.rhai`.  
CEO brief template: `docs/dev/tasks/CEO-BRIEF-next.md`.

## Engineering skills (always)

This repo installs **[addyosmani/agent-skills](https://github.com/addyosmani/agent-skills)** under `.agents/skills/` (24 skills).

**Before non-trivial work**, load `using-agent-skills` and route:

| Phase | Skill | Kit use |
|-------|--------|---------|
| Define | `spec-driven-development` | New surface / engine milestone |
| Plan | `planning-and-task-breakdown` | Write `tasks/plan.md` + `tasks/todo.md` first |
| Build | `incremental-implementation` | Thin slices; test after each; commit |
| Build | `test-driven-development` | Reducer + insta snapshots for TUI |
| Build | `frontend-ui-engineering` | TUI interaction language, a11y, fixed geometry |
| Build | `api-and-interface-design` | Frozen contracts only; never edit contracts as Grok |
| Build | `doubt-driven-development` | PTY, gate semantics, cross-crate seams |
| Verify | `completeness-qa` | After implement, before claiming done. Run `node skills/completeness-qa/scripts/inventory.mjs --root .` — stubs mean not done |
| Verify | `debugging-and-error-recovery` | Repro → localize → fix → guard |
| Review | `code-review-and-quality` | Before every PR |
| Review | `code-simplification` | After green, before ship |
| Review | `security-and-hardening` | Firewall, scope, no credential custody |
| Review | `performance-optimization` | M0 kill: cold start &lt;100ms, idle CPU &lt;1% |
| Ship | `git-workflow-and-versioning` | Atomic commits, ~100-line preference |
| Ship | `documentation-and-adrs` | ADRs in `docs/adr/` for boundary decisions |

### Core operating rules (from using-agent-skills)

1. **Surface assumptions** before implementing.
2. **Stop on confusion** — do not guess contracts.
3. **Push back** on approaches that break kill criteria or crate boundaries.
4. **Simplicity first** — boring over clever.
5. **Scope discipline** — only touch the task's crate/files.
6. **Verify with evidence** — `cargo test` / clippy / snapshots; "looks right" is never done.

---

## Ownership (do not cross)

| Agent | Owns | Must not edit |
|-------|------|----------------|
| **Grok** | Entire `kit-tui` surface (F1–F6), B2-pty later | `kit-core` contracts, `kit-gate`, `kit-cli`, `event.rs` |
| **Codex** | Engine: core port, agents, gate, CLI, CI | `kit-tui` |
| **Claude** | Five contract files, merges, integration | Does not own screen code |

**Contract files (Claude-only):**  
`kit-core` run/config/gate, `kit-agents` trait, `kit-tui/src/event.rs`

**TUI non-negotiables:**

- One clock: only `AppEvent::AnimationTick` (no other timers)
- Hit-testing from the same ratatui rects the renderer drew (F6)
- Navigation mutates `Screen` in the reducer; engine work is `Action`
- Reduced motion: `NO_COLOR`, `KIT_MOTION=off`

Reference architecture: `C:/Users/mzwin/Documents/fennec/crates/fennec-tui`

---

## Commands

```text
# Run the product
cargo run -p kit-cli -- --demo    # Control Room + PRD fixture
cargo run -p kit-cli              # empty Control Room
cargo run -p kit-cli -- doctor

# Verify
cargo test -p kit-tui
cargo clippy -p kit-cli -p kit-tui --all-targets -- -D warnings
cargo fmt --all --check
cargo test --workspace
```

Living status: `docs/dev/CURRENT.md`

Rust workspace: `crates/kit-{core,agents,gate,tui,cli}`  
Legacy Node packages still exist under `packages/` (0.1.x); 1.0 is the Rust binary.

---

## Milestone map (surface focus)

| Done | In flight / next |
|------|------------------|
| M0 event loop (F1) | F4 Dispatch + Board |
| F2 Control Room table | F5 Theme / motion / mascot |
| F3 Run detail | F6 Layout + hit-testing |
| M3 gate port (Codex) | M1 engine (Codex) enables live runs |

---

## Visual / interaction language

- Control Room is the default (`kit` → table of runs)
- One interaction voice across screens (same footer grammar, Esc back)
- Failures surface first error line in the table; detail holds full gate log
- Attach: Esc detaches without kill; `q` disabled while attached
