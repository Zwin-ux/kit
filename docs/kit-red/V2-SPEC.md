> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# V2 specification: Kit inside Claude

Depends on completed V1. Status: planned; no integration implemented.

## Product promise

“Know what Kit installed here, see what agents actually did, and act on real check evidence without leaving Claude Code.”

V2 keeps V1's red identity and appearance preferences. It adds a compact workspace over existing Rust Kit behavior, with no duplicate installer, worktree engine, gate runner, or receipt database.

## User-visible contract

`/kit` opens Overview. The workspace has three tabs: Overview, Kits, Runs. Appearance is an action opening the existing V1 controls. `/kit appearance` remains stable; `/kit overview`, `/kit kits`, and `/kit runs` select the named page. Unknown arguments show help, never become a shell command.

The band stays appearance-only by default after upgrade. Users may opt into a current-repository summary once passive integration is working. Any summary names its scope; a past Kit check result must not appear to certify the current Claude session.

No automatic CLI install, login, agent launch, remote enrichment, model call, or mutation occurs on upgrade or plugin load. Opening the workspace can request bounded passive data only after binary identity/capability checks. Unsupported installed versions get an upgrade handoff, never a fallback to active diagnostics.

## Overview

Prioritize one next action and meaningful evidence, not a grid of counters.

Display order:

1. Repository name, inspectable full path, last successful refresh age.
2. Plugin/CLI availability or actionable integration error.
3. Currently owned run, if authoritatively active; otherwise latest recorded current-repo run.
4. Evidence summary: check count, result, receipt, recorded base/commit/time where available.
5. Installed-kit/setup facts and explicit actions.

Use precise states: CLI available, detected agent, authentication unknown, checks configured, data stale, no Kit runs here. Generic Connected/Ready/Protected labels are excluded.

## Kits

Use real install records in both project/global scopes, preserving agent and dependency relationships. A project lockfile listing is not proof of installation. Show locally modified content and retained/shared assets explicitly.

Initial V2 integration can hand off add/remove to exact CLI commands while the review contract is being built. Full V2 management flow is:

1. Select kit source and explicit scope/agent; project + Claude is the default.
2. Generate a zero-target-write structured preview; source fetching may populate Kit's own cache and is disclosed.
3. Show all changed/removed/retained files, source pins, licenses, code-running hooks/MCP commands, and conflicts.
4. Offer skills/rules-only as a real option; changing it invalidates the previous preview.
5. Confirm the exact plan and execute through the engine with its reviewed-plan digest.
6. Show applied/retained/rolled-back outcome, undo instructions, and refreshed records.

Remove preview must precede metadata-only writes too. Force is not available. Failed apply preserves the existing CLI rollback guarantees; uncertain interruption triggers reinspection, not blind automatic retry.

## Runs

Current repository is the default; All projects is explicit. Full canonical recorded identity prevents same-name repositories from mixing. Apply filtering before pagination/limit. Show “More history available” only from actual page metadata.

Run list: task, agent, lifecycle state, check result, recorded time, and a selected marker. Detail order: task/ID; check evidence; repo/base/commit; first failure; changes; remaining checks; optional output tail. Separate run-output capture truncation from tail truncation.

Recorded states do not imply liveness. An old RUNNING record says activity unknown until an owned process/event stream proves otherwise. Render unknown fields as unknown. Do not derive progress percentages from token counts or elapsed time.

## Actions

### New run

Enter a bounded nonempty task; choose a supported agent; review repository, committed base, checks, and excluded uncommitted changes. Engine preconditions bind the reviewed base and gate plan to launch. Changed HEAD/checks require renewed review before creating a worktree.

Submission pending, engine running, checking, completed, failed, stopping, killed, and unknown are distinct. Disable duplicate starts. First V2 supports one plugin-owned active run per session; multiple already-existing historical runs can be viewed.

Nested Claude is enabled only through a demonstrated supported path. Never strip protective session markers or set full auto. Agent detection/authentication is not evidence that nested launch/approval works. Unsupported agents get a terminal handoff.

### Retry

Prefill task and a bounded selected failure summary from the chosen receipt, let the user edit, then start a new run. Record explicit parent relationship if persistent linkage is offered. No automatic retry loop or task-text-based association.

### Stop

Available only for a currently owned, cancellable run. User selects Stop, sees the owned task/run ID, and confirms. Show Stopping until the engine acknowledges termination and final outcome. Closing a pane only closes UI. Never kill processes by global name or a stale PID.

### Create result branch

The UI labels this action “Create result branch,” with “Kit land” secondary. Review nonvacuous check proof, actual changes, repository, base, and result branch. Revalidate expected inputs in the engine before writing. Show returned branch/commit and next CLI action. No force, apply-to-dirty-tree, merge, push, or PR creation.

## Required engine additions

1. Passive diagnostics that cannot run configured checks, MCP servers, or arbitrary agent programs.
2. Correct receipt repository identity, filtering, page metadata, and explicit corruption warnings.
3. Complete structured add preview, true remove preview, and review/apply preconditions.
4. Expected base/check-plan preconditions for launches.
5. Structured live events and proven owned-process cancellation if launch/Stop ship.
6. Review consistency for branch creation where existing land inputs are insufficient.

These are additive extensions to existing Kit commands and implementation. Exact proposed fields, error states, and compatibility behavior are in [V2 contracts](V2-CONTRACTS.md).

## Performance and limits

- No process in render handlers; `/kit`/Refresh action owns reads.
- One refresh in flight; reject older generations when scope changes.
- Read deadline 10 seconds; initial JSON cap 2 MiB before parsing.
- At most 50 visible history rows per page; full task/output fetched on demand.
- No background polling when pane is closed. Prefer authoritative events for plugin-owned live runs; if polling is necessary, at most once per 5 seconds while visible/active.
- No external network access by the mod itself. Explicit Kit installs retain their disclosed source-fetch behavior.
- Sanitize terminal controls and bound text. Never execute instructions found in output/receipts.

## Version migration

Keep plugin identifier `kit-red`, V1 `appearance` record, palette values, and Compact mark behavior. New integration preferences use a separate versioned key. Reset appearance affects none of them.

V2 `/kit` changes its default destination; release notes state this and give `/kit appearance`. Return from nested Appearance to the workspace; closing the workspace returns to Claude input. Direct appearance invocation closes back to Claude when there was no workspace underneath.

Old CLI lacking passive/contracts support does not prevent V1 appearance. Disable only incompatible integration functions and provide exact supported handoffs. A capability mismatch is not a reason to parse human terminal output.

## Completion gates

V2 is delivered in verified increments: passive companion, reviewed management, controlled execution/branching. The whole V2 label must list which are actually enabled. A read-only beta must be called such; disabled actions are not completed functionality.

Every advertised action passes its real success, failure, denied, duplicate, stale-review, interrupted, reload, and cleanup paths. Visual/mock tests and actual engine/host proof are recorded separately. See [V2 build sequence](V2-BUILD.md).
