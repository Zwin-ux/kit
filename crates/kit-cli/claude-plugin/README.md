# Kit for native Claude Code

Requires Claude Code **2.1.287 or later**. Kit adds four specialist categories to the native host. Claude owns sign-in, the conversation, permissions and native subagent execution.

**Private alpha candidate — 2.0.0-alpha.4.** This native Mod has its own prerelease version, separate from the Kit CLI. Source/package checks and local loading are verified; interactive appearance, visible draft routing and captured-result review still need native acceptance. Four genuine native role tasks and three isolated writer worktrees were verified on the prior alpha.1 archive; that evidence does not accept this revised UI. Loading or a compatible version number alone does not establish Mods rollout access. Keep this candidate local for owner review; public beta and stable are separate promotion steps.

| Role | Native agent | Four preloaded skills |
| --- | --- | --- |
| Frontend | `kit:frontend-ui-builder` | Animate UI guidance, Vercel React best practices, Vercel composition patterns, accessibility |
| Backend | `kit:backend-api-builder` | API design, test-driven development, systematic debugging, Supabase Postgres best practices |
| Security | `kit:security-reviewer` | Audit context building, insecure defaults, sharp edges, variant analysis |
| Product | `kit:product-spec-writer` | Create PRD, product strategy, opportunity solution tree, prioritization frameworks |

Fifteen full upstream skills and one original Kit Animate UI integration guide are bundled, with their supporting resources. Exact commits, paths, adaptations and file hashes are in `provenance.json`; each upstream directory includes attribution. Kit code and the Frontend/Backend/Product guidance use MIT. Trail of Bits Security knowledge remains **CC-BY-SA-4.0**, with its original license and share-alike notices. Vercel's MIT declarations are preserved; that pinned upstream has no root LICENSE file. Animate UI component code is not included; its separate MIT + Commons Clause conditions remain documented.

These are knowledge resources, not installed scanners, component libraries, service connections or extra tools. Native role permissions remain authoritative. Security inspects source with Read/Glob/Grep only. Three writing roles request native worktree isolation. Loading is verified separately from useful execution.

## Prepare and launch

From a Rust Kit build:

```text
kit claude check
kit claude install --dir .kit/claude-plugin
kit claude launch --plugin-dir .kit/claude-plugin --print
kit claude launch --plugin-dir .kit/claude-plugin
```

Installation creates a fresh local directory. An existing destination is preserved; choose a new directory when upgrading. It does not enable a persistent plugin or edit Claude settings. Launch requires an interactive terminal and passes only `--plugin-dir` to native Claude. Complete any normal sign-in or agreements in Claude itself.

Alternatively load this source directory explicitly with `claude --plugin-dir <directory>`. The Node exporter packages the same source with `node scripts/sync-claude-plugin.mjs`; use `--out <fresh-directory>` for another output location.

## Native interface

The compact band above Claude's prompt shows Frontend, Backend, Security and Product. Click a bucket (or focus the band with Claude's native focus shortcut) to inspect its real specialist and skills. Selection never starts work and no bare-letter/digit hotkeys intercept prompt typing. At narrow widths the strip uses two rows when room permits. Short bands omit rules/padding before abbreviating into one row. Kit composes with downstream bands when their row bounds are known; a full or opaque downstream layout keeps the band unchanged, with `/kit open` still available.

`/kit open` opens the native **Kit · Skills** pane. **Prepare draft** fills the native prompt; review it and press Enter yourself. The pane shows all observed Kit work regardless of the selected catalog bucket. **Refresh status** reads current native status; the display is a last-observed snapshot, not a global fleet monitor. Each observed native execution gets its own local task ID, separate specialist ID, native attempt ID and native parent owner. Repeated descriptions are not treated as retries of the same task.

Choose **Kit Red** in Claude's `/theme` picker for the native crimson accent. The plugin ships the theme without changing saved theme preferences or permission styling. Claude controls pane docking, prompt width and scroll regions; the Mod cannot force a full-width prompt beneath a docked pane.

The original 22×16 fox grid renders as one fixed 22×8 terminal Raster in the expanded empty pane when there is room. It stays out of the compact band and hides while attempts are present. Ear/blink gestures return to idle; no body translation or full-screen clearing occurs. **Hide fox**, **Pause motion**, `KIT_MOTION=off`, `NO_COLOR`, or `REDUCE_MOTION=1` reduce motion. Unreadable motion settings default to motion off. The native 100 ms animation clock quantizes the suggested gesture durations. Desktop uses the text/control view without the terminal Raster.

## Explicit task workflow

- `/kit catalog` shows categories, specialists and their four preloaded skills.
- `/kit use frontend` (or backend/security/product) appends only an explicit `@agent-kit:...` mention to the visible prompt. It never copies back or replaces the task text. An observed existing agent mention is left for you to edit. Review and send it yourself. Selection is only intent for the next task. Draft preparation locks before reading the prompt; clear/resume/branch cancel a pending read.
- `/kit jobs` lists native Kit executions in this session. Native execution IDs remain distinct from specialist roles. Completed means **Needs review**. Missing execution data means **Not attached**, with no automatic restart.
- `/kit review <agentId@turnId>` prepares Security's review of that fixed result. `/kit handoff <agentId@turnId> <category>` prepares an explicit continuation. Both append only after observing an empty draft and retain the original result; neither sends anything. Agent mentions inside captured task/answer data are JSON-escaped and decode to the original text.

Claude's fill API has no atomic draft/session guard. Typing after the read is preserved; a fill already submitted when a session changes may still append there. Kit reports that race and never attempts a replacement rollback. Always inspect the visible prompt before sending, including any newer agent mentions or task text.

Kit enables its command and drawings only after successful native registration; refusal preserves downstream command/UI handlers and starts no Kit clock. The newest status request owns the live display. Completion capture retains verified same-session native identity from status reads and a passive successful-start observer. Foreground pruning or temporary status denial does not discard a fixed answer; missing verified attempts stay Not attached. The observer preserves native dispatch unchanged. A pane action still awaiting status is cancelled across a session reset; an already-submitted native UI call cannot be cancelled by Kit.

Results and local task IDs exist only in this running Mod instance; reload, clear, resume, branch or session exit resets them. A review must check original acceptance criteria and the exact artifact or commit; result text is not proof. Writer agents request native worktree isolation and must stop editing if it is unavailable. Security has read-only tools. No global fleet, automatic task dispatch, Stop control or experimental agent teams are implemented.

## Offline verification

```text
claude plugin validate <directory> --strict --json
claude plugin test <source-directory>
node --test scripts/claude-plugin.test.mjs
node --test scripts/native-lifecycle.test.mjs
```

Tests and generated declarations stay in the development source, outside the installed payload. On 2.1.287, `plugin details` reports zero agents even when runtime discovery loads all four. The packaging regression checks the real loader's diagnostic entries, exact declared files and skill preloads in a disposable configuration, using only the local `/kit catalog` command. This proves loading, not model execution or worktree behavior.

The source lifecycle regressions transpile production hooks with existing TypeScript and use controlled host doubles. They verify draft/status/session control flow without Claude's hooks test runner; they do not prove native painting, agent-mention parsing on submission or model execution.

Claude generates `.claude-plugin/types` when the Mod loads. Type-check against the declarations generated by the installed version; do not copy the older public declarations into a release. The development `tsconfig.json` extends that generated configuration.
