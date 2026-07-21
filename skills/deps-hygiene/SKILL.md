---
name: deps-hygiene
description: Clean up dependency drift, unused packages, and risky version ranges.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Inventory direct dependencies and what they are for.
2. Flag unused, duplicate, or overlapping packages.
3. Check for known vulnerabilities and abandoned packages when tooling is available.
4. Prefer minimal version bumps that preserve lockfile integrity.
5. Avoid drive-by major upgrades unless requested.
6. After changes: install, build/test, and confirm lockfile updates are intentional.
7. Report:
   - Removed / replaced
   - Upgraded (and why)
   - Left alone (and why)
