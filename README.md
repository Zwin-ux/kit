# Kit

<p align="center">
  <img src="assets/pixel/kit-idle.gif" alt="Kit idle animation — baby fox silhouette" width="200" />
</p>

<p align="center">
  <strong>Portable agent skills.</strong><br />
  One library. Many agents. A pixel TUI with soul.
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#starter-packs">Packs</a> ·
  <a href="#tui">TUI</a> ·
  <a href="#cli">CLI</a> ·
  <a href="docs/STARTER_PACKS.md">Docs</a>
</p>

---

**Kit** is how you install, apply, and wire [Agent Skills](docs/SKILL_SCHEMA.md) so Claude Code, Grok Build, Codex, and friends share the same playbook.

Offline-first engine. Seven starter packs. A black-and-white fox that actually lives in your terminal.

```text
point at a repo  →  ★ auto-recommend  →  ↵ install  →  apply  →  link  →  doctor green
```

---

## Quick start

**Requirements:** Node 20+, [pnpm](https://pnpm.io) 10

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install
pnpm build
```

**Windows**

```powershell
.\kit.cmd recommend --dir .
.\kit.cmd init --pack essentials
.\kit.cmd pack apply essentials --dir .
.\kit.cmd link --to claude-code --write
.\kit.cmd doctor
.\kit.cmd tui
```

**macOS / Linux**

```bash
./kit.ps1 recommend --dir .    # or: pnpm kit -- recommend --dir .
pnpm kit -- init --pack essentials
pnpm kit -- pack apply essentials --dir .
pnpm kit -- link --to claude-code --write
pnpm kit -- doctor
pnpm kit -- tui
```

> There is no global `kit` binary yet. Use `.\kit.cmd` / `pnpm kit --` from the repo after build.

---

## Starter packs

Seven kits for v1. Stack packs **extend essentials** so base skills always install with the stack.

| | Pack | Best for |
|---|------|----------|
| <img src="docs/assets/packs/essentials.png" width="48" alt="essentials" /> | **essentials** | Any repo — default first install |
| <img src="docs/assets/packs/web-app.png" width="48" alt="web-app" /> | **web-app** | Apps and sites |
| <img src="docs/assets/packs/library.png" width="48" alt="library" /> | **library** | Packages and SDKs |
| <img src="docs/assets/packs/cli-tool.png" width="48" alt="cli-tool" /> | **cli-tool** | Developer CLIs |
| <img src="docs/assets/packs/api-service.png" width="48" alt="api-service" /> | **api-service** | HTTP backends |
| <img src="docs/assets/packs/full-stack.png" width="48" alt="full-stack" /> | **full-stack** | UI + API products |
| <img src="docs/assets/packs/data-ml.png" width="48" alt="data-ml" /> | **data-ml** | Data and ML work |

```bash
pnpm kit -- pack list
pnpm kit -- recommend --dir ../my-app
pnpm kit -- pack install web-app
pnpm kit -- pack apply web-app --dir ../my-app
```

Full format: [docs/STARTER_PACKS.md](docs/STARTER_PACKS.md) · icons: [assets/pixel/packs](assets/pixel/packs)

---

## Point → recommend → install

Kit scans the project you point at and suggests a pack **and** individual skills.

```bash
pnpm kit -- recommend --dir ~/code/my-next-app
# → "my-next-app looks like a web app → web-app"
# → skills: a11y-pass, ship-checklist, pr-ready, …
```

In the TUI: **`o`** types a path, Enter saves it, Home shows ★ summary + suggested skills.

---

## TUI

```bash
pnpm tui
# or
.\kit.cmd tui
```

| Key | Action |
|-----|--------|
| `1`–`7` | First-run pack install |
| `↵` | Install selected toolkit |
| `a` | Apply pack to pointed project |
| `o` | Point at a project (auto-recommend) |
| `k` | Paths — pick harness, **approve folder**, then link |
| `d` | Doctor |
| `e` | Explore registry |
| `l` | Library · `v` validate · `t` test |
| `q` | Quit |

Personality stays with **kit-idle**. Text motion is restrained (status, success, selection).  
Reduced motion: `KIT_REDUCED_MOTION=1`.

Screens: [docs/TUI_SCREENS.md](docs/TUI_SCREENS.md)

---

## CLI

| Command | What it does |
|---------|----------------|
| `init --pack <name>` | First-run install |
| `pack list` / `install` / `apply` | Starter packs |
| `recommend --dir <path>` | Auto-suggest pack + skills |
| `paths` / `link --to <harness> --write` | Wire skills into agents |
| `test` / `doctor` | Quality + health |
| `login` / `whoami` / `logout` | GitHub device flow |
| `explore packs` / `search` | Public registry catalog |
| `tui` | Pixel interface |

---

## Skill format

Each skill is a folder with `SKILL.md` — strict front matter, clear body, multi-agent compatibility.

```yaml
---
name: pr-ready
description: Prepare a clear pull request summary, test plan, and risk notes.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---
```

Schema: [docs/SKILL_SCHEMA.md](docs/SKILL_SCHEMA.md) · catalog: [skills/](skills/)

---

## What ships in Alpha

| Area | Status |
|------|--------|
| Local engine (validate, library, packs, link, test, doctor) | ✅ |
| 7 starter packs + silhouette icons | ✅ |
| Pixel TUI + kit-idle + motion | ✅ |
| Auto-recommend by project | ✅ |
| GitHub login + Railway catalog explore | ✅ |
| Workshop / publish / social | ⏳ next |

Roadmap: [ROADMAP.md](ROADMAP.md) · Architecture: [ARCHITECTURE.md](ARCHITECTURE.md)

---

## Develop

```bash
pnpm install
pnpm build
pnpm test
pnpm typecheck
pnpm kit -- doctor
```

Contributing: [CONTRIBUTING.md](CONTRIBUTING.md) · Agents: [AGENTS.md](AGENTS.md)

Built primarily with [Grok Build](https://x.ai).

---

<p align="center">
  <img src="docs/assets/kit-mascot.png" alt="Kit mascot" width="120" /><br />
  <sub>Kit — a little fox for the tools your agents actually use.</sub>
</p>

<p align="center">
  <sub>MIT · <a href="LICENSE">License</a> · <a href="CHANGELOG.md">Changelog</a></sub>
</p>
