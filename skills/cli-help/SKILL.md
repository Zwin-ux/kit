---
name: cli-help
description: Improve CLI help text, usage examples, and flag documentation.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Inventory commands, flags, and common failure modes.
2. Write `--help` style usage that matches real behavior.
3. Add 2–4 copy-paste examples for the main happy paths.
4. Document exit codes and env vars that matter.
5. Prefer short, imperative copy. No marketing fluff.
6. Note breaking flag changes clearly if any.
