# Implementation Plan: B2 agent adapters + skills in TUI

Skills active: `spec-driven-development`, `planning-and-task-breakdown`,
`incremental-implementation`, `test-driven-development`, `api-and-interface-design`.

## Overview

Enable spinning **real** coding agents (Codex / Claude / Grok / Ollama) from the
Control Room Dispatch form, with the **addyosmani agent-skills** pack injected
into every worktree and prompt — Codex-like headless workflow, multi-agent.

## Architecture decisions

1. **Implementations live in `kit-agents`** against the frozen `Agent` trait.
2. **Skills are filesystem + prompt**, not a new MCP protocol (v1).
3. **Default live when binary on PATH**; `--dry-run` for CI and offline.
4. **Codex sandbox default `workspace-write`**; full auto only via `KIT_FULL_AUTO=1`.
5. **One runner** (`kit-cli` engine) calls adapters; TUI already forwards Dispatch jobs.

## Tasks

### Phase 1 — Skills pack
- [x] Spec in `docs/dev/tasks/B2-agent-adapters.md`
- [x] `kit-agents::skills` resolve + install into worktree + build preamble

### Phase 2 — Adapters
- [x] Shared process handle + streaming stdout/stderr → `RunDelta::Output`
- [x] Codex adapter (`codex exec … --json`)
- [x] Claude adapter (`claude -p`)
- [x] Grok adapter (`grok -p --cwd --always-approve`)
- [x] Ollama adapter (`ollama run`)
- [x] `probe()` via `--version`

### Phase 3 — Wire engine + TUI defaults
- [x] `execute()` calls adapter when live
- [x] Auto live if installed; dry-run fallback
- [x] TUI Dispatch uses live path
- [x] `kit doctor` probes all four
- [x] `kit run` defaults live (`--dry-run` opt-in)

### Phase 4 — Docs / verify
- [x] CURRENT.md + README
- [ ] cargo test + clippy
- [ ] Manual: doctor + dry-run still green

## Risks

| Risk | Mitigation |
|------|------------|
| Agent blocks on approvals | `KIT_FULL_AUTO`; document sandbox |
| Windows .cmd path | `Command::new` uses PATH |
| Huge skill copy cost | copy only if missing; optional env to skip |
| JSONL noise in stream | pass through; pretty later |

## Open questions

- Skill multi-select in Dispatch UI → later (F4.1)
- Attach/PTY → B2-pty
