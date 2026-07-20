/**
 * Pure black 16×16 silhouette icons for each official starter pack.
 * Same visual language as kit-idle: no gray, no AA, clear at small size.
 */

export const PACK_ICON_SIZE = 16;

/** Row-major: true = black pixel. */
export type PackIconBitmap = boolean[];

export const OFFICIAL_PACK_ICONS = [
  "essentials",
  "web-app",
  "library",
  "cli-tool",
  "api-service",
  "full-stack",
  "data-ml",
] as const;

export type OfficialPackIconName = (typeof OFFICIAL_PACK_ICONS)[number];

function g(rows: string[]): PackIconBitmap {
  const out: boolean[] = [];
  for (const row of rows) {
    for (const c of row) {
      out.push(c === "#");
    }
  }
  return out;
}

/**
 * Each grid is 16 chars × 16 rows.
 * # = black · . = empty
 */
const ICONS: Record<string, PackIconBitmap> = {
  // Compact kit mark — ears + curled body (pack default / essentials)
  essentials: g([
    "................",
    "....##....##....",
    "...####..####...",
    "...##########...",
    "....########....",
    ".....######.....",
    "....########....",
    "...##########...",
    "...###....####..",
    "...##......###..",
    "....##....###...",
    ".....######.....",
    "......####......",
    ".......##.......",
    "................",
    "................",
  ]),

  // Browser window with content bars
  "web-app": g([
    "................",
    ".##############.",
    ".#............#.",
    ".##############.",
    ".#............#.",
    ".#..########..#.",
    ".#............#.",
    ".#..######....#.",
    ".#............#.",
    ".#..########..#.",
    ".#............#.",
    ".#..####......#.",
    ".#............#.",
    ".##############.",
    "................",
    "................",
  ]),

  // Book / package
  library: g([
    "................",
    "...##########...",
    "..############..",
    "..##........##..",
    "..##..####..##..",
    "..##........##..",
    "..##..####..##..",
    "..##........##..",
    "..##..####..##..",
    "..##........##..",
    "..##........##..",
    "..############..",
    "...##########...",
    "....########....",
    "................",
    "................",
  ]),

  // Terminal prompt >
  "cli-tool": g([
    "................",
    ".##############.",
    ".#............#.",
    ".#............#.",
    ".#...##.......#.",
    ".#....##......#.",
    ".#.....##.....#.",
    ".#......##....#.",
    ".#.....##.....#.",
    ".#....##......#.",
    ".#...##.......#.",
    ".#............#.",
    ".#....######..#.",
    ".#............#.",
    ".##############.",
    "................",
  ]),

  // Linked nodes / API
  "api-service": g([
    "................",
    "....####........",
    "...######.......",
    "...##..##.......",
    "...######.......",
    "....####........",
    "......##........",
    "......##..####..",
    "......##.######.",
    "......####....#.",
    "........##.####.",
    "........##......",
    ".......####.....",
    "......######....",
    ".......####.....",
    "................",
  ]),

  // Stacked layers (UI + API)
  "full-stack": g([
    "................",
    "....########....",
    "...##########...",
    "....########....",
    "................",
    "...##########...",
    "..############..",
    "...##########...",
    "................",
    "..############..",
    ".##############.",
    "..############..",
    "................",
    "....########....",
    "................",
    "................",
  ]),

  // Chart bars + baseline (data/ml)
  "data-ml": g([
    "................",
    "...........##...",
    "...........##...",
    "......##...##...",
    "......##...##...",
    "..##..##...##...",
    "..##..##...##...",
    "..##..##...##...",
    "..##..##...##...",
    "..##..##...##...",
    "################",
    "................",
    "................",
    "................",
    "................",
    "................",
  ]),
};

// Runtime length check (dev-friendly)
for (const [name, bmp] of Object.entries(ICONS)) {
  if (bmp.length !== PACK_ICON_SIZE * PACK_ICON_SIZE) {
    throw new Error(
      `pack icon ${name}: expected ${PACK_ICON_SIZE * PACK_ICON_SIZE} pixels, got ${bmp.length}`,
    );
  }
}

/** Fallback: small diamond if pack has no dedicated icon. */
const FALLBACK = g([
  "................",
  "................",
  ".......##.......",
  "......####......",
  ".....######.....",
  "....########....",
  "...##########...",
  "....########....",
  ".....######.....",
  "......####......",
  ".......##.......",
  "................",
  "................",
  "................",
  "................",
  "................",
]);

