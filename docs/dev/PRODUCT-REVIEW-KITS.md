# Kit: "a kit is a bundle for one job". Product and Design review

Written 2026-09-26 after Mazen described the goal: installing a kit sets a
coding agent up for one job (an iOS kit, an Apple-design kit) with the
skills, hooks and config it needs, built from free open-source skills, with
saved kits reachable online and a marketplace to find them.

This reverses one line of the 1.0 PRD ("not the product: skill
marketplace"). It builds on, rather than replaces, what already exists:

- Kit 0.1 (Node, `packages/`) had packs (`packs/*/PACK.md`), `pack apply`,
  per-agent skill paths (`packages/core/src/paths/harness.ts`) and a
  recommender. The ideas were right; the Node app was retired.
- Kit 1.0 (Rust) has the part no skills installer has: an isolated run,
  a gate that decides "done", receipts, and `kit land`.
- `skills-lock.json` already pins upstream open-source skills by repo,
  path and content hash.

Agent file locations marked *to verify* below are from 0.1's code and
public docs as I know them; the "Open-source skills for kits" research
thread will confirm them.

---

## 1. Product review

### User problem

An engineer who wants Claude Code or Codex to be good at one job, say
iOS apps, has to find skills scattered across GitHub, copy them into each
agent's own folder, wire an MCP server, add hooks, write AGENTS.md rules,
and work out which build and test commands prove the work. Then do it again
on the next machine and for every teammate. Nobody can tell which version
of which skill they have.

### What Kit becomes

**Kit sets your coding agents up for a job, then proves what they do.**

- **Set up:** `kit add ios` installs one kit: skills, instructions, MCP
  servers, hooks, and the checks that define done for that job.
- **Prove:** `kit run` works exactly as now, but the gate comes from the
  kit (iOS: `swiftformat --lint`, `xcodebuild test`), so "done" means the
  same thing for every agent and every teammate.

The two halves need each other. Other skill installers stop at copying
files; the Control Room without kits has no opinion about the job. A kit
that carries its own definition of done is the thing only Kit offers.

### Why someone picks Kit over copying skills by hand

1. One command installs for every agent they use, in each agent's format.
2. Everything is pinned in `kit.lock` and committed, so a teammate's
   `kit sync` gets the same setup.
3. `kit remove ios` removes exactly what `kit add ios` wrote.
4. Anything that runs code (hooks, MCP servers) is shown before install
   and pinned by hash. A skills marketplace without this is a supply-chain
   hole.
5. The kit's gate proves the agent's work, which no other bundle does.

### First run: Kit asks two questions

Added after Mazen's second message: installing Kit should ask which agents
you use and what you want them to focus on, then set them up.

```console
$ npm i -g @mzwin/kit        # or curl | sh, or cargo install kitctl
$ kit
Kit sets your coding agents up for a job.

Which agents should Kit set up?          (found on this machine)
  [x] Claude Code   2.1.283
  [ ] Codex         not installed
  [ ] Grok          not installed

What should they focus on?               (space to pick, one or more)
  [x] Frontend Design     UI, design systems, accessibility, browser testing
  [ ] Full-stack Design   Frontend Design plus APIs, data, end-to-end tests
  [ ] Backend Engineer    APIs, databases, security, performance
  [ ] LLM Engineer        prompts, evals, RAG, model APIs

Install for:  (•) all my projects   ( ) this repo only

Frontend Design 0.1.0 for Claude Code:
  skills   9   → ~/.claude/skills
  rules        → ~/.claude/CLAUDE.md (marked block)
  mcp      playwright (runs npx @playwright/mcp@x.y.z)   → Claude Code user config
  hooks    prettier after each edit                      → ~/.claude/settings.json
Runs code on your machine: 1 MCP server, 1 hook. Continue? [y/N] y

Done. Claude Code now has the Frontend Design kit in every project.
next      open Claude Code in any repo and ask for a component,
          or: kit run "build a pricing page"
```

- The questions appear the first time bare `kit` runs in a terminal and
  nothing is set up yet. Afterwards bare `kit` opens the Control Room, and
  `kit setup` asks again.
- Detected agents are pre-ticked; missing ones say how to install them.
- The same flow without prompts, for scripts and dotfiles:
  `kit setup --agent claude --kit frontend-design --global --yes`.
- Choices are saved in `~/.kit/config.toml`, so `kit sync --global` on a
  new machine reproduces the setup.
- Prompts are plain terminal questions (a small prompt crate such as
  `dialoguer`), not the Control Room TUI, so they work in any terminal.

### The four starter kits

Proposals from the skills already pinned in `skills-lock.json`
(addyosmani/agent-skills) and well-known open-source tools. The research
thread's catalogue replaces the *candidate* entries with verified sources.

| Kit | Skills | MCP / hooks | Gate (repo installs) |
|-----|--------|-------------|----------------------|
| **Frontend Design** | frontend-ui-engineering, browser-testing-with-devtools, a11y-pass; *candidates:* a frontend/visual design skill, a design-tokens skill | Playwright MCP for seeing the page; format-on-edit hook (prettier) | format, typecheck, lint, test from `package.json` |
| **Full-stack Design** | extends Frontend Design; api-and-interface-design, security-and-hardening, *candidate:* database migrations | Playwright MCP; *candidate:* a Postgres MCP (read-only) | Frontend's gate plus the backend tests |
| **Backend Engineer** | api-and-interface-design, security-and-hardening, performance-optimization, debugging-and-error-recovery, test-driven-development | *candidate:* read-only database MCP; format-on-edit hook for the detected language | format, lint, test for the detected toolchain (as `kit init` does today) |
| **LLM Engineer** | *candidates:* prompt and eval design, model API usage (e.g. an open-source Claude/OpenAI API skill), RAG patterns, cost and latency budgeting | *candidate:* a docs-fetch MCP for current SDK docs | test, plus an eval command when the repo has one |

Every kit also extends `essentials` (spec, plan, review, tests,
shipping), so the base workflow is the same whatever the focus.

### Primary workflow in a repo

```console
$ cd MyApp
$ kit search ios
ios            Swift and SwiftUI apps: HIG, Xcode builds, tests
apple-design   Review UI against Apple's Human Interface Guidelines

$ kit add ios
ios 0.1.0 adds to this repo:
  skills   12  → .claude/skills, .agents/skills
  rules        → AGENTS.md (marked block)
  mcp      xcodebuildmcp (runs npx xcodebuildmcp@1.2.3)   → .mcp.json
  hooks    swiftformat after each edit (Claude Code)       → .claude/settings.json
  gate     format, test                                    → kit.toml
Runs code on your machine: 1 MCP server, 1 hook. Continue? [y/N] y
Added ios 0.1.0. Pinned in kit.lock. Commit kit.toml and kit.lock.
next      kit run "add a settings screen"

$ kit list          # kits in this repo, and whether files drifted
$ kit sync          # teammate: install exactly what kit.lock says
$ kit remove ios    # undo exactly what add wrote
```

A saved kit online, in v0.1, is a Git repo: `kit new my-ios` scaffolds
one, you push it, and anyone can run `kit add github:you/my-ios`.

### Scope for v0.1

1. **First-run setup** (`kit setup`, shown on first bare `kit`) and the
   four starter kits: Frontend Design, Full-stack Design, Backend
   Engineer, LLM Engineer. `essentials` becomes their shared base; the
   other 0.1 packs retire or become kits later.
1. **Kit format** (`KIT.toml`, §2). Job kits like `ios` follow the
   starter four.
2. **`kit add | remove | list | sync | search | new`**, project scope.
3. **Writers for Claude Code, Codex and Grok**: skills and rules for all three;
   MCP and hooks for Claude Code first (its formats are documented);
   Codex MCP and hooks when the research thread confirms the formats.
4. **Index as a Git repo**: `search` and bare names like `ios` resolve
   through `index.toml` in a public repo (`Zwin-ux/kits`), fetched over
   HTTPS and cached. That is the "server" for v0.1, at zero hosting cost.
5. **Kit gate feeds `kit run`**, which otherwise stays as it is.

### Non-goals for v0.1 (argued)

- **A hosted server, accounts, publishing UI, ratings.** Until there are
  outside publishers, a Git index gives the same install experience
  (`kit add ios`) with nothing to run or secure. Build the server when
  publishers exist; the index format moves to it unchanged.
- **Copying other people's skills into our repo.** Kits reference
  upstream skills by repo, path and commit (as `skills-lock.json` does),
  so authors keep credit and licence, and updates are a pin bump.
- ~~Global install~~ (moved into v0.1: the first-run questions set up
  the person's agents for every project, which is per-user).
- **Running unreviewed code.** No kit installs a hook or MCP server
  without the confirmation above; `--yes` exists for CI only.
- **Paid kits, auto-update, dependency solving beyond `extends`.**

### Public contract

- `KIT.toml` schema 1, `kit.lock`, the `[kits]` table in `kit.toml`.
- Install locations per agent, and that every change Kit makes to a
  shared file (AGENTS.md, CLAUDE.md, .mcp.json, settings.json) is inside
  a marked block or keyed entry that `kit remove` can take out cleanly.
- `kit add` never overwrites a file it did not write; a conflict stops
  with the path and the fix.

### Complexity check

- Hooks are the most agent-specific piece. Keep one portable hook event
  in v0.1 (`after_edit`), mapped per agent, and skip it with a notice where
  an agent has no equivalent. Do not invent a hook language.
- `kit init` and `kit add` overlap: `init` becomes "detect the project,
  suggest kits, write the gate", and suggests `kit add <kit>` rather than
  duplicating it.
- One manifest per kit; no separate pack, plugin and bundle concepts. The
  0.1 `PACK.md` and `kit.plugin.json` retire into `KIT.toml`.

---

## 2. Design review

### Commands

```text
kit setup               The first-run questions: agents, focus, scope
kit add <KIT>...        Install kits into this repo (--global: all projects) (name, github:owner/repo, or path)
kit remove <KIT>...     Remove what add wrote
kit list                Kits in this repo, versions, drift
kit sync                Install exactly what kit.lock pins
kit search [TERM]       Find kits in the index
kit new <NAME>          Scaffold a kit to publish as a Git repo

kit init                Detect the project, suggest kits, write the gate
kit run <TASK>          Run an agent in a worktree, prove it with the gate
kit land <RUN>          Put a proven run on a branch
kit receipt [show]      Runs and their proof
kit doctor              Agents, kits, drift
kit                     Control Room
```

`add` over `install` matches `cargo add` and `npm add` and reads as
"add to this repo". The clap grammar from the CLI slice already fits;
these are new variants of the same `Command` enum.

### Kit manifest: `KIT.toml`

```toml
[kit]
name = "ios"
version = "0.1.0"
description = "Swift and SwiftUI apps for iPhone: HIG, Xcode builds, tests"
extends = ["essentials"]

[[skill]]                      # upstream, pinned
source = "github:owner/swift-skills"
path = "skills/swiftui"
rev = "3f2c1ab"

[[skill]]                      # shipped inside the kit
path = "skills/xcode-build"

[rules]                        # appended to AGENTS.md / CLAUDE.md in a marked block
file = "RULES.md"

[mcp.xcodebuild]
command = "npx"
args = ["-y", "xcodebuildmcp@1.2.3"]

[hooks]
after_edit = "swiftformat {file}"

[gate]                         # same keys as a repo's kit.toml
format = "swiftformat --lint ."
test   = "xcodebuild test -scheme App -destination 'platform=iOS Simulator,name=iPhone 16'"
```

`KIT.toml` sits next to `SKILL.md` as the convention for a folder that is
a kit. The repo's own `kit.toml` gains one table:

```toml
[kits]
ios = "0.1.0"
```

and `kit.lock` records each resolved source, commit and content hash.

### Where things go (to verify)

| Piece | Claude Code | Codex | Grok |
|-------|-------------|-------|------|
| skills, this repo | `.claude/skills/<name>/` | `.agents/skills/<name>/` | `.grok/skills/<name>/` (Kit 0.1 convention) |
| skills, all projects | `~/.claude/skills/<name>/` | `~/.codex/skills/<name>/` | `~/.grok/skills/<name>/` |
| rules | `CLAUDE.md` / `~/.claude/CLAUDE.md` block | `AGENTS.md` / `~/.codex/AGENTS.md` block | `AGENTS.md` block |
| mcp | `.mcp.json` / user config | `~/.codex/config.toml` `[mcp_servers]` | unknown; skipped with a notice |
| hooks | `settings.json` `PostToolUse` | none known; skipped with a notice | unknown; skipped |
| gate | `kit.toml` (repo installs only) | same | same |

### Type review (inside kit-cli)

| Type | Why |
|------|-----|
| `KitManifest` | Parsed `KIT.toml`, `deny_unknown_fields` so a typo is an error. |
| `KitSource { Index(name), Git { repo, rev }, Path }` | One parser for `ios`, `github:o/r`, `./dir`. |
| `trait AgentWriter { plan(&KitManifest) -> Plan }` | One per agent. A real trait: two writers now, grok later. |
| `Plan { actions: Vec<Action> }` | What will be written, shown before anything is. `add` prints the plan, asks, then applies. `--print` stops after the plan. |
| `Lock` | `kit.lock`: what was resolved and written, so `remove` and drift checks are exact. |

The plan-then-apply split is the design's main safety property: the
confirmation screen, `--print`, `remove` and drift detection all read the
same `Plan`.

### Errors

| Situation | Message |
|-----------|---------|
| Unknown kit | `kit: no kit named 'iso'. Did you mean 'ios'? See kit search` |
| File Kit did not write | `kit: .claude/skills/swiftui exists and was not written by kit. Move it, or kit add ios --force` |
| Upstream moved | `kit: owner/swift-skills@3f2c1ab no longer exists. Pin a new rev in KIT.toml` |
| Hook or MCP declined | `kit: nothing was changed.` (exit 1) |
| Offline | `kit: cannot reach the kit index. kit add ./path works offline` |

### Alternatives considered

1. **Hosted server first.** Needs accounts, storage, abuse handling and
   uptime before anyone publishes a kit. Rejected for v0.1.
2. **Vendor all skills into one Kit repo.** Simple to serve, but forks
   every author's work and goes stale. Rejected; pin upstream instead.
3. **Separate packs, plugins and bundles** (as in 0.1). Three concepts
   for one idea. Rejected; one `KIT.toml`.

---

## 3. Effect on the work in flight

The CLI slice (clap, default agent, next steps, honest firewall,
`kitctl` rename) stays: it is the "prove" half and none of it assumes
runs are the whole product. The README rewrite waits for this direction,
because its first screen changes from "Control Room" to `kit add`.
