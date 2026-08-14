---
name: completeness-qa
description: "Inventory public functions, flag stubs and untested symbols, and name the next skills that raise SWE quality for this change. Use after implement, before claiming done, or when asked to QA, find missing tests, or catch half-done functions."
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Prefer the git diff. If there is no diff, scan `src/`, `lib/`, `crates/`, `app/`.
2. Run `node scripts/inventory.mjs` from this skill folder (`--root` = project). If Node is missing, follow `references/scan-rules.md` by hand.
3. Do not invent symbols the script did not list.
4. Rank “test next” using `references/swe-quality-ladder.md`. Print at most five rows. Never drop a `stub`.
5. Name at most three follow-on catalog skills, each with a reason from the findings. Do not recommend skills outside the ladder file.
6. Print the script report. Keep `.kit/completeness.json` when the project is writable.
7. Do not write production code or tests unless the user asked. Point at `write-tests` or `fix-bug`.
