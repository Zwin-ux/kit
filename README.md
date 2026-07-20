# Kit

**The terminal-native, pixel-art Agent Skills platform.**

Create, share, test, and run portable skills that work across Claude Code, Grok Build, Codex, Cursor, and more.

Free forever for individuals. Account-based for mass adoption.  
**No web app. Everything lives in a high-quality pixel-art TUI.**

![Kit Mascot](docs/assets/kit-mascot.png)

> A baby fox (a “kit”) that helps you build and share the tools your agents actually use.

## Why Kit exists

Agent Skills are becoming a real standard (SKILL.md and friends), but the ecosystem is still fragmented:

- Different folder conventions per harness
- Weak validation
- Painful sharing
- Almost no good testing story
- Zero soul

Kit fixes that with three hard opinions:

1. **Portable by default** — one skill, many agents
2. **Terminal only** — a beautiful pixel-art TUI is the entire product
3. **Account-native** — free accounts from day one so we can grow into teams, private registries, and more later

## Core Features (Target)

### Local Engine
- Strict skill schema + excellent validation
- Cross-harness path normalization
- Safe script execution
- Local library + offline-first
- **Starter packs** for project types (essentials, web-app, library)
- Built-in skill testing against fixtures or your current repo

### Starter packs (available now)

```sh
kit pack list
kit pack install essentials      # install into ~/.kit
kit pack apply web-app --dir .   # also copy into ./.kit/skills
```

| Pack | Best for |
|------|----------|
| `essentials` | Any repo |
| `web-app` | Apps and sites |
| `library` | Packages and SDKs |

See [docs/STARTER_PACKS.md](./docs/STARTER_PACKS.md).

### Pixel-Art TUI
- Explore & install skills
- Create and edit skills with live validation
- Test skills instantly
- Manage your library and versions
- Profile, following, collections
- Smooth pixel animations and strong personality

### Account Service (Free)
- Signup / login entirely from the TUI
- Publish skills under your name
- Sync library across machines
- Follow creators
- Private skills (limited on free tier)
- Basic analytics on your skills
- Designed so we can later add teams, orgs, verified publishers, etc.

## Mascot

**Kit** — a confident little fox holding a wrench.  
Pixel-art, limited palette, iconic enough for GitHub avatars, splash screens, and in-TUI sprites.

## Tech Direction

- **Core**: High-quality TypeScript package (npm)
- **TUI**: Pixel-art first (Ink or custom + pixel assets, or hybrid with Rust for performance)
- **Backend**: Railway (auth, registry, search, social, sync)
- **Skill Format**: Strict, well-documented extension of the emerging open skills standard
- **Grok Build**: First-class citizen and the preferred coding agent for developing Kit itself

## Repo Status

Public. Planning and foundation phase.

We are using Grok Build as the primary coding agent for this project.

## Roadmap (High Level)

**Phase 0 – Foundation**
- Repo, mascot, vision, architecture docs
- Skill schema v0
- Basic CLI + TUI skeleton with pixel art

**Phase 1 – Local Skills Engine**
- Validator, local install/list/remove
- Cross-harness discovery
- Test runner

**Phase 2 – TUI + Accounts**
- Full pixel-art TUI
- Auth flow in terminal
- Publish + search against Railway registry

**Phase 3 – Social + Polish**
- Following, collections, profiles
- Animations, themes, quality-of-life
- Documentation site (minimal, still terminal-first ethos)

## Development

This project is being built with Grok Build as the main coding agent.

See [ARCHITECTURE.md](./ARCHITECTURE.md) and [ROADMAP.md](./ROADMAP.md) for deeper planning.

---

Made with care for people who live in the terminal.
