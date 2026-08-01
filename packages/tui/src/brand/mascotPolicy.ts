/**
 * Brand / motion policy for agent-class TUI.
 *
 * Menu icons (game assets) always show. Full fox art is opt-in.
 * Lowkey menu pulse uses motionEnabled() only (see MenuButton).
 *
 * Opt-in brand: KIT_SHOW_MASCOT=1, KIT_MASCOT_ANIM=1, KIT_TUI_SPLASH=1
 */

export function mascotVisible(): boolean {
  if (process.env.KIT_NO_MASCOT === "1") return false;
  return (
    process.env.KIT_SHOW_MASCOT === "1" ||
    process.env.KIT_MASCOT_ANIM === "1"
  );
}

/**
 * Soft menu motion (selected row pulse) — on unless reduced motion.
 * Not the full fox GIF; just a quiet cursor beat.
 */
export function menuMotionEnabled(): boolean {
  if (process.env.KIT_REDUCED_MOTION === "1") return false;
  return process.env.KIT_NO_MENU_MOTION !== "1";
}

/** Continuous fox loop — only when explicitly requested. */
export function mascotAnimEnabled(): boolean {
  return mascotVisible() && process.env.KIT_MASCOT_ANIM === "1";
}

/** Nostalgia splash gate — never default. */
export function splashEnabled(): boolean {
  return process.env.KIT_TUI_SPLASH === "1";
}
