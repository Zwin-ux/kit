---
name: code-review
description: Review a change for correctness, risk, and clarity.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Identify the diff or files under review.
2. Check correctness first: logic bugs, edge cases, broken contracts.
3. Check risk: security, data loss, auth, unsafe shell or network use.
4. Check clarity: naming, dead code, confusing control flow.
5. Check tests: missing coverage for new behavior or regressions.
6. Report findings as:
   - Critical
   - High
   - Medium
   - Nit
7. For each finding, state the file, the problem, and a concrete fix.
8. End with a short ship / do-not-ship recommendation.
