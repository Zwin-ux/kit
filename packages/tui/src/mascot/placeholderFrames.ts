import type { PixelFrame } from "./types.js";
import { FRAME_COUNT } from "./types.js";

/**
 * Fallback laying-down fox silhouette (Alpha 1).
 * Pure black, no wrench. Six frames with tail-only motion.
 * Used when PNG masters are missing.
 */

const W = 16;
const H = 16;

function g(rows: string[]): boolean[][] {
  return rows.map((row) => row.split("").map((c) => c === "#"));
}

/** Body locked. Tail slots on the right side. */
function baseBody(): boolean[][] {
  return g([
    "................",
    "....##....##....", // ears
    "...####..####...",
    "...##########...", // head
    "...##########...",
    "....########....",
    ".....######.....", // neck into body
    "....########....", // curled body
    "...##########...",
    "...###....####..", // chest + tail base
    "...##......###..",
    "....##....####..",
    ".....########...",
    "......######....",
    ".......####.....",
    "................",
  ]);
}

function cloneGrid(grid: boolean[][]): boolean[][] {
  return grid.map((row) => row.slice());
}

function set(grid: boolean[][], x: number, y: number, on: boolean): void {
  const row = grid[y];
  if (!row || x < 0 || x >= W || y < 0 || y >= H) return;
  row[x] = on;
}

/** Tail poses: rest → up → peak → down → low → near rest */
const TAIL: Array<Array<[number, number]>> = [
  [
    [12, 9],
    [13, 10],
    [14, 11],
    [13, 12],
  ],
  [
    [13, 8],
    [14, 8],
    [14, 9],
    [15, 10],
  ],
  [
    [13, 7],
    [14, 7],
    [15, 7],
    [15, 8],
  ],
  [
    [13, 8],
    [14, 9],
    [15, 9],
    [14, 10],
  ],
  [
    [12, 10],
    [13, 11],
    [14, 12],
    [13, 12],
  ],
  [
    [12, 9],
    [13, 10],
    [14, 11],
    [14, 12],
  ],
];

function frameAt(index: number): boolean[][] {
  const grid = cloneGrid(baseBody());
  const points = TAIL[index] ?? TAIL[0]!;
  for (const [x, y] of points) {
    set(grid, x, y, true);
  }
  return grid;
}

function toFrame(index: number, grid: boolean[][]): PixelFrame {
  const pixels: boolean[] = [];
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      pixels.push(Boolean(grid[y]?.[x]));
    }
  }
  return {
    index,
    width: W,
    height: H,
    pixels,
    source: "placeholder",
  };
}

export function getPlaceholderFrames(): PixelFrame[] {
  const frames: PixelFrame[] = [];
  for (let i = 0; i < FRAME_COUNT; i++) {
    frames.push(toFrame(i + 1, frameAt(i)));
  }
  return frames;
}
