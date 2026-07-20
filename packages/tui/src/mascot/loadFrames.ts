import { readFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";
import { getPlaceholderFrames } from "./placeholderFrames.js";
import { resolvePixelAssetsDir } from "./resolveAssetsDir.js";
import {
  FRAME_COUNT,
  FRAME_FILES,
  TUI_FRAME_MAX_HEIGHT,
  type PixelFrame,
} from "./types.js";

export interface LoadFramesResult {
  frames: PixelFrame[];
  /** True when at least one frame came from a PNG file. */
  usedFiles: boolean;
  assetsDir?: string;
  /** How many frames were loaded from disk (not placeholder). */
  fileFrameCount: number;
}

/**
 * Load up to 6 idle frames from assets/pixel/kit-frame-N.png.
 * Missing files fall back to built-in silhouette placeholders.
 * High-res masters are nearest-neighbor scaled for terminal display.
 */
export async function loadMascotFrames(): Promise<LoadFramesResult> {
  const placeholders = getPlaceholderFrames();
  const assetsDir = await resolvePixelAssetsDir();

  if (!assetsDir) {
    return {
      frames: placeholders,
      usedFiles: false,
      fileFrameCount: 0,
    };
  }

  const frames: PixelFrame[] = [];
  let fileFrameCount = 0;

  for (let i = 0; i < FRAME_COUNT; i++) {
    const fileName = FRAME_FILES[i];
    if (!fileName) {
      frames.push(placeholders[i]!);
      continue;
    }

    const filePath = path.join(assetsDir, fileName);
    try {
      const buffer = await readFile(filePath);
      const frame = pngToFrame(buffer, i + 1, filePath, {
        maxHeight: TUI_FRAME_MAX_HEIGHT,
      });
      frames.push(frame);
      fileFrameCount += 1;
    } catch {
      frames.push(placeholders[i]!);
    }
  }

  return {
    frames,
    usedFiles: fileFrameCount > 0,
    assetsDir,
    fileFrameCount,
  };
}

export interface PngToFrameOptions {
  /** Nearest-neighbor downscale so TUI stays readable. */
  maxHeight?: number;
}

/**
 * Convert a PNG buffer into a monochrome silhouette frame.
 * Non-transparent dark pixels become black silhouette.
 */
export function pngToFrame(
  buffer: Buffer,
  index: number,
  filePath?: string,
  options: PngToFrameOptions = {},
): PixelFrame {
  const png = PNG.sync.read(buffer);
  const { width, height, data } = png;
  const raw: boolean[] = new Array(width * height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (width * y + x) << 2;
      const r = data[idx] ?? 0;
      const g = data[idx + 1] ?? 0;
      const b = data[idx + 2] ?? 0;
      const a = data[idx + 3] ?? 0;
      const luminance = (r + g + b) / 3;
      raw[y * width + x] = a >= 128 && luminance < 200;
    }
  }

  const maxHeight = options.maxHeight ?? TUI_FRAME_MAX_HEIGHT;
  const scaled =
    height > maxHeight
      ? nearestNeighborScale(raw, width, height, maxHeight)
      : { pixels: raw, width, height };

  const frame: PixelFrame = {
    index,
    width: scaled.width,
    height: scaled.height,
    pixels: scaled.pixels,
    source: "file",
  };

  if (filePath !== undefined) {
    frame.path = filePath;
  }

  return frame;
}

function nearestNeighborScale(
  pixels: boolean[],
  srcW: number,
  srcH: number,
  targetH: number,
): { pixels: boolean[]; width: number; height: number } {
  const targetW = Math.max(1, Math.round((srcW * targetH) / srcH));
  const out: boolean[] = new Array(targetW * targetH);

  for (let y = 0; y < targetH; y++) {
    const srcY = Math.min(srcH - 1, Math.floor((y * srcH) / targetH));
    for (let x = 0; x < targetW; x++) {
      const srcX = Math.min(srcW - 1, Math.floor((x * srcW) / targetW));
      out[y * targetW + x] = pixels[srcY * srcW + srcX] === true;
    }
  }

  return { pixels: out, width: targetW, height: targetH };
}
