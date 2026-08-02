# AGENTS.md — Living Context for Coding Agents

You are working on **Kit 1.0** (Rust control room).

> Supersedes the 0.1 “skill platform / Ink TUI” brief.  
> Product truth: `PRD-1.0.md`. Ownership: `BUILD-ASSIGNMENT.md`.  
> Root agent routing: `/AGENTS.md` + `.agents/skills` ([addyosmani/agent-skills](https://github.com/addyosmani/agent-skills)).

## Core identity

- **Control room for parallel agent work** — dispatch, watch, gate, receipt
- Gate is the differentiator (`GATING` state; nothing ships unproven)
- Terminal is the product (ratatui + crossterm + tokio)
- Local-first, no credential custody, bounded by default

## Skills pack (required)

All agents on this repo should use **addyosmani/agent-skills** under `.agents/skills/`.

| When | Skill |
|------|--------|
| Session start | `using-agent-skills` |
| New feature | `spec-driven-development` → `planning-and-task-breakdown` |
| Implementation | `incremental-implementation` + `test-driven-development` |
| TUI screens | `frontend-ui-engineering` (interaction language, a11y) |
| Contracts / seams | `api-and-interface-design` + Claude-only contract rule |
| Before PR | `code-review-and-quality` + `code-simplification` |
| Perf / idle CPU | `performance-optimization` (M0 kill criteria) |

Surface work queue: `tasks/plan.md`, `tasks/todo.md`.

## Ownership

| Agent | Owns |
|-------|------|
| Grok | `crates/kit-tui` (F1–F6), later B2-pty |
| Codex | engine crates, CI, ports |
| Claude | five contract files, merge review |

**Never:** timers outside the event loop; edit frozen contracts as Grok/Codex.

## Commands

```text
cargo test -p kit-tui
cargo clippy -p kit-tui --all-targets -- -D warnings
cargo test --workspace
```

## Surface status

| Corner | Status |
|--------|--------|
| F1 Event loop | Done |
| F2 Control Room | Done |
| F3 Run detail | Done |
| F4 Dispatch + Board | In progress / landing |
| F5 Theme (light) | Partial |
| F6 Hit-testing | Not started |

## Visual rules (TUI)

- High contrast; monochrome-friendly
- Stable selection (by RunId); fixed geometry on ↑↓
- One interaction language: Esc back, footer grammar consistent
- Reduced motion: `NO_COLOR`, `KIT_MOTION=off`
