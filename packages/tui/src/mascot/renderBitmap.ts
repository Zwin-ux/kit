import type { PixelFrame } from "./types.js";

/**
 * Render a monochrome frame for the terminal.
 * Black pixels become block characters. White stays empty.
 * Each pixel is two columns so the shape stays closer to square.
 */
export function renderFrame(
  frame: PixelFrame,
  options?: { cell?: string; empty?: string },
): string {
  const cell = options?.cell ?? "██";
  const empty = options?.empty ?? "  ";
  const lines: string[] = [];

  for (let y = 0; y < frame.height; y++) {
    let line = "";
    for (let x = 0; x < frame.width; x++) {
      const on = frame.pixels[y * frame.width + x] === true;
      line += on ? cell : empty;
    }
    lines.push(line);
  }

  return lines.join("\n");
}
