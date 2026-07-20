---
name: security-pass
description: Run a focused security pass for common app and API mistakes without a full audit.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Scope the pass: auth, input handling, secrets, dependencies, and public surfaces.
2. Search for secrets, tokens, and credentials in code and config. Flag anything that should be env-only.
3. Check authz on sensitive routes and actions (not just “is the user logged in”).
4. Review user input paths: validation, injection risk (SQL/command/path), file upload handling.
5. Note dependency or framework defaults that are insecure if left open.
6. Prefer concrete fixes over generic advice. Rank findings by severity.
7. Report:
   - Findings (severity · location · why it matters)
   - Recommended fixes
   - What you did not cover
