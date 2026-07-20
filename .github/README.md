# GitHub Actions

This repo has **one** workflow:

| File | Purpose |
|------|---------|
| `workflows/ci.yml` | Build, typecheck, unit tests, pack tests, doctor |

## Safety

- Read-only `contents` permission
- No deploy jobs
- No secrets required
- No force-push or branch protection changes
- Concurrency group is scoped to **this workflow only** (`kit-ci-…`)

If you add more workflows later, give them distinct names and concurrency groups so they do not cancel each other unexpectedly.

## Triggers

- `push` to `main`
- `pull_request` targeting `main`

## What the job runs

```text
pnpm install --frozen-lockfile
pnpm build
pnpm test
pnpm typecheck
node packages/cli/dist/bin.js test --all-packs
node packages/cli/dist/bin.js doctor   # KIT_HOME isolated under runner.temp
```

Local equivalent: `pnpm build && pnpm test && pnpm typecheck && pnpm test:packs && pnpm doctor`.
