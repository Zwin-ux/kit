/** One animation frame as a monochrome bitmap (true = black pixel). */
export interface PixelFrame {
  /** Logical frame index 1–6. */
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

/** Alpha 1: six-frame tail wag. */
export const FRAME_COUNT = 6;
export const FRAME_FILES = [
  "kit-frame-1.png",
  "kit-frame-2.png",
  "kit-frame-3.png",
  "kit-frame-4.png",
  "kit-frame-5.png",
  "kit-frame-6.png",
] as const;

/**
 * Target idle cycle rate (ms between frames).
 * ~5–6 fps matches assets/pixel README (160–220 ms).
 */
export const FRAME_DELAY_MS = 180;

/**
 * Max height when loading high-res PNGs into the TUI.
 * Keep modest so the full fox fits beside menus without wrapping/clipping.
 */
export const TUI_FRAME_MAX_HEIGHT = 16;
