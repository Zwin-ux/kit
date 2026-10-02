---
name: dependency-review
description: Assess dependency provenance, pinned versions and changes without installing packages.
user-invocable: false
---

# Dependency Review

1. Read the project's manifests, lockfiles and the diff. Identify added packages, source URLs, integrity fields, revision pins and license notices.
2. Explain what each new dependency is needed for; flag unnecessary scope or a mismatch with the existing runtime.
3. Check whether a supposedly fixed revision is mutable. Prefer full commit pins for imported agent skills, retaining source, path, revision and license.
4. Treat install scripts, hooks, plugins and MCP startup as executable behavior requiring review. Never start them just to inspect the catalog.
5. Use already available vulnerability evidence; if current advisory access is unavailable, report that check as unverified.
6. Suggest minimal compatible changes, with explicit consequences. Do not perform installs, global configuration edits or unrelated upgrades.
7. Return evidence per finding and an exact list of unchecked assumptions.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
