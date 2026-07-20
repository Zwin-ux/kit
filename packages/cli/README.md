# @mzwin/kit

**One library. Many agents.**

Your Claude Code / Codex / Grok skills are a mess.  
`kit` unifies them into one portable library — then packs and links them everywhere.

```bash
npm i -g @mzwin/kit
```

---

## Install

```bash
npm i -g @mzwin/kit
kit --version
```

Requires **Node 20+**.

| Package | Role |
|---------|------|
| **`@mzwin/kit`** | CLI (`kit` binary) — install this |
| `@mzwin/kit-core` | Engine (dependency) |
| `@mzwin/kit-catalog` | Official packs + skills (dependency) |
| `@mzwin/kit-tui` | Pixel TUI (dependency) |
| `@mzwin/kit-shared` | Shared types (dependency) |

---

## 60-second start

### Option A — Clean the pile you already have

```bash
kit unify                 # dry-run: scan Claude/Codex/Grok skill dumps
kit unify --write         # adopt S/A keepers into ~/.kit (not automation spam)
kit unify --write --link  # + wire into this project’s agents
```

### Option B — Curated pack

```bash
kit init --pack essentials
kit recommend --dir .
kit pack apply essentials --dir .
kit link --to claude-code --write
kit link --to codex --write
kit doctor
```

### Open the TUI

```bash
kit tui
```

---

## `kit unify` (skill OS)

Vibe coders already installed dozens or hundreds of skills. None travel between agents. Most are noise.

```text
UNIFY  skill OS  (dry-run)

  Scanned   998 skill folders
  Noise     809 filtered
  Keepers   4   (S/A · multi-agent or real structure)

  Safe default: adopt keepers — not bulk *-automation dumps.
```

| Flag | Effect |
|------|--------|
| *(none)* | Dry-run report |
| `--write` | Adopt **keepers only** into `~/.kit` |
| `--link` | With `--write`, copy into project harness folders |
| `--all` | Include noise (not recommended) |
| `--json` | Machine-readable report |

---

## Everyday commands

```bash
kit unify
kit unify --write --link
kit init --pack essentials
kit recommend --dir .
kit pack list
kit pack apply <pack> --dir .
kit link --to claude-code --write
kit import --from claude-code --write
kit doctor
kit tui
```

| Command | Job |
|---------|-----|
| `unify` | Rank + clean skill mess → portable library |
| `init` / `pack` | Curated starter packs |
| `recommend` | Suggest pack for a repo |
| `link` | Kit → Claude / Codex / Grok |
| `import` | One harness → Kit |
| `doctor` | Health check |
| `tui` | Pixel terminal UI |

Harness targets: `claude-code` · `codex` · `grok-build`

---

## Starter packs

`essentials` · `web-app` · `library` · `cli-tool` · `api-service` · `full-stack` · `data-ml`

Stack packs extend **essentials**.

```bash
kit pack list
kit pack install web-app
kit pack apply web-app --dir ../my-app
```

---

## Skill format

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

---

## Links

- **GitHub:** [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)
- **npm:** [npmjs.com/package/@mzwin/kit](https://www.npmjs.com/package/@mzwin/kit)
- **License:** MIT

```bash
npm i -g @mzwin/kit
kit unify
```
