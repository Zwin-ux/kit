# Kit 1.0 — Current architecture state

**Date:** 2026-08-01  
**Goal:** production multi-agent control room — Codex-like headless workflow × N agents + skills.

---

## Product one-liner

Dispatch many agents. Watch them in one place. Nothing ships unproven.

## What is real today

| Layer | Status | Prove it |
|-------|--------|----------|
| Control Room TUI | Real (1.0 craft) | `cargo run -p kit-cli -- --demo` — theme tokens, FAIL wash, empty state |
| Dispatch / Board / Detail | Real (1.0 craft) | shared chrome; semantic STATE/GATE colors; concept art in `docs/dev/assets/` |
| Gate (Guardian) | Real | kit-gate tests |
| Worktree + receipt | Real | `kit run --dry-run --json` |
| **Agent adapters** | **Live** | codex / claude / grok / ollama |
| **Skills injection** | **Live** | `.agents/skills` → worktree + prompt |
| PTY attach | Stub | 1.0.1 (CEO stamp) |
| Kill mid-run | **Wired** | `k` → EngineCommand::Kill → CancelHandle + tree kill (Win job / Unix pgid); receipt `Killed` |
| Retry fail | **Wired** | fail-only; gate failure context in new task |
| Max concurrency | **Wired** | semaphore 8 in engine supervisor |
| Vacuous gate | **Wired** | empty gate → infer cargo/npm when live; still empty → UI `UNCONFIGURED` (not PASS); live `kit run` exits 1 unless `--allow-vacuous` |
| JSON envelope | **Wired** | `schemaVersion: 1` on `run --json` + `doctor --json` — see `docs/json-contract.md` |
| Help overlay | **Wired** | `?` / Esc in Control Room surface |

## Spine

```
kit (TUI Dispatch or `kit run`)
  → engine::execute
       → git worktree
       → kit-agents::adapter(kind).spawn  (or dry-run)
            → install .agents/skills (addyosmani pack)
            → skills preamble + user task
            → codex exec | claude -p | grok -p | ollama run
            → stream → RunDelta → Control Room
       → kit-gate
       → ~/.kit/runs/<id>/receipt.json
```

## Agent workflow (Codex-style)

| Agent | Command shape |
|-------|----------------|
| Codex | `codex exec -C <wt> -s workspace-write --json <prompt>` |
| Claude | `claude -p <prompt>` (cwd = worktree) |
| Grok | `grok -p <prompt> --cwd <wt> --always-approve` |
| Ollama | `ollama run $KIT_OLLAMA_MODEL` (stdin prompt) |

`KIT_FULL_AUTO=1` bypasses approval prompts (dangerous — for sandboxes only).  
`KIT_SKILLS_DIR` overrides skill pack path.

**Defaults:** live when the CLI is on PATH; `--dry-run` for CI/offline.  
TUI Dispatch uses the same auto rule.

## How to run

```bash
# Doctor — which agents + skills are ready
cargo run -p kit-cli -- doctor

# Offline CI path
cargo run -p kit-cli -- run --dry-run --task "smoke" --json

# Live Codex (if installed) with skills in the worktree
cargo run -p kit-cli -- run --agent codex --task "add a unit test for X"

# Control Room — d dispatch spins agents live
cargo run -p kit-cli
```

## Specs / plans

- `docs/dev/PRD-1.0.md` — product requirements  
- `docs/dev/tasks/B2-agent-adapters.md` — adapter + skills spec  
- `tasks/plan.md` / `tasks/todo.md` — execution checklist  
- Root `AGENTS.md` + `.agents/skills` — skill routing for every agent  
- **`wiki/kit-1.0/`** — Karpathy llm-wiki: every concept fully specified + workstreams P0–P5  
  - Start: `wiki/kit-1.0/CLAUDE.md` + `wiki/kit-1.0/wiki/index.md`  
  - Roadmap: `wiki/concepts/roadmap/workstreams`  
  - Query: `outputs/queries/2026-08-01-what-next-for-high-quality-v1.md`

## Next (production) — see wiki workstreams + CEO stamp

CEO stamped (`docs/dev/tasks/CEO-STAMP-P1.md`): vacuous C→UNCONFIGURED (P2), board prefill-only, attach 1.0.1, JSON thin (P4), **P1 authorized**.

1. **P0** Land PR #11 once Rust CI stays green 3 OS — **unblocking** (Unix clippy fixed)  
2. **P1** Control plane — **shipped on branch** (EngineCommand + registry + kill/retry/timeout/max-8)  
3. **P2** Gate vacuous UNCONFIGURED + inference — **shipped on branch**; demo deliberate FAIL still open  
4. **P3** 8-concurrent proof harness (dispatch 12 → ≤8 running)  
5. **P4** doctor/receipt JSON envelope (`schemaVersion: 1`) + security flags  
6. **P5** Installers + 1.0.0
