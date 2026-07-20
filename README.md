<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — Portable Agent Skills" width="720" />
</p>

<p align="center">
  <img src="docs/assets/kit-idle.gif" alt="Kit idle — pixel fox mascot" width="200" />
</p>

<p align="center">
  <strong>One library. Many agents.</strong><br />
  Your agents already have skills. They’re a mess.<br />
  Kit unifies them — then packs and links them everywhere.
</p>

<p align="center">
  <a href="#install"><img src="https://img.shields.io/badge/npm-@mzwin/kit-1a1a1a?style=for-the-badge" alt="npm" /></a>
  <a href="#kit-unify"><img src="https://img.shields.io/badge/kit_unify-c45c2a?style=for-the-badge" alt="kit unify" /></a>
  <a href="#starter-packs"><img src="https://img.shields.io/badge/7_packs-1a1a1a?style=for-the-badge" alt="7 packs" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
</p>

---

## Install

**Need:** [Node](https://nodejs.org) 20+

```bash
npm i -g @mzwin/kit
kit --version
```

```bash
# also fine
npx @mzwin/kit --help
```

**Package:** [`@mzwin/kit`](https://www.npmjs.com/package/@mzwin/kit) · bin: `kit`  
**Repo:** [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)

That’s it. Global `kit` command after install.

---

## `kit unify`

**The feature for right now.** Vibe coders already installed 50–900 skills across Claude Code and Codex. None of them travel. Most of them are noise.

```bash
kit unify
```

```text
UNIFY  skill OS  (dry-run)

  Scanned   998 skill folders  (Claude · Codex · Grok · Kit)
  Unique    940
  Noise     809 filtered
  Keepers   4  (grade S/A · real structure or multi-agent)

  Top keepers
  A  careful         claude+codex   Safety guardrails…
  A  freeze          claude+codex   Restrict edits…
  …

  Safe default: adopt keepers — not 809 automation dumps.
```

```bash
kit unify --write           # adopt S/A keepers into ~/.kit
kit unify --write --link    # + wire into this project’s agents
```

| Flag | What it does |
|------|----------------|
| *(none)* | Dry-run: scan, rank, show mess vs keepers |
| `--write` | Adopt **keepers only** (not bulk noise) |
| `--link` | With `--write`, copy into project Claude/Codex/Grok folders |
| `--all` | Include noise (power user; not recommended) |
| `--json` | Machine-readable report |

Kit normalizes messy `SKILL.md` files (missing compatibility, bad names, long descriptions) into a portable schema, then dedupes by name across harnesses.

---

## Full loop

```bash
npm i -g @mzwin/kit

# 1) Clean the pile you already have
kit unify
kit unify --write

# 2) Or start from a curated pack
kit init --pack essentials
kit recommend --dir .
kit pack apply essentials --dir .

# 3) Wire agents
kit link --to claude-code --write
kit link --to codex --write
kit link --to grok-build --write

kit doctor
kit tui
```

### Capture one harness only

```bash
kit import --from claude-code          # dry-run
kit import --from claude-code --write
```

### Skills-only install (any SKILL.md agent)

```bash
npx skills add Zwin-ux/kit
```

---

## How it works

<p align="center">
  <img src="docs/assets/kit-flow.gif" alt="Point → Recommend → Install → Link" width="560" />
</p>

| | |
|:--|:--|
| **Unify** | Skill dumps → one ranked library |
| **Recommend** | Point at a repo → right starter pack |
| **Packs** | Curated skill sets (7 official) |
| **Link** | Kit → Claude Code / Codex / Grok |
| **Import** | Harness → Kit (single-direction capture) |

---

## Starter packs

Seven kits. Stack packs extend **essentials**.

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
kit recommend --dir ../my-app
kit pack install web-app
kit pack apply web-app --dir ../my-app
```

More: [docs/packs.md](docs/packs.md)

---

## Everyday commands

| Command | What it does |
|---------|----------------|
| `unify` | Rank + clean skill mess → portable library |
| `init --pack essentials` | First-run pack install |
| `recommend --dir <path>` | Suggest pack + skills |
| `pack list` / `install` / `apply` | Manage packs |
| `link --to claude-code --write` | Kit → agent |
| `import --from claude-code --write` | Agent → Kit |
| `doctor` | Health check |
| `tui` | Pixel interface |

---

## Skills

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

Browse built-ins in [`skills/`](skills/).

---

## From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit -- unify
pnpm kit -- doctor
```

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
    <a href="https://www.npmjs.com/package/@mzwin/kit">npm @mzwin/kit</a>
    ·
    <a href="CHANGELOG.md">Changelog</a>
    ·
    <a href="CONTRIBUTING.md">Contributing</a>
  </sub>
</p>
