---
name: acceptance-review
description: Assess delivered behavior against the task's stated acceptance criteria.
user-invocable: false
---

# Acceptance Review

1. Identify the original task, acceptance checks and exact deliverable or commit. Ask for missing criteria rather than inventing approval.
2. Build a short criterion-to-evidence mapping using test output, observed behavior and changed files.
3. Mark each criterion verified, failed or unverified. A completed agent turn is a result awaiting review, not an accepted task.
4. Check the primary flow and meaningful failure/recovery states. Separate functionality, visual evidence and automated checks.
5. Explain review rejection with reproducible facts and the smallest revision needed.
6. Preserve the fixed result or commit and review evidence when handing off. Never silently switch to a newer artifact.
7. Report readiness and remaining blockers. Shipping, merging and final acceptance remain explicit user actions.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
