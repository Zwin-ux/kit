<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — Portable Agent Skills" width="720" />
</p>

<p align="center">
  <img src="docs/assets/kit-idle.gif" alt="Kit idle — pixel fox mascot" width="200" />
</p>

<p align="center">
  <strong>One library. Many agents.</strong><br />
  Built for the vibe-coding boom — when you already have too many skills<br />
  and zero portability between Claude Code, Codex, and Grok.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@mzwin/kit"><img src="https://img.shields.io/npm/v/@mzwin/kit?style=for-the-badge&label=npm&color=1a1a1a" alt="npm" /></a>
  <a href="#user-stories"><img src="https://img.shields.io/badge/stories-ready_·_unify-c45c2a?style=for-the-badge" alt="stories" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
</p>

---

## Install

```bash
npm i -g @mzwin/kit
kit                 # situation-aware home — tells you what to run
```

**Package:** [@mzwin/kit](https://www.npmjs.com/package/@mzwin/kit) · **Node 20+**

---

## User stories

### 1. Clean the skill pile

**Who:** You’ve installed every Claude/Codex skill you saw.  
**Pain:** 200 folders, half automation spam, nothing shared across agents.  
**Win:**

```bash
kit unify
kit unify --write
kit unify --write --link
```

Ranks keepers, filters noise, builds one portable library.

### 2. Make this repo agent-ready

**Who:** You just opened a project and want agents useful *now*.  
**Pain:** Don’t want a 12-step ritual every new repo.  
**Win:**

```bash
cd my-app
kit ready              # plan
kit ready --write      # pack + apply + link + doctor
kit ready --write --unify   # also clean personal skill dumps
```

### 3. Same skills in every agent

**Who:** Claude by day, Codex by night, same codebase.  
**Pain:** Skills only live where you installed them.  
**Win:**

```bash
kit init --pack essentials
kit link --to all --write
kit paths
```

### 4. Brand new

```bash
npm i -g @mzwin/kit
kit
kit init --pack essentials
# or: kit unify   if you already have skills elsewhere
```

---

## Why Kit (not another skill dump)

| Default world | With Kit |
|---------------|----------|
| Skills stuck in one agent | One library → Claude + Codex + Grok |
| Install more skills | **Curate** the ones you already have |
| Guess the pack | `recommend` from real project signals |
| Manual folder copy | `link` / `ready` / `unify --link` |

---

## Commands

| Command | Job |
|---------|-----|
| `kit` | Home — your story + next command |
| `kit ready --write` | One-shot: this repo is agent-ready |
| `kit unify --write` | Skill OS: mess → keepers → library |
| `kit recommend --dir .` | Best starter pack for this project |
| `kit pack apply …` | Land a pack on a project |
| `kit link --to all --write` | Broadcast library to agents |
| `kit import --from claude-code --write` | Pull one harness in |
| `kit doctor` / `kit tui` | Health + pixel UI |

---

## Starter packs

Seven official packs (stack packs extend **essentials**):

`essentials` · `web-app` · `library` · `cli-tool` · `api-service` · `full-stack` · `data-ml`

```bash
kit pack list
kit pack apply web-app --dir ../my-app
```

→ [docs/packs.md](docs/packs.md)

---

## Demo (presentation)

```bash
npm i -g @mzwin/kit
kit                          # “here’s your situation”
kit unify                    # gasp: 900 scanned, noise filtered
kit ready --write            # this repo is done
kit doctor
```

---

## From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit -- ready
```

---

<p align="center">
  <img src="docs/assets/kit-wordmark.png" alt="KIT" width="140" /><br />
  <img src="docs/assets/kit-mascot.png" alt="Kit mascot" width="100" /><br />
  <sub>Skills your agents actually use.</sub>
</p>

<p align="center">
  <sub>
    <a href="LICENSE">MIT</a> ·
    <a href="https://www.npmjs.com/package/@mzwin/kit">npm @mzwin/kit</a> ·
    <a href="CHANGELOG.md">Changelog</a>
  </sub>
</p>
