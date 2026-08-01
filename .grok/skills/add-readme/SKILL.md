---
name: add-readme
description: Create a clear README for a new or existing project.
version: 0.1.1
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Scan the repository structure, package manifests, and existing docs.
2. Identify what the project does in one short sentence.
3. Write or update `README.md` with these sections only when they apply:
   - What it is
   - Quick start
   - Usage
   - Configuration
   - Development
   - License
4. Prefer real commands from the repo over generic placeholders.
5. Keep language short, active, and concrete.
6. Do not invent features that the code does not support.
