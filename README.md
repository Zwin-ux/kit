<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — Portable Agent Skills" width="720" />
</p>

<p align="center">
  <img src="docs/assets/kit-idle.gif" alt="Kit idle" width="160" />
</p>

<p align="center">
  <strong>One library. Many agents.</strong><br />
  Install skills once. Use them in Claude Code, Codex, and Grok.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@mzwin/kit"><img src="https://img.shields.io/npm/v/@mzwin/kit?style=for-the-badge&label=npm&color=1a1a1a" alt="npm" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
</p>

---

<p align="center">
  <img src="docs/assets/ad-install.png" alt="Install Kit" width="640" />
</p>

```bash
npm i -g @mzwin/kit
kit --version
```

```bash
kit                       # status + next step
kit ready --write         # set up this project
kit unify --write --link  # clean agent skill folders into one library
```

---

<p align="center">
  <img src="docs/assets/demo-unify.gif" alt="kit unify — filter noise, keep keepers" width="640" />
</p>

<p align="center"><sub>Scan agent folders. Drop noise. Keep what earns a place.</sub></p>

<p align="center">
  <img src="docs/assets/ad-unify.png" alt="Unify still" width="480" />
</p>

---

<p align="center">
  <img src="docs/assets/demo-ready.gif" alt="kit ready — one-shot setup" width="640" />
</p>

<p align="center"><sub>Recommend · install · apply · link · doctor. One command.</sub></p>

<p align="center">
  <img src="docs/assets/ad-ready.png" alt="Ready still" width="480" />
</p>

---

<p align="center">
  <img src="docs/assets/demo-link.gif" alt="kit link — library to agents" width="640" />
</p>

<p align="center"><sub>Library → Claude · Codex · Grok.</sub></p>

<p align="center">
  <img src="docs/assets/ad-link.png" alt="Link still" width="480" />
</p>

---

## Starter packs

<p align="center">
  <img src="docs/assets/packs-strip.png" alt="Starter packs" width="720" />
</p>

A pack is a set of skills for one project type.  
Most packs include **essentials**, then add extra skills.

| Pack | Use when | Extra skills (beyond essentials) |
|------|----------|-----------------------------------|
| **essentials** | Any project. Install this first. | — |
| **web-app** | Sites and UI apps | ship-checklist, a11y-pass, pr-ready |
| **library** | Packages and SDKs | api-docs, changelog, pr-ready |
| **cli-tool** | Command-line tools | cli-help, pr-ready |
| **api-service** | HTTP APIs and backends | api-docs, ship-checklist, pr-ready |
| **full-stack** | UI + API products | ship-checklist, a11y-pass, api-docs, pr-ready |
| **data-ml** | Data and ML work | data-check, write-tests, pr-ready |

```bash
kit pack list
kit recommend --dir .
kit pack apply essentials --dir .
```

---

## Skills

Each skill is a short instruction file. Agents load it when the task matches.

| Skill | What it does |
|-------|----------------|
| **add-readme** | Write a clear project README |
| **project-setup** | Set a clean project baseline for agents and humans |
| **workspace-setup** | Set monorepo and multi-package layout |
| **code-review** | Review a change for correctness, risk, and clarity |
| **write-tests** | Add tests for important behavior |
| **fix-bug** | Find root cause and fix a bug without extra refactors |
| **pr-ready** | Write PR summary, test plan, and risk notes |
| **ship-checklist** | Run a pre-ship checklist for an app release |
| **a11y-pass** | Improve basic accessibility for UI and web flows |
| **api-docs** | Document a library or service API with examples |
| **changelog** | Write a clear changelog entry |
| **cli-help** | Improve CLI help text, usage, and flags |
| **data-check** | Review data scripts and notebooks for clarity and reuse |

```bash
kit list
kit pack show essentials
```

Full pack notes: [docs/packs.md](docs/packs.md)

---

<p align="center">
  <img src="docs/assets/ad-commands.png" alt="Main commands" width="640" />
</p>

| Command | Purpose |
|---------|---------|
| `kit` | Show library status and a next command |
| `kit ready --write` | Recommend pack, install, apply, link agents, run doctor |
| `kit unify --write` | Scan Claude/Codex/Grok skills, keep good ones, drop noise |
| `kit unify --write --link` | Same, then link skills into this project |
| `kit recommend --dir .` | Suggest a pack from project files |
| `kit pack apply <name> --dir .` | Copy pack skills into a project |
| `kit link --to all --write` | Link library skills to Claude, Codex, and Grok |
| `kit import --from claude-code --write` | Copy skills from one agent into Kit |
| `kit doctor` | Check install health |
| `kit tui` | Open the pixel terminal UI |

---

## How it works

1. Skills live in a local library (`~/.kit`).
2. Packs install groups of skills into that library.
3. `link` makes those skills available to each agent.
4. `unify` imports and cleans skills that already exist in agent folders.

Agents: **Claude Code** · **Codex** · **Grok Build**.

---

## From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit -- doctor
```

---

<p align="center">
  <img src="docs/assets/kit-success.gif" alt="Kit ready" width="200" />
</p>

<p align="center">
  <img src="docs/assets/kit-wordmark.png" alt="KIT" width="140" /><br />
  <sub>Skills your agents use.</sub>
</p>

<p align="center">
  <sub>
    <a href="LICENSE">MIT</a> ·
    <a href="https://www.npmjs.com/package/@mzwin/kit">npm</a> ·
    <a href="CHANGELOG.md">Changelog</a>
  </sub>
</p>
