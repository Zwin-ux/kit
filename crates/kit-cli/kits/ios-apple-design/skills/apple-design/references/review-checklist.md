# iOS design review checklist

Use this when asked to review a screen or before calling UI work done.
Report each item as pass, fail (with what to change) or not checked (and
why). Capture evidence where you can: SwiftUI previews for each state, or
simulator screenshots through the XcodeBuildMCP tools.

## States to capture

- [ ] Light mode, default text size
- [ ] Dark mode, default text size
- [ ] Largest accessibility text size (AX5), both modes if layout differs
- [ ] Smallest supported iPhone, and landscape
- [ ] iPad regular width (if the app runs on iPad)
- [ ] Right-to-left pseudo-language

## Hierarchy
- [ ] The screen's main purpose is obvious in one glance.
- [ ] One primary action, visually distinct; secondary actions step back.
- [ ] Grouping uses spacing first, lines and boxes only when needed.

## Type
- [ ] System text styles or custom fonts scaled with `relativeTo:`.
- [ ] Nothing important truncates at AX5; layouts reflow instead.

## Colour
- [ ] Semantic or asset-catalog colours with dark variants; no hard-coded
      black or white.
- [ ] One consistent tint for interactive elements.
- [ ] Meaning is never carried by colour alone.
- [ ] Text is readable in both modes and with Increase Contrast.

## Symbols
- [ ] SF Symbols where possible, weight matching nearby text.
- [ ] Consistent symbol per concept; icon-only buttons are labelled.

## Layout
- [ ] Safe area and layout margins respected.
- [ ] Tap targets at least 44 × 44 pt.
- [ ] Leading/trailing, not left/right.

## Navigation
- [ ] Tab bar for sections only; push for drill-down; sheets for focused
      tasks with a clear way out.
- [ ] System back and dismiss gestures work.

## Liquid Glass
- [ ] Glass only on the navigation and control layer, not content.
- [ ] No glass on glass; nearby glass grouped in a `GlassEffectContainer`.
- [ ] Legible with Reduce Transparency and Increase Contrast.

## Motion and feedback
- [ ] Animations explain changes; Reduce Motion is honoured.
- [ ] Every action has feedback; slow work shows progress.
- [ ] Empty and error states say what to do next.

## Accessibility
- [ ] VoiceOver: every control labelled, sensible order, rows combined.
- [ ] Works with Bold Text and Differentiate Without Colour.

For the principle behind each item, see the matching section and HIG link
in `../SKILL.md`.
