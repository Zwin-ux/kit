# Completeness QA Implementation Plan

> **For agentic workers:** Execute this plan task-by-task. Spec: `docs/dev/specs/2026-08-14-completeness-qa-design.md`.

**Goal:** Ship the `completeness-qa` catalog skill: inventory public functions, flag stubs, name a short SWE skill ladder, present a one-screen report.

**Architecture:** Zero-dep `scripts/inventory.mjs` is the source of truth. `SKILL.md` tells agents when to run it and how to rank. Essentials 0.2.0 installs it. No `kit-gate` changes.

**Tech Stack:** Node 18+ `node:test`, existing Kit skill schema v0, vitest only for the tiny `recommend.ts` bump.

## Global Constraints

- No `crates/kit-gate` edits.
- Skill `compatibility`: `claude-code`, `grok-build`, `codex` only.
- Description: one or two sentences (validator).
- Script exit code always `0`.
- Verdict `done` only if `public > 0` and `stub == 0` and `untested == 0`.
- Report: verdict first, ≤5 test-next rows printed, ≤4 ladder rows.
- JSON version `1` as specified.

## Files

- Create: `skills/completeness-qa/SKILL.md`
- Create: `skills/completeness-qa/scripts/inventory.mjs`
- Create: `skills/completeness-qa/references/scan-rules.md`
- Create: `skills/completeness-qa/references/swe-quality-ladder.md`
- Create: `skills/completeness-qa/tests/inventory.test.mjs`
- Create: `skills/completeness-qa/tests/fixtures/stub-ts/**`
- Create: `skills/completeness-qa/tests/fixtures/clean-rs/**`
- Create: `skills/completeness-qa/tests/fixtures/empty-ts/**`
- Modify: `packs/essentials/PACK.md` (add skill, version 0.2.0)
- Modify: `skills/write-tests/SKILL.md` (one completeness.json line)
- Modify: `packages/core/src/recommend/recommend.ts` (baseline bumpSkill + skillFromPack)
- Modify: `packages/core/tests/recommend.test.ts` (assert completeness-qa listed)
- Modify: `skills/README.md` (catalog row)
- Modify: `docs/packs.md` official table row for essentials

### Task 1: Inventory script + fixtures + tests

- [x] Write fixtures and `node:test` cases from spec §11
- [x] Implement `inventory.mjs` CLI and scan/classify/report
- [x] `node skills/completeness-qa/tests/inventory.test.mjs` green

### Task 2: Skill pack wiring

- [x] SKILL.md + references
- [x] essentials 0.2.0, write-tests hook, recommend bump, docs tables

### Task 3: Verify

- [x] Skill front matter validates (1–2 sentences, known agents)
- [x] Empty scan is not `done`; stub scan is `not_done`
- [x] Commit
