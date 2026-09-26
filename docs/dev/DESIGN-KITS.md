# Kits: the full design

Status: design for v0.1, 2026-09-26. Builds on `PRODUCT-REVIEW-KITS.md`
(why kits) and `PRODUCT-DESIGN-REVIEW.md` (the run and proof half).
Decisions already taken: kits lead the product; the marketplace starts as a
public Git repo of kits; kits point at upstream skills by pinned commit.
Package name for `cargo install` is **`kitctl`** (binary `kit`).

Agent config locations and kit sources come from the "Open-source skills
for kits" research thread (`kit-1.0/kit-catalogue.md`). What nobody has
tried yet is marked **(unverified)**.

Contents:

1. The first five minutes, screen by screen
2. What a kit is: `KIT.toml`
3. The translation layer (the centre of the design)
4. Composition and lifecycle
5. Trust
6. The catalogue and what qualifies as a kit
7. v0.1 cut and build order

---

## 1. The first five minutes

The target: from `npm i -g @mzwin/kit` to an agent that behaves
differently, in under five minutes, with no file editing and no surprises.

### Screen 0: install

```console
$ npm i -g @mzwin/kit
added 2 packages in 3s
$ kit
```

### Screen 1: welcome and detection

Bare `kit` in a terminal, with no `~/.kit/config.toml`, starts setup.
Detection runs the same probes as `kit doctor` (under a second).

```text
Kit sets your coding agents up for a job, and proves what they do.

Looking for agents on this machine…
  Claude Code   2.1.283   logged in
  Codex         not found
  Grok          not found
```

If nothing is found, setup stops here with one line per agent:

```text
No coding agent found. Install one, then run kit again:
  Claude Code   npm i -g @anthropic-ai/claude-code
  Codex         npm i -g @openai/codex
  Grok          see https://github.com/… (verify)
```

### Screen 2: agents

```text
Which agents should Kit set up?   ↑↓ move · space toggle · enter confirm
> [x] Claude Code
  [ ] Codex        not installed (Kit can still write its files)
  [ ] Grok         not installed
```

Installed agents are ticked. An uninstalled agent can be ticked (dotfiles
for a machine that will have it), and says so.

### Screen 3: focus

```text
What should your agents focus on?   space toggle · enter confirm · ? details
> [x] Frontend Design     UI, design systems, accessibility, browser testing
  [ ] Full-stack Design   Frontend Design plus APIs, data, end-to-end tests
  [ ] Backend Engineer    APIs, databases, security, performance
  [ ] LLM Engineer        prompts, evals, RAG, model APIs
      More kits: kit search
```

`?` on a row shows that kit's contents (Screen 5 for one kit), so nobody
picks blind.

### Screen 4: scope

```text
Install for
> (•) All my projects     your agents use it everywhere
  ( ) This repo only      ~/code/shop (committed, shared with your team)
```

The second option appears only inside a git repo. Default is all
projects, because setting up "my agent" is personal.

### Screen 5: the plan

Nothing has been written yet. Every line is a real path.

```text
Frontend Design 0.1.0  (extends Essentials 0.1.0)  → Claude Code, all projects

  skills  11   ~/.claude/skills/
                 frontend-ui-engineering     addyosmani/agent-skills@9c1e2f0  MIT
                 browser-testing-with-devtools  addyosmani/agent-skills@9c1e2f0  MIT
                 accessibility-pass          Zwin-ux/kits@4b7a1d2             MIT
                 … 8 more (kit show frontend-design)
  rules        ~/.claude/CLAUDE.md          + 14 lines in a "kit:frontend-design" block
  mcp          playwright                    npx -y @playwright/mcp@0.0.41       RUNS CODE
  hooks        after each edit               npx prettier --write "$FILE"        RUNS CODE
  check        kit doctor: skills present, playwright MCP starts

Runs code on your machine: 1 MCP server, 1 hook.
Continue?  [y] install  [n] cancel  [s] install without code (skills and rules only)
```

`[s]` is the trust escape hatch: everything that only steers the agent,
nothing that executes.

### Screen 6: install and verify

```text
Fetching  addyosmani/agent-skills@9c1e2f0 … ok (cached)
Writing   11 skills · CLAUDE.md block · playwright MCP · 1 hook
Checking  skills readable · playwright MCP starts · hook command found … ok

Done. Claude Code has Frontend Design in every project.
Saved to ~/.kit/config.toml and ~/.kit/kit.lock.

Try it:
  claude "build a pricing page with three tiers"   (in any repo)
  kit run "build a pricing page with three tiers"  (isolated, proven by your checks)
Undo:  kit remove frontend-design --global
```

