# Completeness QA skill — design

**Status:** Draft for review  
**Date:** 2026-08-14  
**Repo:** [Zwin-ux/kit](https://github.com/Zwin-ux/kit)  
**Slice:** Pack skill only (no `kit-gate` change, no named Control Room agent)

## 1. Problem

Coding agents ship half-work. They add a function name, a happy path, and two tests. The rest is a stub, `TODO`, hardcoded return, or a missing error path. Existing Kit checks do not catch this:

| Surface | What it actually does |
|---|---|
| `write-tests` | “Write three tests and run them.” No inventory. |
| `code-review` | Asks the model to look. No function list. |
| `kit recommend` | Picks a **pack by project type** (web-app vs library). Does not ask “is this change done?” |
| `kit-gate` | Runs `test` / `typecheck`. A green suite of two tests still PASSes. |

The product line is “Nothing ships unproven.” Today, unproven work still ships.

A second, related gap: Kit can install many skills, but it does not tell an agent **which few skills raise software-engineering quality** for *this* change. SWE-bench-class work (reproduce, implement fully, prove, stop) needs a short ladder, not a dump of every catalog entry.

## 2. Goal

Ship one portable catalog skill, `completeness-qa`, that:

1. **Sees the public function surface** of the change (or of the repo if there is no diff).
2. **Flags slop** with deterministic checks (stubs, unimplemented, empty bodies).
3. **Maps symbols to tests** by name mention — not by a coverage runner.
4. **Ranks what to test next** so the list is useful, not every private helper.
5. **Names the next skills** that belong on this change (write-tests, fix-bug, code-review, …) so Kit sets up a SWE-quality ladder, not a random toolkit.
6. **Presents as a tool**, not a lecture: one verdict, a short table, a next command.

It must work wherever Kit already injects `*/SKILL.md` packs: worktrees, `.kit/skills` after `pack apply`, and agent harness folders after `kit link`. The same folder is a valid GitHub Agent Skill if copied to `.github/skills`.

## 3. Non-goals (this slice)

- Do not change `crates/kit-gate`, `kit.toml`, or fail CI.
- Do not add a named QA agent or a new Control Room screen.
- Do not rewrite `recommendToolkits` in `packages/core`. Pack-by-type stays there. This skill complements it.
- Do not parse full ASTs, run Istanbul/tarpaulin, or require tree-sitter.
- Do not write tests unless the user asks. Hand off to `write-tests`.
- Do not demand a test for every private helper.
- Do not add GitHub Copilot to `compatibility` (schema v0 allow-list is `claude-code`, `grok-build`, `codex` only).

A later slice can consume `.kit/completeness.json` as a gate check. This spec only requires the file to be a stable, documented shape so that slice does not invent a second format.

## 4. Product presentation

The skill is a **quality instrument**, not a coverage nag.

### 4.1 Voice

- Short. One screen of output.
- Verdict first: `DONE` / `NOT DONE` / `PARTIAL`.
- Counts second.
- At most **five** “test next” rows.
- At most **four** skill-ladder rows (including “you are here”).
- Every recommended skill has a **reason tied to a finding**, not a generic “testing is good.”
- Skip skills that do not apply. Silence is better than `a11y-pass` on a Rust crate with no UI.

### 4.2 Human report (canonical)

```text
KIT completeness-qa

Surface    14 public  ·  3 stub  ·  6 untested  ·  5 ok
Verdict    NOT DONE — 3 public functions are stubs

Test next
  1. src/auth.ts:login      stub + untested
  2. src/auth.ts:logout     untested
  3. src/auth.ts:resetToken untested

Skill ladder
  1. completeness-qa     you are here
  2. write-tests         6 untested public symbols
  3. code-review         after tests go green

Next    load write-tests and cover login, logout
```

If everything is ok:

```text
KIT completeness-qa

Surface    8 public  ·  0 stub  ·  0 untested  ·  8 ok
Verdict    DONE — public surface is implemented and mentioned in tests

Skill ladder
  1. completeness-qa     you are here
  2. code-review         ship/no-ship on the diff

Next    load code-review
```

### 4.3 Why this is the SWE-quality story

SWE-bench-style success is not “many skills installed.” It is a **small, ordered loop**:

| Step | Job | Kit skill |
|---|---|---|
| See the holes | Inventory + slop | `completeness-qa` (this skill) |
| Prove the bug / new behavior | Failing or missing tests | `write-tests` |
| Finish the functions | No stubs, no half-handlers | stay on implement; re-run this skill |
| Fix a real failure | Root cause, no drive-by | `fix-bug` |
| Ship judgment | Correctness / risk / tests | `code-review` |
| PR text | Only when shipping | `pr-ready` |

Project-type extras (`a11y-pass`, `cli-help`, `api-docs`, `data-check`) appear **only** when repo signals match, and only after the quality ladder. That is how Kit “identifies the best skills to set up”: quality first, flavor second.

`kit recommend` / `kit ready` still choose the pack (essentials, web-app, …). This skill chooses the **next two skills for this change**. Both stay.

## 5. Architecture

```
changed files | src/ lib/ crates/ app/
        │
        ▼
  scripts/inventory.mjs     (deterministic, zero npm deps)
        │
        ▼
  completeness.json
        │
        ├── text report to stdout (section 4.2)
        └── agent ranks + trims “test next”
              then names write-tests / fix-bug / code-review
```

Two units:

| Unit | Job | Depends on |
|---|---|---|
| `scripts/inventory.mjs` | Scan, classify stub/untested/ok, emit JSON + text | Node 18+, git optional |
| `SKILL.md` | When to load, how to run the script, how to rank, what not to do | Script output or the same rules by hand |

If Node is missing, the agent follows the rules in `SKILL.md` / `references/scan-rules.md` and still prints the same report shape. The script is the source of truth, not a hard runtime dependency of the skill.

## 6. Files

```
skills/completeness-qa/
  SKILL.md
  scripts/inventory.mjs
  references/scan-rules.md
  references/swe-quality-ladder.md
  tests/inventory.test.mjs
  tests/fixtures/
    stub-ts/src/auth.ts
    stub-ts/src/auth.test.ts
    clean-rs/src/lib.rs
    clean-rs/tests/foo.rs
```

Also:

- Add `completeness-qa` to `packs/essentials/PACK.md` `skills:` list.
- Bump essentials `version` `0.1.0` → `0.2.0`.
- Add a row to `skills/README.md` (or let `scripts/keep-alive.mjs --sync-only` regenerate the table).
- Point `skillFromPack` in `packages/core/src/recommend/recommend.ts` at essentials for this name **only if** we also `bumpSkill("completeness-qa", …)` when tests exist or when no tests exist. Keep that change tiny: one baseline bump so `kit recommend` lists it. Do not retune pack scoring.

No new pack. Other packs extend essentials, so they inherit the skill.

## 7. SKILL.md contract

Front matter (schema v0):

```yaml
---
name: completeness-qa
description: >
  Inventory public functions, flag stubs and untested symbols, and name the
  next skills that raise SWE quality for this change. Use after implement,
  before claiming done, or when asked to QA, find missing tests, or catch
  half-done functions.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---
```

`description` must stay one or two sentences and must include **when to load** so harness routers fire.

Body steps (normative, short):

1. Prefer the git diff. If none, scan `src/`, `lib/`, `crates/`, `app/`.
2. Run `node scripts/inventory.mjs` from the skill folder, with the project root as cwd (or `--root`). If Node is missing, scan by the rules in `references/scan-rules.md`.
3. Do not invent symbols the script did not list.
4. Rank “test next” using the ladder in `references/swe-quality-ladder.md`. Cap at five.
5. Name at most three follow-on catalog skills with reasons from the findings.
6. Print the report. Write `.kit/completeness.json` when the project is writable.
7. Do not write production code or tests unless the user asked. Point at `write-tests` or `fix-bug`.

## 8. Inventory script

### 8.1 CLI

```text
node scripts/inventory.mjs [--root <dir>] [--json] [--no-write]
```

| Flag | Default | Meaning |
|---|---|---|
| `--root` | cwd | Project to scan |
| `--json` | off | Print only JSON to stdout (text still writes if `--write`) |
| `--no-write` | write on | Do not write `.kit/completeness.json` |

Exit code is always `0` in this slice. This is not a gate. Failure to parse a file skips that file and records a warning.

### 8.2 Scan set

1. If `git rev-parse --is-inside-work-tree` succeeds, take `git diff --name-only` plus `git diff --cached --name-only` plus `git ls-files --others --exclude-standard` for source files. If that set is empty, treat as “no diff.”
2. No diff: walk `src/`, `lib/`, `crates/*/src/`, `app/`, `apps/*/src/` (first existing). Skip `node_modules`, `target`, `dist`, `build`, `.git`, vendor dirs.
3. Only these extensions: `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.rs`, `.py`.
4. Skip files that look like tests: `*.test.*`, `*.spec.*`, `*_test.rs`, `*_test.py`, `test_*.py`, paths under `tests/`, `__tests__/`, `benches/`, `examples/`.

### 8.3 Public symbols

Language-specific, line-oriented, conservative. A symbol is public only if it matches:

**JS/TS**

- `export function name`
- `export async function name`
- `export const name =` / `export let name =` when the right-hand side is a function or arrow
- `export class Name` — then non-private methods on that class (`name(` not starting with `#` or `_`)
- `module.exports.name =` / `exports.name =`

Skip `export type`, `export interface`, `export {` re-exports (v0 does not follow barrels).

**Rust**

- `pub fn name` / `pub async fn name` / `pub(crate) fn` is **not** public for this skill
- Skip files under `tests/`, `benches/`, `examples/`
- Skip `#[cfg(test)]` modules (if the `fn` is inside a `mod tests` block with `#[cfg(test)]` on a previous nearby line, skip)

**Python**

- Module-level `def name` / `async def name` not starting with `_`
- Class methods not starting with `_` (including `__init__` — skip dunders except no: skip all dunders so we do not nag `__repr__`)

Private / ignored names: start with `_`, Rust `test_` in test modules already skipped.

### 8.4 Stub / slop (deterministic)

A symbol is `stub` if its body (until the next top-level symbol or a reasonable brace/indent close, capped at 30 lines) is only:

| Pattern | Languages |
|---|---|
| empty / whitespace only | all |
| `todo!()`, `unimplemented!()`, `todo("…")` | Rust |
| `pass` | Python |
| `raise NotImplementedError` | Python |
| `throw new Error("not implemented")` / `throw new Error('TODO')` | JS/TS |
| `throw new Error(...)` as the **only** statement | JS/TS |
| `return null` / `return undefined` / `return;` as the **only** statement | JS/TS |
| `return None` as the only statement | Python |
| `ok(())` / `Ok(())` as the only statement **and** the fn is not clearly a no-op marker like `fn default()` — v0: still flag `Ok(())` only if there is also a `todo` comment in the body | Rust, conservative |
| body is only comments plus one of the above | all |
| `// TODO implement` / `# TODO` / `// FIXME stub` as the only non-punctuation content | all |

Do not flag a real `return null` that sits among other statements.

### 8.5 Test mention

Collect test-file text from the same extension rules, **including** test paths skipped as sources.

A symbol is `tested` if its **exact identifier** appears in any test file as a word (`\bname\b`). This is crude on purpose. False positives (a comment saying `login`) are acceptable. False negatives (tested only via a wrapper name) are acceptable; the agent may promote those in ranking notes but must not remove script rows.

A symbol is `untested` if not `tested`. A symbol can be both `stub` and `untested`.

Status enum, one per symbol:

| Status | Rule |
|---|---|
| `stub` | slop match (whether or not mentioned in tests) |
| `untested` | not stub, not mentioned in tests |
| `ok` | not stub, mentioned in tests |

### 8.6 JSON shape

Write `.kit/completeness.json` (create `.kit/` if needed):

```json
{
  "version": 1,
  "root": "/abs/project",
  "mode": "diff",
  "scannedFiles": 4,
  "counts": { "public": 14, "stub": 3, "untested": 6, "ok": 5 },
  "verdict": "not_done",
  "symbols": [
    {
      "file": "src/auth.ts",
      "line": 12,
      "name": "login",
      "lang": "ts",
      "status": "stub",
      "reason": "throw new Error(\"not implemented\")",
      "tested": false
    }
  ],
  "testNext": [
    { "file": "src/auth.ts", "name": "login", "status": "stub" }
  ],
  "skillLadder": [
    { "skill": "completeness-qa", "role": "current", "reason": "inventory" },
    { "skill": "write-tests", "role": "next", "reason": "6 untested public symbols" }
  ],
  "warnings": []
}
```

Paths in JSON are project-relative, POSIX slashes.

**Verdict**

| Verdict | When |
|---|---|
| `done` | `stub == 0` and `untested == 0` and `public > 0` |
| `partial` | `stub == 0` and `untested > 0` |
| `not_done` | `stub > 0` |
| `empty` | `public == 0` (report “no public symbols in scan set”; do not pretend DONE) |

**testNext (script default, before agent trim)**  
Order: all `stub` first (file, then name), then `untested`. Cap at 10 in JSON. The agent prints at most 5.

**skillLadder (script default)**

Always include `completeness-qa` as `current`.

Then, in order, include at most three `next` skills:

1. `write-tests` if `untested > 0` or `stub > 0` (reason: counts).
2. `fix-bug` if the user task / git subject looks like a bug **or** if any stub reason is `unimplemented` / `not implemented` on a file that already has tests (half-fix). If neither signal, skip.
3. `code-review` if `public > 0` (reason: “ship/no-ship after holes are closed” when not `done`; “ship/no-ship on the diff” when `done`).
4. Flavor, only if still under the cap of three `next` skills:
   - `a11y-pass` if `package.json` has react/vue/svelte/next
   - `cli-help` if `package.json` `bin` or Cargo `[[bin]]`
   - `api-docs` if library-shaped (package.json without web framework, or `Cargo.toml` lib)
   - `data-check` if `notebooks/` or `pyproject.toml` plus `data/`

Do not recommend `add-readme` or `project-setup` here. Those are `kit recommend` / `kit ready` jobs.

Bug-task heuristic (v0): any of `fix`, `bug`, `error`, `fail`, `panic`, `regress` in `git log -1 --pretty=%s` or in `KIT_TASK` env if set. Otherwise skip `fix-bug`.

## 9. Agent ranking (after the script)

The agent may **reorder or drop** `testNext` rows. It may not add symbols the script did not emit.

Keep a row when any of:

- status is `stub`
- name appears in the user task or open spec
- file is in the diff
- it is user-facing (handler, command, route, exported API)

Drop a row when it is clearly a tiny mapper/`Display`/`Debug` impl and not stub.

Do not expand the skill ladder beyond four rows total. Do not recommend skills outside the Kit catalog listed in `references/swe-quality-ladder.md`.

## 10. Error handling

| Situation | Behavior |
|---|---|
| Not a git repo | Scan default roots; `mode: "tree"` |
| Unreadable file | Skip; `warnings` += path |
| No Node | Agent scans by hand; still emit the text report; skip JSON write if they cannot run the script |
| `.kit/` not writable | Print report; skip write; warning |
| Empty scan | `verdict: empty`; next = “nothing to QA in the scan set” |
| Mixed languages | One list, `lang` per symbol |

Never throw away a stub because ranking is unsure.

## 11. Testing the skill

`tests/inventory.test.mjs` runs the script against fixtures with `node:test` (no extra deps).

Minimum cases:

1. TS fixture with one stub export, one tested export, one untested export → counts 3 / 1 / 1 / 1, verdict `not_done`, `write-tests` on the ladder.
2. Rust fixture with one `pub fn` mentioned in `tests/foo.rs` and no stub → verdict `done`.
3. `--no-write` does not create `.kit/completeness.json`.
4. Test files are not inventoried as public symbols.
5. `export type` / `export interface` are ignored.

Fixtures stay tiny (under 40 lines each).

`SKILL.md` must pass existing `validateSkill` rules (name, semver, compatibility).

## 12. Setup path (how Kit “sets up” the skills)

This slice does not add a new CLI. Setup is the existing path:

```text
kit pack apply essentials --dir .
kit link --to all --write
```

After essentials 0.2.0, `completeness-qa` is in `.kit/skills/` and in linked harness folders. Live Kit runs already copy the pack into the worktree; the description is what makes the agent load it after implement.

The skill report then names `write-tests` / `code-review` / … so the **same session** loads the next skill. That is “identify and set up the best skills” without a second product surface.

`kit recommend` may list `completeness-qa` once we add a small `bumpSkill` (section 6). Applying the pack is still `kit ready` / `pack apply`.

## 13. Acceptance

- `skills/completeness-qa/` validates as a v0 skill.
- `packs/essentials` lists it and is 0.2.0.
- `node skills/completeness-qa/tests/inventory.test.mjs` is green.
- On the TS fixture, stdout matches the voice in 4.2 (verdict first, ≤5 test-next rows, ≤4 ladder rows).
- A repo with only stubs cannot get `verdict: done`.
- A repo with no public symbols cannot get `verdict: done`.
- No edits under `crates/kit-gate`.
- `write-tests` gains one instruction at the top: if `.kit/completeness.json` exists, test those `testNext` symbols first. `code-review` is unchanged.

## 14. Later (not this slice)

- `kit-gate` check: `node .kit/skills/completeness-qa/scripts/inventory.mjs --json` and FAIL on `not_done`.
- Named QA dispatch in the Control Room.
- tree-sitter / real coverage.
- Teaching `recommend.ts` the full quality ladder (keep pack-by-type separate).
