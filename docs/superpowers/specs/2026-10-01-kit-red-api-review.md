# Kit Red: independent mods API and lifecycle review

Date: 2026-10-01. Read-only planning review; no mod installed, no interactive host session launched, no repository code changed.

## Evidence

Official documentation fetched with curl from overview.md, api.md, interface.md, create.md, reference.md, and test.md under https://code.claude.com/docs/en/plugins/mods/. Local `claude --version` returned 2.1.287. The official GitHub mods/types/claude-code.d.ts fetched in this review declares 2.1.277 and is older than current documentation. Current-build generated declarations take precedence.

## Accepted architecture

One plugin over the existing Kit engine is appropriate. The visual layer can work independently of Kit. No npm package, daemon, HTTP bridge, model call, or duplicate execution engine is required for the visual/read-only releases.

## API constraints

Claude statically validates hooks. API calls must use literal $.namespace.method spelling; no namespace aliases or dynamic event names. A top-level function in the same module can receive $. Imported helpers, methods, and nested helper functions cannot receive $. Callbacks may close over $ and call the API directly, as the official examples do. Put registrations and API-owning helpers in register.js; extract only pure parsing/formatting/state transitions/element construction. Imports are static, relative, within the plugin except the supplied claude-code module. No Node APIs or normal timer globals.

Generate .claude-plugin/types from the actual development host before finalizing process-spawn or element signatures. GitHub 2.1.277 lacks process.spawn and calls Spinner terminal-only, whereas 2.1.287 docs expose spawn and list Spinner on terminal/Desktop. Do not resolve that discrepancy by guessing.

## Visual feasibility

AbovePrompt, owned Pane, and supported status treatments can establish the red Kit identity. There is no global host skin API, and the prompt editor and permission prompt are not render sites. Preserve downstream AbovePrompt with await next(e); pass foreign panes through unchanged. A safe first working treatment composes a red adjunct with host drawing.

Spinner reconstruction may lose elapsed time, token counts, and effort: older official declarations describe those as an engine-owned parenthetical absent from word/message/suffix/mode. Replacement is gated on current-build types plus actual host captures. Changing suffix alone does not prove native spinner recoloring.

Portable trees use $.ui.resolve(e), Box/Text/Button/Input/Select. Terminal-only raster/image artwork needs a text fallback in Desktop. Body width comes from bodyColumns. Keyboard focus is conditional, not guaranteed; ordinary typing retains precedence. Avoid digit hotkeys in the always-visible band.

## Lifecycle

Module variables reset on reload. $.state survives module reload but resets on clear/resume/branch. session.start runs on initial load/reload, not these conversation operations. classic.SessionStart sources clear/resume/fork allow rehydration/reset. ui.close carries pane id and person/plugin/unload origin. Timers stop on reload, which does not prove child-tree cleanup. Session-end hooks have a combined 1.5-second deadline; durable receipt writes cannot depend on them.

Reset selection, scope, and refresh generations on conversation changes. Cancel pane refresh on close; closing a pane must not pretend to stop a run. Reject obsolete refresh responses and suppress duplicate actions. Lost ownership means activity unknown and Stop disabled until authoritative recovery; never relaunch automatically.

## Process and mutation boundary

$.process.run uses argv without a shell, buffers results, resolves for nonzero exit, and rejects startup failure/timeout. Default timeout is 30 seconds, maximum ten minutes. Bounded reads can use ten seconds. It is not the general long-running execution mechanism.

Current docs expose $.process.spawn, but exact cancellation/descendant ownership must be established using the installed build. Launch/Stop need real nested-Claude and adapter-specific proof, approval interaction, child cleanup, final receipt, and reload/exit behavior. Never strip protective nested-session variables or enable full auto to pass this gate. Unsupported adapters retain explicit terminal handoffs.

Mods run with user privileges; process calls do not inherit a model Bash confirmation dialog. Review/confirmation is the plugin mutation boundary. Respect managed denials. Publish the validate hooks/calls inventory. No permission interception, model calls, prompt submission, session messaging, credential persistence, or enrichment is needed.

Interactive /kit returns {}. Command return text is model-readable transcript content; fallback text must be bounded and intentionally documented. Receipt viewing should not inject content into the conversation.

## Desktop and headless

Current docs support portable UI in Desktop Code. Access to Kit binaries/checkouts is a separate capability test, not implied by successful painting. Desktop WSL currently lacks plugins. VS Code/headless/cloud cannot display mod UI; fallback depends on plugin and command availability. Do not run discovery processes simply because a headless session loaded the plugin.

## Verification

Native claude plugin validate and claude plugin test are sufficient tooling. Tests need stubs registered before their first API call, explicit session.start when required, and a downstream ui.render stub for passthrough. Failure stubs should return deny; throwing a stub can cause it to be skipped. ui.mount checks trees, not actual painting.

Acceptance suite:
- validate inventory matches release permissions;
- /kit opens with no model turn and no unintended transcript data;
- another mod's band and panes survive;
- argv/cwd remain exact with shell metacharacters in tasks;
- missing/denied/timed-out/nonzero/malformed/truncated/wrong-schema responses stay actionable;
- stale responses cannot overwrite newer scope;
- empty checks never PASS and receipts never imply liveness;
- clear/resume/branch/reload/disable/pane-close/exit behave explicitly;
- real captures at 60x20, 80x24, 120x40 with light/dark/mono/no-color and keyboard-only paths;
- Desktop tree and real-painting/connectivity proof are separate;
- run descendants stop without touching unrelated sessions before Run/Stop ship;
- fresh tagged marketplace installation, update, disable, uninstall preserve unrelated settings/data.

## Final consistency pass

The revised spec and plan incorporate the accepted validator, host-Spinner preservation, lifecycle, current-build types, transcript, Desktop, and child-ownership findings. No P0/P1 contradiction found in these accepted API/lifecycle decisions. All process lifecycle claims remain evidence gates rather than asserted implementation facts.
