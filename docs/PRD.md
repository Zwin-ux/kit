# Kit — PRD: the simple open-source coding agent that proves its work

- **Owner:** Mazen Zwin (Zwin-ux), Westside Web Studio
- **Status:** Draft for owner sign-off, 2026-10-31. Docs only: nothing in this file is built yet. Decisions that need the owner are collected in §14.
- **Base:** Kit `2.0.x` on `main`: kits, `kit run`, the gate, receipts, `kit land` and the Control Room. `v2.0.1` is in PR #42.
- **Builds on:**
  - [`docs/superpowers/specs/2026-09-24-kit-v2-design.md`](superpowers/specs/2026-09-24-kit-v2-design.md): the umbrella design (drivers, policy, event model).
  - [`docs/superpowers/specs/2026-10-26-kit-kickoff.md`](superpowers/specs/2026-10-26-kit-kickoff.md): M0 and decisions K1–K15.
  - `docs/dev/DESIGN-HARNESS.md`: the ACP harness design, unmerged in PR #38.
  - PR #37: the Claude Code mod draft.
  - [`docs/dev/DESIGN-tui.md`](dev/DESIGN-tui.md) and [`docs/QUALITY.md`](QUALITY.md).
- **Research:** done 2026-10-31 from each tool's own docs, repos and issue trackers, then checked by an independent review.
  - Every external fact has a link.
  - **(unverified)** marks facts I could not confirm from a primary source.
  - Benchmark numbers come from secondary mirrors (the tbench.ai and swebench.com leaderboards did not render), so treat them as indicative.

---

## 0. Summary

**Goal (owner, 2026-10-31):** Kit becomes *the best open-source coding agent that stays simple.*

Kit already does two things no other agent does together:
- It **sets agents up**: kits of skills, rules, MCP servers and hooks, installed and removed exactly.
- It **proves their work**: an isolated worktree per run, the repo's own checks as the gate, a receipt, then `kit land`.

What it lacks is a brain of its own. Today every run borrows Claude Code, Codex or Grok.

This PRD adds four things, in this order:

1. **Kit's own agent loop.** A small Rust core with six working tools.
   - Bring-your-own key for Anthropic, OpenAI and xAI, or any OpenAI-compatible local server.
   - Its whole context is visible, and its cost is metered and capped.
   - Its shell is confined to the workspace: by an OS sandbox on macOS and Linux from A0.3, and by an allowlist and boundary prompts everywhere before that.
2. **A full TUI.** A scrollback-native session view for one agent, beside the existing Control Room for many.
3. **A real harness.** Claude Code, Codex and Grok run under Kit with structured events, Kit-answered permissions and cost, each over its documented headless protocol (or ACP).
4. **Claude Code–compatible mods.** Kit's agent reads the same skills, hooks and MCP config as Claude Code, using Claude Code's tool parameter names. Kit also ships a thin Claude Code plugin, so Kit is usable from inside Claude Code.

What stays: the gate and the receipt are still the point. *"Nothing ships unproven"* applies to Kit's own agent exactly as it does to the agents Kit drives.

---

## 1. The landscape (2026-10-31)

### 1.1 Who is out there

