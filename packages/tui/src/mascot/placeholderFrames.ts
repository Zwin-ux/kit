import type { PixelFrame } from "./types.js";

/**
 * Pure black-and-white fox + wrench silhouettes for terminal use.
 * Used until kit-frame-1.png … kit-frame-4.png exist in assets/pixel/.
 * 16x16, high contrast, no gray.
 */

const W = 16;
const H = 16;

/** Shared body template; frames only tweak tail / head. */
function baseTemplate(): boolean[][] {
  // y from top. 1 = black.
  const g = (rows: string[]): boolean[][] =>
    rows.map((row) => row.split("").map((c) => c === "#"));

  return g([
    "................",
    "....##....##....", // ears
    "...####..####...",
    "...##########...", // head
    "...##..##..##...", // snout gap
    "....########....",
    "......####......", // neck
    "...##########...", // torso
    "..####....####..",
    "..###......###..",
    "...##.####.##...", // hips + tail stub area
    "....##....##....", // legs
    "....##....##....",
    "....##....##....",
    "...####..####...",
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

/** Frame 1 — neutral, wrench held at left. */
function frame1(): boolean[][] {
  const g = cloneGrid(baseTemplate());
  // wrench handle + head (left of body)
  set(g, 1, 7, true);
  set(g, 2, 7, true);
  set(g, 1, 8, true);
  set(g, 0, 6, true);
  set(g, 0, 7, true);
  set(g, 0, 8, true);
  // tail low right
  set(g, 13, 10, true);
  set(g, 14, 11, true);
  return g;
}

/** Frame 2 — tail up, slight head lean (ear shift). */
function frame2(): boolean[][] {
  const g = cloneGrid(baseTemplate());
  set(g, 1, 7, true);
  set(g, 2, 7, true);
  set(g, 1, 8, true);
  set(g, 0, 6, true);
  set(g, 0, 7, true);
  set(g, 0, 8, true);
  // ears nudged
  set(g, 3, 1, false);
  set(g, 4, 0, true);
  // tail up
  set(g, 13, 9, true);
  set(g, 14, 8, true);
  set(g, 15, 7, true);
  return g;
}

/** Frame 3 — small body shift right + blink (eye gap). */
function frame3(): boolean[][] {
  const g = cloneGrid(baseTemplate());
  set(g, 1, 7, true);
  set(g, 2, 7, true);
  set(g, 1, 8, true);
  set(g, 0, 6, true);
  set(g, 0, 7, true);
  set(g, 0, 8, true);
  // blink: clear a mid-head pixel
  set(g, 6, 3, false);
  set(g, 9, 3, false);
  // tail mid
  set(g, 13, 10, true);
  set(g, 14, 10, true);
  return g;
}

/** Frame 4 — ready pose, tail curl. */
function frame4(): boolean[][] {
  const g = cloneGrid(baseTemplate());
  set(g, 1, 7, true);
  set(g, 2, 7, true);
  set(g, 1, 8, true);
  set(g, 0, 6, true);
  set(g, 0, 7, true);
  set(g, 0, 8, true);
  set(g, 13, 10, true);
  set(g, 14, 10, true);
  set(g, 14, 9, true);
  set(g, 13, 11, true);
  return g;
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
  return [
    toFrame(1, frame1()),
    toFrame(2, frame2()),
    toFrame(3, frame3()),
    toFrame(4, frame4()),
  ];
}
