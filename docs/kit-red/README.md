# Kit Red

Current direction, 2026-10-01: **Claude Code with Kit’s identity, one conversation, and four useful coding specialties.** Keep the native transcript and prompt central. Use restrained red accents, a compact active-kit line, and an on-demand `/kit` picker. Concept art is provisionally approved as visual guidance; native rendering still needs proof.

Inspect the [exact-source asset preview](production-sprites.html), with the revised fox and four role identities. It is an asset study, not a native screenshot.

Read [the continuation handoff](HANDOFF.md) first, then [the current mod build brief](MOD-BUILD.md) and [session design](SESSION-DESIGN.md). The working draft is in [mods/kit-red](../../mods/kit-red). This is no longer a documentation-only project.

## The four choices

| Choice | Stable kit ID | Work it supports |
| --- | --- | --- |
| Frontend Design | `frontend-design` | UI, design systems, accessibility, browser testing |
| Full-stack Design | `fullstack-design` | Frontend Design plus APIs, data, security, end-to-end tests |
| Backend Engineer | `backend-engineer` | APIs, databases, security, performance, observability |
| LLM Engineer | `llm-engineer` | Prompts, evals, RAG, model APIs, cost and latency |

These are Kit’s existing job kits, not four model providers or four running agents. Multiple choices are allowed. A selected guidance profile does not mean its full toolkit, MCP servers, hooks or skills are installed.

## Scope and status

**V1 — Kit Red:** minimal visual identity on supported native Claude surfaces, with no Kit CLI calls. Preserve native behavior, permissions and composition with other mods.

**V2 — Kit inside Claude:** useful Kit choices inside the conversation. The current draft implements a small first slice: `/kit`, session-only role selection, prompt guidance, a compact context line, and buttons that prepare editable drafts. Full Kit installation remains a separate explicit review flow. Broad receipt/run/land/cancel integration is future work, not part of the current build authority.

Five local scenarios, three native Claude test scenarios and plugin validation pass on Claude `2.1.287`. A real headless CLI loaded the plugin and handled `/kit frontend` with zero model turns and zero API cost. Native test trees cover terminal/Desktop button behavior, first-arrival requests and safe example drafting. Actual on-screen painting, physical keyboard/focus behavior and model response behavior remain unverified. The earlier remote rollout error stopped recurring during this pass; recheck availability when launching rather than assuming it is permanent.

Current source keeps choices in memory: clear/resume/fork reset them, compaction retains them, development reload resets them. Only first-arrival display is recorded after observed pane placement. First interactive activation requests the picker; narrow terminals can use `/kit` explicitly. The fox and role sprites follow the approved motifs. A single selected role uses its emblem in the existing tall-dock header slot; no/multiple choices use the fox. Compact/inline panes use text. Treat native rendering and interaction as open work; see the handoff.

## Design authority

Latest user constraints override older documents and every image. Keep Claude-like minimal UI: no dashboard, sidebar, Appearance panel, permanent four-column roster, staged labels, progress strips, checklist-style onboarding or invented agent statuses. Refined tiny fox/role illustrations support recognition; they must fit real terminal geometry and retain text alternatives. Preserve the native conversation rather than recreating it from a mockup.

[Session design](SESSION-DESIGN.md), [focus research](FOCUS-DESIGN.md), and [art identity notes](concepts/IDENTITY-NOTES.md) provide context. [First-launch studies](first-launch.html) and [concept review](review.html) are visual explorations, not live captures or instructions to add every illustrated control.

The earlier `V1-*`, `V2-*`, `DECISIONS.md`, and `docs/superpowers/` plans are historical research. Their expanded Appearance UI, preference system, dashboards and CLI contracts are not authorized by their presence in this folder. Reassess any useful technical finding against current source before adopting it.

## Source grounding

The exact four choices and their composition were verified against upstream `c35f60855a62534f2ed71edac3d1dc52835b87a3`: root README, `docs/dev/PRODUCT-REVIEW-KITS.md`, `docs/dev/DESIGN-KITS.md`, bundled kit manifests, and `crates/kit-cli/src/kits/setup.rs`. That setup asks agents → focus → scope → installation review. It is a separate CLI workflow; the mod’s lightweight selection must not imply completing it.

The local checkout began at older commit `fb0ddf198256940fc243c8f23318c27d6ff1ea87`. Preserve existing work and recheck versions before continuing. No Rust engine or standalone TUI changes are needed for the current mod slice.
