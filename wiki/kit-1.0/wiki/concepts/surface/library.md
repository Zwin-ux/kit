---
title: Library
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [tui, skills]
status: planned
---

# Library

Minimal supporting screen: **local skills** installed for dispatch substrate.

## 1.0 scope (minimal)

- List skills under configured skills roots (`.agents/skills`, user kit skills)
- Show name + description from SKILL.md frontmatter
- Open path / copy name into Dispatch task as `@skill foo` (optional syntax)

## Out of scope

- Browse remote registry
- Publish / rate / follow
- Workshop editor

## Relationship to skills injection

Library is **browse UX**.  
Injection on run always uses pack on disk (see [[concepts/Skill|Skill]]).  
If Library is cut, injection still works.