export function getPackIconBitmap(packName: string): PackIconBitmap {
  return ICONS[packName] ?? FALLBACK;
}

/**
 * Animation frames for pack logos (4 poses).
 * 0 rest · 1 dilate pulse · 2 rest · 3 bob up
 * Always returns edge×edge pixels for consistent terminal size.
 */
export function getPackIconAnimBitmap(
  packName: string,
  frameIndex: number,
  edge: number = PACK_ICON_SIZE,
): PackIconBitmap {
  const base = scaleBitmap(getPackIconBitmap(packName), PACK_ICON_SIZE, edge);
  const f = ((frameIndex % 4) + 4) % 4;
  if (f === 0 || f === 2) return base;
  if (f === 1) return dilateBitmap(base, edge);
  return shiftBitmap(base, edge, 0, -1);
}

/** Nearest-neighbor scale square bitmap to target edge. */
export function scaleBitmap(
  pixels: PackIconBitmap,
  srcEdge: number,
  destEdge: number,
): PackIconBitmap {
  if (srcEdge === destEdge) return pixels.slice();
  const out: boolean[] = new Array(destEdge * destEdge);
  for (let y = 0; y < destEdge; y++) {
    const sy = Math.min(srcEdge - 1, Math.floor((y * srcEdge) / destEdge));
    for (let x = 0; x < destEdge; x++) {
      const sx = Math.min(srcEdge - 1, Math.floor((x * srcEdge) / destEdge));
      out[y * destEdge + x] = pixels[sy * srcEdge + sx] === true;
    }
  }
  return out;
}

function dilateBitmap(pixels: PackIconBitmap, edge: number): PackIconBitmap {
  const out = pixels.slice();
  for (let y = 0; y < edge; y++) {
    for (let x = 0; x < edge; x++) {
      if (!pixels[y * edge + x]) continue;
      for (const [dx, dy] of [
        [0, -1],
        [0, 1],
        [-1, 0],
        [1, 0],
      ] as const) {
        const nx = x + dx;
        const ny = y + dy;
        if (nx >= 0 && nx < edge && ny >= 0 && ny < edge) {
          out[ny * edge + nx] = true;
        }
      }
    }
  }
  return out;
}

function shiftBitmap(
  pixels: PackIconBitmap,
  edge: number,
  dx: number,
  dy: number,
): PackIconBitmap {
  const out: boolean[] = new Array(edge * edge).fill(false);
  for (let y = 0; y < edge; y++) {
    for (let x = 0; x < edge; x++) {
      if (!pixels[y * edge + x]) continue;
      const nx = x + dx;
      const ny = y + dy;
      if (nx >= 0 && nx < edge && ny >= 0 && ny < edge) {
        out[ny * edge + nx] = true;
      }
    }
  }
  return out;
}

/** Render icon as terminal lines (single-width blocks). */
export function renderPackIconLines(
  packName: string,
  options?: {
    cell?: string;
    empty?: string;
    /** Output size in pixels (default 16). Prefer 6–8 so logos stay under mascot. */
    edge?: number;
    /** Animation frame 0–3. */
    frame?: number;
  },
): string[] {
  const cell = options?.cell ?? "█";
  const empty = options?.empty ?? " ";
  const edge = options?.edge ?? PACK_ICON_SIZE;
  const frame = options?.frame ?? 0;
  const pixels =
    options?.frame !== undefined
      ? getPackIconAnimBitmap(packName, frame, edge)
      : scaleBitmap(getPackIconBitmap(packName), PACK_ICON_SIZE, edge);
  const lines: string[] = [];
  for (let y = 0; y < edge; y++) {
    let line = "";
    for (let x = 0; x < edge; x++) {
      line += pixels[y * edge + x] ? cell : empty;
    }
    lines.push(line);
  }
  return lines;
}

/**
 * Mini list glyphs — ASCII single-cell only.
 * Ambiguous Unicode (◆☰▣) reflows Windows terminals when selection moves.
 * `frame` kept for API compat; list icons are static.
 */
const ASCII_GLYPHS: Record<string, string> = {
  essentials: "*",
  "web-app": "#",
  library: "=",
  "cli-tool": ">",
  "api-service": "+",
  "full-stack": "%",
  "data-ml": "~",
};

/** One-line mini glyph for list rows (always display width 1). */
export function packIconGlyph(packName: string, _frame = 0): string {
  void _frame;
  return ASCII_GLYPHS[packName] ?? "o";
}
