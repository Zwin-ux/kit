# Kit 1.0 Product Knowledge Base

> Schema document — read at the start of every session together with `wiki/index.md`.
> Update after every major compile, ingest batch, or structural change.

## Scope

What this wiki covers:
- Kit 1.0 as a **control room for parallel agent work** (product + engineering)
- Every core concept: Run, Gate, Receipt, Worktree, Agent, Skill, Board, Bounds, Firewall
- Every product surface: Control Room, Run Detail, Dispatch, Board, Doctor, Library, CLI
- Engine pipeline, adapters, skills injection, filesystem layout
- Quality bars, kill criteria, security, performance, distribution
- Honest **status**: what is shipped vs remaining for a high-quality v1.0.0

What this wiki deliberately excludes:
- Kit 0.1 skill marketplace / registry / Railway catalog (cut for 1.0)
- Ink TUI implementation details (archived; only interaction lessons)
- General coding-agent tutorials unrelated to Kit
- Long-term 2.0 multi-tenant cloud control room

## Operations

This wiki follows the llm-wiki skill's five operations: `compile`, `ingest`, `query`, `lint`, `audit`.
Every operation appends an entry to `log/YYYYMMDD.md`.

Primary sources live in the Kit repo (`docs/dev/*`, contracts in `crates/*`).  
`raw/` holds **snapshots / pointers** so the wiki is self-contained; prefer citing both.

## Naming conventions

### Pages
- **Concept pages** (`wiki/concepts/`): Title Case. Prefer folder-split for multi-aspect topics.
- **Entity pages** (`wiki/entities/`): Proper names (tools, crates, agents).
- **Summary pages** (`wiki/summaries/`): kebab-case source slug.

### Wikilinks
- Use `[[concepts/Foo]]` or `[[concepts/Foo/index|Foo]]` for folders.
- Link first mention of each concept.

### Frontmatter
Every wiki page:
```yaml
---
title: <Page Title>
type: concept | entity | summary
created: YYYY-MM-DD
updated: YYYY-MM-DD
sources: []
tags: []
status: shipped | partial | planned | cut
---
```

### Diagrams
- All flows in **mermaid**. No ASCII architecture boxes.

## Source map (canonical repo files)

| Source | Role |
|--------|------|
| `docs/dev/PRD-1.0.md` | Product requirements |
| `docs/dev/BUILD-ASSIGNMENT.md` | Agent ownership |
| `docs/dev/CURRENT.md` | Living implementation truth |
| `docs/dev/tasks/B2-agent-adapters.md` | Adapter + skills spec |
| `crates/kit-core/src/{run,gate,config}.rs` | Frozen contracts |
| Root `AGENTS.md` + `.agents/skills` | Skill routing |

## Open research questions

- Should vacuous gate (empty kit.toml) block PASS in production, or only warn?
- Is Board a real pull-queue in 1.0 or a curated Dispatch prefill list?
- npm dual-path: keep 0.1 packages forever, or hard cut at 1.0.0?
- Default sandbox for Codex: always workspace-write, or profile per repo?
- When is attach/PTY required for "high quality" vs nice-to-have?

## Research gaps / remaining ingest

- [ ] Guardian original guard.js threat model notes (if not fully in kit-gate)
- [ ] trenchwire npm platform package pattern (for M5)
- [ ] fennec-tui interaction language notes (for F5/F6 polish)
- [ ] PR #7 JSON contract surface inventory

## Notes for the LLM

- Prefer **honest status** over aspirational marketing. Tag pages with `status:`.
- Product voice: Simplified Technical English when writing user-facing copy; engineering pages can be denser.
- When specs conflict: PRD product intent wins for *what*; CURRENT wins for *what exists*; contracts win for *shapes*.
- Karpathy guidelines: divide-and-conquer pages, no mega-files, verifiable acceptance on every feature.
