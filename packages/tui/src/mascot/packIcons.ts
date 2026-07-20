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

/** Render icon as terminal lines (single-width blocks). */
export function renderPackIconLines(
  packName: string,
  options?: { cell?: string; empty?: string },
): string[] {
  const cell = options?.cell ?? "█";
  const empty = options?.empty ?? " ";
  const pixels = getPackIconBitmap(packName);
  const lines: string[] = [];
  for (let y = 0; y < PACK_ICON_SIZE; y++) {
    let line = "";
    for (let x = 0; x < PACK_ICON_SIZE; x++) {
      line += pixels[y * PACK_ICON_SIZE + x] ? cell : empty;
    }
    lines.push(line);
  }
  return lines;
}

/** One-line mini glyph for list rows (8×4 half-scale-ish sample). */
export function packIconGlyph(packName: string): string {
  // Distinct short marks per pack for dense lists
  const glyphs: Record<string, string> = {
    essentials: "◆",
    "web-app": "▣",
    library: "▤",
    "cli-tool": "›",
    "api-service": "◈",
    "full-stack": "☰",
    "data-ml": "▥",
  };
  return glyphs[packName] ?? "·";
}
