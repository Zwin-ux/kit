/**
 * Terminal layout scale — content-first, art hard-capped.
 *
 * Full-screen grows *content density*, not logos.
 * Hierarchy: glyph (1) < pack detail (≤4, often off) < mascot rail (≤8–10) << menu.
 */

export type LayoutMode = "narrow" | "normal" | "wide";

export interface LayoutScale {
  mode: LayoutMode;
  columns: number;
  rows: number;
  /**
   * Pixel canvas for mascot rail (letterboxed).
   * Hard-capped — does not grow with monitor size.
   */
  mascotFit: { width: number; height: number };
  /** Rail always single-width █ (never ██ on menus). */
  mascotCell: "single";
  /** Terminal columns reserved for mascot rail (≈ art width + 1). */
  railCols: number;
  /** Pack detail silhouette edge (0 = off). Always ≤ half mascot height. */
  packDetailSize: number;
  /** Whether to show pack detail bitmap under selection. */
  showPackDetail: boolean;
  /** Splash hero canvas (slightly larger than rail, still capped). */
  splashFit: { width: number; height: number };
  splashCell: "single" | "double";
  /** Min columns for content column (menus). */
  contentMinCols: number;
  /** Soft max for long description lines on wide terminals. */
  contentSoftMax: number;
  /** How many list items to show on Home sections. */
  listMaxItems: number;
}

const DEFAULT_COLS = 80;
const DEFAULT_ROWS = 24;

/** Absolute caps — never exceed these for rail art. */
export const LAYOUT_CAPS = {
  /** Default product-screen fox (80×24). */
  mascotFitDefault: 8,
  /** Only when terminal is tall enough that chrome still fits. */
  mascotFitTall: 10,
  railColsMax: 12,
  packDetailMax: 4,
  /** Pack detail only when there is real vertical room. */
  packDetailMinRows: 28,
  splashFitMax: 12,
  splashDoubleColsMax: 20,
} as const;

/**
 * Pick layout from terminal size.
 * Wide = more content, not a bigger fox.
 */
export function layoutScaleFromTerminal(
  columns?: number,
  rows?: number,
): LayoutScale {
  const cols = columns && columns > 0 ? columns : DEFAULT_COLS;
  const r = rows && rows > 0 ? rows : DEFAULT_ROWS;

  let mode: LayoutMode = "narrow";
  if (cols >= 100 || r >= 30) {
    // content knobs: wide when either axis is large (not both required)
    if (cols >= 110 && r >= 32) mode = "wide";
    else if (cols >= 80 && r >= 24) mode = "normal";
    else if (cols >= 100 || r >= 30) mode = "normal";
  } else if (cols >= 80 && r >= 24) {
    mode = "normal";
  }

  // Art: 8 on common 24-row; 10 only when tall. Never grow with width alone.
  const mascotEdge =
    r >= 32
      ? LAYOUT_CAPS.mascotFitTall
      : mode === "narrow"
        ? 8
        : LAYOUT_CAPS.mascotFitDefault;
  const mascotFit = { width: mascotEdge, height: mascotEdge };
  // Tight rail — no 3-col dead gutter
  const railCols = Math.min(LAYOUT_CAPS.railColsMax, mascotEdge + 1);

  // Pack detail: off by default on short screens; max 4 and ≤ half fox
  const packDetailSize =
    r >= LAYOUT_CAPS.packDetailMinRows
      ? Math.min(LAYOUT_CAPS.packDetailMax, Math.floor(mascotEdge / 2))
      : 0;
  const showPackDetail = packDetailSize > 0;

  // Splash: always single ≤12 — no double billboard
  const splashEdge = Math.min(
    LAYOUT_CAPS.splashFitMax,
    mode === "narrow" ? 10 : 12,
  );
  const splashFit = { width: splashEdge, height: splashEdge };
  const splashCell: "single" = "single";

  const listMaxItems = mode === "wide" ? 8 : mode === "normal" ? 4 : 3;
  const contentMinCols = mode === "narrow" ? 28 : 40;
  const contentSoftMax = mode === "wide" ? 96 : mode === "normal" ? 72 : 48;

  return {
    mode,
    columns: cols,
    rows: r,
    mascotFit,
    mascotCell: "single",
    railCols,
    packDetailSize,
    showPackDetail,
    splashFit,
    splashCell,
    contentMinCols,
    contentSoftMax,
    listMaxItems,
  };
}

/** Splash fit helper for MascotPlayer. */
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
