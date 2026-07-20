/**
 * Fluid terminal layout — grows with the window.
 *
 * Priority: readable menus + stable geometry + a11y.
 * Mascot is brand chrome — never steals the menu on small windows,
 * but on fullscreen it scales up so the UI doesn't look like a postage stamp.
 *
 * Breakpoints (cols × rows):
 *   stack  — < 72 cols OR < 22 rows  → full-width menu, optional top brand
 *   split  — normal desktop           → left rail + menu (rail grows from 80×24)
 *   wide   — ≥ 110 cols AND ≥ 32 rows → larger rail + denser lists
 *
 * Within split/wide, sizes are continuous in terminal cells — not fixed caps.
 */

export type LayoutMode = "stack" | "split" | "wide";

export type MascotPlacement = "hidden" | "top" | "rail";

export interface LayoutScale {
  mode: LayoutMode;
  columns: number;
  rows: number;

  /** Where the mascot lives this frame (resize only). */
  mascotPlacement: MascotPlacement;
  mascotFit: { width: number; height: number };
  mascotCell: "single";
  railCols: number;
  railRows: number;

  /** Outer page padding (cells). */
  padX: number;
  padY: number;

  /** Menu/content column — primary focus. */
  contentMinCols: number;
  /** Soft max for truncated descriptions / action lines. */
  contentSoftMax: number;
  /** How many secondary list items (installed skills, etc.). */
  listMaxItems: number;
  /** Max pack rows visible before “+N more” (0 = show all). */
  packListMax: number;

  packDetailSize: number;
  showPackDetail: boolean;

  splashFit: { width: number; height: number };
  splashCell: "single" | "double";

  /**
   * 1-based terminal row where the first menu list item starts
   * (for mouse hit-testing). Approximate; updated by shells when possible.
   */
  listStartRowHint: number;
}

const DEFAULT_COLS = 80;
const DEFAULT_ROWS = 24;

export const LAYOUT_CAPS = {
  /** Baseline rail at common 80×24 (split). Grows from here on larger terms. */
  railColsSplit: 10,
  railRowsSplit: 8,
  /** Soft floors for wide; actual wide rail is proportional. */
  railColsWide: 14,
  railRowsWide: 12,
  /** Hard ceilings so the fox never eats the menu. */
  railColsMax: 28,
  railRowsMax: 22,
  /** Tiny top brand when stacked (optional height). */
  topBrandRows: 3,
  packDetailMax: 4,
  splashFitMax: 20,
  railFrameDelayMs: 220,
  /** Below this width, menu owns 100% of the frame. */
  stackMaxCols: 72,
  stackMaxRows: 22,
  wideMinCols: 110,
  wideMinRows: 32,
  /** Always leave this many cols for the menu when rail is shown. */
  menuMinCols: 42,
  /** Always leave this many rows for chrome + tools when rail is shown. */
  menuMinRows: 14,
} as const;

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

/**
 * Continuous rail size from terminal dims.
 * 80×24 → 10×8; 120×40 → ~14×14; 180×55 → ~20×19; 200×60 → ~22×21.
 */
function railSizeFor(
  cols: number,
  rows: number,
  mode: LayoutMode,
): { railCols: number; railRows: number } {
  // Grow from the 80×24 baseline so maximize feels intentional, not fixed.
  let railCols = Math.round(
    LAYOUT_CAPS.railColsSplit + (cols - DEFAULT_COLS) * 0.1,
  );
  let railRows = Math.round(
    LAYOUT_CAPS.railRowsSplit + (rows - DEFAULT_ROWS) * 0.35,
  );

  if (mode === "wide") {
    // Wide gets a slightly stronger brand presence.
    railCols = Math.max(railCols, LAYOUT_CAPS.railColsWide);
    railRows = Math.max(railRows, LAYOUT_CAPS.railRowsWide);
  }

  railCols = clamp(railCols, LAYOUT_CAPS.railColsSplit, LAYOUT_CAPS.railColsMax);
  railRows = clamp(railRows, LAYOUT_CAPS.railRowsSplit, LAYOUT_CAPS.railRowsMax);

  // Never starve the menu.
  railCols = Math.min(railCols, Math.max(8, cols - LAYOUT_CAPS.menuMinCols));
  railRows = Math.min(railRows, Math.max(6, rows - LAYOUT_CAPS.menuMinRows));

  return { railCols, railRows };
}

/**
 * Compute layout from terminal size.
 * Changes only on resize — never on animation tick.
 */
