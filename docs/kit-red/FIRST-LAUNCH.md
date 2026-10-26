# First launch: make something with Kit

Status: visual direction provisionally approved; interaction and native implementation unverified. Extends [SESSION-DESIGN.md](SESSION-DESIGN.md). Keep Claude's conversation/prompt as the daily surface, with a small Kit addition and playful, optional guidance around the existing setup.

## Intended experience

Help someone choose useful capabilities, understand the changes, install what they approve and optionally try their own task. The fox adds recognition and brief warmth. The outcome is a usable coding tool and inspectable work, not a score or completed tutorial.

Do not display “stage,” “mission progress,” “milestone,” percentages, levels, unlocks, checklists or a permanent setup tracker. Internal implementation state is allowed; it is not product copy. No new dashboard, Appearance screen, compulsory tour or pet-care mechanic.

The provisionally approved art resolves the visual exploration; this document no longer asks the user to select among three onboarding directions. Preserve the accepted conversational composition, four job identities and compact fox while implementing the requirements below.

## Entry and existing configuration

On a genuinely new setup, show a compact welcome alongside the current setup interaction:

`Let's make your first thing.`
`Choose what you want help with, or skip the guide.`

The native prompt remains usable wherever supported. No splash delay, typewriter lock, sound or fake loading. A brief fox acknowledgement is optional and nonblocking. If integration cannot place guidance natively, use the existing Kit setup flow and an explicit return to Claude; do not create an imitation host prompt.

`Skip guide` ends optional guidance. It does not install a default kit, bypass missing-agent requirements or launch a task. If configuration is required for the requested action, explain only that requirement through existing setup. With no supported coding tool, give the actual install/connect remedy; do not show a locked level.

Existing configuration or changing cwd alone never triggers a fresh onboarding replay. Use current Kit setup detection/records and a minimal guidance preference only if needed; do not overwrite existing installed state to force the experience.

## Choose useful work

Keep the exact four job kits in their existing order and the existing multi-select picker:

| Kit | Description |
|---|---|
| Frontend Design | UI, design systems, accessibility, browser testing |
| Full-stack Design | Frontend Design plus APIs, data, security, end-to-end tests |
| Backend Engineer | APIs, databases, security, performance, observability |
| LLM Engineer | Prompts, evals, RAG, model APIs, cost and latency |

Question: `What should your agents focus on?`
Helper: `Choose one or more. You can add or remove kits later.`

Provider selection remains the existing separate question; its folded answer can supply context above Focus. Provider logos do not replace job-kit identities. Keep existing iOS/other catalogue discovery and Essentials inheritance. Selecting kits configures capabilities; it never starts four workers.

Use the accepted small browser/layers/server/model-node symbols where rendering supports them, and full names everywhere. Cursor focus and selection are visibly different. Descriptions remain near the names. Selection feedback does not change row height or move the prompt.

The ordinary setup order remains provider → focus → scope → plan → confirmation. An optional task idea may be retained as an editable draft, but do not insert a compulsory goal-classification step before that sequence. No new model call solely to recommend or animate a kit.

## Review and install

Keep actual saved/default scope behavior and spell out scope before any mutation. Do not import the rejected integration proposal's project-default rule into existing global/saved setup behavior.

`Here's what Frontend Design will add to Claude Code.`
`Scope: All my projects` or the actual selected repository.

Use the current plan and confirmation: real paths, sources/pins/licences, dependencies, retained/conflicting content and RUNS CODE disclosures. Preserve the skills/rules-only option. The consequential action says Install/Continue through the existing flow, not an ambiguous game verb. Previewing, selecting or inspecting does not approve installation.

Pause decorative reactions while the user reads the plan or host permissions. No reward for enabling hooks/MCP, choosing global scope, accepting permission or spending tokens. No countdown or default-Yes.

After the installer confirms apply and ownership recording, show the actual outcome:

`Frontend Design installed for Claude Code in this project.`

Use actual provider, kit names and scope. Keep undo instructions from the installer. A short fox nod is optional. No progress strip such as “Goal chosen · Kit installed · First task next.” Successful installation does not claim the current Claude session loaded new context or that any task passed checks.

On failure, state the cause, known rollback outcome and concrete remedy. Preserve safe selections; do not blindly retry an uncertain write. No sad pet, lost progress, broken streak, blame or forced restart.

## Optional first task

Installation alone completes setup. Ask once:

`Want to try it on something useful?`

Offer `Try a task` and `Finish setup` with equal access. Finishing is not lesser progress. A dismissed offer does not repeatedly return on launch.

If the user chooses to try a task, use one editable native draft. A selected example only prefills text; it never submits, installs, invokes a provider or starts a check. Examples can be drawn from existing kit manifests, but are suggestions, not assertions about the repository. The user can replace the entire draft.

If a task chooser is shown, it selects one draft, unlike the multi-select kit picker. Use one focus cursor, not multiple checkboxes. Do not invent framework/project context before authorized inspection.

Work begins only after explicit native submission. Keep real provider approvals and cancellation. If the selected kit cannot be applied to the existing session, report the required action rather than changing an “active” label cosmetically. SESSION-DESIGN.md defines installed versus in-session labels.

When invoking Kit's isolated run flow, retain its real review of repository, committed base, checks and exclusion of uncommitted edits. A native Claude edit is not automatically a Kit run and gets no synthetic receipt. If no Git repo/checks exist, offer the real preparation path or finish setup; do not manufacture a toy project or fake passing test to guarantee success.

