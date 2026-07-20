import type { MascotVariant, PixelFrame } from "./types.js";
import {
  FRAME_COUNT,
  SCAN_FRAME_COUNT,
  SUCCESS_FRAME_COUNT,
} from "./types.js";

/**
 * Fallback laying-down fox silhouette (Alpha 1).
 * Pure black, no wrench. Variants: idle (tail), scan (ear tilt), success (bob).
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

function clearRow(grid: boolean[][], y: number): void {
  const row = grid[y];
  if (!row) return;
  for (let x = 0; x < W; x++) row[x] = false;
}

/** Shift whole silhouette vertically (success bob). */
function shiftY(grid: boolean[][], dy: number): boolean[][] {
  const out = g(Array.from({ length: H }, () => ".".repeat(W)));
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      if (grid[y]?.[x]) {
        const ny = y + dy;
        if (ny >= 0 && ny < H) set(out, x, ny, true);
      }
    }
  }
  return out;
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

/** Ear tilt offsets for scan: left ear, right ear tip nudge. */
const EAR_SCAN: Array<{ left: [number, number][]; right: [number, number][] }> =
  [
    {
      left: [
        [4, 1],
        [5, 1],
      ],
      right: [
        [10, 1],
        [11, 1],
      ],
    },
    {
      left: [
        [3, 1],
        [4, 1],
        [4, 2],
      ],
      right: [
        [11, 1],
        [12, 1],
      ],
    },
    {
      left: [
        [4, 1],
        [5, 1],
      ],
      right: [
        [10, 0],
        [11, 1],
        [12, 1],
      ],
    },
    {
      left: [
        [4, 0],
        [5, 1],
      ],
      right: [
        [10, 1],
        [11, 1],
      ],
    },
  ];

function applyTail(grid: boolean[][], index: number): void {
  const points = TAIL[index % TAIL.length] ?? TAIL[0]!;
  for (const [x, y] of points) {
    set(grid, x, y, true);
  }
}

function idleFrameAt(index: number): boolean[][] {
  const grid = cloneGrid(baseBody());
  applyTail(grid, index);
  return grid;
}

/** Scan: faster tail + ear tip variations (looking around). */
function scanFrameAt(index: number): boolean[][] {
  const grid = cloneGrid(baseBody());
  // Clear default ear tips then re-apply variant
  clearRow(grid, 0);
  set(grid, 4, 1, false);
  set(grid, 5, 1, false);
  set(grid, 10, 1, false);
  set(grid, 11, 1, false);
  // restore body ear bases from base
  const base = baseBody();
  for (let x = 0; x < W; x++) {
    set(grid, x, 1, Boolean(base[1]?.[x]));
    set(grid, x, 2, Boolean(base[2]?.[x]));
  }
  const ears = EAR_SCAN[index % EAR_SCAN.length]!;
  for (const [x, y] of ears.left) set(grid, x, y, true);
  for (const [x, y] of ears.right) set(grid, x, y, true);
  // faster tail cycle (use odd indices)
  applyTail(grid, (index * 2) % TAIL.length);
  return grid;
}

/** Success: vertical bob + high tail. */
function successFrameAt(index: number): boolean[][] {
  const bob = [0, -1, 0, 1][index % 4] ?? 0;
  let grid = cloneGrid(baseBody());
  applyTail(grid, 2); // peak tail
  if (bob !== 0) {
    grid = shiftY(grid, bob);
  }
  // small sparkle above head on peak frames
  if (index % 2 === 1) {
    set(grid, 7, Math.max(0, 0 + bob), true);
    set(grid, 8, Math.max(0, 1 + bob), true);
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

export function getPlaceholderFrames(
  variant: MascotVariant = "idle",
): PixelFrame[] {
  const frames: PixelFrame[] = [];
  if (variant === "scan") {
    for (let i = 0; i < SCAN_FRAME_COUNT; i++) {
      frames.push(toFrame(i + 1, scanFrameAt(i)));
    }
    return frames;
  }
  if (variant === "success") {
    for (let i = 0; i < SUCCESS_FRAME_COUNT; i++) {
      frames.push(toFrame(i + 1, successFrameAt(i)));
    }
    return frames;
  }
  for (let i = 0; i < FRAME_COUNT; i++) {
    frames.push(toFrame(i + 1, idleFrameAt(i)));
  }
  return frames;
}
