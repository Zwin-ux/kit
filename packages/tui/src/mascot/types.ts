/** One animation frame as a monochrome bitmap (true = black pixel). */
export interface PixelFrame {
  /** Logical frame index 1–4. */
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

export const FRAME_COUNT = 4;
export const FRAME_FILES = [
  "kit-frame-1.png",
  "kit-frame-2.png",
  "kit-frame-3.png",
  "kit-frame-4.png",
] as const;

/** Target idle cycle rate (ms between frames). */
export const FRAME_DELAY_MS = 220;