Use contextual help only when the relevant thing exists: an edit to inspect, a check output to open, or a failure to fix. No overlay tour through unavailable controls.

## Respond to actual results

| Evidence | Copy/behavior |
|---|---|
| Matching nonempty configured checks pass | `3/3 configured checks passed.` Use actual count; offer actual diff/receipt inspection |
| Check fails | `Typecheck failed. Open the failure.` Preserve the output and offer a reviewed retry if supported |
| Changes exist but checks are absent | `Change ready to review. Checks aren't configured.` |
| Agent made no changes | `No changes were made.` |
| Result/operation state cannot be established | Explain the uncertainty and inspect authoritative state; no success claim or automatic replay |

A short fox acknowledgement may follow a confirmed successful outcome, never delaying input or hiding evidence. No confetti for failed/unconfigured work. A model's “done” message cannot trigger verification. Merely opening a diff does not mean the user reviewed or approved it.

No achievement system is required. Installation records and actual run/check evidence supply the facts. Optional guidance state may remember dismissal or an unfinished draft, but never authorizes a write, proves an install or resumes a process.

## Return naturally

Returning users land in the ordinary native conversation/prompt with the small Kit identity and truthful context. Do not replay arrival art or guidance on every launch. If a supported conversation can resume, identify its actual task and let the user choose. A stored receipt alone is `Last recorded result`, not a running session.

An interrupted setup can offer `Continue setup`; first reconcile actual install records if a write may have occurred. Continuing guidance is different from restarting an installer or coding agent. Help may reopen guidance without resetting installed kits or redisplaying already acknowledged results as new wins.

## Motion, keyboard and narrow behavior

- One optional arrival/confirmed-success fox gesture under 700ms; selection acknowledgement, if used, around 120–180ms. These are targets to verify, not proof the artwork animates.
- Reuse existing supported animation infrastructure; no per-control timers, continuous companion motion, blinking or default sound.
- Respect reduced-motion and NO_COLOR. A resting fox and plain text communicate everything.
- Preserve stable geometry and the native prompt/draft. Full names, review details and host permissions take precedence over sprites.
- Existing picker: arrows move, Space toggles kit selection, Enter confirms, Esc/Ctrl-C cancel before installation. Any newly proposed control must have equivalent keyboard access and a verified host binding.
- Esc from optional guidance dismisses that guidance. It never silently confirms a plan, cancels an unrelated native turn or undoes an installed kit. During a real operation, use its existing explicit cancellation behavior and report the result.
- Serious errors and permission decisions use plain language; playful framing never renames Install, Cancel or Remove into ambiguous game actions.

## Concrete acceptance

| Test | Pass condition |
|---|---|
| Fresh entry and skip | Native work remains reachable; zero implicit installation, model call or task submission |
| Select Frontend and LLM | Two kits selected, one focused row; no implication of two running agents |
| Cancel plan | Existing installer semantics preserved; no false installed/context label |
| Skills/rules-only | Actual plan changes accordingly; no hidden code-running integration |
| Successful install | Real kit/provider/scope and undo shown; current-session application not assumed |
| Failed/interrupted install | Cause and actual rollback/uncertainty shown; no blind repeat on return |
| Try task then edit/abandon | Draft can change or be discarded without a model call; only submit starts work |
| Finish setup | Normal workspace; optional task prompt does not recur uninvited |
| Fail/no-changes/no-checks/pass results | Correct evidence-scoped copy; no synthetic reward or receipt |
| Returning launch / changed cwd | No tutorial replay, automatic process resume or duplicate install |
| 60×20, 80×24, 120×40 / text-only / reduced motion | Prompt, full necessary labels, focus and decision controls remain usable |
| Host permission request | Native request and every decision stay visible and operable |

Capture actual native flows with tested versions, command/event, observed result and artifact path. Generated concept boards and render-tree fixtures do not count as native interaction proof.

## Artwork and research boundary

The current art is provisionally approved; retain its restrained conversation, fox and four visual identities. Its minor selection-marker, texture and layout inconsistencies are resolved by the behavioral requirements here, not copied into implementation. In particular: kit choice is multi-select; starter-task choice is one draft; only one control owns keyboard focus; the implemented background stays flat; no visible stage/progress jargon is introduced from a board caption.

The artwork proves a visual direction only. It does not prove animation, host render support, keyboard behavior, installation, session transport or check execution. Do not build a fake chat runtime to match it.

Microsoft's [human-AI interaction guidelines](https://www.microsoft.com/en-us/research/?p=564561) motivate clear consequences, dismissal and correction. The original [Ryan, Rigby and Przybylski study](https://www.researchgate.net/publication/225998888_The_Motivational_Pull_of_Video_Games_A_Self-Determination_Theory_Approach) links autonomy/competence experiences with game enjoyment. Applying those principles to Kit is a design hypothesis, not evidence that this mascot or onboarding improves retention.

Validate with a small formative mix of new and experienced terminal users plus a text-only/reduced-motion user. Check whether they distinguish kit/provider/run, selected/installed/applied, and check evidence/model confidence; can skip, decline, find undo and recover; and find the fox helpful rather than distracting. Preserve comprehension and control over faster approval or more time spent in the product.