### After setup

- Bare `kit` opens the Control Room from then on; `kit setup` repeats
  the questions and shows a diff against what is installed.
- No terminal (CI, a pipe): bare `kit` prints
  `kit: setup needs a terminal. Use: kit setup --agent claude --kit frontend-design --global --yes`.
- Every screen answers `q` or Ctrl-C with "nothing was changed".

---

## 2. What a kit is: `KIT.toml`

A kit is a folder with a `KIT.toml`, optional local skills, and an
optional rules file. It lives in a Git repo (one kit per repo, or many
under `kits/<name>/` in an index repo).

```toml
schema = 1

[kit]
name        = "frontend-design"
title       = "Frontend Design"
version     = "0.1.0"
description = "UI, design systems, accessibility, browser testing"
extends     = ["essentials"]
agents      = ["claude", "codex", "grok"]     # which agents it was written for
licence     = "MIT"                           # the kit's own files

# Skills: upstream (pinned) or local to the kit.
[[skill]]
name    = "frontend-ui-engineering"
source  = "github:addyosmani/agent-skills"
path    = "skills/frontend-ui-engineering"
rev     = "9c1e2f0a…"          # full commit sha; kit.lock adds the content hash
licence = "MIT"

[[skill]]
name = "accessibility-pass"
path = "skills/accessibility-pass"   # inside this kit

# Rules: text appended to the agent's instruction file in a marked block.
[rules]
file = "RULES.md"

# MCP servers: portable description, translated per agent.
[mcp.playwright]
command = "npx"
args    = ["-y", "@playwright/mcp@0.0.41"]
env     = {}                         # names only for secrets: env = { API_KEY = "$API_KEY" }

# Hooks: a small set of portable events, translated per agent.
[[hook]]
on      = "after_edit"
glob    = "*.{ts,tsx,js,jsx,css,md}"
run     = "npx prettier --write \"$FILE\""

# Gate: used only when the kit is installed into a repo.
[gate]
format = "npx prettier --check ."

# Smoke check: how kit doctor proves the kit is live.
[check]
mcp_starts = ["playwright"]          # start it, list tools, stop it
commands   = ["npx prettier --version"]
```

Design choices:

- **Portable events, not agent hook syntax.** v0.1 has one event,
  `after_edit`, because every agent that has hooks can express it. Kits
  that need more wait for a second event to prove itself.
- **Secrets are never in a kit.** `env` values can only reference the
  user's environment (`"$NAME"`). Kit refuses a literal that looks like a
  key.
- **`name` on every skill** is the folder name it installs as; two kits
  that bring the same name must bring the same source and rev (§4).
- **`agents`** is a claim, not a filter: installing for an agent not
  listed warns that the kit was not tested with it.

---

## 3. The translation layer

This is why Kit exists instead of "copy these folders". One `KIT.toml`,
each agent's native format, and an exact record so it can be undone.

```text
                 KIT.toml
                    │ resolve (extends, pins, fetch)
                    ▼
                 Resolved kit
                    │ plan (one writer per agent × scope)
        ┌───────────┼────────────┐
        ▼           ▼            ▼
   ClaudeWriter  CodexWriter  GrokWriter      → Plan: [Action]
        └───────────┼────────────┘
                    ▼
      show plan → confirm → apply → record in kit.lock → check
```

### Mapping table

Confirmed by the research thread (`kit-1.0/kit-catalogue.md` §2 and
`kit-research/`), 2026-09-26. **(unverified)** marks what comes from docs
nobody has tried yet.

| Kit piece | Claude Code | Codex | Grok Build |
|-----------|-------------|-------|------------|
| skill (all projects) | `~/.claude/skills/<name>/` | `~/.agents/skills/<name>/` | reads `~/.agents/skills` natively and `~/.claude/skills` via compat |
| skill (this repo) | `.claude/skills/<name>/` | `.agents/skills/<name>/` | reads `.claude/skills` via compat (unverified); `.grok/skills` natively |
| rules (all projects) | `~/.claude/CLAUDE.md` block | `~/.codex/AGENTS.md` block | reads `CLAUDE.md` and `AGENTS.md` |
| rules (this repo) | `CLAUDE.md` block | `AGENTS.md` block | same files |
| mcp (all projects) | `claude mcp add --scope user` (Claude owns `~/.claude.json`) | `~/.codex/config.toml` `[mcp_servers.<name>]` | reads `~/.claude.json` via compat; else `~/.grok/config.toml`, same TOML as Codex |
| mcp (this repo) | `.mcp.json` `mcpServers.<name>` | `.codex/config.toml` (loads only in trusted projects) | reads `.mcp.json`; else `.grok/config.toml` |
| hook `after_edit` | `settings.json` `hooks.PostToolUse`, matcher `Edit\|Write\|MultiEdit` | `hooks.json` `PostToolUse`, matcher `Edit\|Write` (same shape) | reads Claude's hooks; project hooks need `/hooks-trust` once |
| gate | `kit.toml` `[gate]` (repo scope only) | same | same |

