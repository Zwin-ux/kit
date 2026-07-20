---
name: pr-ready
description: Prepare a clear pull request summary, test plan, and risk notes.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Summarize what changed in plain language (what / why, not a file dump).
2. List user-visible impact and any migrations or env changes.
3. Write a short test plan: what you ran, what a reviewer should verify.
4. Call out risks, rollbacks, and anything intentionally left out.
5. Suggest a PR title under ~72 characters.
6. Keep the write-up scannable with bullets.
