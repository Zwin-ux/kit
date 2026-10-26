# Continue Kit Red

Continue the existing mod under `mods/kit-red/`. The user wants a useful, minimal Claude Code experience with Kit identity. Concept art is provisionally approved; it is not native runtime proof. Latest user constraints and this current scope override archived expanded plans.

## Start here

1. Read the applicable repository instructions and inspect `git status --short`, the diff, and any active work before changing files. Preserve user changes in `tasks/` and other agents’ edits.
2. Read [README](README.md), [MOD-BUILD](MOD-BUILD.md), [SESSION-DESIGN](SESSION-DESIGN.md), then the actual mod files below. Resolve conflicts in favor of latest user direction and observed source; do not revive old Appearance/dashboard plans.
3. Check `claude --version` and the installed build’s mod API/help. Last verified build: `2.1.287`. Local Git started at `fb0ddf198256940fc243c8f23318c27d6ff1ea87`; the Kit catalogue was reviewed at upstream `c35f60855a62534f2ed71edac3d1dc52835b87a3`. Recheck before any upstream-dependent work. Do not reset or replace the dirty checkout.

## Product contract

Keep one native conversation and prompt. `/kit` opens a small picker for exactly these featured choices: **Frontend Design, Full-stack Design, Backend Engineer, LLM Engineer**. Their IDs are `frontend-design`, `fullstack-design`, `backend-engineer`, `llm-engineer`. They describe coding work, not providers or live agent processes. Full-stack includes frontend guidance; multiple choices are allowed.

Use restrained Kit red, a compact active-kit line, and the refined fox/role visual family where real terminal space permits. Keep labels usable without art or color. No permanent four-card workspace, sidebar, dashboard, Appearance settings, staged flow labels, progress strip or checklist onboarding. Preserve downstream native UI, transcript, permissions and ordinary typing. Never silently submit a sample prompt.

V1 Kit Red is the visual layer and makes zero Kit CLI calls. The user also authorized an effective mod; current work includes a limited V2 slice that adds role guidance to the next normal prompt. This does not authorize broad CLI management or execution. Selection does not install skills, hooks or MCP servers. “Set up the full toolkit” prepares an editable request to inspect official CLI help and a no-write plan, then obtain review before installation. It does not run the CLI itself. Keep that distinction visible and testable.

## Existing source map

| File under `mods/kit-red/` | Responsibility |
| --- | --- |
| `.claude-plugin/plugin.json` | Plugin identity/version |
| `hooks/hooks.json` | Module registration |
| `hooks/register.js` | Literal host hook registrations, command, picker, prompt guidance and draft fill |
| `hooks/roles.js` | Pure role data, parsing, examples and context composition |
| `hooks/art.js` | Static terminal Raster assets |
| `tests/kit.test.ts` | Three passing native scenarios: context, terminal/Desktop buttons, first arrival/editable examples |
| `tests/local.test.mjs` | Five offline scenarios for activation/errors, context/lifecycle and drawing/draft behavior |
| `package.json` | Private Node test entry; no runtime dependencies |
| `scripts/preview-art.mjs` | Development-only exact-source SVG preview generator |

Reinspect before editing: parallel work may have expanded tests since this handoff. Keep API calls in the host-supported module shape and imported helpers pure. No Rust engine/TUI changes or external runtime dependencies are needed for this slice.

Current role selection is **session-only**. Clear/resume/fork clear it; compaction retains it. Development reload clears module variables and supplies off guidance on the next prompt. One plugin-owned `intro-seen` boolean suppresses first-arrival replay after a pane was actually placed; it does not persist role choices. MOD-BUILD describes these exact boundaries. Verify actual host lifecycle behavior before promising restored state.

Current assets include a fox and four role sprites; the picker uses a single selected role emblem in the existing tall-dock header slot, or the fox with no/multiple choices. Inline/short panes use text. Native geometry and painting remain to be checked. Do not claim asset existence proves the accepted visual has been implemented.

## Native evidence and reproducible checks

Run these from the repository root, recording each exit code and full relevant output separately:

```sh
claude --version
npm --prefix mods/kit-red test
claude plugin validate ./mods/kit-red
claude plugin test ./mods/kit-red
```

