import { readFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";
import { getPlaceholderFrames } from "./placeholderFrames.js";
import { resolvePixelAssetsDir } from "./resolveAssetsDir.js";
import { normalizeFrame } from "./renderBitmap.js";
import {
  FRAME_COUNT,
  FRAME_FILES,
  SCAN_FRAME_COUNT,
  SCAN_FRAME_FILES,
  SUCCESS_FRAME_COUNT,
  SUCCESS_FRAME_FILES,
  TUI_FRAME_MAX_HEIGHT,
  type MascotVariant,
  type PixelFrame,
} from "./types.js";

/** Canonical pixel canvas for all loaded mascot frames (no size jump). */
const NORMALIZED_EDGE = 16;

export interface LoadFramesResult {
  frames: PixelFrame[];
  variant: MascotVariant;
  /** True when at least one frame came from a PNG file. */
  usedFiles: boolean;
  assetsDir?: string;
  /** How many frames were loaded from disk (not placeholder). */
  fileFrameCount: number;
}

export interface LoadAllMascotFramesResult {
  idle: PixelFrame[];
  scan: PixelFrame[];
  success: PixelFrame[];
  assetsDir?: string;
  usedFiles: boolean;
}

function fileListFor(variant: MascotVariant): readonly string[] {
  if (variant === "scan") return SCAN_FRAME_FILES;
  if (variant === "success") return SUCCESS_FRAME_FILES;
  return FRAME_FILES;
}

function countFor(variant: MascotVariant): number {
  if (variant === "scan") return SCAN_FRAME_COUNT;
  if (variant === "success") return SUCCESS_FRAME_COUNT;
  return FRAME_COUNT;
}

/**
 * Load mascot frames for a variant from assets/pixel/.
 * Missing files fall back to built-in silhouette placeholders.
 * High-res masters are nearest-neighbor scaled for terminal display.
 */
export async function loadMascotFrames(
  variant: MascotVariant = "idle",
): Promise<LoadFramesResult> {
  const placeholders = getPlaceholderFrames(variant);
  const assetsDir = await resolvePixelAssetsDir();
  const files = fileListFor(variant);
  const count = countFor(variant);

  if (!assetsDir) {
    return {
      frames: placeholders,
      variant,
      usedFiles: false,
      fileFrameCount: 0,
    };
  }

  const frames: PixelFrame[] = [];
  let fileFrameCount = 0;

  for (let i = 0; i < count; i++) {
    const fileName = files[i];
    if (!fileName) {
      frames.push(placeholders[i]!);
      continue;
    }

    const filePath = path.join(assetsDir, fileName);
    try {
      const buffer = await readFile(filePath);
      const raw = pngToFrame(buffer, i + 1, filePath, {
        maxHeight: TUI_FRAME_MAX_HEIGHT,
      });
      // Same 16×16 box every frame so animation never resizes the rail
      frames.push(normalizeFrame(raw, NORMALIZED_EDGE, NORMALIZED_EDGE));
      fileFrameCount += 1;
    } catch {
      frames.push(placeholders[i]!);
    }
  }

  return {
    frames,
    variant,
    usedFiles: fileFrameCount > 0,
    assetsDir,
    fileFrameCount,
  };
}

/** Load idle + scan + success in one pass (TUI boot). */
export async function loadAllMascotFrames(): Promise<LoadAllMascotFramesResult> {
  const [idle, scan, success] = await Promise.all([
    loadMascotFrames("idle"),
    loadMascotFrames("scan"),
    loadMascotFrames("success"),
  ]);

  const assetsDir = idle.assetsDir ?? scan.assetsDir ?? success.assetsDir;
  const result: LoadAllMascotFramesResult = {
    idle: idle.frames,
    scan: scan.frames,
    success: success.frames,
    usedFiles: idle.usedFiles || scan.usedFiles || success.usedFiles,
  };
  if (assetsDir !== undefined) {
    result.assetsDir = assetsDir;
  }
  return result;
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
