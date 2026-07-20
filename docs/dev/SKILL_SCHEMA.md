# Skill Schema v0

This document defines the skill format for Kit.

## Goal
Make one skill work in many agents.
Keep the format strict and easy to validate.

## File Structure
A skill is a folder.
The folder must contain a file named `SKILL.md`.

Optional files:
- scripts/
- assets/
- tests/

## Required Fields in SKILL.md

Use YAML front matter.

```yaml
---
name: example-skill
description: Short description of what the skill does.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---
```

Rules:
- name must be lowercase
- name must use only letters, numbers, and hyphens
- description must be one or two sentences
- version must follow semver
- compatibility must list supported agents

Allowed compatibility agents in schema v0:
- `claude-code`
- `grok-build`
- `codex`

The parser lives in `@kit-skills/core`:
- `parseSkillMd` — YAML front matter + body
- `validateSkill` — field rules
- `loadSkill` — load a skill folder from disk
- `parseAndValidateSkillMd` — parse and validate from string

## Body
The body after the front matter contains the instructions.
Write clear steps.
Use simple language.

## Validation Rules
The validator must reject a skill if:
- SKILL.md is missing
- required fields are missing
- name does not match the rules
- version is not valid semver
- compatibility list is empty

## Example

```yaml
---
name: add-readme
description: Create a clear README for a new project.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
---

# Instructions

1. Read the project structure.
2. Write a short README.
3. Include installation steps.
4. Include basic usage.
```
