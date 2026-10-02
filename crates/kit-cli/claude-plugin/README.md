# Kit for native Claude Code

Requires Claude Code **2.1.287 or later**. Kit adds four specialist categories to the native host. Claude owns sign-in, the conversation, permissions and native subagent execution.

| Category | Native agent | Bundled Kit knowledge |
| --- | --- | --- |
| Frontend | `kit:frontend-ui-builder` | Interface craft, accessibility |
| Backend | `kit:backend-api-builder` | API design, backend verification |
| Security | `kit:security-reviewer` | Security review, dependency review |
| Product | `kit:product-spec-writer` | Product specification, acceptance review |

These eight skills are original Kit starter knowledge under MIT. They are not installed Animate UI, AWS, Azure, Trail of Bits or gstack integrations. See `provenance.json` and the repository's `docs/dev/CLAUDE-NATIVE.md` for readiness and future sources.

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

The compact band above Claude's prompt shows Frontend, Backend, Security and Product. Click a bucket (or focus the band with Claude's native focus shortcut) to inspect its real specialist and skills. Selection never starts work and no bare-letter/digit hotkeys intercept prompt typing. At narrow widths the strip uses two rows.

`/kit open` opens the native **Kit · Skills** pane. **Prepare draft** fills the native prompt; review it and press Enter yourself. The pane shows all observed Kit work regardless of the selected catalog bucket. **Refresh status** reads current native status; the display is a last-observed snapshot, not a global fleet monitor. Each observed native execution gets its own local task ID, separate specialist ID, native attempt ID and native parent owner. Repeated descriptions are not treated as retries of the same task.

Choose **Kit Red** in Claude's `/theme` picker for the native crimson accent. The plugin ships the theme without changing saved theme preferences or permission styling. Claude controls pane docking, prompt width and scroll regions; the Mod cannot force a full-width prompt beneath a docked pane.

The original 22×16 fox grid renders as one fixed 22×8 terminal Raster in the expanded empty pane when there is room. It stays out of the compact band and hides while attempts are present. Ear/blink gestures return to idle; no body translation or full-screen clearing occurs. **Hide fox**, **Pause motion**, `KIT_MOTION=off`, `NO_COLOR`, or `REDUCE_MOTION=1` reduce motion. The native 100 ms animation clock quantizes the suggested gesture durations. Desktop uses the text/control view without the terminal Raster.

## Explicit task workflow

- `/kit catalog` shows categories, specialists and their two preloaded skills.
- `/kit use frontend` (or backend/security/product) fills the visible prompt with an explicit `@agent-kit:...` invocation and preserves the task draft. Review and send it yourself. Selection is only intent for the next task.
- `/kit jobs` lists native Kit executions in this session. Native execution IDs remain distinct from specialist roles. Completed means **Needs review**. Missing execution data means **Not attached**, with no automatic restart.
- `/kit review <agentId@turnId>` prepares Security's review of that fixed result. `/kit handoff <agentId@turnId> <category>` prepares an explicit continuation. Both require an empty draft and retain the original result; neither sends anything.

Results and local task IDs exist only in this running Mod instance; reload, clear, resume, branch or session exit resets them. A review must check original acceptance criteria and the exact artifact or commit; result text is not proof. Writer agents request native worktree isolation and must stop editing if it is unavailable. Security has read-only tools. No global fleet, automatic task dispatch, Stop control or experimental agent teams are implemented.

## Offline verification

```text
claude plugin validate <directory> --strict --json
claude plugin test <source-directory>
node --test scripts/claude-plugin.test.mjs
```

Tests and generated declarations stay in the development source, outside the installed payload. On 2.1.287, `plugin details` reports zero agents even when runtime discovery loads all four. The packaging regression checks the real loader's diagnostic entries, exact declared files and skill preloads in a disposable configuration, using only the local `/kit catalog` command. This proves loading, not model execution or worktree behavior.

Claude generates `.claude-plugin/types` when the Mod loads. Type-check against the declarations generated by the installed version; do not copy the older public declarations into a release. The development `tsconfig.json` extends that generated configuration.
