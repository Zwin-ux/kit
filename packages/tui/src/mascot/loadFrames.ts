import { readFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";
import { getPlaceholderFrames } from "./placeholderFrames.js";
import { resolvePixelAssetsDir } from "./resolveAssetsDir.js";
import {
  FRAME_COUNT,
  FRAME_FILES,
  type PixelFrame,
} from "./types.js";

export interface LoadFramesResult {
  frames: PixelFrame[];
  /** True when at least one frame came from a PNG file. */
  usedFiles: boolean;
  assetsDir?: string;
}

/**
 * Load up to 4 idle frames from assets/pixel/kit-frame-N.png.
 * Missing files fall back to the built-in silhouette placeholders.
 * If some PNGs exist and some do not, placeholders fill the gaps
 * so the cycle always has 4 frames.
 */
export async function loadMascotFrames(): Promise<LoadFramesResult> {
  const placeholders = getPlaceholderFrames();
  const assetsDir = await resolvePixelAssetsDir();

  if (!assetsDir) {
    return { frames: placeholders, usedFiles: false };
  }

  const frames: PixelFrame[] = [];
  let usedFiles = false;

  for (let i = 0; i < FRAME_COUNT; i++) {
    const fileName = FRAME_FILES[i];
    if (!fileName) {
      frames.push(placeholders[i]!);
      continue;
    }

    const filePath = path.join(assetsDir, fileName);
    try {
      const buffer = await readFile(filePath);
      const frame = pngToFrame(buffer, i + 1, filePath);
      frames.push(frame);
      usedFiles = true;
    } catch {
      frames.push(placeholders[i]!);
    }
  }

  return {
    frames,
    usedFiles,
    assetsDir,
  };
}

/**
 * Convert a PNG buffer into a monochrome silhouette frame.
 * Non-transparent dark pixels become black silhouette.
 */
export function pngToFrame(
  buffer: Buffer,
  index: number,
  filePath?: string,
): PixelFrame {
  const png = PNG.sync.read(buffer);
  const { width, height, data } = png;
  const pixels: boolean[] = new Array(width * height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (width * y + x) << 2;
      const r = data[idx] ?? 0;
      const g = data[idx + 1] ?? 0;
      const b = data[idx + 2] ?? 0;
      const a = data[idx + 3] ?? 0;
      // Silhouette: opaque and dark enough counts as black.
      const luminance = (r + g + b) / 3;
      const on = a >= 128 && luminance < 200;
      pixels[y * width + x] = on;
    }
  }

  const frame: PixelFrame = {
    index,
    width,
    height,
    pixels,
    source: "file",
  };

  if (filePath !== undefined) {
    frame.path = filePath;
  }

  return frame;
}