export function layoutScaleFromTerminal(
  columns?: number,
  rows?: number,
): LayoutScale {
  const cols = columns && columns > 0 ? columns : DEFAULT_COLS;
  const r = rows && rows > 0 ? rows : DEFAULT_ROWS;

  // --- mode ---
  let mode: LayoutMode = "split";
  if (cols < LAYOUT_CAPS.stackMaxCols || r < LAYOUT_CAPS.stackMaxRows) {
    mode = "stack";
  } else if (cols >= LAYOUT_CAPS.wideMinCols && r >= LAYOUT_CAPS.wideMinRows) {
    mode = "wide";
  }

  // --- mascot placement + fluid size ---
  let mascotPlacement: MascotPlacement = "rail";
  let railCols = 0;
  let railRows = 0;

  if (mode === "stack") {
    // Tiny top strip only if we have vertical room; else hide entirely
    mascotPlacement = r >= 20 ? "top" : "hidden";
    railCols = Math.min(cols - 4, clamp(Math.round(cols * 0.35), 12, 24));
    railRows = mascotPlacement === "top" ? LAYOUT_CAPS.topBrandRows : 0;
  } else {
    mascotPlacement = "rail";
    ({ railCols, railRows } = railSizeFor(cols, r, mode));
  }

  // Fill the rail slot — no postage-stamp art with empty padding
  const mascotFit = {
    width: Math.max(4, railCols > 0 ? railCols : 8),
    height: Math.max(3, railRows > 0 ? railRows : 6),
  };

  // Padding grows slightly on huge windows so the UI doesn't hug edges
  const padX =
    mode === "stack" ? 1 : mode === "wide" ? clamp(Math.round(cols / 70), 2, 4) : 2;
  const padY = mode === "stack" ? 0 : 1;

  // Content budget: leftover after rail + padding
  const chromeX = padX * 2 + (mascotPlacement === "rail" ? railCols + 3 : 0);
  const contentAvail = Math.max(20, cols - chromeX);
  // Use the width — old hard caps (72/96) left fullscreen looking empty
  const contentSoftMax = clamp(
    contentAvail - 2,
    mode === "stack" ? 24 : 36,
    mode === "stack" ? 64 : 140,
  );
  const contentMinCols = mode === "stack" ? 20 : 36;

  // Density scales with free rows so fullscreen shows more tools
  // Chrome budget: header~2 + status~2 + footer~2 + project~2 + margins~4 ≈ 12
  const chromeBudget = 12 + padY * 2;
  let listMaxItems: number;
  if (mode === "stack") {
    listMaxItems = 1;
  } else if (mode === "split") {
    listMaxItems = r >= 30 ? 3 : 2;
  } else {
    listMaxItems = clamp(3 + Math.floor((r - 32) / 6), 3, 8);
  }

  const packListMax = clamp(
    r - chromeBudget - listMaxItems - (mode === "stack" ? 4 : 6),
    mode === "stack" ? 4 : 5,
    mode === "wide" ? 20 : 12,
  );

  // Never render solid █ pack detail blocks next to descriptions.
  const packDetailSize = 0;

  // Splash also grows on large terminals (still capped)
  const splashEdge = clamp(
    mode === "stack" ? 10 : 10 + Math.floor((Math.min(cols, r) - 24) / 4),
    10,
    LAYOUT_CAPS.splashFitMax,
  );

  // Header(1) + pad + optional top mascot + section labels ≈ list start
  const listStartRowHint =
    1 + padY + 1 + (mascotPlacement === "top" ? railRows + 1 : 0) + 4;

  return {
    mode,
    columns: cols,
    rows: r,
    mascotPlacement,
    mascotFit,
    mascotCell: "single",
    railCols,
    railRows,
    padX,
    padY,
    contentMinCols,
    contentSoftMax,
    listMaxItems,
    packListMax,
    packDetailSize,
    showPackDetail: packDetailSize > 0,
    splashFit: { width: splashEdge, height: splashEdge },
    splashCell: "single",
    listStartRowHint,
  };
}

/** @deprecated alias for older call sites */
export type LayoutModeLegacy = "narrow" | "normal" | "wide";

export function splashMascotFit(scale: LayoutScale): {
  width: number;
  height: number;
  cell: "single" | "double";
} {
  return {
    width: scale.splashFit.width,
    height: scale.splashFit.height,
    cell: scale.splashCell,
  };
}
