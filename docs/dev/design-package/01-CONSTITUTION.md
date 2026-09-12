# Kit Project Constitution (Reality-Adjusted)

**Last updated:** 2026-08-13  
**Source of truth:** Honest review + design synthesis

## Ground Truth

- This is a credible **alpha** (`1.0.0-alpha.1`), not 1.0.
- The product has **never dogfooded itself**.
- Docs and wiki are claims. Code + tests + doctor + dry-run are facts.
- No `kit.toml` currently exists in the repo → gates on Kit itself are weak.
- Two products still coexist (Rust control room + legacy Node).
- Live kill / live concurrency / real Guardian suite / TUI cold-start / idle CPU are not proven.

## Settled Product Direction

These remain valid and should not be reopened lightly:

1. **Engine owns** run lifecycle, worktrees, gates, receipts, and concurrency.
2. **TUI is a reactive client** — the emotional / acquisition surface (live operations board).
3. **Every run** is isolated (worktree) + gated + produces a receipt.
4. **Recommended path** uses high-signal SWE skills.
5. **Vibe mode** is an explicit lower-proof escape hatch (never the advertised default).
6. **In-process Engine** is correct for this phase. Design the command/event boundary so a later local daemon is possible without rewriting the TUI.
7. **Local-first**, no credential custody.
8. **Primary quality metric** = the real multi-run loop:  
   `dispatch → truthful live state → kill/retry with context → receipt`

## Non-Negotiables

- Gate always runs.
- Receipts are structured and useful.
- TUI density, keyboard feel, and FAIL visibility are first-class.
- Headless (`--json`) behavior must stay semantically aligned with the TUI.
- Prefer dogfood + truth + `kit.toml` over new features right now.

## Pragmatic for Current Phase

- No background daemon yet.
- No multi-language client generation required yet.
- No PTY, npm platform installer, or marketplace work.
- Prefer small, evidence-producing, reviewable changes.
- Do not treat stale wiki / CURRENT / todo items as fact.

## Immediate Rules for Agents

1. Start from the honest review (00-HONEST-STATE.md).
2. Prefer making Kit able to gate and dogfood itself before polishing.
3. After every meaningful change: run what you can, produce evidence (tests, receipts, doctor output), and critique against the honest state.
4. Keep the 30-second / multi-run loop as the north star for TUI work.
