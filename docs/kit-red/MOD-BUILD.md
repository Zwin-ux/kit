# Build the small Kit mod

Current authority, 2026-10-01. The user has provisionally approved the concept direction and authorized an effective Claude Code mod. Keep the native conversation, prompt, permissions and tool output. Add Kit red, a compact fox when space permits, and four useful coding choices. No visible stages, progress strip, dashboard or Appearance panel. This replaces the earlier expanded build plans.

## What exists

Implementation lives in `mods/kit-red/`. It has no external runtime dependencies, no Kit process calls, no network calls and no new chat engine. It can provide guidance without the `kit` binary. The four featured roles are **Frontend Design, Full-stack Design, Backend Engineer and LLM Engineer**; they are Kit work specialties, not model providers or running agents.

| Implemented source behavior | Evidence / remaining limit |
|---|---|
| `/kit` command, four multi-select choices, off, aliases and combinations | Local logic and real headless command checked; physical focus remains unverified |
| Selected guidance appended to ordinary prompt context | Existing text, attachments and downstream context preserved in local check; model delivery not proven |
| Full-stack includes frontend guidance | Local context assertions pass |
| Compact AbovePrompt line preserves downstream tree | Local composition and narrow-name fallback checked; actual allocation unverified |
| First interactive activation requests a picker | Local/native tests cover request and stubbed placement; physical popup unverified |
| One saved `intro-seen` boolean after observed placement | Role choices themselves are not persisted; save errors remain visible |
| Editable examples and full-toolkit request | Local checks cover occupied prompt, denied fill and text typed during fill; no submission API called |
| Fox + four static role sprites | Packed geometry checked; selected role or fox uses one existing header slot; no native painting proof |
| Clear/resume/fork withdraw the prior choice; compaction keeps it | Local lifecycle checks pass; actual host lifecycle still unverified |

The concept art is direction, not a screenshot of this implementation. No standalone Kit theme edits, released package, persistent plugin installation or MCP installation have been performed.

## User flow

First activation requests the small native Kit pane. It asks `What are you making?` and shows the four named choices with a short description. Tab moves between native controls; Enter toggles a choice or invokes a button; Esc closes the pane. Return immediately to ordinary typing through `Back to chat`. No selection starts a model turn, installs a toolkit or launches workers.

`/kit frontend`, `/kit fullstack`, `/kit backend`, `/kit llm` select a role. Full stable IDs are accepted too. `/kit frontend,backend` composes multiple roles; duplicate names collapse. An invalid member rejects the whole replacement and retains the existing selection. `/kit off` withdraws Kit role preferences. Command results supply context, and the prompt hook reattaches the current preference to subsequent submitted messages.

`Try an example` fills an empty native draft. It reads the existing draft first and refuses an occupied prompt. The final fill uses append, so text typed between read and fill survives. The user reviews and submits it through Claude's normal editor. A failed fill keeps the pane open and explains that nothing was sent.

`Set up the full toolkit` prepares an editable request for Claude to inspect the official CLI, its current help and `kit add <ids> --agent claude --print`. It asks for actual files, scope, sources and code-running entries before installation. The mod itself neither invokes Kit nor approves that plan. This is a handoff to the existing installer, not an embedded installer or a claim that skills/MCP/hooks are available.

## Activation contract

| Situation | Required behavior |
|---|---|
| First interactive activation | Request native pane with focus and Esc dismissal |
| Pane genuinely placed | Save only `intro-seen: true` in plugin-owned store |
| Host defers automatic pane | Do not mark arrival complete; keep `/kit` entry |
| User explicitly runs `/kit` | Host may place it at any width; record successful placement |
| Later launch after saved arrival | Normal Claude prompt; no introduction replay |
| Headless invocation | Register command; provide argument help; no popup |
| Store/open fails | Continue normal session; explain the problem; role arguments remain available |

