---
name: error-ux
description: Improve user-facing error messages so failures are actionable.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. List the failure modes a user can hit (CLI, API, UI).
2. For each important path, ensure errors say: what failed, why, and what to do next.
3. Strip stack traces and internal jargon from end-user surfaces; keep detail in logs.
4. Use stable error codes or short names when scripts depend on output.
5. Prefer one clear message over nested rethrows that bury the root cause.
6. Add or update tests for the friendliest failure cases.
7. Report before/after examples for the worst messages.
