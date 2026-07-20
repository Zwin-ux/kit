# GitHub Actions

This repo has **one** workflow:

| File | Purpose |
|------|---------|
| `workflows/ci.yml` | Build, unit tests, pack tests, doctor |

## Safety

- Read-only `contents` permission
- No deploy jobs
- No secrets required
- No force-push or branch protection changes
- Concurrency group is scoped to **this workflow only** (`kit-ci-…`)

If you add more workflows later, give them distinct names and concurrency groups so they do not cancel each other unexpectedly.
