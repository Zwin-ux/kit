# GitHub Actions

| File | Purpose |
|------|---------|
| `workflows/ci.yml` | Build, typecheck, unit tests, pack tests, doctor |
| `workflows/keep-alive.yml` | Weekly catalog promote + PR (queue → skills) |

## Safety

- CI: read-only `contents`; no deploy; no secrets
- Keep-alive: `contents` + `pull-requests` write only to open PRs (never force-push `main`)
- Distinct concurrency groups: `kit-ci-…` and `kit-keep-alive-…`

## CI triggers

- `push` to `main`
- `pull_request` targeting `main`

## Keep-alive triggers

- Cron: Mondays 15:00 UTC
- `workflow_dispatch`

## Local equivalents

```bash
pnpm build && pnpm test && pnpm typecheck && pnpm test:packs && pnpm doctor
pnpm keep-alive -- --check
pnpm keep-alive
```
