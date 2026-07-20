---
name: git-commit
description: Write clear, honest commit messages and small reviewable commits.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Inspect the full diff. Group unrelated changes into separate commits when practical.
2. Prefer imperative subject lines under ~72 characters (what the commit does).
3. Body: why it changed, not a file list. Call out breaking changes and follow-ups.
4. Do not hide refactors, renames, and behavior changes in one opaque commit.
5. Never invent changes that are not in the working tree.
6. Suggest a subject + optional body the user can paste.