| Agent | Licence | BYO model | Local models | MCP | Hooks / plugins | Sandbox | Notes |
|---|---|---|---|---|---|---|---|
| Claude Code | Proprietary ([LICENSE](https://github.com/anthropics/claude-code/blob/main/LICENSE.md)) | Claude only: API, Bedrock, Vertex, gateway ([docs](https://code.claude.com/docs/en/third-party-integrations)) | No official path | Yes | Hooks, skills, plugins, marketplaces ([docs](https://code.claude.com/docs/en/features-overview)) | Opt-in Seatbelt / bubblewrap; none on native Windows ([docs](https://code.claude.com/docs/en/sandboxing)) | Deepest extension system |
| Codex CLI | Apache-2.0 ([repo](https://github.com/openai/codex)) | Yes (`model_providers`) | Yes (`--oss`) | Yes | Hooks, skills, plugins ([hooks](https://learn.chatgpt.com/docs/hooks.md)) | **On by default**: Seatbelt, bubblewrap, native Windows ([docs](https://learn.chatgpt.com/docs/sandboxing.md)) | Rust; `apply_patch` (V4A) edits |
| Aider | Apache-2.0 ([repo](https://github.com/Aider-AI/aider)) | Yes (LiteLLM) | Yes | Not documented | No | None | Repo map; auto-commits; releases have slowed ([PyPI](https://pypi.org/project/aider-chat/#history)) |
| OpenCode | MIT ([repo](https://github.com/anomalyco/opencode)) | Yes | Yes | Yes (unverified on the page fetched) | Plugins, SDK | None; tools **allowed by default** ([tools](https://opencode.ai/docs/tools/)) | Client/server; 2.x line out 2026-10-27 |
| Cline | Apache-2.0; JetBrains plugin closed ([repo](https://github.com/cline/cline)) | Yes | Ollama, LM Studio | Yes | SDK hooks, rules, skills | Per-action approval, checkpoints | VS Code first, plus a CLI |
| Goose | Apache-2.0 ([repo](https://github.com/block/goose)) | Yes (15+ providers) | Ollama | Yes (core) | Recipes, extensions | (unverified) | Rust; now under the Linux Foundation's AAIF |
| Grok Build | Apache-2.0; **no outside PRs** ([repo](https://github.com/xai-org/grok-build)) | Yes (3 wire protocols) | Custom endpoint | Yes ([docs](https://docs.x.ai/build/features/mcp-servers)) | Hooks, skills, plugins | Profiles ([docs](https://docs.x.ai/build/features/sandbox)) | ACP via `grok agent stdio` ([docs](https://docs.x.ai/build/cli/headless-scripting)) |
| Pi | MIT ([repo](https://github.com/badlogic/pi-mono)) | Yes | (unverified) | **No, by design** | TS extensions, packages | **None, by design** | Four tools; prompt plus tools under 1k tokens ([post](https://mariozechner.at/posts/2025-11-30-pi-coding-agent/)) |
| mini-SWE-agent | MIT ([repo](https://github.com/SWE-agent/mini-swe-agent)) | Yes (LiteLLM) | Yes | No | No | Container | About 100 lines; bash only, no tool calling ([site](https://mini-swe-agent.com/latest/)) |
| Crush | FSL-1.1-MIT, source-available ([repo](https://github.com/charmbracelet/crush)) | Yes | Auto-discovers | Yes | (unverified) | Prompts; `--yolo` | Metrics on by default |
| Gemini CLI | Apache-2.0; consumer access retired 2026-06-18 ([discussion](https://github.com/google-gemini/gemini-cli/discussions/27274)) | Gemini only | No | Yes | Extensions | (unverified) | Users moved to Antigravity CLI |

### 1.2 What people love, what they complain about

| Agent | Loved for | Recurring complaints |
|---|---|---|
| Claude Code | Extension depth; multi-file reasoning ([InventiveHQ](https://inventivehq.com/blog/terminal-ai-coding-clis-compared-2026)); guardrails out of the box | Quota drain ([#41930](https://github.com/anthropics/claude-code/issues/41930), [MacRumors](https://www.macrumors.com/2026/03/26/claude-code-users-rapid-rate-limit-drain-bug/)); TUI flicker and scrollback bugs in tmux and Windows Terminal ([#37283](https://github.com/anthropics/claude-code/issues/37283), [#51393](https://github.com/anthropics/claude-code/issues/51393)); context you can't see ([Zechner](https://mariozechner.at/posts/2025-11-30-pi-coding-agent/)); subscription tokens banned in third-party tools ([The Register](https://www.theregister.com/2026/02/20/anthropic_clarifies_ban_third_party_claude_access)) |
| Codex CLI | Open source; sandbox on by default; local models | Quota drain ([forum](https://community.openai.com/t/codex-users-report-rapid-quota-drains-since-may-10/1380649)); Windows sandbox friction ([#23712](https://github.com/openai/codex/issues/23712)); alt-screen breaks scrollback and copy ([#10331](https://github.com/openai/codex/issues/10331)) |
| Aider | Git-native undo, repo map, any model | Adding files by hand; noisy auto-commits; stalled upstream ([rywalker](https://rywalker.com/research/aider)) |
| OpenCode | Vendor-neutral; plan/build split; LSP | Memory and CPU blowups ([#11868](https://github.com/anomalyco/opencode/issues/11868)); a local HTTP server CVE ([CVE-2026-22812](https://cloud.projectdiscovery.io/library/CVE-2026-22812)); 1.x and 2.x clash |
| Cline | Per-action approval, checkpoints, local models | A $300 HTML task ([#8027](https://github.com/cline/cline/issues/8027)); an unprompted `rm -rf` of a home directory, closed "not planned" ([#7572](https://github.com/cline/cline/issues/7572)) |
| Goose | Early, deep MCP; clear tool activity | Wrong context limits for local models ([#8835](https://github.com/aaif-goose/goose/issues/8835)); weak tool calling on local models ([docs](https://goose-docs.ai/docs/troubleshooting/known-issues)); heavy config |
| Grok Build | Open source, fast, ACP | Uploaded whole repos, secrets included, in July 2026 ([Wikipedia](https://en.wikipedia.org/wiki/Grok_Build)); no outside contributions |
| Pi | Tiny, readable, cheap per task ([Composio](https://composio.dev/blog/pi-agent-vs-claude-code)); a TUI with no flicker | Full system access from the first prompt; build-it-yourself; new contributors' PRs auto-closed |

### 1.3 Where a simple open-source agent wins

| # | Gap (evidence above) | Kit's answer | § |
|---|---|---|---|
| W1 | Vendor lock-in: Anthropic's subscription ban, Gemini CLI retired, Amp picks your model | Provider-neutral core, MIT, no account, no Kit server | 6, 11 |
| W2 | Hidden, shifting context; MCP schemas cost tens of thousands of tokens ([Willison](https://www.simonwillison.net/2025/Aug/22/too-many-mcps)) | Hard token caps on the prompt and on the skills/MCP index; `kit context` prints exactly what is sent; MCP schemas load only when used | 5.4 |
| W3 | Quota drains and surprise bills | Live token and dollar meter, hard budgets, loop detection | 5.6 |
| W4 | TUI flicker and broken scrollback | Scrollback-native session view with synchronized-output updates | 7 |
| W5 | Permissions at the extremes: allow-all or prompt fatigue | No prompts for work inside the workspace; one prompt when a boundary is crossed; an OS sandbox where the OS has one | 5.5 |
| W6 | Local servers as attack surface (the OpenCode CVE) | No TCP server. Only per-session IPC that only the current user can open, and a loopback proxy guarded by a per-session token | 10 |
| W7 | Local and weak models break tool calling | A shell-only text protocol fallback; context window read from the server | 6.3 |
| W8 | Telemetry and repo uploads | Zero telemetry; network only to configured endpoints and on explicit user commands | 10 |
| W9 | Governance: closed PRs, licence changes, stalls | MIT, no CLA, written succession, outside contributions welcome | 11 |
| W10 | "Simple" usually means weak | Measured: published evals against the same model in other harnesses | 13 |

W10 is plausible for two reasons:
- The harness alone moves benchmark results by double digits ([survey](https://arxiv.org/pdf/2606.20683); [Vaughan](https://codex.danielvaughan.com/2026/06/11/terminal-bench-2-1-june-2026-benchmark-landscape-codex-cli-harness-engineering-model-scores/)).
- Minimal harnesses such as mini-SWE-agent and Terminus 2 are competitive.

The target is to match a big harness running the same model. Kit makes no claim about a score before running one.

**Kit's unfair advantage:** none of the agents above ships the proof loop. A worktree per task, the repo's own checks as the definition of done, a receipt and a land step are already built, tested and released in Kit 2.0.

---

## 2. Vision and principles

> **Kit is the coding agent you can read in an afternoon, run on any model, and trust because it proves its work.**

"Simple" is made concrete as budgets CI enforces, not as adjectives:

| Principle | Concrete rule | Enforced by |
|---|---|---|
| Small core | `crates/kit-loop` ≤ 3,000 lines, `crates/kit-ext` ≤ 1,500, each provider adapter ≤ 600 (Rust code lines: non-blank, non-comment, tests excluded; module estimates in §8.1) | `scripts/loc-budget.mjs` in `rust.yml` |
| Readable | One file per tool. No `macro_rules!` and no proc macros of our own (derive and `async_trait` only). Every public item has a doc comment. `docs/dev/ARCHITECTURE.md` explains the loop in under 300 lines | Review checklist; `#![warn(missing_docs)]` |
| Visible context | System prompt plus the eight tool schemas ≤ 1,500 tokens; the skills and MCP index ≤ 1,000 tokens (cut with a marker beyond that). Counted offline with `tiktoken-rs` `o200k_base` as an estimate and labelled `≈`. `kit context` prints the exact next request | Prompt snapshot test and token-count test, no network |
| One-command install | Unchanged: npm, curl, PowerShell or Cargo, each one line and SHA-256 checked | `installers.yml`, `npm-smoke.mjs` |
| Fast | Cold start < 100 ms (`kit --version`, measured on Ubuntu, Windows and, from A0.0, macOS); idle TUI < 1% of one core; first token on screen within 50 ms of the provider's first byte | `startup budget` job; `scripts/idle-cpu.mjs`; a loop benchmark with a scripted provider |
| BYO model and key | Works with an Anthropic, OpenAI or xAI API key, or with no key against a local server. Never needs a Kit account | Provider contract tests |
| Proven | `kit run` always ends in the gate; a session in a repo with `kit.toml` offers it | Existing gate tests |
| Private | No telemetry and no crash upload. During an agent session, network goes only to the configured provider and MCP endpoints. Other egress happens only on explicit commands (§10) | Recording-connector test (§13.2) |
| Cross-platform | Windows, macOS and Linux are equal; a Windows-only failure blocks a release. Where an OS lacks a feature (an OS sandbox on Windows), Kit says so on screen | 3-OS CI (exists) |

---

## 3. Target users

1. **The provider-neutral developer.** Pays for API keys, not subscriptions, and switches models often. *Job:* "fix this bug with whatever model, and tell me what it cost."
2. **The multi-agent power user** (today's Kit user). Has Claude Code, Codex and Grok. *Job:* "send this to three agents and land the one that passes."
3. **The local-first developer.** Runs Ollama or LM Studio, often on a network that forbids sending code out. *Job:* "a useful agent with no data leaving the machine."
4. **The careful team lead.** Wants agents in CI and on teammates' machines with the same setup (`kit.lock`), a sandbox, budgets and receipts. *Job:* "let agents open PRs, but only ones that pass our checks, with a cost line."
5. **The contributor.** Wants to read, fork and extend an agent without a 500k-line codebase. *Job:* "add a tool or a provider in an evening."

Plus the five people in [`docs/QUALITY.md`](QUALITY.md): Riya, Tomás on Windows, Sam with a screen reader and `NO_COLOR`, Lee on a light terminal over SSH, and a CI job.

---

## 4. Product shape

### 4.1 Commands

| Command | What it does | Status |
|---|---|---|
| `kit` | First run: `kit setup`. After that: the Control Room until A1.0, then the session view (§7.1, decision D1) | Exists; changes at A1.0 |
| `kit chat` | The session view with Kit's agent in the current folder, with checkpoints | New (A0.2) |
| `kit -p "<task>"` | One-shot, non-interactive session; prints events (`--plain`) or `--json`; exits with the outcome. The flag avoids collisions such as a task named `list` | New (A0.1) |
| `kit run "<task>" [--agent kit\|claude\|codex\|grok]` | The task in its own worktree, then the gate, then a receipt. The default agent stays as today until the owner opts in (§6.4, D11) | Exists; `kit` agent new |
| `kit room` | The Control Room | New alias (A0.2) |
| `kit land` / `kit receipt …` / `kit init` / `kit doctor` | Unchanged | Exists |
| `kit add / remove / list / sync / search / show / new` | Kits, unchanged | Exists |
| `kit gate [--quick]` | Run `kit.toml` checks in the current tree; `--quick` runs only `format` (and `lint` once that key exists, §8.4) | New |
| `kit trust [--revoke]` | Trust this repo's project config, hooks and MCP servers (§10) | New (A0.5) |
| `kit auth add\|list\|remove <provider>` | Store a provider key in the OS keychain (§6.4) | New (A0.4) |
| `kit models` / `kit prices [--update]` | Reachable models, context sizes, prices and their date; `--update` fetches the price file | New (A0.4) |
| `kit context` | Print the exact next request with a token count by part | New (A0.2) |
| `kit sessions` / `kit undo` | List sessions; undo the last turn's file changes | New (A0.2) |
| `kit eval` | Kit's eval suite (§13.3) | New (A0.1) |
| `kit claude install\|remove\|statusline` | The Kit plugin for Claude Code (§9.3) | New (A0.6) |
| `kit mcp [--session <id>]` | Kit as an MCP server (§9.3), and per-session approval bridge (§6.5) | New (A0.5) |

Flags shared by `chat`, `-p` and `run --agent kit`:
- `--model`, `--profile read|workspace|full`, `--budget <usd>` (a plain number such as `--budget 2.00`, so the shell cannot expand `$2`) and `--max-turns`.
- `--resume <id>` and `--fork <id>`, plus `--plain` and `--json`.

### 4.2 How the pieces fit

```text
            ┌──────────────── kit (one binary) ────────────────┐
 you ──TUI──┤ session view │ Control Room │ CLI │ kit mcp       │
            ├───────────────────────────────────────────────────┤
            │ engine: worktrees · scheduler · gate · receipts   │  (exists)
            ├────────────────┬──────────────────────────────────┤
            │ Kit's agent    │ harness drivers                   │
            │ kit-loop       │ claude · codex · grok · acp       │
            │  ├ kit-ext     │ (kit-agents, upgraded)            │
            │  │ hooks · MCP client · skills index              │
            │  └ kit-providers: anthropic · openai · xai · compat│
            ├────────────────┴──────────────────────────────────┤
            │ kit-sandbox (macOS, Linux) · kits (exist)          │
            └────────────────────────────────────────────────────┘
```

Kit's agent is one more agent behind the `Agent` interface the harness uses: it gets an `AgentKind::Kit` variant (a contract change, §8.4). So the gate, receipts, Control Room, Dispatch fan-out and `kit land` work for it with no special case.

---

## 5. Kit's own agent: the core loop

### 5.1 The loop

```text
load rules (AGENTS.md, CLAUDE.md), skills index, MCP server index
loop:
  request = system prompt + tools + rules + index + history (compacted if needed)
  stream the reply → show text and tool calls as they arrive
  for each tool call:
     checkpoint if the tool can write (§5.7) → permission check (§5.5)
       → hooks PreToolUse (§9.2) → run → hooks PostToolUse
     append the result (cut past 30 KB with a marker; full output kept in the session file)
  stop when the model ends its turn with no tool calls, the budget is hit, the user interrupts,
       or the loop detector fires (§5.6)
then: kit run always runs the gate; kit chat / kit -p offer it when the repo has kit.toml
```

- One turn at a time, with no hidden sub-loops. Parallel tool calls in one reply run concurrently only when all are read-only (`Read`, `Grep`, `Glob`); otherwise they run in order.
- Interrupt (`Esc` in the TUI, Ctrl-C on the CLI) cancels the stream and the running tool with the existing process-tree kill, and keeps the session.
- No subagents in 1.0. Parallelism is Kit's job at the fleet level (Dispatch, best-of-N), where every branch is gated.

### 5.2 Tools

The model sees **eight tools**: six that do work, plus two loaders. Each lives in one file.

Names and parameter names are **Claude Code's**, so hooks, permission rules and prompts written for Claude Code see the same shapes (§9.2):

| Tool | Parameters | Behaviour |
|---|---|---|
| `Read` | `file_path`, `offset?`, `limit?` | Text with line numbers. Images go to vision models as attachments. Binary files are refused with a size line |
| `Write` | `file_path`, `content` | Creates or overwrites; the parent folder must exist |
| `Edit` | `file_path`, `old_string`, `new_string`, `replace_all?` | Exact replace; `old_string` must match once unless `replace_all`. On no match it retries ignoring trailing whitespace and CRLF/LF only, and says so in the result. Nothing fuzzier |
| `Bash` | `command`, `timeout?` (ms; default 120,000, max 600,000) | Unix: `bash`. Windows: Git Bash when installed (as Claude Code uses on Windows); otherwise PowerShell, with the shell named in the system prompt. Output is cut at head and tail |
| `Grep` | `pattern`, `path?`, `glob?` | ripgrep semantics through the `grep-searcher` and `ignore` crates (no external binary); honours `.gitignore` |
| `Glob` | `pattern`, `path?` | Paths only, newest first, `.gitignore` honoured, capped at 500 |
| `Skill` | `name` | Loads a skill's body (§5.3) |
| `Mcp` | `server`, then `tool` and `arguments` | Lists one MCP server's tools with their schemas, then calls them (§5.4) |

**Why these six workers:**
- Read, write, edit and shell are the common core of Claude Code, OpenCode and Pi.
- OpenCode also makes grep and glob tools, while Claude Code on macOS and Linux searches through its shell ([Claude Code tools](https://code.claude.com/docs/en/tools-reference); [OpenCode tools](https://opencode.ai/docs/tools/); [Pi](https://mariozechner.at/posts/2025-11-30-pi-coding-agent/)). Kit makes search a tool so it behaves the same on Windows.
- Exact-match `Edit` is the format Claude Code and OpenCode use. A patch tool (`apply_patch`, as in Codex) is a 1.1 candidate only if the Aider polyglot eval (§13.3) shows models edit better with it.

**Not tools:** web fetch and search, todo lists, subagents and LSP. Each can come from an MCP server or a kit, and costs only an index line until it is used.

### 5.3 Rules, skills and memory

- **Rules:**
  - `AGENTS.md` and `CLAUDE.md`, walked from the repo root down to the working folder, plus `~/.kit/AGENTS.md`. Codex walks AGENTS.md the same way ([docs](https://learn.chatgpt.com/docs/agent-configuration/agents-md.md)).
  - Concatenated and capped at 32 KiB, with the cut stated.
- **Skills:**
  - `SKILL.md` folders in the agentskills.io format, which Claude Code and Codex read, from `.claude/skills/`, `.agents/skills/` and their home-folder twins.
  - Only `name` and `description` go into the index. The body loads through `Skill`, or when the user types `/name`.
- **Memory:** none hidden. The session file is the memory, and `kit chat --resume <id>` continues it.

### 5.4 Context management

- **Window:**
  - Read from the provider's model list. For OpenAI-compatible servers: `/v1/models` fields where present, then Ollama's `/api/show`.
  - It can be set per model in `~/.kit/config.toml`. Never assumed: Goose's 128k assumption is a documented failure ([#8835](https://github.com/aaif-goose/goose/issues/8835)).
- **Compaction:**
  - At 80% of the window, Kit summarises older turns. The summary appears in the transcript as a card the user can open, and the original turns stay in the session file. Never silent.
- **Prompt caching:**
  - Anthropic: cache breakpoints on the system prompt, tools and history prefix.
  - OpenAI and xAI: cache automatically.
  - Cached tokens are shown separately.
- **MCP is deferred:**
  - The index carries each configured server's name and a one-line summary.
  - Schemas reach the model only after it calls `Mcp` with that server.
  - With ten servers this costs ten index lines inside the 1,000-token index cap, instead of every schema.
- **`kit context`:** prints the next request, its token count by part, and the ten largest items.
- **No repo map or embeddings in 1.0.** `Grep` and `Glob` come first; a repo map comes only if evals show the gain is worth the code (§12).

### 5.5 Permissions and sandbox

**Profiles.** One switch: `--profile`, `/profile` in the session, or `[agent] profile` in config.

| Profile | Reads | Edits | Shell | Network from the shell |
|---|---|---|---|---|
| `read` | Anywhere the user can read, except denied paths | Never | Read-only allowlist only | Ask |
| `workspace` (**default**) | Same | Inside the workspace (the repo, or the run's worktree) without asking | See the confinement table below | Ask once per host per session |
| `full` | Same | Anywhere except protected paths | Unconfined | Allowed |

**How `workspace` confines the shell, by milestone and OS:**

| Situation | Shell runs without asking | Everything else |
|---|---|---|
| A0.1–A0.2, every OS (no OS sandbox yet); and Windows at every milestone | The **read-only allowlist**: `git status/diff/log/show/blame/branch --list`, `ls`, `cat`, `head`, `tail`, `wc`, `pwd`, `which`, `rg`, plus the repo's own `kit.toml` gate commands (the user wrote them) and commands matching a `[permissions] allow` rule | Asks. The status bar reads `unsandboxed` |
| A0.3 onwards, macOS (Seatbelt) | Any command | Writes outside the workspace and temp, and network to unapproved hosts, are blocked by the OS and turned into a prompt |
| A0.3 onwards, Linux with bubblewrap (and user namespaces allowed) | Any command | Same as macOS |
| Linux with Landlock only | Any command (file rules only) | Network is enforced only on kernel ≥ 6.7 (Landlock ABI v4). Older kernels show `network: unsandboxed` |

Ubuntu 23.10+ restricts unprivileged user namespaces through AppArmor, which can stop bubblewrap. `kit doctor` checks this and says which tier applies.

**Prompting:**
- **Boundary prompts only.** A prompt appears only when an action crosses a boundary: an edit outside the workspace, a blocked or unlisted command, a new network host, or a protected or denied path. This is Codex's `workspace-write` idea ([docs](https://learn.chatgpt.com/docs/sandboxing.md)), with Claude Code's rule syntax for exceptions.
- **Rules:** `[permissions] allow / ask / deny` in `~/.kit/config.toml` and the trusted repo's `.kit/config.toml`. They use Claude Code's syntax (`Bash(git push *)`, `Edit(//etc/**)`; deny, then ask, then allow; first match wins ([docs](https://code.claude.com/docs/en/permissions))), so users of both tools learn one syntax.
- **Always ask**, even in `full`, unless an explicit allow rule matches:
  - commands: `git push`, `npm publish`, `cargo publish`, deploy commands, `sudo`, `curl … | sh`, and `rm -rf` of an absolute path or `~`;
  - writes to protected paths: `.git/` internals, `kit.toml`, `.kit/**`, `.claude/**`, `~/.kit/**`.

  This extends the flagged list in DESIGN-HARNESS §4 (Y4, PR #38) from a label to a prompt.
- **Who answers:**
  - In `kit chat`, the user, inline.
  - In runs launched from the TUI (Dispatch, `r` retry), the run enters `ASK` in the Control Room. If nobody answers within the escalation timeout (default 10 minutes, `[agent] ask_timeout`), it becomes a deny.
  - In `kit -p`, `kit run` without a TTY, and CI, every would-be prompt is an immediate deny. The reason goes back to the model and is recorded in the receipt.

**Rule precedence**, so `kit.toml`'s existing rules and the new ones don't fight:

| Layer | Decides | When |
|---|---|---|
| Protected paths and always-ask (above) | Never silently allowed | Every tool call |
| `[permissions]` (user, then trusted repo) | Allow, ask or deny a tool call | Before the call |
| Profile | The default for anything no rule matched | Before the call |
| `kit.toml` `[firewall]` (exists) | Blocks commands by blast radius | Before a shell call; a block is a deny with the firewall's reason |
| `kit.toml` `[gate.scope]` (exists) | What a run may change | After the run, in the gate; fails the gate |

**Environment:**
- The shell's environment is DESIGN-HARNESS Y5's allowlist **minus every provider key**, plus variables the user names. Kit's agent never needs keys in a tool's environment.
- Harness drivers keep Y5 as written, because those agents need their own keys.
- `.env*`, `~/.ssh`, `~/.aws`, `~/.kit` and the keychain paths are in the default deny-read list.

### 5.6 Cost, tokens and runaway protection

- **Meter:** every turn shows input, cached, output and reasoning tokens and dollars. The status bar shows the session total and the context fill. Receipts record totals per model (§8.4).
- **Prices:**
  - A versioned `prices.toml` ships in the binary: per model input, cached input, cache write and output, with an "as of" date.
  - `kit prices --update` fetches it from the Kit repo, only when the user runs it.
  - Where a tool or provider reports cost, Kit shows that figure: xAI `cost_in_usd_ticks` (1 USD = 10¹⁰ ticks ([docs](https://docs.x.ai/developers/cost-tracking))), and Claude Code's `total_cost_usd` (itself a client estimate).
  - Computed figures are marked `≈`.
- **Budgets:** `--budget 2.00`, `--max-turns 50`, or `[agent] budget_usd`. At 80% the status bar warns. At 100% Kit ends the turn cleanly and the receipt state is `budget` (a new state, §8.4).
- **Loop detector:** the same tool with the same input three times in a row, or ten turns with no file change and no new read, pauses with a prompt. Who answers follows §5.5.

### 5.7 Sessions, checkpoints and undo

- A session is a JSONL file in `~/.kit/sessions/<repo-key>/<ulid>.jsonl`. It holds every request, reply, tool call, result and cost, is readable by people, and `kit sessions` lists them.
- **Checkpoints** are taken before any tool that can write: `Edit`, `Write`, and `Bash` outside the read-only allowlist. Kit uses plain git plumbing on a temporary index and never touches the user's index, HEAD or branches:
  - **Snapshot:**
    1. `GIT_INDEX_FILE=<tmp> git add -A` (untracked files included, ignored files excluded).
    2. `git write-tree`, then `git commit-tree`.
    3. `git update-ref refs/kit/checkpoints/<session>/<n>`.
  - **Undo** (`kit undo`, or `/undo` in the session):
    1. Read the checkpoint tree into a temporary index.
    2. `git checkout-index -a -f` into the working tree.
    3. Delete files that exist now, are not ignored, and are absent from the checkpoint.
    4. Show the restored and deleted list before doing it.
  - **Pruning:** refs are pruned when a session is deleted, and sessions older than 30 days are pruned by `kit sessions --prune` or automatically (configurable).
  - **Outside a git repo:** Kit copies each file before its first write into the session folder. Undo restores those copies and removes new files it created.
- **Resume and fork:** `kit chat --resume <id>`, `kit chat --fork <id>`.

---

## 6. Providers and harnesses

### 6.1 The provider interface (Kit's own agent)

```rust
/// One model API. Implementations live in crates/kit-providers, one file each.
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str; // "anthropic", "openai", "xai", "openai-compatible"
    async fn models(&self) -> Result<Vec<ModelInfo>, ProviderError>; // id, window, vision, tools, price key
    async fn stream(&self, req: Request) -> Result<EventStream, ProviderError>;
}

pub enum Event {
    TextDelta(String),
    ThinkingDelta(String),
    ToolCall { id: String, name: String, args: serde_json::Value },
    Usage { input: u64, cached_input: u64, cache_write: u64, output: u64, reasoning: u64, cost_usd: Option<f64> },
    Stop(StopReason), // end_turn | tool_use | max_tokens | refusal
}

pub enum ProviderError {
    Auth, RateLimited { retry_after: Option<Duration> }, ContextOverflow,
    Transient(String), BadRequest(String), Fatal(String),
}
```

- **Retries:**
  - `RateLimited` and `Transient` back off with jitter: three tries, honouring `retry-after`.
  - `ContextOverflow` compacts once, then retries.
  - `Auth` stops and names the fix (`kit auth add <provider>`).
- **Transport:** SSE over `reqwest` with rustls, so there is no OpenSSL on any OS.

### 6.2 Providers at 1.0

| Provider | API | Auth | Notes |
|---|---|---|---|
| `anthropic` | Messages API: streaming, tool use, prompt caching | API key | API keys only. Never a Claude subscription token ([The Register](https://www.theregister.com/2026/02/20/anthropic_clarifies_ban_third_party_claude_access)) |
| `openai` | Responses API (Chat Completions for compatibility) | API key | Reasoning tokens shown separately |
| `xai` | `POST https://api.x.ai/v1/responses` ([docs](https://docs.x.ai/developers/rest-api-reference/inference/responses)) | API key | Reports cost in ticks |
| `openai-compatible` | `/v1/chat/completions` at a base URL | Optional key | Ollama, LM Studio, llama.cpp server, vLLM, OpenRouter, LiteLLM, corporate gateways |

Bedrock and Vertex come later, through gateways or a dedicated adapter, if users ask.

### 6.3 Models that can't call tools

For models without reliable tool calling (from `ModelInfo`, or `tools = "text"` in config), Kit uses a **text protocol**:
- The model writes one fenced command block per turn: ` ```bash ` on Unix and on Windows with Git Bash, ` ```powershell ` on Windows without it.
- Kit runs the block under the same permissions and returns the output.
- Edits happen through the shell, constrained by §5.5.

This is mini-SWE-agent's design ([site](https://mini-swe-agent.com/latest/)). It is tested in CI with a scripted model on all three OSes, and the Kit eval suite runs it on Windows too.

### 6.4 Credentials and the default agent

Storing keys **changes a 2.0 promise** ("Kit never stores API keys"), so it is decision D2.

- **Order:**
  1. The environment variable (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`).
  2. The OS keychain entry `kit:<provider>`, via the `keyring` crate with its backends enabled explicitly: macOS Keychain, Windows Credential Manager, Linux Secret Service.
  3. `key_command` in config: runs a command that prints the key, like Claude Code's `apiKeyHelper` ([docs](https://code.claude.com/docs/en/authentication)).
- **Storage rules:**
  - Never a plain-text key in `~/.kit`.
  - `kit auth add` reads from a hidden prompt or stdin, never argv.
  - On Linux without Secret Service, `kit auth add` refuses and points to the env var or `key_command`.
  - On macOS, keychain access is tied to the binary's code signature. Unsigned releases make macOS ask again after each upgrade; this is accepted until binaries are signed (§15).
- **Never exposed:** keys are not logged, not in receipts, not in a tool's environment. The redactor (V2 §6.5) masks key-shaped strings in sessions and receipts.
- **The default agent does not switch silently.** A Claude Code user with `ANTHROPIC_API_KEY` set must not be moved to Kit's metered agent by an upgrade. `kit run` keeps today's default until the user runs `kit auth add` or sets `[agent] default = "kit"`, and A1.0 revisits this (D11).
- **For the harness, nothing changes.** Claude Code, Codex and Grok keep their own logins. Kit checks them with each tool's status command (`claude auth status`, `codex login status`) and never reads their credential files.

### 6.5 Harness drivers (external agents)

Flags were checked against each tool's docs on 2026-10-31. Kit's current invocations are in `crates/kit-agents/src/`.

**Claude Code** ([CLI reference](https://code.claude.com/docs/en/cli-reference), [headless](https://code.claude.com/docs/en/headless))
- **Launch:** `claude -p --output-format stream-json --verbose --include-partial-messages --session-id <uuid> --permission-mode default --permission-prompt-tool mcp__kit__approve --mcp-config <kit session config> --strict-mcp-config --setting-sources <user,local, plus project only if the repo is trusted> --max-budget-usd <b>`, with the prompt on stdin.
- **Events:** `system/init`, `assistant`, `user`, `stream_event`, `result`.
- **Permissions:** `kit mcp --session <id>` is the MCP server Claude starts. It forwards each prompt over the session IPC (§10) to the Kit process that owns the run, which answers with §5.5's rules. Runs without a TTY may use `--permission-prompts none` (v2.1.259+) instead.
- **Cost:** `result.total_cost_usd`, a client estimate that already includes earlier spend on resume, so it is never summed ([docs](https://code.claude.com/docs/en/agent-sdk/cost-tracking)).
- **Change from today:** structured stream instead of log lines, Kit-answered approvals, budget passed through, and project settings loaded only when trusted.

**Codex CLI** ([non-interactive](https://learn.chatgpt.com/docs/non-interactive-mode))
- **Launch:** `codex exec --json -C <wt> -s workspace-write --color never -`, with the prompt on stdin. Under `full`, add `--dangerously-bypass-approvals-and-sandbox`. Never `--full-auto`, which is removed.
- **Events:** `thread.started`, `turn.started`, `turn.completed{usage}`, `turn.failed` and `item.*`, where item types include `agent_message`, `command_execution`, `file_change` and `mcp_tool_call`.
- **Permissions:** Codex's own sandbox. `exec` runs with approvals off unless `--approve-for-me` routes them to Codex's auto-reviewer. Kit maps profiles to `-s`.
- **Cost:** computed from `usage` × `prices.toml`, because the stream has no cost field.
- **Change from today:** usage and cost added. `app-server` stays out while its docs call it experimental ([docs](https://learn.chatgpt.com/docs/app-server.md)).

**Grok Build**
- **Launch, preferred:** ACP via `grok agent stdio` ([headless](https://docs.x.ai/build/cli/headless-scripting)).
- **Launch, fallback:** today's headless line, `grok -p … --always-approve --output-format streaming-json`.
- **Events:** ACP `session/update`; or the headless JSON stream.
- **Permissions:**
  - ACP: Kit answers `session/request_permission`, picking an offered option by kind (DESIGN-HARNESS §3).
  - Headless: stays gated behind `KIT_FULL_AUTO=1`, as in 2.0.
- **Cost:** ACP `usage_update` when sent (unverified for Grok); headless as reported.
- **Change from today:** the ACP route can drop the `KIT_FULL_AUTO` requirement, once a live check shows ACP permission requests arrive for edits and commands. Grok's repo docs describe edit-only headless modes (`--permission-mode acceptEdits`, `--prompt-file`), but xAI's published CLI pages do not list them, so Kit does not rely on them **(unverified)**. DESIGN-HARNESS §3's "Grok: none" for ACP is out of date.

**Any ACP agent** (new generic driver)
- **Launch:** a manifest per agent (`command`, `args`, `env`), e.g. `@agentclientprotocol/codex-acp`, pinned exactly.
- **Permissions:** ACP permission requests, answered by kind.
- **Cost:** `usage_update` when sent.

**Ollama** (today's adapter, ADR-0003)
- **Deprecated in 2.x** with a warning that names the replacement: `--agent kit` with the `openai-compatible` provider.
- **Removed in 3.0**, recorded in a new ADR that supersedes ADR-0003.

**Driver contract:**
- Each driver turns its tool's stream into `KitEvent` (V2 §4.3).
- Replay tests use recorded transcripts in `crates/kit-agents/src/fixtures/`, plus fake binaries for exit codes, kill and resume, following the existing pattern in `crates/kit-cli/tests/land_cli.rs`.
- A version probe records the CLI version in the receipt. A stream that changed shape fails loudly, naming the version.

**Policy risk (D6).** Anthropic's Agent SDK docs say third-party products may not offer claude.ai login or rate limits without approval ([overview](https://code.claude.com/docs/en/agent-sdk/overview)). Kit 2.0 already launches the user's own signed-in `claude`. Whether that counts needs a written answer from Anthropic before Kit markets subscription use. Kit's own agent uses API keys only.

---

## 7. The TUI

One binary, two surfaces, one design system: fox red `#FF5A1F`, teal for live work, every state also a word ([`DESIGN-tui.md`](dev/DESIGN-tui.md)).

### 7.1 Session view (`kit chat`; bare `kit` from A1.0)

**Scrollback-native.** Finished output is printed once to the terminal's normal scrollback, and only a live region at the bottom is redrawn.
- **Mechanics:** ratatui's inline viewport, with each update wrapped in synchronized-output codes (`CSI ? 2026 h/l`) so it lands as one frame where the terminal supports them. This answers the flicker and scrollback complaints in §1.2, and is the approach Pi documents ([post](https://mariozechner.at/posts/2025-11-30-pi-coding-agent/)). Copy, search and scroll are the terminal's own. Decision D3.

```text
  (scrollback: earlier turns, printed once)
  you › the login test fails on Windows
  kit › Reading the test first.
    ▸ Read  tests/login.rs                          1.2 KB
    ▸ Grep  "CRLF" crates/auth                      3 matches
    ▸ Edit  crates/auth/src/token.rs  +4 −1          #3
    ▸ Bash  cargo test -p auth login                 PASS 2.1s
  kit › Fixed: the token parser split on \n only. One edit, test passes.
──────────────────────────────────────────────────────────────── live region
  ⠹ thinking · 4s
 ┌───────────────────────────────────────────────────────────────────────┐
 │ › _                                                                   │
 └───────────────────────────────────────────────────────────────────────┘
  claude-sonnet-5-5 · 18.2k/200k ctx 9% · ≈$0.041 · workspace · main · ? help
```

**Tool cards:**
- Each card is one numbered line: verb, target, outcome, size or duration.
- Text already in scrollback is never edited. `/show 3` opens card 3's output or diff in a pager overlay (the alternate screen, exited with `q`), and `/diff` shows the last turn's diff the same way.
- Failed cards print in the danger colour with the first error line, as in the Control Room.

**Approvals:**
- Asked inline in the live region, never as a modal over the transcript.
- The prompt shows the exact command or path, and why it crosses a boundary.
- Choices: `y` once, `a` always (writes a rule), `n` deny, `e` edit the command.

**Composer:**
- `Enter` sends.
- `Ctrl-J` inserts a newline everywhere. `Shift-Enter` also works where the terminal reports it (the kitty keyboard protocol).
- `/` lists commands and skills; `@` completes paths with `Tab`.
- `!cmd` runs your own shell command and adds it to context.
- `Up` on an empty composer recalls history.

**Status bar:** model, context fill, session cost, profile (`unsandboxed` when §5.5's table says so), branch, help. Under 80 columns it drops fields from the right. Never colour alone.

**Resize and switching:**
- On resize, the live region is re-laid out and the scrollback is left to the terminal.
- Switching to the Control Room (`F2` or `/room`) enters the alternate screen. Coming back reprints the live region.

**Tested on:** tmux (which must pass synchronized-output codes through; older versions degrade to plain redraws), Windows Terminal, conhost and macOS Terminal.

### 7.2 Control Room (exists; `kit room`, or `F2` from a session)

The fleet table, Run detail (stream / gate / diff), Dispatch and Board, as shipped in 2.0. Additions:

- Kit's agent appears in the agent strip and in Dispatch.
- Run detail's stream pane renders structured events from every driver with the same tool cards, replacing raw log lines.
- A `$` column for cost, with a total in the header.
- `ASK` is a new state for runs waiting on an approval (§5.5). `Enter` on the row answers it in Run detail; the timeout turns it into a deny.

### 7.3 Keys

| Where | Key | Action |
|---|---|---|
| Everywhere | `?` (on an empty composer) | Help (the credit rides its border) |
| Everywhere | `Esc` | Back / close; in a session, interrupt the turn |
| Everywhere | `Ctrl-C` twice | Quit |
| Session | `Enter` / `Ctrl-J` | Send / newline |
| Session | `Shift-Tab` | Cycle profile `read → workspace → full` (`full` asks to confirm) |
| Session | `F2` or `/room` | Control Room |
| Session | `/diff`, `/show <n>`, `/undo`, `/context`, `/profile`, `/model`, `/budget` | As named |
| Session | `Ctrl-R` | Search prompt history |
| Control Room | as today | `↑↓ Enter g d b f k r l q` |

Letter keys never act while the composer has focus. Actions are slash commands or chords. `Alt-Enter` is not used, because Windows Terminal takes it for fullscreen.

### 7.4 States every view must have

- **Empty:** the first run says in one line which provider was found, or how to add one (`kit auth add`, or start Ollama).
- **Loading:** a spinner plus the word (`thinking`, `running`, `gating`).
- **Error:** the provider error class in words, e.g. `rate limited, retrying in 8s`, `context full, compacting`, `key rejected: kit auth add anthropic`.
- **Offline:** a network failure says so and keeps the session.
- **Too small:** below 60×12, the existing screen.

**Accessibility:**
- `NO_COLOR`, `KIT_MOTION=off`, `KIT_THEME=high` and light terminals work as in 2.0.
- The session view is plain text in scrollback, which screen readers handle better than a repainting full-screen app.
- `--plain` prints events as lines with no live region, for screen readers and CI logs.

---

## 8. Architecture

### 8.1 Crates and size budget

| Crate | Status | Role | Budget (code lines) |
|---|---|---|---|
| `kitctl` (`kit-cli`) | exists | The binary: clap, wiring, kits, `kit mcp` server | — |
| `kitctl-core` | exists | Run, config, gate and receipt types, plus `KitEvent` (V2's `kit-proto` role without a new crate, per K2) | — |
| `kitctl-gate` | exists | The gate | — |
| `kitctl-agents` | exists, upgraded | Harness drivers (§6.5), process management | — |
| `kitctl-tui` | exists, extended | Control Room plus session view | — |
| **`kitctl-loop`** (`crates/kit-loop`) | new | Kit's loop and the work it does per turn (modules below) | ≤ 3,000 |
| **`kitctl-ext`** (`crates/kit-ext`) | new | Hooks runner (§9.2), MCP client (stdio and streamable HTTP, deferred schemas), skills and rules index | ≤ 1,500 |
| **`kitctl-providers`** | new | Model APIs, one file each, plus shared SSE and retry | ≤ 600 per adapter, ≤ 400 shared |
| **`kitctl-sandbox`** | new (A0.3) | Seatbelt profiles, bubblewrap and Landlock, the loopback network proxy | ≤ 1,500 |

What fits in `kit-loop`'s 3,000 lines (estimates, checked at A0.1):

| Module | Lines |
|---|---|
| Loop and turn state | 450 |
| Six tools | 650 |
| Permissions: profiles, rule matcher, prompts | 450 |
| Context and compaction | 300 |
| Sessions, resume, fork | 300 |
| Checkpoints and undo | 250 |
| Cost meter and prices | 200 |
| Loop detector | 80 |
| Text protocol | 200 |
| Prompt | 100 |

That totals about 2,980 lines, which is tight on purpose. When a module outgrows its share, the answer is to simplify it or move it out of the core, not to raise the budget.

The legacy TypeScript tree goes first (A0.0, K10). A simple, readable repo cannot carry a second, dead product.

### 8.2 Data flow (Kit's agent in `kit run`)

```text
kit run "task" → engine: worktree at HEAD → Agent::spawn(Kit)
  → kit-loop ⇄ provider stream
       tool call → checkpoint → permissions → sandbox → hooks → tool
       every step → KitEvent → session JSONL + receipt + TUI
  → turn ends → kit-gate (kit.toml checks) → receipt v2
  → PASS → kit land <id>
```

### 8.3 Dependencies

New crates must be MIT or Apache-2.0 (or compatible), maintained, and pinned in `Cargo.lock`. `cargo deny` checks licences and advisories in CI.

Planned:
- `reqwest` (rustls), plus an SSE parser of about 100 lines.
- `keyring`, with explicit backend features.
- `grep-searcher` and `ignore`, `globset`.
- `tiktoken-rs` for offline token estimates.
- An MCP client (`rmcp`, the official Rust SDK, or our own if smaller).
- `agent-client-protocol` (Apache-2.0, pinned exactly, as DESIGN-HARNESS advises).

### 8.4 Contract changes (each needs a CHANGELOG entry and a pinning test, per K15)

| Change | Surface | Milestone |
|---|---|---|
| `AgentKind::Kit` and `kit` as an `--agent` value | CLI, `--json`, receipts | A0.1 |
| `RunState::Budget` (`budget`) | receipts, `--json`, Control Room | A0.1 |
| Receipt v2: `model`, `provider`, `cliVersion`, `profile`, `costUsd`, `costEstimated`, `tokens{input,cached,output,reasoning}`, `denials[]`, `flagged[]`, `sandbox` (tier). Old receipts still read | receipts, `kit receipt`, `docs/json-contract.md` | A0.1 |
| `RunDelta` carries structured `KitEvent`s, not just `Output(String)` | engine ↔ TUI (internal) | A0.2 |
| `RunState::Ask` (`ask`), and an approval channel in the `Agent` trait (answers flow back to a running agent) | receipts, `--json`, Control Room | A0.5 |
| `kit.toml` `[gate] lint` key (optional), used by `kit gate --quick` | `kit.toml` | A0.6 |
| `[agent]`, `[permissions]`, `[mcp_servers]`, `[hooks]` sections in `~/.kit/config.toml` and `.kit/config.toml` | config | A0.1 → A0.6 |

---

## 9. Mods and extensibility (Claude Code–compatible)

### 9.1 One format, three agents

Kit invents no extension format. It reads and installs Claude Code's formats and agentskills.io's, which Codex and Grok Build also read: Codex skills ([docs](https://learn.chatgpt.com/docs/build-skills.md)), Grok skills and plugins ([docs](https://docs.x.ai/build/features/skills-plugins-marketplaces)).

| Piece | Format | Kit's agent reads | Kits install into |
|---|---|---|---|
| Skills | `SKILL.md` folders ([Claude Code](https://code.claude.com/docs/en/skills)) | `.claude/skills`, `.agents/skills`, home twins | Claude Code, Codex, Grok (exists) |
| Rules | `AGENTS.md`, `CLAUDE.md` | Yes | Marked blocks (exists) |
| MCP | `.mcp.json` / `claude mcp add-json` ([docs](https://code.claude.com/docs/en/mcp)); `[mcp_servers]` TOML for Codex and Grok | `.mcp.json` (trusted repos) and `[mcp_servers]`; stdio and HTTP | Exists |
| Hooks | Claude Code's hooks schema ([docs](https://code.claude.com/docs/en/hooks)) | `~/.kit/settings.json`, `.kit/settings.json` (trusted repos), and `.claude/settings.json` hooks when `[hooks] read_claude = true` | Claude Code (exists); Codex `hooks.json` (new) |
| Commands | `.claude/commands/*.md` (legacy, still supported by Claude Code) | Yes, as `/name` | — |

### 9.2 Hooks in Kit's agent

The goal: hooks written for Claude Code's tools work in Kit when they read the fields Kit shares. The compatibility suite (§13.2) defines exactly which fields; where it fails, the hook doesn't fit.

- **Events (1.0):** `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Stop`, `SessionEnd`. Claude Code has many more events; the rest are not supported and are listed in `docs/EXTENDING.md`.
- **Matchers:** Claude Code's rules against Kit's tool names, which are Claude's names (`Bash`, `Read`, `Edit`, `Write`, `Grep`, `Glob`), plus `Skill` and `Mcp`.
- **Payload:**
  - Fields shared with Claude Code: `session_id`, `cwd`, `hook_event_name`, `permission_mode`, `tool_name`, `tool_input` (Claude's parameter names, §5.2), `tool_response`, `prompt`.
  - `transcript_path` points to **Kit's** session JSONL, which is not Claude's transcript format. This is documented.
  - Kit adds `kit_run_id` when in a run.
- **Handler:** `command` (with exec-form `args`, `timeout` and `shell`). On Windows, hooks run in Git Bash when installed (as Claude Code does), else PowerShell. `http` and `prompt` handlers come later.
- **Exit codes:**
  - `0` continues; stdout is parsed as JSON if it is a single object.
  - `2` blocks, with stderr as the reason.
  - Any other code is a non-blocking error.
- **Output:**
  - `PreToolUse`: `hookSpecificOutput.permissionDecision` (`allow | deny | ask`) and `updatedInput` (same shape as `tool_input`).
  - `PostToolUse` and `Stop`: top-level `decision: "block"` with `reason`.
  - `SessionStart` and `UserPromptSubmit`: `additionalContext`, capped at 10,000 characters as in Claude Code.
  - Claude Code's `defer` decision is treated as `ask`.
- **Environment:** `KIT_PROJECT_DIR`, and `CLAUDE_PROJECT_DIR` set to the same value.
- **Trust:** hooks are code. They run only from user config or a trusted repo (`kit trust`), and `kit doctor` lists them. This is the same rule as kits: anything that runs code is shown first.

### 9.3 Kit inside Claude Code (the mod)

A thin plugin, not a repaint: Claude Code's UI stays theirs. This resolves K7 and replaces PR #37's large draft with a small, testable piece (D4).

- **Packaging:** this repo becomes a Claude Code marketplace (`.claude-plugin/marketplace.json`) with one plugin, `kit` ([marketplace reference](https://code.claude.com/docs/en/plugins/marketplace-reference)). The plugin contains:
  - **Skills** (as slash commands): `/kit:run <task>` (dispatch a gated run), `/kit:status`, `/kit:land <id>` and `/kit:receipt <id>`. Each calls the `kit` CLI with `--json`.
  - **An MCP server,** `kit mcp` over stdio, with tools `kit_run`, `kit_status`, `kit_receipt` and `kit_land`. `kit_land` is annotated destructive and refuses without `confirm: true`.
  - **Hooks.** An opt-in `Stop` hook (through the plugin's `userConfig`) runs `kit gate --quick`. A failure returns `decision: "block"` with the first error line, so Claude keeps working. `SessionStart` adds one line of Kit context (installed kits, `kit.toml` present).
- **Runs outlive the MCP call without a daemon.** `kit_run` starts `kit run` as a detached process and returns the run id at once. The run writes its receipt and event log under `~/.kit/runs/<id>/` (as today), and `kit_status` reads them. If the Claude session ends, the run continues and lands in the Control Room like any other.
- **Install:**
  - Claude Code users run `claude plugin marketplace add Zwin-ux/kit`, then `claude plugin install kit@kit` ([CLI reference](https://code.claude.com/docs/en/plugins/cli-reference)).
  - `kit claude install` runs the same two commands after showing the plan and getting a yes; `kit claude remove` undoes them.
  - CI validates the plugin with `claude plugin validate --strict` where `claude` is available, else against the manifest schema.
- **Status line:** plugins cannot ship one ([manifest reference](https://code.claude.com/docs/en/plugins/manifest-reference)). `kit claude statusline` prints a script the user can opt into. It shows Kit's active runs plus Claude's session cost and context fill from the status-line JSON (`cost.total_cost_usd`, `context_window.used_percentage` ([docs](https://code.claude.com/docs/en/statusline))). Kit never writes `statusLine` itself.
- **What happens to PR #37:**
  - Kept: the specialist skill content (security review, API design, …) moves into kits, where it serves all three agents.
  - Dropped: the native strip, the pane UI, the theme and the fox inside Claude Code.

### 9.4 Stability promise

| Surface | Stability |
|---|---|
| `KIT.toml`, `kit.toml`, `kit.lock`, `--json`, receipts | Semver of the `kit` product (exists); changes listed in §8.4 |
| Skills, hooks and MCP formats | Follow Claude Code's documented schemas; Kit-only fields take a `kit_` prefix |
| Hook payloads Kit sends | Pinned by compat fixtures; additive only within a major |
| `Provider` and `Agent` Rust traits | **Unstable until Kit 3.0**, then semver within 3.x, documented in `docs/EXTENDING.md` |
| The plugin's slash commands and MCP tool names | Semver with the product |

---

## 10. Security and privacy

- **No TCP server, ever.** Kit listens on no port reachable from another machine or another user. The local endpoints it does open:
  - **Session IPC:**
    - What it is: a Unix socket at mode 0600 in a 0700 folder under `~/.kit/run/`, or on Windows a named pipe whose ACL allows only the current user.
    - When it exists: one per session or run that needs it, and only while that session runs.
    - Who uses it: `kit mcp --session <id>` uses it to forward Claude Code's permission prompts to the owning Kit process.
    - Its path goes to the child in an environment variable, plus a random per-session token that must be presented.
  - **Network proxy** (A0.3, macOS and Linux):
    - Bound to `127.0.0.1` on a random port, for the sandboxed shell only.
    - Requires a per-session token in `Proxy-Authorization`.
    - Lives only while its session runs.
  - **`kit mcp`:** stdio only.
- **A daemon** (V2 M4, K12) is still deferred. If one ever ships, it uses the same IPC rules.
- **No telemetry,** no crash upload, no update check.
- **Network egress, all of it:**
  - During an agent session: the configured provider and MCP endpoints.
  - On explicit commands only: `kit add` / `kit sync` / `kit search` (github.com, as today), `kit prices --update`, the installer scripts, and `kit claude install` (through `claude`).
  - Nothing else.
- **Untrusted repos:**
  - A project's `.kit/config.toml`, hooks, `.mcp.json` and Claude project settings load only after `kit trust` in that repo, which is a one-time prompt in the TUI.
  - A cloned repo cannot run code through Kit by itself. This extends 2.0's `kit.lock` rule.
- **Prompt injection:**
  - Repo files, tool output and MCP results are data, and the system prompt says so.
  - The structural defence is the confinement and boundary prompts in §5.5, not the prompt.
- **Supply chain:**
  - Release archives keep SHA-256 sums and build attestations (exists).
  - `cargo deny` and `cargo audit` gate PRs.
  - Kit MCP servers stay pinned to exact versions (exists).
- **`SECURITY.md`:** private reports through GitHub security advisories, with a 90-day disclosure window.

---

## 11. Open-source plan

- **Licence:**
  - MIT, unchanged. Inbound = outbound, **no CLA**.
  - Commits carry a DCO `Signed-off-by`, checked by a GitHub app (D7).
- **New files:**
  - `CONTRIBUTING.md`: setup in three commands, the QUALITY.md bar, how to add a tool, a provider or a driver.
  - `CODE_OF_CONDUCT.md`: Contributor Covenant 2.1.
  - `SECURITY.md`.
  - `GOVERNANCE.md`: who merges, how a contributor becomes a maintainer, and what happens if the owner is unavailable for 90 days.
  - `docs/EXTENDING.md`.
  - `docs/dev/ARCHITECTURE.md`, updated.
- **Issues:**
  - Templates for bugs (with `kit doctor --json`), provider requests and eval regressions.
  - `good first issue` on real, scoped work.
- **Releases:** the existing pipeline (npm, crates.io, GitHub Release, installers), a changelog per release, and semver per §9.4.
- **Third-party content:** kit skills keep their upstream licence and pin (exists). Vendored skills keep their LICENSE file (`.agents/skills/LICENSE-addyosmani-agent-skills`).
- **Launch and marketing:**
  - **The demo:**
    - README hero, landing page and launch video show a terminal with Kit's agent on a real task: PASS, the receipt, then `kit land`.
    - Then the Control Room fanning one task out to Kit, Claude Code and Codex.
  - **How it's made:**
    - Built with the **stageshow** skill (benjitaylor/skills, MIT, installed at user level). Its workflow starts from a gold example and films the result, so the cut is reviewed before release.
    - stageshow is not vendored into this repo, and no artwork from it ships: its `assets/app-icons.ts` artwork is outside its MIT licence (its NOTICE.md).
  - **Proof:**
    - The VHS recordings in `docs/media/`, made from the real binary, stay as the proof next to the hero.
    - Every published number links to its eval run (§13.3).
  - **Credit:** "Kit by Westside Web Studio" (QUALITY.md).

---

## 12. Non-goals (for Kit 3.0)

- A Kit account, a hosted backend, cloud runners, a web dashboard, or a model of our own.
- Reusing Claude or ChatGPT **subscription** logins inside Kit's own agent.
- Telemetry of any kind, opt-in included, until a written proposal is accepted.
- An IDE extension. Kit is terminal-first, and editors can drive `kit mcp`.
- Subagents or swarms inside one session. Fan-out is Kit's fleet, and it is gated.
- A repo map, embeddings or a vector index.
- An OS sandbox on native Windows. Windows gets the allowlist, boundary prompts and an `unsandboxed` label.
- Repainting or theming Claude Code, Codex or Grok's own UIs (DESIGN-HARNESS §2).
- In-tree providers beyond §6.2. Others come through `openai-compatible`.
- The legacy TypeScript 0.1 tree on trunk.

---

## 13. Milestones, acceptance and evals

Milestones are the agent's own versions, A0.0 → A1.0. Each ships as a Kit release: A0.0 = 2.0.2, A0.1 = 2.1, and so on; A1.0 = **Kit 3.0**. This supersedes the kickoff's K1 plan for 2.1 ("proof integrity"); its receipt and scope items are folded into A0.1 and A0.3 (D12).

How acceptance is checked:
- Every milestone ends green on three OSes, with its automated acceptance in CI.
- Checks marked **(owner-signed)** need real models or real CLIs. They are committed as result files in `docs/evals/` or `docs/dev/dogfood-notes/`, with the date, versions and cost.
- Nothing is tagged without the owner.

### 13.1 Milestones

| Milestone | Scope | Acceptance |
|---|---|---|
| **A0.0 Groundwork** (2.0.2) | K10 as written: tag `legacy-0.1-final`, then delete `packages/`, `apps/`, `catalog/`, the Node `CI` workflow, `keep-alive.yml` and the root pnpm files if unused. `scripts/loc-budget.mjs`, `cargo deny`, macOS startup budget. CONTRIBUTING, SECURITY, GOVERNANCE, CODE_OF_CONDUCT | Tag exists; the folders and workflows are gone; the LOC and deny jobs run in `rust.yml`; the startup budget runs on three OSes |
| **A0.1 The loop** (2.1) | `kit-loop` with the eight tools; `anthropic` and `openai-compatible`; `workspace` profile with the read-only allowlist (`unsandboxed` everywhere); `kit -p`, `--plain`, `--json`; sessions; cost meter; `kit run --agent kit` gated; receipt v2, `AgentKind::Kit`, `budget` state (§8.4); the Kit eval suite (30 tasks) and `kit eval` | `kit-loop` ≤ 3,000 lines; prompt plus tools ≤ 1,500 tokens (test); scripted-provider tests for each tool, interrupt, budget stop, loop detector and text protocol on three OSes; receipt v2 contract test; Kit suite ≥ 70% with `claude-sonnet-5-5` **(owner-signed)**; cold start < 100 ms |
| **A0.2 Session view** (2.2) | `kit chat` inline TUI (§7.1): cards, inline approvals, composer, status bar, pager, `kit context`, checkpoints and `kit undo`, compaction, `kit sessions`, resume and fork; `kit room`; structured `RunDelta` | Snapshots for every §7.4 state at 80 and 60 columns; a render-count test proves no full redraw while streaming; idle CPU < 1% via `scripts/idle-cpu.mjs` (30 s sample); checkpoint tests cover untracked files, deletions, a clean tree and a `Bash`-made change; resume round-trips |
| **A0.3 Sandbox** (2.3) | `kit-sandbox`: Seatbelt (macOS), bubblewrap or Landlock (Linux), loopback proxy with per-host prompts; the `kit doctor` sandbox tier; Windows stays on the allowlist | Escape tests on macOS and Linux CI: a write outside the workspace, a read of `~/.ssh` and a connection to an unapproved host all fail; the profile matrix is tested; `unsandboxed` shows on Windows and on Landlock kernels < 6.7 for network |
| **A0.4 Providers** (2.4) | `openai` (Responses), `xai`, keychain `kit auth`, `prices.toml` and `kit prices`, `kit models`; text-protocol hardening; Ollama adapter deprecation warning | Recorded-stream contract tests per provider (no network in CI); a redaction test shows keys never appear in sessions or receipts; text mode ≥ 50% on the Kit suite with `llama3.1:8b` on Ollama **(owner-signed)** |
| **A0.5 Harness v2** (2.5) | Claude stream-json with `kit mcp --session` approvals over IPC; Codex usage and cost; Grok over ACP; the generic ACP driver; `ask` state and approval channel; structured Run detail and `$` column | Replay fixtures per driver; kill, resume and version-drift tests with fake binaries; the IPC is 0600 / user-only ACL (test); Grok over ACP edits a file without `KIT_FULL_AUTO` **(owner-signed)**; one task fanned to Kit, Claude and Codex in a VHS recording **(owner-signed)** |
| **A0.6 Compatibility and the mod** (2.6) | Hooks (§9.2), `.mcp.json`, commands, `kit trust`; the Kit plugin and marketplace; `kit mcp` tools; `kit claude install/remove/statusline`; `[gate] lint` | ≥ 20 hook fixtures give the same decision for Kit's runner and for the recorded Claude Code payload; `claude plugin validate --strict` passes **(owner-signed where `claude` isn't on the runner)**; a recorded session in which `/kit:run` returns a run id and the run reaches PASS |
| **A0.7 Public evals** (2.7) | Harbor agent adapter; Terminal-Bench subset, 50-task SWE-bench Verified subset and Aider polyglot in `evals.yml` (manual or weekly, budget-capped) | Results committed to `docs/evals/` with date, model, cost and run link **(owner-signed: paid runs)**; Kit within 5 points of Codex CLI with the same model on the same Terminal-Bench subset, or the gap published with its reason |
| **A1.0 = Kit 3.0** | Bare `kit` opens the session view (D1); the default agent decision (D11); Ollama adapter removed; traits stabilised in `docs/EXTENDING.md`; launch per §11 | Fresh-machine smoke test (`docs/SMOKE-TEST.md`) on three OSes, with an API key and with Ollama **(owner-signed)**; every earlier acceptance still green; README numbers link to eval runs |

### 13.2 Test plan

| Layer | How | Where |
|---|---|---|
| Tools | Unit tests on temp dirs: CRLF, non-UTF-8, unique-match failures, symlinks leaving the workspace, huge outputs, Windows paths | `kit-loop` |
| Loop | A **scripted provider** replays a JSON script of replies through the real loop: tool order, parallel reads, interrupt, budget, loop detector, compaction, the text protocol (bash and PowerShell variants) | `kit-loop/tests` |
| Providers | Recorded SSE streams per provider (success, tool call, rate limit, context overflow, auth failure) replayed through the adapter. A `live` cargo feature, off in CI, runs the same cases against real APIs | `kit-providers/tests` |
| Harness | Replayed transcripts per driver plus fake binaries for exit, kill, resume and version drift | `kit-agents`, `kit-cli/tests` |
| Hooks | Fixtures: a hook script, an event, the expected decision, run through Kit's runner and checked against recorded Claude Code payloads | `kit-ext/tests` |
| Permissions and sandbox | A rule-matching table test; the precedence table (§5.5) as tests; escape tests per OS | `kit-loop`, `kit-sandbox` |
| Checkpoints | Untracked, deleted, renamed, ignored and binary files; a clean tree; a `Bash` edit; undo twice | `kit-loop` |
| IPC | Permissions of the socket or pipe; a wrong token is refused; the endpoint is gone after the session | `kit-cli/tests` |
| TUI | insta snapshots for every state and width; render-count tests for streaming; `NO_COLOR` and high-contrast cases | `kit-tui` |
| Privacy | The HTTP client is built through one constructor; in tests it takes a recording connector, and a scripted session asserts it reached only the configured provider host. Shell network is covered by the sandbox escape tests | `kit-loop/tests` |
| Budgets | LOC, prompt tokens, cold start, idle CPU | `rust.yml` |
| Release | Existing: crates.io package, npm smoke, installers, release dry run | `rust.yml`, `installers.yml`, `release-npm.yml` |

### 13.3 Evals

- **Kit suite (`kit eval`, in repo, A0.1):**
  - **Tasks:** 30 small, real tasks in fixture repos (Rust, Python, TypeScript, Go). Each is graded by its own `kit.toml` gate, so no model judges the result.
  - **Categories:** fix a failing test, a feature with a test, a refactor that keeps behaviour, a Windows/CRLF bug, a multi-file rename, read-only questions (graded by an exact-answer check), and a refusal case (a task that needs `git push` and must end in a denied prompt).
  - **Reported per model:** pass rate, median cost, median turns, tokens.
  - **When it runs:** on every PR with the scripted provider, which checks the harness only. Real-model runs are owner-signed and capped with `--budget`.
- **Terminal-Bench** (A0.7):
  - Through Harbor (Apache-2.0): `harbor run -d terminal-bench/terminal-bench@<pinned> --agent <kit adapter> --model …`, after an oracle run ([repo](https://github.com/harbor-framework/terminal-bench)).
  - Kit's Harbor adapter runs `kit -p --plain --profile full` inside the task container.
  - Version 4.0 (66 tasks) is not comparable with 2.x results (BenchLM, secondary).
- **SWE-bench Verified** (A0.7):
  - A fixed 50-task subset, with the MIT harness in Docker ([repo](https://github.com/SWE-bench/SWE-bench)).
  - Cost is about $0.30 per task with mid-tier models ([swebench blog](https://www.swebench.com/post-250820-mini-roulette.html)), so roughly $15 per run (my arithmetic).
  - Verified is saturated and contamination-prone, so it is a regression check, not a headline.
- **Aider polyglot** (A0.7):
  - 225 exercises in Docker ([README](https://github.com/Aider-AI/aider/blob/main/benchmark/README.md)).
  - It tests edit quality, and decides whether `apply_patch` earns its place (§5.2).
- **Rule for claims:**
  - No number goes in the README or a post without a committed result file and a run link.
  - Comparisons use the same model, date and task subset, and state all three.

---

## 14. Decisions for the owner

| # | Decision | Recommendation |
|---|---|---|
| D1 | What bare `kit` opens | **The session view, from A1.0 (Kit 3.0).** Until then it stays the Control Room, and the session is `kit chat` |
| D2 | Kit stores provider keys in the OS keychain (never plain text), changing the 2.0 "never stores keys" promise | **Yes**: env var, then keychain, then `key_command`. Rewrite the promise as "never in plain text, never sent anywhere but the provider" |
| D3 | Session view inline (scrollback-native) or full-screen | **Inline.** The Control Room stays full-screen |
| D4 | The Claude Code mod is a thin plugin (commands, `kit mcp`, opt-in Stop gate), and PR #37's UI work is closed | **Yes.** It resolves K7 |
| D5 | Milestones A0.x ship as Kit 2.x minors; A1.0 = Kit 3.0 | **Yes** |
| D6 | Ask Anthropic, in writing, whether launching the user's signed-in `claude` from Kit is allowed | **Yes, before A0.5 ships**, and before any marketing mentions Claude subscriptions |
| D7 | DCO sign-off on contributions (no CLA) | **Yes** |
| D8 | Delete the legacy TS tree now (K10) as A0.0 | **Yes** |
| D9 | Default profile `workspace`, with boundary prompts; `full` needs an explicit switch | **Yes** |
| D10 | PR #38 (ACP harness design) | **Merge** as the reference for §6.5's ACP driver (K8), with a note that Grok now has ACP |
| D11 | When `kit run` defaults to Kit's own agent | **Only after an explicit `kit auth add` or `[agent] default = "kit"`**; revisit at 3.0. Never because an env var is set |
| D12 | This PRD supersedes K1's "2.1 = proof integrity" | **Yes**: receipt v2 lands in A0.1 and the sandbox in A0.3, and the firewall-wiring item becomes the precedence rules in §5.5 |

---

## 15. Risks

| Risk | Likelihood / impact | Mitigation |
|---|---|---|
| Scope sprawl turns "simple" into a second Claude Code | High / high | Line and token budgets in CI; §12; one milestone at a time; simplify rather than raise a budget |
| Upstream CLIs change stream shapes (`codex --full-auto` and `codex mcp-server` were both removed this year) | High / medium | Version probe, replay fixtures, loud failure naming the version, one driver file per tool |
| Anthropic policy on harnessing `claude` | Medium / high | D6; Kit's own agent never depends on it |
| Windows has no OS sandbox | Certain / medium | The read-only allowlist, boundary prompts and an `unsandboxed` label; parity elsewhere |
| Linux sandbox availability (AppArmor userns restriction; Landlock network needs 6.7+) | Medium / medium | `kit doctor` tiering; labels on screen; bubblewrap preferred |
| A small agent scores poorly | Medium / high | Evals from A0.1; publish honestly; patch tool or repo map only on evidence |
| Key storage becomes a target, or macOS re-prompts after unsigned upgrades | Low / high; certain / low | Keychain only, redaction tests, no keys in tool environments; accept re-prompts until binaries are signed (the signing cost is noted in RELEASING.md) |
| Hook compatibility drifts from Claude Code | Medium / medium | Fixtures from recorded payloads; supported fields listed in `docs/EXTENDING.md` |
| Contributor load on a solo maintainer | Medium / medium | GOVERNANCE.md, small scoped issues, the QUALITY.md review checklist |

---

## Appendix A — verification notes (2026-10-31)

- **Kit today** (`crates/kit-agents/src/{codex,grok}.rs`):
  - Codex runs as `codex exec -C <wt> -s workspace-write --json --color never [--dangerously-bypass-approvals-and-sandbox] -`, with the prompt on stdin.
  - Grok runs as `grok -p <prompt> --cwd <wt> --always-approve --output-format streaming-json`, and is refused without `KIT_FULL_AUTO`.
  - Run states are queued, running, gating, pass, fail, unconfigured, killed and error. The receipt is v1.
- **Codex:** latest release is `rust-v0.166.0` (2026-10-29). Its docs moved from developers.openai.com/codex to learn.chatgpt.com/docs (a 308 redirect).
- **Grok Build:** xAI's published CLI pages list `-p`, `-m`, `-s`, `-r`, `-c`, `--cwd`, `--output-format plain|json|streaming-json`, `--always-approve` and `grok agent stdio`. `--permission-mode` and `--prompt-file` appear only in the repo's user guide **(unverified for the released CLI)**.
- **Claude Code:** `claude agents --json` (active sessions) is documented; its schema is not. Kit passes `--verbose` with `stream-json` because the docs always pair them.
- **Open questions:**
  - ACP permission behaviour and usage reporting in `grok agent stdio`.
  - Whether `codex-acp` honours headless auto-approval.
  - All Terminal-Bench figures, which come from secondary mirrors.
