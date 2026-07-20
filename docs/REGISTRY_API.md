# Registry API (Railway)

## Goal
Host a public catalog of Kit packs and skills.
Keep the TUI offline-first; registry is optional Explore.

## MVP (v0.1)

- Read-only HTTP API
- Seeded official packs/skills
- Deployed on Railway
- No auth yet (publish comes later)

## Base URL

Production (Railway project `kit-registry`):

```
KIT_REGISTRY_URL=https://kit-registry-production.up.railway.app
```

Examples:

```sh
curl https://kit-registry-production.up.railway.app/health
curl https://kit-registry-production.up.railway.app/v1/packs
curl "https://kit-registry-production.up.railway.app/v1/search?q=readme"
```

## Endpoints

See `apps/registry-api/README.md`.

## Client (future CLI)

```sh
# planned
kit explore search "readme"
kit explore install essentials --from registry
```

## GitHub App (Kit-skills)

| Setting | Value |
|---------|--------|
| App name | Kit-skills |
| App ID | `4343273` |
| Client ID | `Iv23liinXwHgamjhILSP` |
| Homepage | https://github.com/Zwin-ux/kit |
| Callback | `https://kit-registry-production.up.railway.app/auth/github/callback` |
| Webhook | `https://kit-registry-production.up.railway.app/webhooks/github` |

Railway env (never commit):

```text
GITHUB_APP_ID=4343273
GITHUB_CLIENT_ID=Iv23liinXwHgamjhILSP
GITHUB_CLIENT_SECRET=...
GITHUB_WEBHOOK_SECRET=...
PUBLIC_BASE_URL=https://kit-registry-production.up.railway.app
```

**Enable Device Flow** in the GitHub App settings (required for CLI/TUI login).

Auth endpoints:

- `POST /auth/github/device/start`
- `POST /auth/github/device/poll` body: `{ "device_code": "..." }`
- `GET /auth/github/login` (browser)
- `GET /auth/github/callback`
- `POST /webhooks/github`

### CLI login + explore

```sh
.\kit.cmd login
.\kit.cmd whoami
.\kit.cmd explore packs
.\kit.cmd explore search readme
.\kit.cmd explore show essentials
.\kit.cmd logout
```

Override registry: `KIT_REGISTRY_URL=https://kit-registry-production.up.railway.app`

## Safety

- Catalog GET is public
- Secrets only on Railway
- CI for monorepo stays test-only and does not deploy Railway automatically