Earlier on 2026-10-01, build `2.1.287` reported:

```text
hooks modules are turned off in this process: the rollout switch served off, and a plugin's tests run only while it is on
```

That error stopped recurring later in this pass. All three native tests now pass; plugin validation and five local scenarios pass too. The real headless CLI successfully loaded the plugin and handled `/kit frontend` as a local command with no model turn. Exact `2.1.287` declarations now exist in the ignored `.claude-plugin/types/` directory. Recheck rollout/build availability; never bypass protection controls. For an interactive visual check, use a disposable project in a normal standalone terminal:

```sh
claude --plugin-dir /Users/entreprenurecosystem/kit/mods/kit-red
```

Check the installed help if flags change. Do not remove nested-session protections to launch Claude from inside another Claude session. Desktop requires its own rendering/input check; terminal success does not establish Desktop support.

Current evidence: Node `v24.14.1`, five local scenarios; Claude `2.1.287`, validation and three native tests pass. Headless `/kit frontend` succeeds with zero model turns/API cost. Native tests stub placement and the model; they do not establish physical popup placement or model response behavior. Automatic panes are deferred on narrow terminals; `/kit` remains the explicit entry. Computer control refused access to Terminal for safety reasons, so actual terminal painting/focus/keyboard inspection remains open. Do not treat that UI limitation as a remote mods outage.

## Finish the current slice

1. **Guidance correctness:** cover all four IDs and aliases, comma-separated combinations, duplicate/invalid input, selection replacement and off. Preserve user text and pre-existing context. Verify Full-stack’s frontend composition. Establish the intended repeated-prompt behavior explicitly; current code reattaches selected guidance after a choice, so test stale guidance withdrawal and subsequent prompts, not only the first selection.
2. **Lifecycle truth:** test fresh session, clear, resume and fork against actual event semantics. Keep the default session-only. The visible active names and injected context must agree; never imply a saved project preference.
3. **Safe drafts:** examples and toolkit setup fill an empty prompt only. Existing input must survive; rejected/failed fill must explain that nothing was sent. A successful fill with a rejected pane close must keep the inserted draft and report that it was added. Command-registration rejection must preserve startup and pass through the conflicting command. Closing, selecting and rendering must make no Kit process/model/install calls. Missing Kit CLI must not break lightweight role guidance.
4. **Native composition and input:** foreign panes pass through, downstream AbovePrompt content survives, `/kit` works with and without arguments, multiple choices toggle correctly, off updates the line, Tab/Enter/Esc work and focus returns to the prompt. Check invalid input and noninteractive command text.
5. **Visual proof:** compare an unmodified Claude session with this mod. Check normal and narrow terminal sizes, long combined names, `NO_COLOR`, readable text fallback, fox/role-art cell geometry and no clipped buttons. Use screenshots from actual native sessions, clearly separate from concept art. Keep the native prompt/transcript dominant.
6. **Honest evidence:** remove stale test assumptions, add focused pure/offline cases where useful, then native cases when enabled. Record exact commands, versions, results, screenshots and unresolved platform gates. A local mock validates logic only; a manifest pass validates packaging only.

Do not mark native acceptance complete until hooks execute and the interactive flow is exercised. Publishing, global installation, changing standalone Kit, and the old broad V2 receipt/run/land/cancel designs are separate future work.

## Copyable continuation request

> Continue the existing Kit mod using docs/kit-red/README.md and HANDOFF.md, the latest user constraints, and actual mods/kit-red source. Preserve Claude’s native conversation and prompt. Keep the four real job kits—Frontend Design, Full-stack Design, Backend Engineer, LLM Engineer—with compact Kit identity and effective session-only guidance. Finish focused logic tests, safe editable drafts, lifecycle truth, keyboard/composition and terminal visual checks. Do not revive archived dashboards, Appearance panels, stage/progress/checklist flows, persistence systems or broad CLI integration. Recheck native runtime availability and preserve the passing tests; do not bypass the Terminal computer-control restriction or claim physical UI proof from render-tree tests. Report exact changes, commands/results, proof artifacts and remaining limitations.
