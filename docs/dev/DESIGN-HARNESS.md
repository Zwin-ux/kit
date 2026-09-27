# Kit as the harness: one red room for Claude Code, Codex and Cursor

Status: design for after 2.0.0, 2026-09-27, revised after the skeptic review
(`/mnt/project-files/kit-2.0/harness/review.md`, product and design thread).
Asked for by Mazen ("combine kit to be an agent harness that can run all 3
and make it red… like a skin for codex/claude/cursor… utilize the open
source"). Nothing here changes 2.0.0 or its release.

Builds on `DESIGN-KITS.md` (kits lead, proof second), `DESIGN-tui.md`,
ADR-0001 (local process adapters) and the fox-red look from #34. Claims
nobody has run in Kit yet are marked **(unverified)**.

Contents:

1. The answer in one paragraph
2. Why not a literal skin
3. The protocol: ACP
4. Product (mind one): what a person gets
5. Design (mind two): what it looks like
6. Engine: how it fits the crates
7. Trust, boundaries and licences
8. Cut, build order, kill criteria
9. Decisions
10. Review log

---

## 1. The answer

Kit does not repaint Claude Code, Codex or Cursor. Kit draws its own
interface, in fox red, and drives each agent over the **Agent Client
Protocol (ACP)**, an open JSON-RPC protocol. Cursor speaks it natively;
Claude Code and Codex can be driven over it through open-source adapters.
The agent keeps its brain, its login and its model; Kit owns the screen,
the answers to the agent's permission prompts, the worktree, the gate and
the receipt. The pitch holds: "Kit sets your coding agents up for one job,
then proves what they do." The harness is how you *watch and steer* a run
while it does it.

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
newline-delimited. Kit is the **client**; the agent process is the
**agent**.

What Kit sends: `initialize`, `authenticate` (only to learn that the agent
needs a login, see §7), `session/new` (cwd = the run's worktree, MCP
servers), `session/prompt`, `session/cancel`, `session/load` (resume),
`session/set_mode`.

What comes back:

- `session/update` notifications: `agent_message_chunk`,
  `agent_thought_chunk`, `tool_call` / `tool_call_update` (kind, title,
  status, locations, diff content), `plan` (entries with status),
  `available_commands_update`, `current_mode_update`.
- `session/request_permission` requests. The agent offers options, each
  with an agent-chosen `optionId` and a `kind` (`allow_once`,
  `allow_always`, `reject_once`, `reject_always`). Kit picks an offered
  option **by kind**, falling back to the `_once` kind when `_always` isn't
  offered. Open prompts are answered `cancelled` on `session/cancel` and on
  kill.
- Tool kinds are `read`, `edit`, `delete`, `move`, `search`, `execute`,
  `think`, `fetch`, `switch_mode`, `other`, **as reported by the agent**.
  There is no "network" kind: a `curl` is `execute`.
- Optional client capabilities Kit may offer: `fs/read_text_file`,
  `fs/write_text_file`, `terminal/*`.

How each agent is driven (npm and crates.io checked 2026-09-27; Cursor's
docs page read the same day):

| Agent | Launch | Support | Notes |
|---|---|---|---|
| Cursor | `agent acp` | Native | Permissions and `session/load` yes; **no client fs, no client terminal**; modes agent/plan/ask. May require a `cursor_login` authenticate method and `cursor/*` extension handlers **(unverified)** |
| Claude Code | `@agentclientprotocol/claude-agent-acp` 0.81.2 (Apache-2.0, Node ≥ 22) | Adapter | Embeds `@anthropic-ai/claude-agent-sdk` (exact pin, licence "SEE LICENSE IN README"), which ships **its own** Claude Code per platform. Not the user's `claude` binary |
| Codex | `@agentclientprotocol/codex-acp` 1.13.1 (Apache-2.0, Node package) | Adapter | Embeds `@openai/codex` on a **caret** range (`^0.156.1`). Not the user's `codex` binary |
| Grok | none | — | Stays on today's headless adapter; the YOLO switch (§4) replaces `KIT_FULL_AUTO=1` |
| Ollama | — | Not in scope | Nobody owns checking whether ADR-0003's Codex bridge survives `codex-acp` |

Both adapters were renamed once already (from `@zed-industries/*`, now
deprecated). Kit's pin must be one line to move, and `kit doctor` says when
a pinned adapter is deprecated on npm.

Rust side: the `agent-client-protocol` crate (Apache-2.0, 2.2.0,
github.com/agentclientprotocol/rust-sdk) gives typed messages. It went
1.3.0 → 2.0.0 in three days in July, so Kit pins it exactly (`=2.2.0`).

## 4. Product (mind one)

### Who it's for

Someone with two or three agent subscriptions who is tired of three
terminals in three looks. They want one place to hand out work, watch it,
steer it, and only keep what passes.

### What they get that no single agent gives them

1. **Same task, several agents, one screen.** Dispatch already fans out;
   now each run shows its messages, tool calls and plan live, not a log.
2. **YOLO by default, one switch for every agent.** DECISION (Mazen,
   2026-09-27): the harness runs full power. YOLO means **anything in the
   worktree**, flagged when it reaches out.
   - **Allow, with one path check (Y1).** Kit allows every tool call at
     once: `read`, `edit`, `execute`, `fetch` and the rest. The exception
     is an `edit`, `delete` or `move` whose location resolves (after
     symlinks) outside the run's worktree: that gets one prompt, or a
     reject when there is no TTY. Normal work never sees it. It can't stop
     a shell command doing the same (§7).
   - **Per-agent switch, recorded (Y7).** Over ACP, YOLO **leaves the agent
     in a mode that asks**, and Kit answers every prompt at once. The user
     still sees no prompts and gets full speed, and every edit passes
     through Y1's path check. (An agent in a bypass mode never sends
     `request_permission`, so Kit would never see the edit.) The
     permissive flag is used only on headless paths, where Kit sees no
     prompts anyway: Claude `--permission-mode bypassPermissions`, Cursor
     `--force`, Grok `--always-approve`, Codex its full-auto sandbox
     setting **(flags unverified per agent version)**. The receipt records
     `permissions: yolo` plus which applied: "ACP, Kit answered" with the
     mode the agent reports, or the headless flag. As a second net, Kit
     checks the locations in every `tool_call` update (and in headless
     streams that report tool calls) and marks any edit outside the
     worktree `flagged` after the fact.
   - **Label only risky rows (Y4).** `execute` rows matching push,
     publish, deploy, `curl … | sh`, `sudo`, or `rm` of an absolute path,
     and any `fetch`, get a `flagged` tag. The receipt ends with one line
     ("ran 14 commands, 2 flagged: git push, curl"), and `kit land` prints
     it above the gate result. "Anything that runs code is labelled" stays
     true under YOLO.
   - **Trimmed environment (Y5, slice 2c).** A per-OS allowlist:
     - everywhere: PATH, HOME, USER, LANG, `LC_*`, TERM, TMPDIR, the
       network variables `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`,
       `SSL_CERT_FILE`, `NODE_EXTRA_CA_CERTS`;
     - Linux: the `XDG_*` dirs;
     - Windows: `SystemRoot`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`,
       `PATHEXT`, `COMSPEC`, `TEMP`, `TMP`;
     - each agent's own auth variables (`CURSOR_API_KEY`,
       `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, …), and its provider's
       credentials when that agent's provider switch is set (e.g.
       `CLAUDE_CODE_USE_BEDROCK` passes the AWS variables,
       `CLAUDE_CODE_USE_VERTEX` the Google ones);
     - the `$VAR`s the kit's MCP servers name.
     Anything else needs `--env NAME`. `kit doctor` lists the names passed,
     never the values. Logins still work; a stray `AWS_SECRET_ACCESS_KEY`
     no longer rides along to an agent that isn't on Bedrock.
   - **Setup asks once (Y8)**, before the first run, default yes: "Agents
     can run any command without asking. Kit's gate still decides what
     lands in your branch. Use YOLO mode? [Y/n]". Without a TTY the answer
     is not assumed: runs use the default and the header and receipt say
     YOLO. `kit doctor` shows the mode and how to change it
     (`kit config permissions careful` or `--careful`).
   - **The gate decides what lands in your branch (Y3).** It does not stop
     side effects before landing: a `git push` from the worktree, a
     deploy, an upload, an `rm` outside the tree.
   - Today's per-agent workarounds go: Claude's approvals were
     all-or-nothing in headless mode, and Grok needed `KIT_FULL_AUTO=1`.
   - **When `kit run` changes (F4).** Until slice 6, `kit run` keeps 2.0's
     behaviour: headless Claude uses `acceptEdits` and can't run shell
     unasked, and Grok needs `KIT_FULL_AUTO=1`. Slice 6 flips the default
     to YOLO for `kit run` too, CI included, with a changelog line ("`kit
     run` now lets agents run commands without asking; `--careful` keeps
     the old behaviour") and the mode in every receipt. From slice 6 Grok
     becomes auto-pickable under YOLO, last in the pick order (Claude,
     Codex, Cursor, Grok, Ollama).
   - **Careful mode** (`--careful`, slice 3b) sets each agent's asking
     mode and turns prompts on. It refuses to start an agent that can't
     ask (Grok headless, and Cursor headless if M2 holds): "Grok can't ask
     before it acts; drop --careful or pick another agent". It never
     quietly runs one in YOLO (Y6).
3. **Nothing ships unproven.** A steered run ends in the same land → gate
   → receipt flow as any run. Steering is free; landing is gated.
4. **Kits apply to all of them.** The kit's skills, rules and MCP servers
   already land per agent; ACP's `session/new` also takes MCP servers, so a
   kit can add session-only servers without touching config files.

### Runs you can steer

- **Run** (exists): one task in, receipt out. Stays the default for
  Dispatch and CI.
- **Steer** (new): open a live run from the Control Room and tell it what
  to change. Follow-ups go into the same ACP session in the same worktree.
  Replaces the `Attached` stub.

### What we deliberately lose

- Each agent's own slash-command UI and look. ACP's
  `available_commands_update` becomes a `/` menu, which covers most of it.
- Features an adapter doesn't expose. The PTY escape hatch (`o`, "open in
  agent") hands the worktree to the real CLI for anything missing.

### Positioning

Lead with kits is unchanged. The harness is the second screen: "set up with
a kit, then run them side by side in the Control Room." Not an IDE, not a
chat app: no file tree, no editor. **Rule: no screen in Kit starts with an
empty conversation.** The Control Room stays home; the steer view only
opens from a run that already has a task, and its input says "tell it what
to change".

## 5. Design (mind two)

Same grammar as today: Control Room table → Enter → run. Colour tokens are
the ones in `DESIGN-tui.md`; agent identity comes from the #34 marks and
tiles, never from recolouring their output.

### Steer view (replaces `Attached`)

```text
 kit ▸ fix-login-redirect ▸ Claude Code              YOLO  RUNNING ⠋
 ─────────────────────────────────────────────────────────────────────
 task  The login redirect loops on Safari. Find it and fix it.
 ◆     Looking at the auth middleware first.
 ▸ read   src/auth/middleware.ts                                  done
 ▸ edit   src/auth/middleware.ts  +4 −1                           done
 ▸ exec   npm test -- auth                                        done
 ▸ exec   git push origin fix-login                       flagged  done
 plan  ✓ find loop   ✓ patch cookie SameSite   ● run tests   ○ land
 ─────────────────────────────────────────────────────────────────────
 i tell it what to change          d diff · l land · Esc back
```

A prompt, which under YOLO appears only for an edit outside the worktree
(Y1) and in careful mode for anything the agent asks about:

```text
 ▸ exec   npm test -- auth                                  [ASK]
 ┌ Allow exec: npm test -- auth ─────────────────────────────────┐
 │  y once   a always for this run   n no                        │
 └───────────────────────────────────────────────────────────────┘
```

- **Colour.** `accent` (fox red) means "focus / needs you". ASK is an
  `accent` **filled chip** with the word ASK, and the prompt box border is
  `accent`. Never `danger` (that is FAIL) and never a red row. `live` stays
  RUNNING (teal). On ANSI16, where accent and danger both map to red, the
  chip's reverse video and the word carry the meaning; colour is never the
  only signal. The `YOLO` badge in the header uses `accent` text, not a
  chip, so it is visible without shouting. The Control Room shows one small
  `YOLO` in its header row, never in the STATE column.
- **Focus model.** The transcript has focus by default, so single-letter
  keys are commands. `i` or `/` moves focus into the input (`/` opens the
  command menu); Esc leaves the input, and Esc again goes back. While a
  permission box is open it captures `y`/`a`/`n` and nothing else. The
  footer always shows which keys are live for the current focus.
- **Keys match the Control Room:** lowercase `l` for land (as `[l]and`
  today), `d` diff, Esc back.
- Transcript rows: the task first (`task`), follow-ups as `you` in bold,
  agent text plain, tool calls one collapsible row each (kind, target,
  diff size, status on the right). Only risky rows carry a `flagged` tag
  (`warn`), per §4.2; ordinary rows carry nothing.
- Plan line from ACP `plan` updates, pinned above the input.
- **Motion and streaming.** The header spinner is the only motion, on the
  one clock. Streaming chunks mark state dirty; redraws happen at most once
  per `AnimationTick`, never per chunk, so a chatty agent can't break the
  1% idle CPU budget. A test counts frames per second while a fake agent
  streams.
- Narrow terminals: the plan line and tool status column drop first.

### Control Room additions

- STATE gains `ASK` (the accent chip, careful mode only) and `IDLE`
  (session open, agent finished its turn; `dim`). Under `NO_COLOR` /
  `KIT_MOTION=off` both are words only, no chip, no spinner.
- Footer: `Enter open · l land · k kill`. `a answer` appears only while
  some run is in ASK (careful mode, or a Y1 out-of-worktree prompt) and
  jumps to the oldest open prompt; under YOLO it would be a dead key.

### Plain CLI

- `kit run --watch` prints the same rows as text. A careful-mode prompt is
  `Allow exec: npm test? [y/a/N]` on a TTY. **When stdin is not a TTY it
  never blocks:** it answers by the configured mode and prints the
  decision (`auto-allowed exec: npm test (yolo)` or
  `rejected exec: npm test (careful, no terminal)`).
- `--json` emits Kit's own delta events as JSON lines, not raw ACP. It
  never includes `session/new` params, and env values are redacted.

## 6. Engine: how it fits the crates

Contract files are Claude-only; each change below is called out.

1. **`kit-agents`: new `acp` module** (ordinary work). One client, driven by
   a per-agent `AcpLaunch { program, args, env }` where `program` is
   `node` and `args[0]` is the adapter's resolved bin file (§6.7), or
   Cursor's `agent`. It maps ACP updates to deltas and parks permission
   requests on a channel. Headless adapters stay as the fallback when no
   ACP launcher is installed.
2. **`kit-core` `RunDelta` (contract):** add `Message { role, text }`,
   `Tool { id, kind, title, status, diff_stat }`, `Plan(Vec<PlanEntry>)`,
   `Ask { id, title, options }`, `Mode(String)`. `Output(String)` stays for
   headless agents. The receipt's optional `transcript` field is decided
   together with the after-2.0 receipt JSON contract (P10), so the version
   question is settled once.
3. **`kit-agents` `Agent` trait (contract):** add
   `fn session(&self) -> Option<Box<dyn SessionAgent>>` with
   `prompt(text)`, `answer(ask_id, kind)`, `cancel()`. Default `None`, so
   Grok and headless paths compile unchanged.
4. **`kit-core` `AgentKind` (contract):** add `Cursor`. Its binary is
   `agent`, a generic name, so probe confirms both that `agent --version`
   identifies Cursor and that it resolves to a user-installed path; that
   is part of slice 1's acceptance test.
5. **`kit-tui` `event.rs` (contract):** no new variants;
   `RunUpdate(RunId, RunDelta)` carries the new deltas.
6. **Permissions config.** YOLO needs no policy engine: allow everything,
   plus the Y1 path check (§4.2). The config below arrives with careful
   mode, after slice 3.
   - `mode = "yolo" | "ask"` and an optional allowlist of exact commands
     that auto-allow in `ask` mode.
   - **Only the user can loosen.** `~/.kit/config.toml` and `--yolo` may
     set `yolo`. A repo `kit.toml` or a marketplace `KIT.toml` may only
     tighten: set `ask` or add `ask` rules. A loosening entry there is
     ignored with a doctor note, so a cloned repo can't override a user
     who chose careful.
   - A `KIT.toml` may *suggest* allowlist entries; `kit add` shows them as
     "lets the agent run: …" behind the same `Continue? [y/N]` as hooks
     before they are written into the user's config. A kit never sets
     `mode`.
   - Gate checks auto-allow in `ask` mode only when they are checks Kit
     recorded (the same rule as DECISION 18:32).
   - No `network` key: ACP has no network kind, and Kit won't promise what
     it can't enforce.
   - Evaluated in `kit-core`, answered by the ACP client, every answer
     recorded in the receipt.
7. **Adapter install.** Never `npx`. Kit ships a lockfile per adapter and
   installs it into `~/.kit/adapters/<name>-<version>/` by running npm's own
   JS entry with node (`node <npm-cli.js> ci --ignore-scripts`), never
   `npm.cmd`, so nothing runs through cmd.exe and no install script runs
   before the user has seen the label. Slice 2c checks that both adapters
   work without scripts (the SDK's per-platform binaries arrive as
   optional dependencies). Kit
   records its hash like any kit entry, and spawns `node <resolved bin>`
   directly. That pins every transitive dependency (the SDK, and Codex
   despite its caret range), downloads once instead of per session, needs
   no network at session start, and avoids `npx.cmd`, which on Windows
   would run through cmd.exe, the shell Kit promises never to use.
   `kit setup` offers the install and labels it as code that runs; `kit
   doctor` reports each adapter's version, hash and deprecation, and
   checks that Node is 22 or later (the Claude adapter needs it).

ADR-0001's "output is text, not a rich shared protocol" is what this
changes; write **ADR-0004: drive agents over ACP where offered** before
slice 2a.

## 7. Trust, boundaries and licences

- **The worktree, and later a sandbox, is the only boundary.** ACP lets
  Kit answer the prompts an agent chooses to send; the agent's own mode and
  settings decide whether it sends one, and the tool kind is whatever the
  agent says. So Kit's permission answers are advisory in both modes. Kit
  makes them provable, not assumed: it sets each agent's mode at start,
  records the mode the agent reports, and records every prompt and answer.
  The one hard rule Kit adds is Y1: under YOLO it refuses tool-call edits
  outside the worktree. A container or OS sandbox is a later option, not a
  precondition.
- YOLO is honest about its reach: a shell command can touch anything the
  user can, and the gate decides what lands in the branch, not what
  happens on the machine. Setup's one question says so in plain words, and
  `kit doctor` shows the mode. The trimmed environment (§4.2) is the
  cheapest real reduction of blast radius before a sandbox.
- **Kit never holds credentials.** Every agent logs in through its own CLI.
  If an agent returns `auth_required` or offers an authenticate method
  (Cursor's `cursor_login`, `codex-acp` opening a browser), Kit stops the
  run and says "log in with `agent login`" (or `claude` / `codex`). Kit
  never runs a login flow and never takes a key.
- **Secrets stay out of records.** MCP env for `session/new` is resolved
  from `$VAR` and written only to the agent's stdin. Transcripts are 0600
  under `~/.kit/runs/<id>/`, never in the repo's team copy, and never
  contain `session/new` params. Tool-call content can hold whatever the
  agent read (a `.env` included), which is why transcripts stay private.
- **What Kit causes to be installed.** Kit doesn't bundle Claude Code, the
  Claude Agent SDK, Codex or Cursor. But installing an adapter does fetch
  the SDK (proprietary, "SEE LICENSE IN README") and a copy of Codex onto
  the user's machine, and Kit is what triggers that. Setup and doctor show
  each adapter's licence **and** the licence of the agent it embeds, and
  say that the adapter runs its own copy of the agent, not the user's
  installed one (version, settings and bugs can differ). Running it with
  the user's own account is the intended use; redistributing it is not
  ours to do.
- `allow-always` (careful mode) is scoped to the run, never global.

## 8. Cut, build order, kill criteria

Thin slices, each shippable and each skeptic-reviewed by another thread.

| # | Slice | Proves | Size |
|---|---|---|---|
| 0 | ADR-0004 + this doc reviewed | Direction agreed | doc |
| 1 | Cursor as a 4th agent, headless (`agent -p --output-format stream-json`) on today's adapter. First check whether print mode can edit without also running shell unasked (`--force`); if not, Cursor sits behind the YOLO switch like Grok, and the row says so **(unverified)** | Cursor works in Kit at all | S (edit-only matters only for careful mode, 3b) |
| 2a | ACP client in `kit-agents` behind `KIT_ACP=1`, run mode, YOLO answers with the Y1 path check, `Output` deltas only, plus a **fake ACP agent fixture** in the repo (CI can't log into Cursor and the container can't run real agents). Cursor first only if M1's auth check passes, else the fixture only | ACP maps onto runs | M |
| 2b | `RunDelta` additions, mode and flag recording, risky-row flags and the receipt count line, private transcript | Receipts carry the proof | M |
| 2c | Adapter install from a Kit lockfile into `~/.kit/adapters` (§6.7), env allowlist | Pinned, shell-free launch | M |
| 3 | Claude and Codex via adapters, same run mode | All three on one protocol | M |
| 3b | Careful mode and the permissions config (§6.6): tighten-only from repos and kits, refuse agents that can't ask | Careful is honest | S |
| 4 | Steer view in the TUI, `ASK`/`IDLE` states, follow-ups, focus model, frame-rate test | The harness experience | L |
| 5 | `kit run --watch` plain-CLI parity, non-TTY answers, JSON lines | Not TUI-only | S |
| 6 | ACP on by default where an adapter is installed; headless stays fallback. `kit run` defaults to YOLO (F4), changelog line, Grok auto-pickable | Ship | S |

Kill criteria (stop and rethink if any hold after slice 3):

- An adapter needs Kit to hold or proxy credentials.
- Kit can't answer permission prompts for one of the three.
- In careful mode, an agent runs an `execute` tool without asking.
- A YOLO run changes a file outside its worktree through an `edit`,
  `delete` or `move` tool call Kit allowed (Y1 failed).
- A repo or kit loosens a user's careful choice (Y2 failed).
- Cold start regresses past 100 ms or idle CPU past 1% (M0 kill).
- A run can land without a passing gate.

## 9. Decisions (Mazen, 2026-09-27)

1. Cursor is in: slice 1.
2. YOLO is the default: Kit allows everything inside the worktree without
   asking you; careful mode is opt-in.
3. Grok stays on its headless adapter until xAI ships ACP. The YOLO switch
   replaces its own `KIT_FULL_AUTO=1`.

## 10. Review log

Skeptic review by the product and design thread, 2026-09-27
(`kit-2.0/harness/review.md`). Direction approved. Fixed in this revision:

- H1 adapter names → §3 table, rename risk, doctor deprecation check.
- H2 adapters embed their own agent, caret range → §3 table, §6.7 lockfile
  install, §7 licences.
- H3 prompts are advisory, no network kind → §4.2, §6.6, §7, new kill
  criterion.
- H4 kits can't grant shell → §6.6.
- M1 Cursor auth **(unverified)** → §3, §7, slice 2a. M2 Cursor headless
  corner → slice 1. M3 secrets → §7, §5 JSON. M4 permission options by
  kind → §3. M5 slice split → 2a–2c, 3b. M6 steering, not chat → §4.
- D1 ASK colour → §5. D2 focus model → §5. D3 frame coalescing → §5.
  D4 non-TTY → §5 Plain CLI.
- L1 wording, L2 probe test, L3 Ollama out of scope, L4 receipt field tied
  to P10.

Second pass on b3b7e15, same thread. H1/H2 were still open there and are
fixed above. Added:

- Y1 YOLO is "anything in the worktree" → §4.2, kill criterion.
- Y2 only the user loosens → §6.6, kill criterion.
- Y3 "the gate decides what lands in your branch" → §4.2, §7.
- Y4 label risky rows only → §4.2, §5, slice 2b. Y5 env allowlist → §4.2,
  slice 2c. Y6 careful refuses agents that can't ask → §4.2, slice 3b.
  Y7 per-agent YOLO flag recorded → §4.2, slice 2b. Y8 setup question →
  §4.2.
- Design: mockup is the YOLO default, prompt in a second mock; `a answer`
  only while a run is in ASK; `YOLO` in the Control Room header only.
- Careful mode and the allowlist moved to slice 3b.

Final check on 6643135, same thread: approved for ADR-0004 after F1.

- F1 Y7 switched Y1 off → over ACP YOLO keeps the agent asking and Kit
  answers; permissive flags on headless paths only; after-the-fact flag on
  out-of-worktree edits; decision 9.2 reworded.
- F2 per-OS env allowlist, provider credentials by switch → §4.2.
- F3 npm via node, `--ignore-scripts`, Node ≥ 22 check → §6.7.
- F4 `kit run` flips to YOLO in slice 6 with a changelog line; Grok
  auto-pickable from then → §4.2, slice 6.
- L-a, L-b fixed.
