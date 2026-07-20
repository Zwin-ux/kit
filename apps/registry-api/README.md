# Kit Registry API

Public read-only catalog for Kit packs and skills.  
Runs on Railway.

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Health + counts + auth flag |
| GET | `/v1/packs` | List packs (`?tag=` `?projectType=`) |
| GET | `/v1/packs/:name` | Pack + skills |
| GET | `/v1/skills` | List skills (`?agent=`) |
| GET | `/v1/skills/:name` | Skill detail |
| GET | `/v1/search?q=` | Search packs + skills |
| POST | `/auth/github/device/start` | Start device flow (CLI/TUI) |
| POST | `/auth/github/device/poll` | Poll device flow (`{ "device_code" }`) |
| GET | `/auth/github/login` | Browser OAuth start |
| GET | `/auth/github/callback` | Browser OAuth callback |
| POST | `/webhooks/github` | GitHub App webhooks |

## Railway env

```text
GITHUB_APP_ID=
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=
GITHUB_WEBHOOK_SECRET=
PUBLIC_BASE_URL=https://kit-registry-production.up.railway.app
```

Never commit secrets.

## Local

```sh
cd apps/registry-api
npm install
npm run build
npm start
# http://localhost:3000/health
```

## Deploy

Deploy the `apps/registry-api` directory to Railway (root directory for the service).

## Next

- Postgres for durable catalog
- GitHub auth / device code
- Publish endpoint
- TUI Explore client
