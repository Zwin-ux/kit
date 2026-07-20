---
name: api-docs
description: Document a library or service API with clear usage examples.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Find the public surface: exports, routes, or CLI commands.
2. Document each public item with:
   - Purpose
   - Inputs
   - Outputs or side effects
   - One short example
3. Prefer real examples from the codebase.
4. Mark unstable or experimental APIs clearly.
5. Keep docs next to the code or in the project’s established docs path.
6. Do not document private helpers as public API.
