# @mzwin/kit

**One library. Many agents.**

Install skills once. Use them in Claude Code, Codex, and Grok.

```bash
npm i -g @mzwin/kit
kit --version
```

Requires **Node 20+**.

---

## Quick start

```bash
kit                       # status and next step
kit ready --write         # set up this project for agents
kit unify --write --link  # clean agent skill folders into one library
```

---

## Packs

A pack is a set of skills for one project type.  
Most packs include **essentials**, then add more skills.

| Pack | Use when | Adds (on top of essentials) |
|------|----------|-----------------------------|
| **essentials** | Any project. Start here. | base set only |
| **web-app** | Sites and UI apps | ship-checklist, a11y-pass, pr-ready |
| **library** | Packages and SDKs | api-docs, changelog, pr-ready |
| **cli-tool** | Command-line tools | cli-help, pr-ready |
| **api-service** | HTTP APIs | api-docs, ship-checklist, pr-ready |
| **full-stack** | UI + API products | ship-checklist, a11y-pass, api-docs, pr-ready |
| **data-ml** | Data and ML work | data-check, write-tests, pr-ready |

```bash
kit pack list
kit recommend --dir .
kit pack apply essentials --dir .
```

---

## Skills

Each skill is a short instruction file for a common task.

| Skill | Purpose |
|-------|---------|
| **add-readme** | Write a clear project README |
| **project-setup** | Set a clean project baseline |
| **workspace-setup** | Set monorepo / multi-package layout |
| **code-review** | Review a change for correctness and risk |
| **write-tests** | Add tests for important behavior |
| **fix-bug** | Find root cause and fix a bug |
| **pr-ready** | Write PR summary, test plan, and risks |
| **ship-checklist** | Run a pre-ship checklist |
| **a11y-pass** | Improve basic UI accessibility |
| **api-docs** | Document a library or service API |
| **changelog** | Write a clear changelog entry |
| **cli-help** | Improve CLI help and usage text |
| **data-check** | Review data scripts and notebooks |

```bash
kit list
kit pack show web-app
```

---

## Commands

| Command | Purpose |
|---------|---------|
| `kit` | Show status and a next command |
| `kit ready --write` | Install pack, apply to project, link agents, run doctor |
| `kit unify --write` | Import good skills from Claude/Codex/Grok; skip noise |
| `kit unify --write --link` | Same, then link into this project |
| `kit recommend --dir .` | Suggest a pack from project files |
| `kit pack apply <name> --dir .` | Apply a pack to a project |
| `kit link --to all --write` | Link library skills to all agents |
| `kit import --from claude-code --write` | Copy skills from one agent into Kit |
| `kit doctor` | Check health |
| `kit tui` | Open the pixel terminal UI |

---

## How it works

1. Kit stores skills in `~/.kit`.
2. Packs install groups of skills.
3. `link` makes skills available to each agent.
4. `unify` imports and cleans skills that already exist in agent folders.

---

## Links

- GitHub: [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)
- npm: [npmjs.com/package/@mzwin/kit](https://www.npmjs.com/package/@mzwin/kit)
- License: MIT

```bash
npm i -g @mzwin/kit
kit pack list
```
