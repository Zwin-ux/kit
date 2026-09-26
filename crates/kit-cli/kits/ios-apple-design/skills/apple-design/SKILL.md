---
name: apple-design
description: Design and review native iOS and iPadOS interfaces the way Apple's platforms expect - hierarchy, typography with Dynamic Type, semantic colour and dark mode, SF Symbols, layout and size classes, Liquid Glass, motion, and accessibility. Use when building or reviewing any SwiftUI or UIKit screen, choosing a navigation pattern or control, or when the user asks for something to "look native", "feel like an Apple app" or "follow the HIG".
---

# Apple design for native iOS

This skill is Kit's own summary of how native iOS apps are expected to look
and behave. It paraphrases principles; it does not reproduce Apple's text.
The authority is Apple's Human Interface Guidelines (HIG). Every section
links the HIG page it summarises. When a decision depends on detail, read
that page:

- With the `sosumi` MCP server (installed by this kit), fetch the page by
  its path, for example `/design/human-interface-guidelines/materials`.
- Otherwise open the URL. If neither is possible, say which page you could
  not check instead of guessing.

When skills disagree, follow this order: Apple's HIG and documentation,
then Apple's own Xcode skills if the user has exported them, then
`swiftui-expert-skill`, then `swiftui-pro`, then `swiftui-liquid-glass`.

## How to work

1. **Start from the system.** Use standard components (`NavigationStack`,
   `TabView`, `List`, `Form`, `toolbar`, `sheet`, `Menu`, system buttons).
   They come with Dynamic Type, dark mode, accessibility, Liquid Glass and
   platform behaviour for free. Customise only what the product needs.
2. **Decide the hierarchy before styling.** Name the one thing each screen
   is for, then make it the most prominent element. Everything else steps
   back.
3. **Check the five states of every screen:** light, dark, the largest
   accessibility text size, VoiceOver, and Reduce Motion or Reduce
   Transparency. A screen that only works in light mode at default size is
   not done. Use `references/review-checklist.md` for a review.
4. **Show, don't claim.** When you change UI, say how you checked it: a
   preview in each state, a simulator screenshot, or the XcodeBuildMCP
   tools this kit installs.

## Principles

### Hierarchy, clarity, deference
HIG: https://developer.apple.com/design/human-interface-guidelines/designing-for-ios

- Content is the point. Chrome, decoration and controls should support it
  and never compete with it.
- Prefer one primary action per screen. Make it visually distinct (a
  prominent button style or the trailing toolbar position). Secondary
  actions go in toolbars, menus or context menus.
- Use whitespace and grouping to show structure before reaching for lines,
  boxes or colour.
- Keep text short and specific. Buttons are verbs ("Save", "Add Photo"), not
  "OK" or "Submit". HIG: https://developer.apple.com/design/human-interface-guidelines/writing

### Typography and Dynamic Type
HIG: https://developer.apple.com/design/human-interface-guidelines/typography

- Use the system text styles (`.largeTitle`, `.title`, `.headline`,
  `.body`, `.callout`, `.subheadline`, `.footnote`, `.caption`) rather
  than fixed point sizes. They scale with the user's text size setting.
- If you need a custom font, scale it with the matching text style
  (`.custom(_:size:relativeTo:)`), and scale custom spacing or icon sizes
  with `@ScaledMetric`.
- Layouts must survive the largest accessibility sizes. Let text wrap
  instead of truncating important content. Switch horizontal arrangements
  to vertical when text is large (`ViewThatFits`, or check
  `dynamicTypeSize.isAccessibilitySize`).
- Build hierarchy with weight and style, not with many sizes. Two or three
  levels per screen are usually enough.

### Colour and dark mode
HIG: https://developer.apple.com/design/human-interface-guidelines/color ·
https://developer.apple.com/design/human-interface-guidelines/dark-mode

- Use semantic colours (`.primary`, `.secondary`, `Color(.systemBackground)`,
  `Color(.secondarySystemBackground)`, `Color(.label)`, `.tint`). They adapt
  to light, dark and increased contrast automatically.
- Custom colours belong in the asset catalog with light, dark and
  high-contrast variants. Never hard-code a colour that only works on one
  background.
- Pick one tint colour for interactive elements and use it consistently.
  Do not use the tint for non-interactive text.
- Never carry meaning by colour alone. Pair it with a symbol, a label or a
  shape.
- Check contrast in both modes. Body text needs strong contrast against its
  background; secondary text still has to be readable.

### SF Symbols and icons
HIG: https://developer.apple.com/design/human-interface-guidelines/sf-symbols ·
https://developer.apple.com/design/human-interface-guidelines/icons

- Prefer SF Symbols (`Image(systemName:)`) for interface icons. They match
  the system font's weight and scale with Dynamic Type.
- Match the symbol weight to the adjacent text, and choose a rendering mode
  deliberately (monochrome by default; hierarchical, palette or multicolour
  when they add meaning).
