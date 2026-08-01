# Kit Roadmap

> **Superseded by [`PRD-1.0.md`](PRD-1.0.md).**
>
> Kit 1.0 is a Rust binary and a control room for parallel agent work. The
> registry track below — publish API, durable catalog, profiles, teams,
> verified publishers — is **cut**, not deferred. Skills remain as local
> substrate; they are no longer the product. See PRD section 7.
>
> This file is kept for the 0.1 history. Milestones and their kill criteria
> live in PRD section 8; who builds what lives in
> [`BUILD-ASSIGNMENT.md`](BUILD-ASSIGNMENT.md).

## 1.0 — at a glance

| Milestone | Delivers |
|---|---|
| M0 | Rust workspace, frozen contracts, one event loop, 3-OS CI, startup budget |
| M1 | One run end to end — worktree, stream, receipt |
| M2 | Control Room — N concurrent runs, attach, kill, diff |
| M3 | The gate — Guardian port, `GATING` state, retry with failure context |
| M4 | Board and fan-out — shared queue, one task across N repos |
| M5 | Ship — npm platform packages, cargo, installer, docs, 1.0.0 |

---

## Alpha v1 — shipped (0.1.x, TypeScript)

- [x] Name, monorepo, skill schema, local library
- [x] Seven starter packs + `extends` dependency skills
- [x] Pack silhouette icons (TUI + GitHub assets)
- [x] Cross-harness paths / link
- [x] test + doctor + CI
- [x] Pixel TUI: splash, first-run, home, packs, library, explore, doctor, paths
- [x] kit-idle live loop + restrained text motion
- [x] Point-at-project auto-recommend
- [x] GitHub device login + Railway public catalog explore
- [x] README / LICENSE / CHANGELOG for public alpha
- [x] Global npm: `npm i -g @mzwin/kit`
- [x] `kit import` from Claude/Codex/Grok harness folders
- [x] Keep-alive cron + skill queue
- [x] `kit unify` — scan/normalize/dedupe/rank/adopt keepers
- [x] E2E release proof: compiled-CLI harness, 3-OS CI, JSON contract v1

## Cut in 1.0

Recorded so the decision is not relitigated. Each was a real plan; none survives
contact with "what would a developer miss if Kit vanished."

- Authenticated publish API
- Postgres-backed durable catalog
- Profiles, following, collections
- Teams / private registries, verified publishers
- Workshop skill editor
- Services lane / trading plugins

## Success metrics

Superseded by PRD section 12. The 0.1 metrics measured pack installs; 1.0
measures whether anyone trusts the gate.
