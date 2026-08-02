---
title: Agent Adapters
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [b2-agent-adapters, current]
tags: [agents]
status: shipped
---

# Agent Adapters

Isolated implementations of the frozen `Agent` trait in `kit-agents`.  
A broken adapter degrades **one** agent, never the control room.

## Trait (conceptual)

```
probe() -> AgentStatus
spawn(spec, worktree, tx) -> AgentHandle
AgentHandle: wait / kill
```

## Implementations

| Kind | Binary | Headless | Skills |
|------|--------|----------|--------|
| Codex | codex | `exec -C -s workspace-write --json` | yes |
| Claude | claude | `-p` | yes |
| Grok | grok | `-p --cwd --always-approve` | yes |
| Ollama | ollama | `run $MODEL` + stdin | yes |

Windows: spawn via `cmd /C` so npm `.cmd` shims work.

## Env

| Var | Effect |
|-----|--------|
| `KIT_FULL_AUTO=1` | Bypass approvals (dangerous) |
| `KIT_SKILLS_DIR` | Skills root override |
| `KIT_OLLAMA_MODEL` | Default llama3.2 |

## Probe semantics

- Installed = version/help succeeds
- Authenticated = currently assumed if installed (no secret inspection)
- Remedy string always when missing

## v1.0 hardening

- [ ] Real auth checks without reading secrets (e.g. `codex login status` if exists)
- [ ] Handle registry wired to kill
- [ ] JSONL pretty-print improved for Codex events
- [ ] Integration test with mocked binary
