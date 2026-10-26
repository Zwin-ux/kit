# Kit Red: four focus kits

Update: [SESSION-DESIGN.md](SESSION-DESIGN.md) now governs the main daily experience. This document covers supporting setup behavior. Its punctuation-based role marks and old fox geometry are superseded by the user's request for better role logos and a refined fox.

Status: concept and implementation handoff, not a running build. Replaces the rejected Appearance/dashboard direction. Source reviewed: `c35f60855a62534f2ed71edac3d1dc52835b87a3`.

## The experience

Keep Kit's existing terminal setup. A person chooses their coding tools, chooses the work to equip them for, reviews what will be installed, and continues in their coding tool. They should not need to understand agent orchestration to get a useful setup.

The four featured choices are the existing **Frontend Design, Full-stack Design, Backend Engineer, and LLM Engineer**. They configure skills, rules, MCP servers and hooks; they are not four active workers. Multiple kits may be selected. Full-stack already extends Frontend, and the shared Essentials base installs through existing composition logic.

Keep the original agent chooser and its Claude Code, Codex and Grok identities. After confirmation it folds into the existing `Agents` summary above the focus question. No new provider roster is needed. Keep the later Control Room unchanged.

## Small visual proposal

Add one three-character ASCII identifier beside each of the four existing focus rows. Keep full names, descriptions, order, selection markers and keyboard behavior. Use warm red for the Kit identity and small marks, neutral text for names, and readable muted descriptions. Preserve the original fox geometry and provider colors.

| Kit | Proposed mark | Meaning | Existing example task |
| --- | --- | --- | --- |
| Frontend Design | `[_]` | Screen | Build a pricing page with three tiers |
| Full-stack Design | `===` | Layers | Add a sign-up form that saves to the database |
| Backend Engineer | `{ }` | Code/API | Add rate limiting to the login endpoint |
| LLM Engineer | `(*)` | Model/prompt node | Write an eval for the summarizer prompt |

These are proposed category marks, not existing logos. If a mark proves unclear, omit it; the full title carries meaning. Do not replace provider logos with these marks. Do not build a font, image loader, registry or animation system.

Exact baseline text arrangement (descriptions in this example shortened for width; implementation reads manifests):

```text
✓ Agents       Claude Code, Codex

What should your agents focus on?

> [x] [_] Frontend Design     UI, accessibility, browser testing
  [ ] === Full-stack Design  Frontend plus APIs, data, tests
  [ ] { } Backend Engineer   APIs, databases, security
  [ ] (*) LLM Engineer       Prompts, evals, RAG, model APIs
  [ ]     iOS / Apple Design Native iOS, SwiftUI, accessibility
  [ ]     Essentials         Shared base: plan, build, test, review

↑↓ move · space select · enter confirm · esc cancel
kit show <kit> for details
```

The cursor `>` and selected checkbox are independent. Moving the cursor never changes selection or starts a process. Extra catalogue choices remain available: featuring four does not delete iOS or Essentials. No new More menu is required.

## Preserve the complete journey

| Moment | User needs | Existing behavior to retain |
| --- | --- | --- |
| Agents | Which tools can I configure? | Original provider marks, actual version/login details, saved selection |
| Focus | What work should they be equipped for? | Four starter kits first, descriptions, multi-select, saved choices |
| Scope | Where will this apply? | All projects / this repo; saved scope and current defaults |
| Review | What exactly changes? | Paths, pinned sources, licences, rule blocks, MCP/hook commands, RUNS CODE labels |
| Confirm | Install this setup or stop? | Existing confirmation, skills/rules-only option, cancellation |
| Finish | What do I do next? | Concrete example command, undo command, truthful install result |
| Return | How do I inspect or change setup? | Existing setup/list/show/add/remove/sync commands; original Control Room for runs |

Selecting a kit is not installing it. Installation is not a passed agent run. Deselecting a previously installed kit does not remove it; retain the existing completion reminder and remove command. Missing tools may still receive config for later use; keep the existing explanation.

## Terminal constraints

- Wide: retain one row per choice and descriptions beside names. No columns of cards.
- 80 columns: shorten visually through the existing ellipsis behavior; do not hard-code shortened manifest descriptions.
- 60 columns: preserve full titles and selection before artwork; omit marks if necessary. Original fox already hides below 66 columns.
- Short terminals: retain the existing cursor-centered list window and truthful overflow indicators. Never claim a tall mockup fits 12 rows.
- Monochrome: shape, text, cursor and selection remain distinct. Preserve NO_COLOR and plain output.
- No new motion or clocks. Preserve existing resize and cancellation behavior.

Current source has no `?` detail binding even though an earlier design proposed it. Use the actual `kit show <kit>` hint. A selected-row preview, new detail pane, role-specific mascot, or separate settings page is outside this first pass.

## Minimum implementation handoff

Work from current upstream in an isolated checkout; the local checkout is older and contains user work. Read applicable ownership instructions before editing application code.

1. `crates/kit-cli/src/kits/setup.rs::pick_kits`: associate the four stable IDs with their tiny display marks. Continue loading titles, descriptions, ordering and defaults from the existing catalogue/logic.
2. `crates/kit-cli/src/kits/picker.rs`: support an optional plain mark without overloading `Item.agent`. Keep display mark separate from label so folded summaries remain clean. Reserve equal space for unmarked rows. No manifest schema change.
3. Reuse existing Paint/theme behavior for accent changes; review all callers before changing a shared token. Provider identities and semantic warning/error/success colors remain distinct.
4. Extend existing picker checks for marked/unmarked rows, alignment, monochrome, narrow width, resize, windowing and preserved multi-selection. Check starter order and saved choices. Run the existing setup integration suite in its isolated test environment.
5. Capture actual setup at wide/80/60 widths, short height and NO_COLOR. Compare the same moment against the original capture. Stop before an actual install for visual-only inspection; installation verification belongs in a disposable scope.

Relevant commands at this source revision:

```sh
cargo test -p kitctl kits::picker
cargo test -p kitctl --test kits_cli
cargo fmt --all --check
cargo clippy -p kitctl --all-targets -- -D warnings
```

These commands are handoff instructions, not tests run for this documentation/art task.

## V1 and V2 boundary

V1: original Kit setup with restrained red identity and the four focus rows made easier to scan. No new installer, daemon, orchestration engine or management dashboard.

V2: expose Kit inside Claude through verified supported surfaces. Reuse Kit's manifests, installation plan, ownership and exact removal. Start with the existing four choices and inspectable contents; do not revive the rejected Overview/Kits/Runs layout. Host integration remains a separate engineering task: this concept does not demonstrate a working mod, MCP installation, nested execution or permission handling.

## Evidence and acceptance

Primary product evidence: `PRODUCT-REVIEW-KITS.md`, `DESIGN-KITS.md`, current README and the FIRST array in `setup.rs`. Runtime behavior comes from source, not unimplemented examples in older designs. Original visual reference: `docs/media/setup-plan.png`; media rules: `docs/media/README.md`.

Accept the concept only if the original setup is recognizable, all four exact names are immediately readable, more than one selection remains possible, and the mark helps rather than competes with the title. Reject provider/role confusion, fake installed/running badges, replacement dashboards, hidden installation details, or removal of other catalogue entries.

Generated art is illustrative. The text layout and source behavior above are authoritative where image-generation typography drifts. A reviewed image is not terminal runtime proof.
