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

## Safety

- Public GET only for now
- No secrets in the client for read endpoints
- CI for monorepo stays local/test-only and does not deploy Railway automatically
