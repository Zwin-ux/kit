import type { PixelFrame } from "./types.js";

export interface RenderFrameOptions {
  cell?: string;
  empty?: string;
  /**
   * Drop empty border rows/cols so the silhouette isn’t clipped by
   * wasted margin eating terminal height — then re-pad by `pad`.
   */
  tight?: boolean;
  /** Padding cells after tight crop. Default 1 when tight. */
  pad?: number;
}

/**
 * Render a monochrome frame for the terminal.
 * Black pixels become block characters. White stays empty.
 * Default cell is two columns so the shape stays closer to square.
 */
export function renderFrame(
  frame: PixelFrame,
  options?: RenderFrameOptions,
): string {
  return renderFrameLines(frame, options).join("\n");
}

/** Same as renderFrame but one terminal row per array entry (avoids Ink wrap). */
export function renderFrameLines(
  frame: PixelFrame,
  options?: RenderFrameOptions,
): string[] {
  const cell = options?.cell ?? "██";
  const empty = options?.empty ?? "  ";
  const tight = options?.tight === true;
  const pad = options?.pad ?? (tight ? 1 : 0);

  let pixels = frame.pixels;
  let width = frame.width;
  let height = frame.height;

  if (tight) {
    const cropped = cropToContent(pixels, width, height);
    pixels = cropped.pixels;
    width = cropped.width;
    height = cropped.height;
  }

  if (pad > 0) {
    const padded = padFrame(pixels, width, height, pad);
    pixels = padded.pixels;
    width = padded.width;
    height = padded.height;
  }

  const lines: string[] = [];
  for (let y = 0; y < height; y++) {
    let line = "";
    for (let x = 0; x < width; x++) {
      const on = pixels[y * width + x] === true;
      line += on ? cell : empty;
    }
    lines.push(line);
  }
  return lines;
}

/** Cell width of one rendered line (for Box sizing). */
export function renderedCellWidth(
  frame: PixelFrame,
  options?: RenderFrameOptions,
): number {
  const lines = renderFrameLines(frame, options);
  const first = lines[0] ?? "";
  // Count display columns: each code unit roughly 1 in block art we use
  return first.length;
}

function cropToContent(
  pixels: boolean[],
  width: number,
  height: number,
): { pixels: boolean[]; width: number; height: number } {
  let minX = width;
  let minY = height;
  let maxX = -1;
  let maxY = -1;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (pixels[y * width + x]) {
        if (x < minX) minX = x;
        if (y < minY) minY = y;
        if (x > maxX) maxX = x;
        if (y > maxY) maxY = y;
      }
    }
  }

  if (maxX < 0) {
    return { pixels, width, height };
  }

  const w = maxX - minX + 1;
  const h = maxY - minY + 1;
  const out: boolean[] = new Array(w * h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      out[y * w + x] = pixels[(minY + y) * width + (minX + x)] === true;
    }
  }
  return { pixels: out, width: w, height: h };
}

function padFrame(
  pixels: boolean[],
  width: number,
  height: number,
  pad: number,
): { pixels: boolean[]; width: number; height: number } {
  const w = width + pad * 2;
  const h = height + pad * 2;
  const out: boolean[] = new Array(w * h).fill(false);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      out[(y + pad) * w + (x + pad)] = pixels[y * width + x] === true;
    }
  }
  return { pixels: out, width: w, height: h };
}