So Kit needs **two writers**, Claude and Codex. Grok is covered by its
Claude compatibility plus `~/.agents/skills`; `kit doctor` confirms it
loaded and, if compat fails, a Grok-native writer reuses the Codex TOML
emitter. Two different tools install as `grok` (xAI's Grok Build and the
community `grok-cli`); Kit checks `grok --version` before writing for it.

A piece an agent cannot take is a line in the plan
(`hooks  after_edit  skipped for Codex: no edit hook`), never silent.

### Writing into shared files

- **Markdown (CLAUDE.md, AGENTS.md):** one block per kit,
  `<!-- kit:frontend-design 0.1.0 -->` … `<!-- /kit:frontend-design -->`.
  Kit only ever edits between its own markers.
- **JSON / TOML (settings.json, .mcp.json, config.toml):** Kit edits the
  parsed document, touches only keys it adds, keeps key order and
  comments where the format allows (`toml_edit` for TOML), and records
  each key path in `kit.lock`. A key that already exists and was not
  written by Kit is a conflict, not an overwrite.
- **Skill folders:** Kit writes a `.kit-owned` marker with the content
  hash inside each skill folder it creates, so `remove` and drift checks
  never touch a folder a person made.

### Types (inside `kitctl`)

```rust
pub struct KitManifest { kit: KitMeta, skill: Vec<SkillRef>, rules: Option<Rules>,
                         mcp: BTreeMap<String, McpServer>, hook: Vec<Hook>,
                         gate: Option<GateConfig>, check: Check }

pub enum Scope { Global, Repo(PathBuf) }

pub trait AgentWriter {
    fn agent(&self) -> AgentKind;
    fn plan(&self, kit: &ResolvedKit, scope: &Scope) -> Vec<Action>;
}

pub enum Action {
    CopySkill   { name: String, from: PathBuf, to: PathBuf, hash: String },
    RulesBlock  { file: PathBuf, kit: String, text: String },
    JsonKey     { file: PathBuf, pointer: String, value: serde_json::Value },
    TomlKey     { file: PathBuf, path: Vec<String>, value: toml_edit::Item },
    ClaudeMcp   { name: String, value: serde_json::Value },  // `claude mcp add-json --scope user`
    Skip        { piece: String, why: String },
}
```

`Plan` is the single source for the confirm screen, `--print`, `apply`,
`kit.lock`, `remove` and drift. Each `Action` knows its inverse, so
remove is "apply the inverses recorded in the lock", not a guess.

### Whose record Kit trusts

A repo is untrusted input: anyone can commit a `kit.lock`. So Kit acts
only on its own record, written by `kit add` after the user said yes:
`~/.kit/kit.lock` for global installs and `~/.kit/repos/<id>/kit.lock`
for a repo, where `<id>` comes from the repo's git folder (each clone has
its own; `kit run` worktrees share their checkout's).

- The record is data, never commands. Undo runs Kit's own code, plus
  `claude mcp remove --scope user <name>` built by Kit from a validated
  name. No record carries an argv.
- Paths are stored relative to the repo (or home) and refused on load if
  absolute, if they contain `..`, or, for home, if they are outside
  `.claude`, `.agents` and `.codex`. In a repo, Kit also refuses to write
  or delete through a link that leads outside it.
- Hooks and `kit doctor` run only the hooks and `[check]` approved at
  `kit add`, from that record; the hook command names its scope
  (`kit hook after-edit NAME --scope repo`).
- The repo's own `kit.lock` is a copy for the team, with no machine
  paths. Kit writes it and never reads it back, except for `kit list`
  naming kits it lists that are not installed here. `kit sync` must treat
  it as a proposal: show the plan and ask, exactly like `kit add`.

---

### Format decision: KIT.toml in, Agent Plugins out

The research found Agent Plugins (agent-plugins.org, 1.0, steered by
Amazon, Cursor, Microsoft, OpenAI and Vercel): `plugin.json` + `skills/` +
`mcp.json` + `hooks/`, loaded natively by Codex and accepted by Cursor.

**Decision: keep `KIT.toml` as the source a kit author writes, and emit
Agent Plugins as an install target.** Why:

