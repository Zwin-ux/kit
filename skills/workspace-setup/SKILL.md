---
name: workspace-setup
description: Set up monorepo and multi-package workspaces for agents and humans.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Map packages/apps and how they depend on each other.
2. Document install, build, test, and run commands from the repo root.
3. Keep agent instructions short: where code lives, what not to touch.
4. Prefer workspace scripts over deep nested one-offs.
5. Note env files per package without committing secrets.
6. Summarize the workspace map and how to verify a change end-to-end.
