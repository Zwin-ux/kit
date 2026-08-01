---
name: write-tests
description: Add focused tests for important project behavior.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Find the existing test runner and patterns in the repo.
2. Choose the smallest behavior that matters to users or agents.
3. Prefer tests for:
   - Core happy path
   - One important edge case
   - One failure or validation path
4. Match local style. Do not introduce a new test framework unless none exists.
5. Make tests deterministic. Avoid network and real clocks when possible.
6. Run the tests and fix failures you introduce.
7. Summarize what is covered and what is still untested.
