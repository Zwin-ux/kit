---
user-invocable: false
name: animate-ui
description: Integrate or refine Animate UI components in a React web project when the user requests Animate UI, animated shadcn components, or an existing Animate UI interaction. Preserve native controls, reduced motion, and the project design.
license: MIT
metadata:
  author: Kit contributors
  version: "1.0.0"
---

# Animate UI integration

This is original Kit guidance, not an official Animate UI skill. Its MIT license covers this guidance only. Animate UI components and documentation have their own MIT + Commons Clause license condition. No Animate UI component code is bundled here.

## Establish the existing interface

Read the requested screen, package manifest, lockfile, component aliases, and any `components.json`. Identify the installed React version, styling system, Motion dependency, and existing component behavior before proposing a change. Apply this workflow to React web UI; it does not supply native Claude terminal widgets.

Keep the user's approved design, content, and working controls. Choose an animation for a concrete interaction such as revealing content or confirming a state change. Do not add looping decoration, replace an existing component system, or initialize shadcn in an already configured project merely because this skill is loaded.

## Integrate the requested component

1. Read the official component page and [installation guidance](https://animate-ui.com/docs/installation). Use available tools; this skill grants no additional tool permissions. If current documentation cannot be retrieved, use the pinned source links below and state the version limit rather than inventing component names, props, or registry identifiers.
2. Check the component's required dependencies, aliases, styling, and client boundary against the current project. Animate UI uses the shadcn installation workflow. Add only the component needed for the requested change, using the project's package manager and existing configuration. Inspect generated changes and dependency additions before keeping them; preserve unrelated files and settings.
3. Retain semantic controls and their keyboard behavior. Preserve accessible names, visible focus, disabled states, dialog focus management, validation messages, and the user's ability to interrupt an interaction. Animation must not change the reading order or move a target away from the pointer or keyboard focus.
4. Follow the official [accessibility guidance](https://animate-ui.com/docs/accessibility). When Motion is present, use its `MotionConfig` with `reducedMotion="user"` at the appropriate existing provider boundary; avoid redundant providers. Check CSS animations and non-Motion effects separately. Prefer a stable final state under reduced motion and keep feedback understandable without animation.
5. Check the changed interaction at a narrow viewport and in the project's supported themes. Exercise keyboard entry, activation, dismissal, focus return, repeated input, and reduced motion. Run relevant existing checks. Distinguish source inspection, automated checks, and observed browser behavior; report unavailable checks explicitly.

## Licensing and delivery

Upstream declares MIT + Commons Clause and restricts selling or redistributing the components themselves in their original form, alone or bundled. Do not describe the component library as unrestricted MIT or ship its source as a Kit skill resource. For a requested application integration, retain applicable notices and review the current upstream license against the intended distribution. This skill does not install components, add MCP servers, or change permissions merely by being loaded.

Report the component and interaction changed, dependencies actually added, verification performed, and any remaining browser or assistive-technology checks. Do not claim accessibility conformance from an automated score alone.

## Official source snapshot

Documentation reviewed at `imskyleen/animate-ui` revision `efeb96ffd7a3b7a4868667e4ac3c346620fb3044`:

- [Installation source](https://github.com/imskyleen/animate-ui/blob/efeb96ffd7a3b7a4868667e4ac3c346620fb3044/apps/www/content/docs/installation.mdx)
- [Accessibility source](https://github.com/imskyleen/animate-ui/blob/efeb96ffd7a3b7a4868667e4ac3c346620fb3044/apps/www/content/docs/accessibility.mdx)
- [Upstream license](https://github.com/imskyleen/animate-ui/blob/efeb96ffd7a3b7a4868667e4ac3c346620fb3044/LICENSE.md)

The pinned documents are provenance for this guidance, not evidence that a project's installed component version matches that revision.
