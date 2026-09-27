# Kit as the harness: one red room for Claude Code, Codex and Cursor

Status: design for after 2.0.0, 2026-09-27. Asked for by Mazen ("combine kit
to be an agent harness that can run all 3 and make it red… like a skin for
codex/claude/cursor… utilize the open source"). Nothing here changes 2.0.0,
PR #34 or the version.

Builds on `DESIGN-KITS.md` (kits lead, proof second), `DESIGN-tui.md`,
ADR-0001 (local process adapters) and the fox-red look in #34. Claims nobody
here has run yet are marked **(unverified)**.

Contents:

1. The answer in one paragraph
2. Why not a literal skin
3. The protocol: ACP
4. Product (mind one): what a person gets
5. Design (mind two): what it looks like
6. Engine: how it fits the crates
7. Trust and licences
8. Cut, build order, kill criteria
9. Decisions

---

## 1. The answer

Kit does not repaint Claude Code, Codex or Cursor. Kit draws its own
interface, in fox red, and drives each agent over the **Agent Client
Protocol (ACP)**, an open JSON-RPC protocol that all three now speak. The
agent keeps its brain, its login and its model; Kit owns the screen, the
approvals, the worktree, the gate and the receipt. One pitch still holds:
"Kit sets your coding agents up for one job, then proves what they do." The
harness is how you *watch and steer* them while they do it.

## 2. Why not a literal skin

| Route | What you'd get | Verdict |
|---|---|---|
| Recolour their TUI | Needs their source. Claude Code and Cursor's CLI are closed. | Impossible for two of three |
| Fork Codex's TUI (Apache-2.0) | Works for one agent, then a fork to rebase forever | No |
| Host their TUI in a PTY pane (the old B2-pty "attach") | A red frame around *their* colours. Keystrokes, resize, alt-screen, mouse all proxied. Kit sees bytes, not meaning, so it can't gate or summarise | Keep only as an escape hatch |
| **Drive them over ACP, render ourselves** | Structured messages, tool calls, diffs, plans and permission prompts; Kit draws every pixel | **Do this** |

The PTY route also breaks a TUI rule: one clock. Their screen repaints on
its own timers inside ours.

## 3. The protocol: ACP

ACP (from Zed, Apache-2.0) is JSON-RPC 2.0 over the agent's stdio,
newline-delimited. Kit is the **client**; the agent CLI is the **agent**.

What Kit sends: `initialize`, `session/new` (cwd = the run's worktree, MCP
servers), `session/prompt`, `session/cancel`, `session/load` (resume),
`session/set_mode`.

What comes back:

- `session/update` notifications: `agent_message_chunk`,
  `agent_thought_chunk`, `tool_call` / `tool_call_update` (kind, title,
  status, locations, diff content), `plan` (entries with status),
  `available_commands_update`, `current_mode_update`.
- `session/request_permission` requests: the agent asks before a tool runs;
  Kit answers `allow-once`, `allow-always` or `reject-once`.
- Optional client capabilities Kit may offer: `fs/read_text_file`,
  `fs/write_text_file`, `terminal/*`.

How each agent speaks it (checked 2026-09-27 against
agentclientprotocol.com/get-started/agents and cursor.com/docs/cli/acp):

| Agent | Launch | Support | Notes |
|---|---|---|---|
| Cursor | `agent acp` | Native | Permissions yes, session/load yes, **no client fs, no client terminal**; modes agent/plan/ask; auth via `agent login` or `CURSOR_API_KEY` |
| Claude Code | `claude-agent-acp` (Zed's adapter, npm, built on the Claude Agent SDK) | Adapter | Uses the user's own Claude login **(unverified in Kit)** |
| Codex | `codex-acp` (agentclientprotocol org) | Adapter | Uses the user's Codex login **(unverified in Kit)** |
| Grok | none listed | — | Stays on today's headless adapter; the YOLO switch replaces `KIT_FULL_AUTO=1` |
| Ollama | via Codex bridge (ADR-0003) | — | Rides `codex-acp` if the bridge holds **(unverified)** |

Rust side: the `agent-client-protocol` crate (Apache-2.0, 2.2.0,
github.com/agentclientprotocol/rust-sdk) gives typed messages, so Kit does
not hand-roll the schema.

Because Cursor offers no client fs or terminal, **Kit cannot rely on being
the file firewall**. The worktree stays the boundary, as it is today.

## 4. Product (mind one)

### Who it's for

Someone with two or three agent subscriptions who is tired of three
terminals in three looks. They want one place to hand out work, watch it,
say yes or no to risky steps, and only keep what passes.

### What they get that no single agent gives them

1. **Same task, several agents, one screen.** Dispatch already fans out;
   now each run is a live conversation, not a log.
2. **YOLO by default; Kit answers every permission prompt.** DECISION
   (Mazen, 2026-09-27): the harness runs full power. Every
   `session/request_permission` from every agent is answered `allow-once`
   by Kit at once, and shows as a row in the transcript and the receipt.
   Setup asks once ("Run agents in YOLO mode? They can run any command
   in their worktree. [Y/n]", default yes) and stores it in
   `~/.kit/config.toml`. `--careful` or `[permissions] mode = "ask"` in a
   kit turns prompts back on. This also removes today's per-agent tricks
   (Claude's `acceptEdits`, Grok's `KIT_FULL_AUTO=1`): one switch for all.
   The safety net is the gate, not the prompt: nothing lands without a
   pass.
3. **Nothing ships unproven.** A session ends in the same Land → gate →
   receipt flow as a run. Talking is free; landing is gated.
4. **Kits apply to all of them.** The kit's skills, rules and MCP servers
   already land per agent; ACP's `session/new` also takes MCP servers, so a
   kit can add session-only servers without touching config files.

### Two modes, one room

- **Run** (exists): one task in, receipt out, no conversation. Stays the
  default for Dispatch and CI.
- **Session** (new): open a run and talk to it. Follow-ups go into the same
  ACP session in the same worktree. Replaces the `Attached` stub.

### What we deliberately lose

- Each agent's own slash-command UI and look. We surface ACP's
  `available_commands_update` as a `/` menu, which covers most of it.
- Features an adapter doesn't expose. The PTY escape hatch (`o`, "open in
  agent") hands the worktree to the real CLI for anything missing.

### Positioning

Lead with kits is unchanged. The harness is the second screen: "set up with
a kit, then run them side by side in the Control Room." It is not a new
product and not an IDE; no file tree, no editor.

## 5. Design (mind two)

Same grammar as today: Control Room table → Enter → run. Fox red `#FF5A1F`
is the accent and only the accent; agent identity comes from the #34 marks
and tiles, never from recolouring their output.

### Session screen (replaces `Attached`)

```text
 kit ▸ fix-login-redirect ▸ Claude Code                     RUNNING ⠋
 ─────────────────────────────────────────────────────────────────────
 you   The login redirect loops on Safari. Find it and fix it.
 ◆     Looking at the auth middleware first.
 ▸ read   src/auth/middleware.ts                                  done
 ▸ edit   src/auth/middleware.ts  +4 −1                           done
 ▸ shell  npm test -- auth                                   ASK  ▌
 ┌ Allow shell: npm test -- auth ────────────────────────────────┐
 │  y once   a always for this run   n no                        │
 └───────────────────────────────────────────────────────────────┘
 plan  ✓ find loop   ✓ patch cookie SameSite   ● run tests   ○ land
 ─────────────────────────────────────────────────────────────────────
 > type a follow-up                       Enter send · Esc back · L land
```

- Transcript rows: `you` in bold, agent text plain, tool calls as one
  collapsible row each (kind, target, diff size, status on the right).
- In YOLO (default) the permission box never appears; auto-allowed steps
  get a small `yolo` tag on their row. The header shows `YOLO` in fox red
  so it is never a secret.
- In `--careful` mode permission prompts are the only red box on screen.
  Waiting rows turn the Control Room STATE cell red (`ASK`), distinct from
  `RUN` per the #34 review (M3: running is not red).
- Plan line from ACP `plan` updates, pinned above the input.
- Motion stays on the one clock: the spinner in the header, nothing else.
- Diff view: `d` opens the existing RunDetail diff pane for the worktree.
- Narrow terminals: the plan line and tool status column drop first.
- Plain CLI parity: `kit run --watch` prints the same rows as text
  (no boxes; permission prompt becomes `Allow shell: npm test? [y/a/N]`).
  `--json` passes ACP updates through as JSON lines.

### Control Room additions

- STATE gains `ASK` (waiting on you) and `IDLE` (session open, agent done
  with its turn).
- Footer: `Enter open · a answer · L land · k kill`. `a` jumps to the oldest
  pending permission across all runs.

## 6. Engine: how it fits the crates

Contract files are Claude-only; each change below is called out.

1. **`kit-agents`: new `acp` module** (ordinary work). One client, driven by
   a per-agent `AcpLaunch { program, args, env }`. It maps ACP updates to
   deltas and parks permission requests on a channel. Existing headless
   adapters stay as the fallback when the ACP binary is missing.
2. **`kit-core` `RunDelta` (contract):** add
   `Message { role, text }`, `Tool { id, kind, title, status, diff_stat }`,
   `Plan(Vec<PlanEntry>)`, `Ask { id, title, options }`. `Output(String)`
   stays for headless agents. Receipt gets an optional `transcript` path;
   receipt `version` stays 1 if the field is additive and optional,
   otherwise bump to 2.
3. **`kit-agents` `Agent` trait (contract):** add
   `fn session(&self) -> Option<Box<dyn SessionAgent>>` with
   `prompt(text)`, `answer(ask_id, choice)`, `cancel()`. Default `None`, so
   Grok and headless paths compile unchanged.
4. **`kit-core` `AgentKind` (contract):** add `Cursor` (binary `agent`,
   which is a generic name: probe must confirm it is Cursor's by
   `agent --version` output before trusting it).
5. **`kit-tui` `event.rs` (contract):** no new variants needed;
   `RunUpdate(RunId, RunDelta)` carries the new deltas.
6. **Policy:** `mode = "yolo"` (default) or `"ask"`, from
   `~/.kit/config.toml`, overridable by `[permissions]` in a repo kit.toml
   and by `--careful` / `--yolo`. In `ask` mode an allowlist of exact
   commands (the gate's checks by default) still auto-allows. Evaluated in
   `kit-core`, answered by the ACP client, every answer recorded in the
   receipt.
7. **Adapter install:** `kit doctor` reports the ACP launcher per agent;
   `kit setup` offers to install it with the same pinned-launcher rules as
   MCP servers (e.g. `npx -y @zed-industries/claude-agent-acp@<pinned>`).

ADR-0001's "output is text, not a rich shared protocol" becomes the thing
this changes; write **ADR-0004: drive agents over ACP where offered** before
slice 2.

## 7. Trust and licences

- Kit never holds credentials. Every agent uses the user's own login; Kit
  passes env through, never stores it (same rule as today).
- Kit never bundles Claude Code, the Claude Agent SDK or Cursor. It runs
  whatever the user installed. The adapters are fetched pinned, like MCP
  servers, and labelled as code that runs.
- Adapter licences must be checked and shown before slice 2 (Zed and the
  ACP org publish under Apache-2.0 as far as we know **(unverified per
  repo)**). The Claude Agent SDK under the adapter has Anthropic's own
  terms; running it on the user's machine with the user's account is the
  intended use, redistributing it is not ours to do.
- YOLO is honest about its reach: the worktree is where the agent works,
  not a sandbox. A shell command can touch anything the user can. Setup's
  one question says so in plain words, and `kit doctor` shows the mode.
  A real sandbox (container or OS sandbox) is a later option, not a
  precondition.
- `allow-always` (careful mode) is scoped to the run, never global.
  Receipts list every permission asked and the answer, YOLO included.

## 8. Cut, build order, kill criteria

Thin slices, each shippable and each skeptic-reviewed by another thread.

| # | Slice | Proves | Size |
|---|---|---|---|
| 0 | ADR-0004 + this doc reviewed | Direction agreed | doc |
| 1 | Cursor as a 4th agent, headless (`agent -p --output-format stream-json`) on today's adapter | Cursor works in Kit at all | S |
| 2 | ACP client in `kit-agents` behind `KIT_ACP=1`, Cursor first, run mode only (one prompt, YOLO answers) | ACP maps onto runs and receipts | M |
| 3 | Claude and Codex via adapters, same run mode | All three on one protocol | M |
| 4 | Session screen in the TUI + `ASK` state + follow-ups | The harness experience | L |
| 5 | `kit run --watch` plain-CLI parity + JSON lines | Not TUI-only | S |
| 6 | ACP on by default where the launcher is present; headless stays fallback | Ship | S |

Kill criteria (stop and rethink if any hold after slice 3):

- An adapter needs Kit to hold or proxy credentials.
- Kit can't answer permission prompts for one of the three (then YOLO and
  careful mode both break for it).
- Cold start regresses past 100 ms or idle CPU past 1% (M0 kill).
- A session can land without a passing gate.

## 9. Decisions (Mazen, 2026-09-27)

1. Cursor is in: slice 1.
2. YOLO is the default: Kit auto-allows every permission prompt; careful
   mode is opt-in.
3. Grok stays on its headless adapter until xAI ships ACP. Under YOLO it
   no longer needs its own `KIT_FULL_AUTO=1`: the YOLO switch covers it.
