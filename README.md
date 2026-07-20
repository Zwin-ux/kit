<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — Portable Agent Skills" width="720" />
</p>

<p align="center">
  <img src="docs/assets/kit-idle.gif" alt="Kit idle — pixel fox mascot" width="200" />
</p>

<p align="center">
  <strong>One library. Many agents.</strong><br />
  Point Kit at a project. Install a starter pack.<br />
  Wire skills into Claude Code, Grok Build, and Codex.
</p>

<p align="center">
  <a href="#install"><img src="https://img.shields.io/badge/install-1a1a1a?style=for-the-badge" alt="Install" /></a>
  <a href="#starter-packs"><img src="https://img.shields.io/badge/7_packs-1a1a1a?style=for-the-badge" alt="7 packs" /></a>
  <a href="#open-the-tui"><img src="https://img.shields.io/badge/pixel_tui-1a1a1a?style=for-the-badge" alt="Pixel TUI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-c45c2a?style=for-the-badge" alt="MIT" /></a>
</p>

---

## What Kit does

<p align="center">
  <img src="docs/assets/kit-flow.gif" alt="Point → Recommend → Install → Link" width="560" />
</p>

| | |
|:--|:--|
| **1. Point** | Aim Kit at any project folder |
| **2. Recommend** | Get the right pack + skills for that repo |
| **3. Install** | Drop curated skills into your library |
| **4. Link** | Connect Claude Code, Grok Build, or Codex |

<p align="center">
  <img src="docs/assets/readme-terminal.png" alt="kit recommend → apply → link" width="560" />
</p>

Same skills. Multiple agents. No rewriting prompts per tool.

---

## Install

**Need:** [Node](https://nodejs.org) 20+

### Global CLI (recommended)

```bash
npm i -g @mzwin/kit

kit init --pack essentials
kit recommend --dir .
kit pack apply essentials --dir .
kit link --to claude-code --write
kit doctor
kit tui
```

Wire other agents the same way:

```bash
kit link --to codex --write
kit link --to grok-build --write
```

### Capture skills you already use

Import skills from Claude Code / Codex / Grok folders into your Kit library:

```bash
kit import --from claude-code          # dry-run
kit import --from claude-code --write  # install into ~/.kit
kit import --from all --write
```

### Skills-only (any agent that supports SKILL.md)

```bash
npx skills add Zwin-ux/kit
```

### From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit -- doctor
```

---

## Starter packs

Seven kits. Stack packs extend **essentials**, so the basics always come along.

<p align="center">
  <img src="docs/assets/packs/essentials.png" width="48" alt="essentials" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/web-app.png" width="48" alt="web-app" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/library.png" width="48" alt="library" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/cli-tool.png" width="48" alt="cli-tool" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/api-service.png" width="48" alt="api-service" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/full-stack.png" width="48" alt="full-stack" />
  &nbsp;&nbsp;
  <img src="docs/assets/packs/data-ml.png" width="48" alt="data-ml" />
</p>

| | Pack | Best for |
|:---:|:-----|:---------|
| <img src="docs/assets/packs/essentials.png" width="36" alt="" /> | **essentials** | Any repo — start here |
| <img src="docs/assets/packs/web-app.png" width="36" alt="" /> | **web-app** | Apps and sites |
| <img src="docs/assets/packs/library.png" width="36" alt="" /> | **library** | Packages and SDKs |
| <img src="docs/assets/packs/cli-tool.png" width="36" alt="" /> | **cli-tool** | Developer CLIs |
| <img src="docs/assets/packs/api-service.png" width="36" alt="" /> | **api-service** | HTTP backends |
| <img src="docs/assets/packs/full-stack.png" width="36" alt="" /> | **full-stack** | UI + API products |
| <img src="docs/assets/packs/data-ml.png" width="36" alt="" /> | **data-ml** | Data and ML work |

```bash
pnpm kit -- recommend --dir ../my-app
pnpm kit -- pack install web-app
pnpm kit -- pack apply web-app --dir ../my-app
```

More detail: [docs/packs.md](docs/packs.md)

---

## Open the TUI

```bash
pnpm tui
# Windows: .\kit.cmd tui
```

| Key | Action |
|:---:|:-------|
| **1–7** | Pick a starter pack |
| **↵** | Install the selected pack |
| **o** | Point at a project |
| **a** | Apply pack to that project |
| **k** | Link an agent harness |
| **d** | Doctor |
| **q** | Quit |

<p align="center">
  <img src="docs/assets/kit-idle.gif" width="120" alt="kit-idle" />
</p>

---

## Everyday commands

| Command | What it does |
|---------|----------------|
| `init --pack essentials` | First-run install |
| `recommend --dir <path>` | Suggest pack + skills |
| `pack list` / `install` / `apply` | Manage packs |
| `link --to claude-code --write` | Kit → agent harness |
| `import --from claude-code --write` | Agent harness → Kit library |
| `doctor` | Health check |
| `tui` | Pixel interface |

Harness targets: `claude-code` · `grok-build` · `codex`

---

## Skills

Skills are plain markdown with a short front matter block:

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

Browse the built-in set in [`skills/`](skills/).

---

<p align="center">
  <img src="docs/assets/kit-wordmark.png" alt="KIT" width="140" /><br />
  <img src="docs/assets/kit-mascot.png" alt="Kit mascot" width="100" /><br />
  <sub>Skills your agents actually use.</sub>
</p>

<p align="center">
  <sub>
    <a href="LICENSE">MIT</a>
    ·
    <a href="https://www.npmjs.com/package/@mzwin/kit">npm</a>
    ·
    <a href="CHANGELOG.md">Changelog</a>
    ·
    <a href="CONTRIBUTING.md">Contributing</a>
  </sub>
</p>
