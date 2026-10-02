---
name: ui-craft
description: Build a scoped interface with explicit states, consistent geometry and evidence from the running surface.
user-invocable: false
---

# Ui Craft

1. Read the task, relevant project instructions, existing components and the selected design. Name missing design evidence before changing visual direction.
2. Define the primary user action and acceptance checks. Cover loading, empty, populated, error, disabled and success states that the task actually needs.
3. Reuse the project's design tokens and interaction patterns. Keep spacing, text hierarchy, focus and layout consistent across affected screens.
4. Implement the smallest end-to-end interaction. Keep view state separate from external effects; do not invent backend results or claim a successful operation without a response.
5. Check the narrowest supported viewport and keyboard path. Reserve fixed geometry for icons and motion; honor reduced motion.
6. Run relevant existing checks and inspect the rendered result when tooling is available. Report blocked visual verification explicitly.
7. Return changed files, the exact checks and outcomes, and remaining discrepancies against the design. Do not publish, merge or change auth configuration without explicit authorization.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
