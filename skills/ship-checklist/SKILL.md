---
name: ship-checklist
description: Run a practical pre-ship checklist for an app release.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Confirm the change set and intended release scope.
2. Check build and typecheck if the project has them.
3. Run tests that cover the changed area.
4. Verify env and config examples still match real usage.
5. Check for secrets, debug flags, and temporary code.
6. Confirm README or changelog notes if user-facing behavior changed.
7. Produce a pass/fail checklist with remaining blockers only.
