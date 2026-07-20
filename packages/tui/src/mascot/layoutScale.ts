/**
 * Menu-first terminal layout.
 *
 * Priority: readable menus + stable geometry + a11y.
 * Mascot is brand chrome — never steals the menu on small windows.
 *
 * Breakpoints (cols × rows):
 *   stack  — < 72 cols OR < 22 rows  → full-width menu, no side rail
 *   split  — normal desktop           → compact left rail + menu
 *   wide   — ≥ 110 cols AND ≥ 32 rows → split + denser lists
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
  /** Side rail only when split/wide — keep small so menu breathes. */
  railColsSplit: 10,
  railRowsSplit: 8,
  railColsWide: 12,
  railRowsWide: 10,
  /** Tiny top brand when stacked (optional height). */
  topBrandRows: 3,
  packDetailMax: 4,
  splashFitMax: 12,
  railFrameDelayMs: 220,
  /** Below this width, menu owns 100% of the frame. */
  stackMaxCols: 72,
  stackMaxRows: 22,
  wideMinCols: 110,
  wideMinRows: 32,
} as const;

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

  // --- mascot placement (menu-first) ---
  let mascotPlacement: MascotPlacement = "rail";
  let railCols = 0;
  let railRows = 0;
  if (mode === "stack") {
    // Tiny top strip only if we have vertical room; else hide entirely
    mascotPlacement = r >= 20 ? "top" : "hidden";
    railCols = Math.min(cols - 4, 16);
    railRows = mascotPlacement === "top" ? LAYOUT_CAPS.topBrandRows : 0;
  } else if (mode === "wide") {
    mascotPlacement = "rail";
    railCols = LAYOUT_CAPS.railColsWide;
    railRows = LAYOUT_CAPS.railRowsWide;
  } else {
    mascotPlacement = "rail";
    railCols = LAYOUT_CAPS.railColsSplit;
    railRows = LAYOUT_CAPS.railRowsSplit;
  }

  const mascotFit = {
    width: Math.max(4, railCols > 0 ? railCols - 1 : 8),
    height: Math.max(3, railRows > 0 ? railRows : 6),
  };

  // Padding: tighter on small screens so content fits
  const padX = mode === "stack" ? 1 : 2;
  const padY = mode === "stack" ? 0 : 1;

  // Content budget: leftover after rail + padding
  const chromeX = padX * 2 + (mascotPlacement === "rail" ? railCols + 2 : 0);
  const contentAvail = Math.max(20, cols - chromeX);
  const contentSoftMax =
    mode === "wide" ? Math.min(96, contentAvail) : Math.min(72, contentAvail);
  const contentMinCols = mode === "stack" ? 20 : 32;

  // Density
  const listMaxItems = mode === "wide" ? 8 : mode === "split" ? 5 : 3;
  const packListMax = mode === "wide" ? 0 : mode === "split" ? 8 : 6;

  // Detail silhouettes only when tall enough (don't eat menu)
  const packDetailSize =
    mode !== "stack" && r >= 28
      ? Math.min(LAYOUT_CAPS.packDetailMax, 4)
      : 0;

  const splashEdge = Math.min(
    LAYOUT_CAPS.splashFitMax,
    mode === "stack" ? 10 : 12,
  );

  // Header(1) + pad + optional top mascot + section labels ≈ list start
  const listStartRowHint =
    1 +
    padY +
    1 +
    (mascotPlacement === "top" ? railRows + 1 : 0) +
    4;

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
