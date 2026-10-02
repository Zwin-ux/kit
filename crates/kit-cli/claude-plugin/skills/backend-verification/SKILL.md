---
name: backend-verification
description: Verify backend changes against real failure modes and repeatable checks.
user-invocable: false
---

# Backend Verification

1. Derive checks from the task's behavior and boundaries, not from the new implementation's branches.
2. Identify external dependencies, fixtures and required configuration. Use project test facilities; do not copy or inspect private credentials.
3. Cover resource ownership, validation, error mapping, transaction rollback and retry behavior where relevant.
4. Exercise race conditions when shared state is changed. State which concurrency properties the tests do and do not prove.
5. Run the narrow useful test first, then the existing relevant package checks. Preserve failing output with the command and first actionable error.
6. Distinguish a mocked unit result from integration evidence. Never label an unrun check as passing.
7. Return reproducible verification steps and unresolved risks. Do not repair unrelated systems during this task.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
