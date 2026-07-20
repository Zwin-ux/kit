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
  /**
   * Letterbox into a fixed pixel canvas after crop/pad.
   * Prevents frame-to-frame size jumps (tail wag used to change height).
   */
  fit?: { width: number; height: number };
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
  const prepared = preparePixels(frame, options);

  const lines: string[] = [];
  for (let y = 0; y < prepared.height; y++) {
    let line = "";
    for (let x = 0; x < prepared.width; x++) {
      const on = prepared.pixels[y * prepared.width + x] === true;
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
  return first.length;
}

/** Fixed pixel canvas size after prepare (for reserving layout). */
export function preparedPixelSize(
  frame: PixelFrame,
  options?: RenderFrameOptions,
): { width: number; height: number } {
  const prepared = preparePixels(frame, options);
  return { width: prepared.width, height: prepared.height };
}

function preparePixels(
  frame: PixelFrame,
  options?: RenderFrameOptions,
): { pixels: boolean[]; width: number; height: number } {
  const tight = options?.tight === true;
  const pad = options?.pad ?? (tight ? 1 : 0);
  const fit = options?.fit;

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

  if (fit && fit.width > 0 && fit.height > 0) {
    return letterbox(pixels, width, height, fit.width, fit.height);
  }

  return { pixels, width, height };
}

/**
 * Scale content to fit inside dest, then center (letterbox).
 * Nearest-neighbor — pure silhouette, no AA.
 */
export function letterbox(
  pixels: boolean[],
  srcW: number,
  srcH: number,
  destW: number,
  destH: number,
): { pixels: boolean[]; width: number; height: number } {
  if (srcW <= 0 || srcH <= 0) {
    return {
      pixels: new Array(destW * destH).fill(false),
      width: destW,
      height: destH,
    };
  }

  const scale = Math.min(destW / srcW, destH / srcH);
  const drawW = Math.max(1, Math.round(srcW * scale));
  const drawH = Math.max(1, Math.round(srcH * scale));
  const ox = Math.floor((destW - drawW) / 2);
  const oy = Math.floor((destH - drawH) / 2);

  const out: boolean[] = new Array(destW * destH).fill(false);
  for (let y = 0; y < drawH; y++) {
    const srcY = Math.min(srcH - 1, Math.floor((y * srcH) / drawH));
    for (let x = 0; x < drawW; x++) {
      const srcX = Math.min(srcW - 1, Math.floor((x * srcW) / drawW));
      if (pixels[srcY * srcW + srcX]) {
        const dx = ox + x;
        const dy = oy + y;
        if (dx >= 0 && dx < destW && dy >= 0 && dy < destH) {
          out[dy * destW + dx] = true;
        }
      }
    }
  }
  return { pixels: out, width: destW, height: destH };
}

/** Normalize a frame into a fixed canvas (for load pipeline). */
export function normalizeFrame(
  frame: PixelFrame,
  destW: number,
  destH: number,
): PixelFrame {
  const cropped = cropToContent(frame.pixels, frame.width, frame.height);
  const fitted = letterbox(
    cropped.pixels,
    cropped.width,
    cropped.height,
    destW,
    destH,
  );
  return {
    ...frame,
    width: fitted.width,
    height: fitted.height,
    pixels: fitted.pixels,
  };
}

export function cropToContent(
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