- `plugin.json` describes what is *in a folder*. A kit also has to say
  where each piece *comes from* and whether Kit may copy it (vendor,
  fetch at install, link only), its pin and licence, what it `extends`,
  the gate that proves the work, and the check that proves it loaded.
  None of that is in the plugin schema; putting it in an extension blob
  would make the real manifest the blob.
- Several kit pieces cannot be inside a redistributable folder at all
  (fetch-only CC-BY-SA skills, link-only Apple skills), so a kit is not
  a finished plugin until it is resolved on the user's machine. That
  resolved output is exactly what an Agent Plugin is.
- Emitting it gives the benefit anyway: for Codex (and Cursor later),
  "install a kit" becomes "write one plugin folder", and uninstall is
  deleting it. `kit export <kit>` can also publish a resolved kit to a
  Codex or Cursor marketplace.

The on-disk spec could not be fetched from this environment, so the
emitter is built against Codex's documented layout and checked against
the spec before it ships. This is reversible: KIT.toml maps onto
`plugin.json` plus an extension table if the standard grows these fields.

## 4. Composition and lifecycle

**Kits stack.** A person can be Frontend Design and LLM Engineer at once;
forcing one primary kit would make the four starter kits compete instead
of compose. Rules for stacking:

- `extends` is resolved first; a shared base (`essentials`) installs once.
- Same skill name, same source and rev: installed once, owned by both.
  Same name, different source or rev: conflict, plan stops and names both.
- Rules blocks are ordered by install order; each is independent.
- MCP servers and hooks with the same name: conflict unless identical.

**Lifecycle commands:**

```console
$ kit list --global
frontend-design   0.1.0   claude          11 skills · 1 mcp · 1 hook   ok
llm-engineer      0.1.0   claude, codex    8 skills · 0 mcp · 0 hook   1 file changed by hand

$ kit update frontend-design
frontend-design 0.1.0 → 0.2.0
  skills   ~ frontend-ui-engineering   9c1e2f0 → 1d44a09   (12 lines changed, kit show --diff)
           + design-tokens              Zwin-ux/kits@7e0c311  MIT
  mcp      ~ playwright                 @0.0.41 → @0.0.45   RUNS CODE
Continue? [y/N]

$ kit remove frontend-design --global
Removes 11 skills, 1 CLAUDE.md block, playwright MCP, 1 hook. Continue? [y/N]
```

- `update` never runs silently and re-asks whenever anything that runs
  code changes.
- A skill edited by hand shows as drift; `update` and `remove` leave it
  in place unless `--force`, and say which file.
- `kit sync` proposes what a repo's `kit.lock` lists (new machine, new
  teammate) and installs it only after the same plan and yes as `kit add`.
  The repo copy is committed; Kit's own record lives in `~/.kit/`.
- `kit doctor` runs each installed kit's `[check]` and reports drift.

---

## 5. Trust

Skills steer the agent (a prompt-injection surface); hooks and MCP servers
execute code. The rules:

1. **Show before write.** The plan lists every file, key and command.
2. **Code is labelled.** `RUNS CODE` on every hook and MCP line, a count
   in the confirm prompt, and `[s]` to install without it.
3. **Everything is pinned.** Skills by commit and content hash; MCP
   packages by exact version (`@0.0.41`, never `@latest`; Kit refuses
   unpinned `npx` args in index kits).
4. **Licences are shown** per skill; the official index accepts only
   OSI licences, and a skill without one cannot be in an index kit.
5. **Three source levels**, shown in the plan header:
   - *Official*: kits in `Zwin-ux/kits`, reviewed by the maintainers.
   - *Index*: kits listed in the index by PR, reviewed at listing.
   - *Direct*: `kit add github:someone/kit`. Allowed, with
     `Direct source, not reviewed by Kit` above the plan.
6. **Nothing phones home.** Fetching is `git` over HTTPS from the named
   hosts; no telemetry.

---

## 6. The catalogue

### What qualifies as a kit

A kit must be all of:

1. **One job** a person would name as their role or their task this
   month ("iOS developer", "technical writer"), not a technology list.
2. **Changes behaviour:** at least three skills or rules that make the
   agent do that job differently, each from a licensed source.
3. **Provable:** a `[check]` that shows it is live, and for repo installs
   a `[gate]` that shows the work is right.
4. **Pinned and small:** everything pinned; if a person cannot read the
   plan in one screen, it should be two kits.

### Starter kits (v0.1)

Bundled in `crates/kit-cli/kits/`, sources confirmed by the research
thread and pinned to commits checked on 2026-09-26.