- Use the same symbol for the same idea everywhere in the app. Do not reuse
  a well-known system symbol for a different meaning.
- Icon-only buttons need an accessibility label that says what they do.

### Layout, spacing and size classes
HIG: https://developer.apple.com/design/human-interface-guidelines/layout ·
https://developer.apple.com/design/human-interface-guidelines/layout-and-organization

- Respect the safe area and the system layout margins. Let the system
  default spacing do the work before inventing a spacing scale; if you add
  one, use a small set of consistent values.
- Interactive targets need to be comfortably tappable: aim for at least
  44 × 44 points, even when the visible glyph is smaller.
- Design for compact and regular width. On iPad and in split view, use
  `NavigationSplitView` or adaptive layouts rather than a stretched phone
  layout. Test the smallest supported iPhone and landscape.
- Keep related controls near the content they affect. Keep destructive
  actions away from frequent ones.

### Navigation and presentation
HIG: https://developer.apple.com/design/human-interface-guidelines/tab-bars ·
https://developer.apple.com/design/human-interface-guidelines/toolbars ·
https://developer.apple.com/design/human-interface-guidelines/sheets ·
https://developer.apple.com/design/human-interface-guidelines/modality

- Tab bars switch between top-level sections. They are not for actions,
  and they stay visible across the app's main areas.
- Push navigation (`NavigationStack`) drills into detail. Titles say where
  the user is; use large titles on top-level screens.
- Use a sheet for a focused, self-contained task, and give it a clear way
  to finish or cancel. Use detents when a partial sheet keeps useful
  context visible.
- Use alerts only for important information that needs a decision now.
  Prefer an undo or a confirmation dialog tied to the action.
  HIG: https://developer.apple.com/design/human-interface-guidelines/alerts
- Support the system back gesture and swipe-to-dismiss. Do not trap users.

### Liquid Glass and materials
HIG: https://developer.apple.com/design/human-interface-guidelines/materials ·
Adopting Liquid Glass: https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass

- Liquid Glass is for the layer that floats above content: navigation,
  tab bars, toolbars and controls. Content itself (lists, cards, media,
  text) should not be made of glass.
- Standard components adopt it when built with the current SDK. Remove
  custom backgrounds and opaque bars that fight it before adding your own
  effects.
- For custom controls, use `.glassEffect()`, and group nearby glass elements
  in a `GlassEffectContainer` so they blend and animate together. Use the
  glass button styles for buttons that sit on the glass layer.
- Use glass sparingly. Do not stack glass on glass, and do not put glass
  over busy content where it hurts legibility.
- Check Reduce Transparency and Increase Contrast. The system adjusts
  standard glass; custom effects must stay legible in both.

### Motion and feedback
HIG: https://developer.apple.com/design/human-interface-guidelines/motion ·
https://developer.apple.com/design/human-interface-guidelines/feedback ·
https://developer.apple.com/design/human-interface-guidelines/playing-haptics

- Motion should explain a change (where something came from or went), not
  decorate. Prefer the system's spring animations and transitions.
- Honour Reduce Motion (`@Environment(\.accessibilityReduceMotion)`):
  replace movement with fades or no animation.
- Give feedback for every action: a state change, a symbol effect, a
  progress indicator for anything slow, and haptics
  (`.sensoryFeedback`) only where they reinforce a meaningful moment.
- Show loading states that keep layout stable, and empty states that say
  what to do next. HIG: https://developer.apple.com/design/human-interface-guidelines/loading

### Accessibility
HIG: https://developer.apple.com/design/human-interface-guidelines/accessibility

- Every interactive element needs a clear label, and the right traits and
  value. Combine related elements (`.accessibilityElement(children: .combine)`)
  so VoiceOver reads a row as one thing.
- VoiceOver order must follow the visual order and the task.
- Support Dynamic Type, Bold Text, Increase Contrast, Reduce Motion,
  Reduce Transparency and Differentiate Without Colour.
- Use `swift-accessibility-skill` (installed by this kit) for a full audit.

### Inclusion and localisation
HIG: https://developer.apple.com/design/human-interface-guidelines/inclusion ·
https://developer.apple.com/design/human-interface-guidelines/right-to-left

- Use leading and trailing, never left and right, so layouts mirror for
  right-to-left languages.
- Leave room for longer translations. Never bake text into images.
- Use inclusive, plain language and avoid idioms.

## Things that look wrong on iOS

- A custom tab bar, navigation bar or back button that imitates the system
  one but behaves differently.
- Fixed font sizes, or text truncated at large Dynamic Type sizes.
- Hard-coded black or white text and backgrounds that break in dark mode.
- Web-style layouts: hamburger menus, underlined links as buttons, hover
  effects, heavy drop shadows and borders around everything.
- Alerts for routine confirmations, or modal sheets with no way out.
- Glass applied to content, glass on glass, or custom blur that ignores
  Reduce Transparency.
- Icon-only buttons without accessibility labels, and targets smaller than
  a fingertip.
