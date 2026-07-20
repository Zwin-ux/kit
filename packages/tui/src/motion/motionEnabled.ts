/**
 * When KIT_REDUCED_MOTION=1, all text animations jump to their final frame.
 * Prefer this over long decorative loops in accessibility-sensitive sessions.
 */
export function motionEnabled(): boolean {
  return process.env.KIT_REDUCED_MOTION !== "1";
}
