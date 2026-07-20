---
name: fix-bug
description: Find root cause and fix a bug without drive-by refactors.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Restate the bug with expected vs actual behavior.
2. Reproduce it or gather the best available evidence.
3. Form a root-cause hypothesis before changing code.
4. Make the smallest fix that addresses the root cause.
5. Add or update a test that would have caught the bug.
6. Avoid unrelated cleanup in the same change.
7. Report:
   - Root cause
   - Fix
   - How you verified it
