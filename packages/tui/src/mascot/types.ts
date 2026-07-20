/** One animation frame as a monochrome bitmap (true = black pixel). */
export interface PixelFrame {
  /** Logical frame index 1–N. */
  index: number;
  width: number;
  height: number;
  /** Row-major pixels. length === width * height. */
  pixels: boolean[];
  /** Where the frame came from. */
  source: "file" | "placeholder";
  /** Absolute path when loaded from disk. */
  path?: string;
}

/** Mascot animation variants used across TUI screens. */
export type MascotVariant = "idle" | "scan" | "success";

/** Alpha 1: six-frame tail wag (idle). */
export const FRAME_COUNT = 6;
export const FRAME_FILES = [
  "kit-frame-1.png",
  "kit-frame-2.png",
  "kit-frame-3.png",
  "kit-frame-4.png",
  "kit-frame-5.png",
  "kit-frame-6.png",
] as const;

/** Scan loop: looking / ear tilt (4 frames). */
export const SCAN_FRAME_COUNT = 4;
export const SCAN_FRAME_FILES = [
  "kit-scan-1.png",
  "kit-scan-2.png",
  "kit-scan-3.png",
  "kit-scan-4.png",
] as const;

/** Success loop: celebratory bob (4 frames). */
export const SUCCESS_FRAME_COUNT = 4;
export const SUCCESS_FRAME_FILES = [
  "kit-success-1.png",
  "kit-success-2.png",
  "kit-success-3.png",
  "kit-success-4.png",
] as const;

/**
 * Target idle cycle rate (ms between frames).
 * ~5–6 fps matches assets/pixel README (160–220 ms).
 */
export const FRAME_DELAY_MS = 180;

/** Slightly snappier for scan (busy work). */
export const SCAN_FRAME_DELAY_MS = 140;

/** Success holds a bit longer per pose. */
export const SUCCESS_FRAME_DELAY_MS = 160;

/**
 * Max height when loading high-res PNGs into the TUI.
 * Keep modest so the full fox fits beside menus without wrapping/clipping.
 */
export const TUI_FRAME_MAX_HEIGHT = 16;

export function frameDelayForVariant(variant: MascotVariant): number {
  if (variant === "scan") return SCAN_FRAME_DELAY_MS;
  if (variant === "success") return SUCCESS_FRAME_DELAY_MS;
  return FRAME_DELAY_MS;
}