The [official pane rules](https://code.claude.com/docs/en/plugins/mods/interface#when-a-pane-waits-for-a-wider-terminal) defer unsolicited panes below 144 terminal columns, or below 110 once the person has opened that pane themselves. Requesting a popup is not proof it appeared. Native `/kit` is the reliable user-triggered path. Desktop placement and focus require their own proof.

## State and lifecycle

Selections are session-only module variables. Startup begins with guidance off. Clear, resume and fork clear the choice explicitly. Compaction retains it. Every later prompt receives current guidance, including an explicit off context that withdraws earlier preferences.

Development hot reload clears module variables. The next prompt receives off guidance, rather than silently retaining an old profile while the label shows none. This is a deliberate current ceiling. If retaining choices during live reload becomes required, migrate the choice to declared host `$.state`; do not add a database or project preference system. Do not confuse the globally shared first-arrival boolean with per-project role settings.

`NO_COLOR` suppresses branding color and Raster art. Failure to read the environment defaults to plain text. The fox requires a docked terminal pane, at least 58 allocated columns and 24 body rows. Read width from `e.props.bodyColumns` and docked height from `e.props.scroll.bodyRows`; the inline pane's current body height does not establish its maximum capacity. Inline and compact pickers use the text mark and keep full role names/controls ahead of artwork. Long combined names collapse to a role count in the band; `/kit` retains full names. A single selected role displays its emblem in that same slot; no/multiple choices display the fox. This adds no per-option height. Do not add four eight-row illustrations to a small pane.

## Engineering boundaries

Keep all literal `$` API calls and any helper receiving `$` in `hooks/register.js`. Imported `roles.js` and `art.js` remain pure. Preserve `next(e)` for foreign panes and native lifecycle events, and compose `await next(e)` for AbovePrompt. No render handler writes files, calls processes, runs checks or invokes models. Host validation enforces API declarations; a Node mock cannot enforce every native tree or permission rule.

Existing Claude permission prompts remain authoritative. The mod does not restyle them or change approval policy. Selection expresses desired working guidance; instructions from the current user/project still apply. The fox is inside supported Kit surfaces; no exposed API has been proven to replace Claude's global logo or recolor its entire UI.

## Run checks

From the Kit repository root:

```sh
npm --prefix mods/kit-red test
claude plugin validate mods/kit-red
claude plugin test mods/kit-red
```

Latest observed on 2026-10-01: Node `v24.14.1` passes **5 local scenario tests**; Claude `2.1.287` passes validation and **3 native test scenarios**. Earlier runs returned:

```text
hooks modules are turned off in this process: the rollout switch served off, and a plugin's tests run only while it is on
```

The [official troubleshooting guide](https://code.claude.com/docs/en/plugins/mods/troubleshoot) identifies that error as Anthropic remotely disabling installed mods; no local setting enables them. It stopped recurring during this pass without any protection bypass. Record it as historical, and recheck availability on later launches. Native automated tests now run successfully; generated screenshots still do not establish physical UI proof.

## Finish native visual verification

1. Recheck Claude version, validate, generate declarations for the installed build and run native tests. Exact `2.1.287` declarations have now been generated under the ignored `.claude-plugin/types/` directory; they win over the older public GitHub snapshot.
2. In a disposable project, launch `claude --plugin-dir /Users/entreprenurecosystem/kit/mods/kit-red`. Compare first arrival, subsequent arrival, explicit `/kit` and headless behavior. Record actual placement, not only API calls.
3. Submit harmless fixture prompts after each role, combination and off; verify native context delivery and preservation of user text/attachments. Check clear, resume, fork, compaction and development reload.
4. Exercise Tab, Enter, Esc, focus return, occupied draft, failed fill and a rapid typing/fill race. Verify selection and every control make no hidden model/installer call. Missing Kit CLI must not block role guidance.
5. Capture native terminal rendering at 60×20, 80×24, 120×40 and 160×40, with a second harmless mod, `NO_COLOR` and a prompt already in progress. Check prompt visibility, native permission visibility, clipping and scrolling. Verify Desktop separately or explicitly limit support.
6. Inspect fox/role art at native cell size before adding role visuals. Preserve the accepted visual family with minimal native geometry. No new dashboard or art generation is required by the remaining work.
7. Record build, commands, exit codes, screenshots and unresolved cases. Complete native acceptance before packaging/release claims. Publishing or global installation is not implied by local validation.

No alternate plugin-skill command is needed for the current working runtime. If mods become unavailable again, reassess that temporary path explicitly; it would be a normal model-turn command with different semantics and no native picker.

## Current engineering and asset pass

Command-registration conflicts now preserve the host session and pass the conflicting command downstream. An unreadable arrival preference still permits the popup attempt. Open/save failures have separate messages; a successful retry clears the prior error. A filled draft followed by a rejected pane close reports that the draft was added, rather than falsely reporting a fill failure. Five local scenarios cover these paths alongside context, lifecycle, geometry and safe drafts.

The three native tests explicitly raise startup and stub API calls. They check role context, mount terminal/Desktop trees and exercise role/off buttons, then check that first arrival requests a pane once and an example fills a draft without submitting. All three now pass in the real Claude test runner. They prove native hook/tree/callback behavior with stubs; they do not prove physical painting or actual model output.

[Production asset preview](production-sprites.html) decodes the exact packed source at an 8×16 px sample cell size, 2× inspection and a light-background study. It is not a screenshot of Claude. The grids use one red, cream, ink and grey palette; transparent cells retain the host background. The revised fox narrows the face, moves the eyes inward and tapers the muzzle. Role motifs match the approved browser/layers/drawers/three-node family. Light-background cream contrast remains a visual issue to resolve before claiming theme parity. Full text names remain usable.

Regenerate the static preview from repository root after changing art:

```sh
node mods/kit-red/scripts/preview-art.mjs > docs/kit-red/production-sprites.html
```

The preview script is development-only; the plugin still contains no runtime encoder, asset download or external dependency. Source-level independent engineering/design reviews found no further material behavioral issue after these corrections; they do not substitute for native painting, permission or keyboard proof.

## Real CLI load evidence

In a disposable `/tmp/kit-red-native.*` folder, this command loaded the actual plugin for one headless process without installing it globally or enabling any MCP server:

```sh
claude -p '/kit frontend' \
  --plugin-dir /Users/entreprenurecosystem/kit/mods/kit-red \
  --tools '' --strict-mcp-config --mcp-config '{"mcpServers":{}}' \
  --setting-sources '' --no-session-persistence \
  --output-format json --max-turns 1
```

Observed exit code 0; `local_command: custom`, `is_error: false`, `num_turns: 0`, `duration_api_ms: 0`, `total_cost_usd: 0`. Result: `kit-red: Kit: Frontend Design. Guidance for this conversation; no tools installed.` This proves real CLI loading/command handling, not delivery to an actual model turn.

The native session wrote build `2.1.287` declarations and a local extending tsconfig, both ignored. Computer control refused the Terminal app for safety reasons before an interactive launch, so physical painting/keyboard/focus inspection remains unfinished. Do not bypass that control or describe the old rollout error as the current blocker.
