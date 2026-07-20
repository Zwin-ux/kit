# Kit Registry API

Public read-only catalog for Kit packs and skills.  
Runs on Railway.

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Health + counts |
| GET | `/v1/packs` | List packs (`?tag=` `?projectType=`) |
| GET | `/v1/packs/:name` | Pack + skills |
| GET | `/v1/skills` | List skills (`?agent=`) |
| GET | `/v1/skills/:name` | Skill detail |
| GET | `/v1/search?q=` | Search packs + skills |

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
