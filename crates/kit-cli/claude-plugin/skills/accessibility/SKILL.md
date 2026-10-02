---
name: accessibility
description: Review and improve keyboard use, labels, focus, semantics and readable state changes.
user-invocable: false
---

# Accessibility

1. List the controls and reading order in the changed flow. Each control needs a visible purpose and an accessible name.
2. Use the host's native controls and semantics. Every pointer action needs a keyboard path; focus must remain visible and predictable.
3. Follow the entire flow using the keyboard, including dialogs and error recovery. Restore focus to a sensible element when closing an overlay.
4. Check text and control contrast with the project's existing tools. Never convey success, failure or selection by color alone.
5. Make validation errors concrete, associate them with their fields, preserve entered data, and announce consequential state changes.
6. Respect reduced motion and avoid animations that move the user's target or change the reading order.
7. Report each issue with its affected control, reproduction, fix and verification. Separate observed failures from untested assumptions.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