| Kit | Extends | Skills (source, licence) | MCP / hooks |
|-----|---------|--------------------------|-------------|
| Essentials | — | spec, plan, incremental build, TDD, review, debugging, git, shipping (addyosmani/agent-skills, MIT) | none: installs nothing that runs code |
| Frontend Design | Essentials | `frontend-design` (anthropics/skills, Apache-2.0); `accessibility`, `core-web-vitals` (addyosmani/web-quality-skills, MIT); `frontend-ui-engineering`, `browser-testing-with-devtools` (addyosmani, MIT) | Chrome DevTools MCP 1.10.1; prettier after edits |
| Full-stack Design | Frontend Design | API design, security, migrations, observability (addyosmani, MIT); `supabase-postgres-best-practices` (supabase/agent-skills, MIT) | inherits Frontend's |
| Backend Engineer | Essentials | API design, security, performance, observability, migrations, CI (addyosmani, MIT); `security-review` (getsentry/skills, Apache-2.0 and CC-BY-SA-4.0, fetched only); `find-bugs` (getsentry, Apache-2.0); Postgres best practices (supabase, MIT) | none yet (database MCPs need a connection string; stack-conditional, after v0.1) |
| LLM Engineer | Essentials | `claude-api`, `mcp-builder` (anthropics/skills, Apache-2.0); `prompt-optimizer` (getsentry, Apache-2.0); context engineering, source-driven, doubt-driven (addyosmani, MIT); `llm-evals`, `llm-cost-latency` (Kit, MIT) | OpenAI Docs MCP (remote, no code runs) |

Kit authors next, from the research gaps: a cross-agent format/lint hook
(replacing the prettier hook), `design-review` with screenshots at three
widths, `design-tokens`, a Tailwind v4 skill, an xAI/Grok API skill
(none exists), `provider-router`, and for the iOS kit an `apple-design`
skill that paraphrases and cites the HIG.

### Next kits (index, after v0.1)

| Kit | Why people want it |
|-----|--------------------|
| iOS / Apple | Swift, SwiftUI, HIG review, Xcode build and test (Mazen's example) |
| Apple Design | HIG and platform design review, separate from building |
| Android / Mobile | Kotlin or React Native, device testing |
| Data Engineering | SQL, pipelines, data quality checks |
| DevOps / Platform | CI, containers, IaC review, deploy safety |
| Security Review | threat modelling, dependency and secret scanning, safe fixes |
| Technical Writing | docs structure, API reference, READMEs, changelogs |
| Game Dev | Godot or Unity patterns, asset pipelines |
| Rust Systems | idiomatic Rust, clippy pedantic, unsafe review (Kit dogfoods this) |

### The index

A public repo, `Zwin-ux/kits`:

```text
kits/
  frontend-design/KIT.toml RULES.md skills/…
  …
index.toml        # name → path or github:owner/repo, rev, source level, summary
```

`kit search` and `kit show` read a cached `index.toml`
(`~/.kit/index/`, refreshed at most daily or with `--refresh`). The later
server serves the same `index.toml`; clients do not change.

```console
$ kit search design
frontend-design     Official   UI, design systems, accessibility, browser testing
fullstack-design    Official   Frontend Design plus APIs, data, end-to-end tests
apple-design        Index      HIG and platform design review

$ kit show frontend-design
(the Screen 5 plan for the agents in ~/.kit/config.toml, without writing)
```

---

## 7. v0.1 cut and build order

**In v0.1:** `kit setup` (Screens 1–6), `kit add | remove | list | show`
with `--global`, the Claude Code writer complete, Codex and Grok writers
for skills and rules, the five starter kits, `kit.lock`, `[check]` in
`kit doctor`, trust rules 1–6.

**After v0.1:** `kit update`, `kit sync`, `kit search` and the public
index, Codex MCP, hooks beyond Claude Code, `kit new`, the next kits,
the server.

Build order (each a tested commit on `claude/kit-product-design-uz9njt`):

1. `KIT.toml` parser with `deny_unknown_fields`, bundled starter kits,
   `kit show <kit>` printing a plan (no writes).
2. Fetch and pin upstream skills with `git` into `~/.kit/cache`, content
   hashes.
3. Claude Code writer and `Action` apply/undo, `kit.lock`,
   `kit add | remove | list`.
4. `kit setup` screens and `~/.kit/config.toml`.
5. Codex writer (skills, rules, MCP, hooks); Grok via compat, confirmed
   by `kit doctor`.
6. `[check]` in `kit doctor`.
7. README rewritten around Screen 0 to Screen 6.
