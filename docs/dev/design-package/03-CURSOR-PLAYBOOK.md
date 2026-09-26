# Cursor Ultra + Grok 4.6 Playbook for Kit

**Optimized for:** long context, multi-file reasoning, terminal use, sequential phases with verification.

## How to Use This Package

1. Put this entire `kit-design-package/` folder (or its contents) into context.
2. Start every session by forcing the agent to read `00-HONEST-STATE.md` and `01-CONSTITUTION.md`.
3. Follow the phased plan below. Do not skip ahead.
4. After every phase the agent must produce evidence and run the self-critique rubric.

---

## Project Constitution (short form — always keep in context)

```text
GROUND TRUTH (2026-08-13):
- Credible alpha only (1.0.0-alpha.1). Never dogfooded.
- Docs/wiki are claims. Code + tests + doctor + dry-run are facts.
- No kit.toml in the repo yet.
- Two products still in the tree (Rust + legacy Node).
- Live proof gaps exist.

SETTLED DIRECTION:
- Engine owns run lifecycle, worktrees, gates, receipts.
- TUI is a reactive client (live operations board).
- Every run = isolation + gate + receipt.
- Recommended = high-signal SWE skills; vibe = explicit lower-proof escape hatch.
- Primary metric = multi-run loop quality (dispatch → live state → kill/retry with context → receipt).

IMMEDIATE RULES:
- Prefer dogfood + truth + kit.toml over new features.
- Do not start PTY, npm installer, or marketplace work.
- Prefer small, evidence-producing changes.
- After every phase: show evidence and self-critique.
```

---

## Self-Critique Rubric (mandatory after every phase)

The agent must answer explicitly:

1. Does this improve the real multi-run loop or the honesty of the product?
2. Is the Engine still the intended source of truth?
3. Did I reopen any settled decision or ignore the honest review?
4. Is the change incremental and reviewable?
5. What evidence did I produce (tests, receipts, doctor output, before/after)?
6. What is the biggest remaining gap vs the honest state?

---

## Phased Attack Plan

### Phase 0 — Orient (mandatory first)

- Read `00-HONEST-STATE.md`, `01-CONSTITUTION.md`, `02-TUI-WORKFLOW.md`.
- Explore the actual repo structure (especially `crates/kit-tui`, `crates/kit-core`, `crates/kit-gate`, `crates/kit-cli`).
- Run `cargo test --workspace`, `kit doctor`, and a dry-run if possible.
- Report current state vs the honest review.
- **No large edits yet.**

### Phase 1 — Dogfood + kit.toml (P0)

Highest priority.

1. Execute dogfood sessions from `docs/dev/DOGFOOD.md` (or create them if missing).
2. Use Kit on the Kit repository itself.
3. Produce real receipts.
4. Add a meaningful `kit.toml` to this repository so the gate is non-vacuous.
5. Show evidence that the product can now gate itself.

**Done when:** There are real dogfood receipts and a `kit.toml` that makes the gate real on this repo.

### Phase 2 — Truth in Docs (P0/P1)

- Update README, CURRENT.md, tasks/todo.md, and relevant wiki pages so they match actual HEAD.
- Stop the lie. Make the status matrix and checklists reflect what is proven vs claimed.

**Done when:** A new agent reading the docs will not be misled about the current state.

### Phase 3 — TUI Excellence on What Exists (P1)

Improve the **real** Control Room toward the intent in `02-TUI-WORKFLOW.md`:

- FAIL wash + first error visibility
- Keyboard grammar consistency
- Retry carries previous gate failure context
- Density and selection clarity
- Narrow terminal behavior
- Footer / help accuracy
- Filter improvements if still weak

**Done when:** The 30-second / multi-run loop feels clearly better and the emotional targets in the workflow document are closer.

### Phase 4 — Ownership & Structure

- Clarify Engine vs TUI ownership.
- Start reducing the pressure on the 2033-line `app.rs` if it is blocking progress.
- Keep everything in-process.

### Phase 5 — Only After the Above

- Skill posture hardening (recommended vs vibe)
- Deeper protocol cleanup
- Live adapter proof
- Isolation of remaining 0.1 Node machinery
- Anything else from the earlier design work

---

## Drop-Ready Master Prompt

Paste this (or adapt it) when starting a Cursor session:

```text
You are working on the Kit repository.

First read and internalize:
- 00-HONEST-STATE.md (ground truth)
- 01-CONSTITUTION.md
- 02-TUI-WORKFLOW.md (the human intent for the Control Room)
- 03-CURSOR-PLAYBOOK.md

Treat the 2026-08-13 honest review as ground truth. Docs and wiki are claims. Code, tests, doctor, and dry-run are facts.

Current state: credible alpha (1.0.0-alpha.1). Never dogfooded. No kit.toml in the repo. Stale docs. Two products still in the tree. Live proof gaps exist.

Follow the phased attack plan in 03-CURSOR-PLAYBOOK.md strictly.
Start with Phase 0 (Orient). Report current state vs the honest review. Do not make large changes yet.

After every phase: produce evidence and run the self-critique rubric.

Constraints:
- Prefer dogfood + truth + kit.toml over new features.
- Do not start PTY, npm platform installer, or marketplace work.
- Prefer small, reviewable, evidence-producing changes.
- Use the terminal. Be critical of your own work.
```

---

## Priority Order (never forget)

1. Dogfood + `kit.toml` (make it gate itself)
2. Truthful docs
3. TUI excellence on the real Control Room
4. Ownership / structure cleanup
5. Everything else
