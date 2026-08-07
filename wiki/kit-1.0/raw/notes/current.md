# Kit 1.0 — Current architecture state

**Date:** 2026-08-01  
**Goal:** production multi-agent control room — Codex-like headless workflow × N agents + skills.

---

## Product one-liner

Dispatch many agents. Watch them in one place. Nothing ships unproven.

## What is real today

| Layer | Status | Prove it |
|-------|--------|----------|
| Control Room TUI | Real | `cargo run -p kit-cli -- --demo` |
| Dispatch / Board / Detail | Real | keys `d` / `b` / Enter |
| Gate (Guardian) | Real | kit-gate tests |
| Worktree + receipt | Real | `kit run --dry-run --json` |
| **Agent adapters** | **Live** | codex / claude / grok / ollama |
| **Skills injection** | **Live** | `.agents/skills` → worktree + prompt |
| PTY attach | Stub | B2-pty |
| Kill mid-run | Seam only | needs handle registry |

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

- `docs/dev/tasks/B2-agent-adapters.md` — adapter + skills spec  
- `tasks/plan.md` / `tasks/todo.md` — execution checklist  
- `docs/dev/PRD-1.0.md` — product requirements  
- Root `AGENTS.md` + `.agents/skills` — skill routing for every agent

## Next (production)

1. Handle registry for kill/retry from TUI  
2. B2-pty attach/detach  
3. Skill multi-select in Dispatch UI  
4. Real gate configs in dogfood repos  
5. M5 distribution
