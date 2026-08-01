---
name: project-setup
description: Set up a clean project baseline for agents and humans.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Inspect the repo. Note language, package manager, and entry points.
2. Ensure these exist when useful:
   - `README.md` with install and run steps
   - `.gitignore` for the stack
   - `AGENTS.md` or agent instructions with project rules
   - A simple test or check command
3. Keep setup minimal. Do not add frameworks the project does not need.
4. Write commands that work from the repo root.
5. Record any required env vars in an `.env.example` without secrets.
6. Summarize what you added and how to verify the setup.
