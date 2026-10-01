# Native Claude Code checkpoint

The current request is a native Claude Code Mod with four categories: Frontend, Backend, Security and Product. It is independent of Kit's existing headless `claude -p` execution adapter and ratatui Control Room. The historical 1.0 plans do not define this new surface.

## Implemented

- `kit claude check` checks only the installed version against 2.1.287. It does not check authentication.
- `kit claude install` copies the embedded native payload into a fresh directory, preserving existing destinations. `--print` writes nothing.
- `kit claude launch` opens the native interactive CLI with only the explicit plugin path; `--print` previews the argument vector. A pipe or JSON-mode launch fails before opening Claude.
- `crates/kit-cli/claude-plugin` is the single package source, included in the Rust crate and shared by `scripts/sync-claude-plugin.mjs`.
- Four explicitly declared, flat agent files preload eight original Kit starter skills. Category hierarchy lives in `hooks/catalog.ts` rather than depending on recursive agent discovery.
- `/kit` lists the catalog, prepares visible invocation drafts, shows session-local native jobs, and prepares review/handoff drafts tied to immutable result IDs. It never submits a prompt or spawns an agent. Missing status remains unknown or Not attached. Completion requires review.

No third-party suite has been installed. The current skills are a bounded first-party scaffold, not equivalent upstream integrations. There are no approval-intercept hooks, MCP declarations, hidden prompt additions or authentication changes. The three writing roles request native worktree isolation. Actual isolated execution still needs verification with an explicitly requested user task.

## Discovery evidence and limits

Claude Code **2.1.287** was already installed; Node **24.18.0** and Rust/Cargo **1.97.1** were available. Setup did not install or update them. The locked Cargo build dependencies were downloaded for this checkout.

`plugin details` reports `Agents (0)` even with a minimal flat agent fixture. The real runtime loads all four explicitly declared Kit files and reports `Total plugin agents loaded: 4`, plus eight skills, during the local no-model `/kit catalog` command. `scripts/claude-plugin.test.mjs` packages a fresh bundle and checks these exact loader entries under a disposable Claude configuration. It also checks native IDs, manifest paths, skill preloads and preservation of an existing output directory. This is loading evidence, not successful model execution.

Installed-version declarations are generated locally under `.claude-plugin/types` and excluded from Git and the shipped payload. The public declaration file was older than the installed runtime. Offline hook tests use the generated 2.1.287 definitions, including complete `command.run` and `turn.complete` fixtures.

Passing at the checkpoint: TypeScript no-emit check, 9 offline Mods tests, 2 packaging/runtime-discovery tests, strict plugin validation, 4 native Rust unit tests, 2 real-binary Rust integration tests and workspace formatting. Cargo's package file list includes the payload and excludes generated declarations. The full workspace clippy and test gates remain pending before treating the feature as complete.

The completeness inventory found 13 public functions and zero stubs. It heuristically flagged four helpers as untested: `catalogText`, `findAgent` and `resultDraft` are exercised through the native command tests; `command_name` has a direct Rust unit test. The inventory is a prompt for review, not execution coverage evidence.

## Next work

1. Implement the parent-reviewed terminal character grid and layout specification. The native UI, fox and red theme are not implemented in this checkpoint. The parent owns comparison to the approved reference.
2. Keep the primary four-category strip in AbovePrompt and details in a supported right Pane. Do not use the old long project/path table. Preserve native input and approval surfaces.
3. Verify UI interactions and narrow terminal behavior; run full Rust gates and all Mods/package tests.
4. Perform an explicitly requested live task only after native login/agreements, if needed. Verify worktree isolation and real result/acceptance evidence before calling execution verified. Do not add a fake Stop control.

## Curated source queue (not installed)

The following pins were provided by the parent research pass. They describe candidates, not installed files or runtime readiness. Keep each upstream license and dependency closure when integration is later authorized.

| Category | Source and full revision | Candidate / constraints |
| --- | --- | --- |
| Frontend / Backend | `addyosmani/agent-skills` at `2686b620fc1fed2e8f60c704839c766b8594c6b6` | `frontend-ui-engineering`, `api-and-interface-design`; MIT |
| Frontend | `addyosmani/web-quality-skills` at `afa8da942115f2961fdbfa80807ea0b232ff6c00` | accessibility / core-web-vitals; MIT; preserve `../performance/references/MEASUREMENT.md` dependency |
| Backend | `supabase/agent-skills` at `551274ed2fe97c8fea1325f7ceb05803a542f8df` | Postgres v1.1.1; MIT |
| Backend | `aws/agent-toolkit-for-aws` at `acc890da1028c9c9e7c4d496238789d4d77f201d` | `skills/core-skills/aws-well-architected-review`; Apache-2.0; not deprecated samples |
| Backend | `microsoft/GitHub-Copilot-for-Azure` at `fda21b063fb8f2f93f423c8e7c61e75f2e58b7ad` | `plugins/azure-skills/skills/azure-enterprise-infra-planner`; MIT; do not inherit provisioning or `npx @latest` MCP startup |
| Security | `trailofbits/skills` at `82fe8226252622fa807643bdca1710901198553a` | differential-review / sharp-edges / variant-analysis; CC-BY-SA-4.0; retain attribution and license |
| Security / Product | gstack at `7fca42ad8b6c707b8a38f579f72bf3c4f7de6d85` | cso / office-hours / plan-ceo-review / spec; MIT; full runtime dependencies, optional providers/scanners/local state; never imply ready from prose alone |

Animate UI has no `SKILL.md` and its license includes Commons Clause. Use original Kit guidance and links, not a vendored skill. Readiness should distinguish **catalogued**, **files installed**, **runtime ready**, and **execution verified**. Do not preload entire large upstream suites.

Kit's existing `KIT.toml` schema remains unchanged. Its skill provenance uses `source`, `path`, full `rev`, and `licence`; native first-party metadata is separately recorded in `provenance.json`.

## Primary references

- [Mods overview](https://code.claude.com/docs/en/plugins/mods/overview), [create](https://code.claude.com/docs/en/plugins/mods/create), [test](https://code.claude.com/docs/en/plugins/mods/test), [interface](https://code.claude.com/docs/en/plugins/mods/interface), [reference](https://code.claude.com/docs/en/plugins/mods/reference)
- [Plugin components](https://code.claude.com/docs/en/plugins/components), [manifest reference](https://code.claude.com/docs/en/plugins/manifest-reference), [skills](https://code.claude.com/docs/en/skills), [subagents](https://code.claude.com/docs/en/sub-agents)

No relevant canonical Kit Notion/Brain page was found in the initial context search. Current explicit product instructions and installed-runtime evidence take precedence over the historical local plans.
