<p align="center">
  <img src="docs/assets/readme-banner.png" alt="KIT — multi-agent control room" width="720" />
</p>

<p align="center">
  <strong>Dispatch many agents. Watch them in one place. Nothing ships unproven.</strong><br />
  Local Control Room for Codex, Claude, Grok, and Ollama — worktrees, gates, receipts.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
  <img src="https://img.shields.io/badge/status-1.0%20alpha-00E6CC?style=for-the-badge" alt="1.0 alpha" />
</p>

---

## 30 seconds

```bash
# From this repo
cargo run -p kit-cli -- doctor          # which agents are ready
cargo run -p kit-cli -- --demo          # Control Room with FAIL + retry fixture
cargo run -p kit-cli                    # empty room; d = dispatch live agents
cargo run -p kit-cli -- run --dry-run --task "smoke" --json
```

| Command | What it does |
|---------|----------------|
| `kit` / `kit --demo` | Control Room TUI |
| `kit run --task "…"` | One isolated run (live agent if on PATH) |
| `kit doctor` | Probe codex / claude / grok / ollama + skills pack |
| `kit run --dry-run --json` | Offline CI path (worktree → stream → gate → receipt) |
| `kit receipt list` | Browse proof under `~/.kit/runs/` |
| `kit receipt show <id>` | One receipt (+ `--output` for log tail) |

**Product loop:** dispatch → table of runs → FAIL wash + first error → `r` retry with gate context → receipt under `~/.kit/runs/<id>/`.

Keys: `↑↓` select · `Enter` open · `g` gate · `d` dispatch · `b` board · `k` kill · `r` retry · `?` help · `q` quit.

---

## Install (1.0 alpha)

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
cargo build -p kit-cli --release
./target/release/kit doctor
./target/release/kit --demo
```

Requires Rust stable and (for live runs) at least one of: `codex`, `claude`, `grok`, `ollama` on PATH.  
Kit uses each provider's existing login. It does not store model API keys.

Env flags:

| Env | Effect |
|-----|--------|
| `KIT_FULL_AUTO=1` | Bypass agent approval prompts (dangerous — sandboxes only) |
| `KIT_SKILLS_DIR=…` | Override skill pack root (see Skills) |
| `KIT_OLLAMA_MODEL=…` | Model for Ollama adapter (default `llama3.2`) |
| `NO_COLOR` / `KIT_MOTION=off` | Monochrome / reduced motion |

Architecture: [`docs/dev/CURRENT.md`](docs/dev/CURRENT.md) · PRD: [`docs/dev/PRD-1.0.md`](docs/dev/PRD-1.0.md)

---

## Skills

Every live run copies a skill pack into the worktree and prepends routing to the prompt.

**Default:** [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) at `.agents/skills` (coding lifecycle: spec → plan → build → verify → review).

**Any `*/SKILL.md` pack works**, including [Harness skills](https://github.com/harness/harness-skills):

```bash
git clone https://github.com/harness/harness-skills.git
# Point Kit at the skills tree (not the repo root)
export KIT_SKILLS_DIR="$PWD/harness-skills/skills"
cargo run -p kit-cli -- run --agent claude --task "debug my failed pipeline"
```

Or place a `skills/` directory (Harness layout) in the target repo — Kit discovers it after `.agents/skills`.

**Harness note:** those skills expect the [Harness MCP v2 server](https://github.com/harness/mcp-server) and API credentials. They are a **domain pack**, not Kit's default. Without MCP, agents can still read the markdown but cannot call Harness tools.

Resolution order: `KIT_SKILLS_DIR` → `<repo>/.agents/skills` → `<repo>/skills` → walk up from cwd.

---

## Control Room

- Live table of runs (repo · agent · task · STATE · GATE)
- FAIL rows get a danger wash and a `^ first error` annotation
- Dispatch fans out repo × agent (cap 16) with one task
- Board is prefill-only in 1.0 (Enter → Dispatch)
- Gate: empty checks render **UNCONFIGURED**, never silent PASS
- Kill mid-run (`k`) and retry failed (`r`) with gate failure context

```bash
cargo run -p kit-cli -- --demo   # lands on a FAIL so the proof loop is obvious
```

## Receipts

Every run writes `~/.kit/runs/<id>/` (`receipt.json`, `output.log`, optional `diff.patch` / `gate.json`).

```bash
cargo run -p kit-cli -- receipt list
cargo run -p kit-cli -- receipt show 01KZD0… --output
cargo run -p kit-cli -- receipt list --json --limit 10
```

Override data root with `KIT_HOME`. Contract: [`docs/json-contract.md`](docs/json-contract.md).  
Dogfood path: [`docs/dev/DOGFOOD.md`](docs/dev/DOGFOOD.md).

---

## Legacy: Kit 0.1.x npm workbench

Earlier alpha (`npm i -g @mzwin/kit`) was a Node skill/workbench TUI. Still in `packages/` for history; **1.0 is the Rust binary above**. Prefer Control Room commands.

<details>
<summary>0.1 workbench notes</summary>

```bash
npm i -g @mzwin/kit
kit ready --write
kit tui workbench
```

See [Workbench architecture](docs/dev/WORKBENCH_ARCHITECTURE.md) for the archived design.

</details>

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

## Attach a CLI service

Kit can run a local CLI from its own checkout. The plugin supplies one small
manifest.

```bash
kit plugin add ../trenchwire --write
kit plugin doctor trenchwire
kit plugin task trenchwire
kit plugin task trenchwire health
kit plugin task trenchwire market
```

Kit stores the plugin path and manifest digest. It does not copy the binary.
Kit passes arguments without a shell. The plugin keeps its own safety rules.
Kit stops if the manifest changes after registration.

Read the [plugin contract](docs/plugins.md).

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
| **completeness-qa** | Inventory public functions, flag stubs, name the next SWE skill |
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
| `kit plugin add <path> --write` | Register a local CLI |
| `kit plugin doctor <name>` | Check its binary and manifest |
| `kit plugin task <name> [task]` | List or run a fixed read-only task |
| `kit plugin run <name> -- <args>` | Run it without a shell |
| `kit doctor` | Install health |
| `kit tui` | Terminal UI |
| `kit tui workbench` | Coding runners and attached services |

---

## How it works

1. Skills and plugin registrations live in `~/.kit`.
2. Packs install groups into the skill library.
3. `link` exposes skills to each coding runner.
4. Workbench starts provider CLIs and service tasks without a shell.
5. `unify` imports and cleans skills already in agent folders.

**TUI:** menu screens keep the mascot in a fixed rail. Workbench gives the job
and output the full terminal. `KIT_REDUCED_MOTION=1` freezes motion.

Agents: **Claude Code** · **Codex** · **Grok Build**.

---

## From source

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
pnpm install && pnpm build
pnpm kit doctor
pnpm kit tui
# or:  pnpm tui
# also: pnpm kit -- tui   (npm-style; leading -- is stripped)
```

Run from the **repo root**, not your home folder. `kit tui` needs an interactive terminal (real TTY).

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
