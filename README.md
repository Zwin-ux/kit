<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — Portable Agent Skills" width="720" />
</p>

<p align="center">
  <img src="docs/assets/kit-idle.gif" alt="Kit" width="140" />
</p>

<p align="center">
  <strong>One skill library. Claude, Codex, and Grok.</strong><br />
  Install once. Link everywhere. Clean the mess when agents pile up.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@mzwin/kit"><img src="https://img.shields.io/npm/v/@mzwin/kit?style=for-the-badge&label=npm&color=1a1a1a" alt="npm" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
</p>

---

## Install

```bash
npm i -g @mzwin/kit
kit --version
```

Node 20+. Command name: `kit`.

```bash
kit                  # status + next command
kit ready --write    # pack → install → apply → link → doctor
kit unify --write --link
kit tui              # keyboard console + calm side mascot
```

---

<p align="center">
  <img src="docs/assets/ad-install.png" alt="Install" width="640" />
</p>

---

## Clean agent skill folders

<p align="center">
  <img src="docs/assets/demo-unify.gif" alt="kit unify" width="640" />
</p>

<p align="center"><sub>Scan · filter noise · keep what earns a place.</sub></p>

```bash
kit unify
kit unify --write
kit unify --write --link
```

---

## One-shot project setup

<p align="center">
  <img src="docs/assets/demo-ready.gif" alt="kit ready" width="640" />
</p>

<p align="center"><sub>Recommend · install · apply · link · doctor.</sub></p>

```bash
kit ready
kit ready --write
kit ready --write --unify
```

---

## Link the library to agents

<p align="center">
  <img src="docs/assets/demo-link.gif" alt="kit link" width="640" />
</p>

<p align="center"><sub>Library → Claude · Codex · Grok.</sub></p>

```bash
kit link --to all --write
kit import --from claude-code --write
```

---

## Starter packs

<p align="center">
  <img src="docs/assets/packs-strip.png" alt="Starter packs" width="720" />
</p>

A pack is a skill set for one project type. Most extend **essentials**.

| Pack | Use when | Extra skills |
|------|----------|--------------|
| **essentials** | Any project. Start here. | — |
| **web-app** | Sites and UI | ship-checklist, a11y-pass, pr-ready |
| **library** | Packages and SDKs | api-docs, changelog, pr-ready |
| **cli-tool** | CLIs | cli-help, pr-ready |
| **api-service** | HTTP APIs | api-docs, ship-checklist, pr-ready |
| **full-stack** | UI + API | ship-checklist, a11y-pass, api-docs, pr-ready |
| **data-ml** | Data / ML | data-check, write-tests, pr-ready |

```bash
kit pack list
kit recommend --dir .
kit pack apply essentials --dir .
```

---

## Skills

Short instruction files. Agents load them when the task matches.

| Skill | Does |
|-------|------|
| **add-readme** | Project README |
| **project-setup** | Clean baseline for agents and humans |
| **workspace-setup** | Monorepo / multi-package layout |
| **code-review** | Correctness, risk, clarity |
| **write-tests** | Tests for important behavior |
| **fix-bug** | Root cause + fix without drive-by refactors |
| **pr-ready** | PR summary, test plan, risk |
| **ship-checklist** | Pre-ship checklist |
| **a11y-pass** | Basic UI accessibility |
| **api-docs** | API docs with examples |
| **changelog** | Changelog entry |
| **cli-help** | Help text, usage, flags |
| **data-check** | Data scripts / notebooks |

```bash
kit list
kit pack show essentials
```

More: [docs/packs.md](docs/packs.md)

---

## Commands

| Command | Purpose |
|---------|---------|
| `kit` | Status + next step |
| `kit ready --write` | Make this repo agent-ready |
| `kit unify --write` | Clean Claude/Codex/Grok skill dumps |
| `kit unify --write --link` | Clean + link into the project |
| `kit recommend --dir .` | Suggest a pack |
| `kit pack apply <name> --dir .` | Apply pack skills |
| `kit link --to all --write` | Link library to agents |
| `kit import --from claude-code --write` | Import from one agent |
| `kit doctor` | Install health |
| `kit tui` | Terminal UI |

---

## How it works

1. Skills live in `~/.kit`.
2. Packs install groups into that library.
3. `link` exposes them to each agent.
4. `unify` imports and cleans skills already in agent folders.

**TUI:** fixed side rail for the mascot (animation does not resize the menu). ↑↓ shows direction. `KIT_REDUCED_MOTION=1` freezes motion.

Agents: **Claude Code** · **Codex** · **Grok Build**.

---

## From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit -- doctor
pnpm kit -- tui
```

Run builds and local `kit` from the **repo root** (`agent-sandbox/projects/kit`), not your home folder.

---

<p align="center">
  <img src="docs/assets/kit-success.gif" alt="ready" width="180" />
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
