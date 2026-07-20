# @mzwin/kit

**One library. Many agents.**

Built for the vibe-coding boom: you run Claude Code, Codex, maybe Grok —  
and your skills are scattered, duplicated, and stuck in one tool.

```bash
npm i -g @mzwin/kit
kit                 # tells you YOUR next move
```

---

## Install

```bash
npm i -g @mzwin/kit
kit --version
```

Requires **Node 20+**.

---

## Who is this for? (real stories)

### 1) “I have 200 skills and I trust none of them”

You installed every Claude/Codex skill you saw. Half are `*-automation` junk.  
Nothing works the same in both agents.

```bash
kit unify                 # see mess vs keepers
kit unify --write         # adopt S/A keepers only
kit unify --write --link  # + wire into this project
```

### 2) “I just opened a repo — make agents useful”

You don’t want a 12-step setup. You want Claude/Codex competent on *this* folder.

```bash
cd my-app
kit ready                 # dry-run plan
kit ready --write         # pack + apply + link + doctor
kit ready --write --unify # also clean your personal skill pile
```

### 3) “I bounce between Claude and Codex every day”

Same product, two agents, two skill folders. Kit is the shared library.

```bash
kit init --pack essentials
kit link --to all --write
kit paths
```

### 4) “Brand new install”

```bash
npm i -g @mzwin/kit
kit                       # situation-aware home
kit init --pack essentials
# or if you already have skills elsewhere:
kit unify
```

---

## Commands that matter

| Command | When you use it |
|---------|-----------------|
| `kit` | Open Kit — reads your setup, names your story, gives next command |
| `kit ready --write` | One-shot: this repo is agent-ready |
| `kit unify --write` | Clean chaotic personal skills → portable library |
| `kit recommend --dir .` | What pack fits this project? |
| `kit pack apply <name> --dir .` | Land pack skills on a project |
| `kit link --to all --write` | Broadcast library → Claude/Codex/Grok |
| `kit import --from claude-code --write` | Pull one harness into Kit |
| `kit doctor` | Health check |
| `kit tui` | Pixel terminal UI |

---

## `kit unify` (skill OS)

```text
Scanned   998 skill folders
Noise     809 filtered   (automation bulk, stubs)
Keepers   4              (S/A · multi-agent or real structure)

Safe default: adopt keepers — not hundreds of dumps.
```

| Flag | Effect |
|------|--------|
| *(none)* | Dry-run |
| `--write` | Adopt keepers into `~/.kit` |
| `--link` | Also copy into project harness folders |
| `--all` | Include noise (not recommended) |
| `--json` | Machine report |

---

## Starter packs

`essentials` · `web-app` · `library` · `cli-tool` · `api-service` · `full-stack` · `data-ml`

```bash
kit pack list
kit pack apply web-app --dir ../my-app
```

---

## Links

- GitHub: [github.com/Zwin-ux/kit](https://github.com/Zwin-ux/kit)
- npm: [npmjs.com/package/@mzwin/kit](https://www.npmjs.com/package/@mzwin/kit)
- License: MIT

```bash
npm i -g @mzwin/kit
kit
kit ready --write
```
