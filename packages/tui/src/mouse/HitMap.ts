export interface HitRegion {
  id: string;
  /** 1-based inclusive rows (terminal cells). */
  row0: number;
  row1: number;
  col0: number;
  col1: number;
  /** Payload for App (e.g. list index). */
  data?: Record<string, string | number | boolean>;
}

/**
 * Simple axis-aligned hit test registry.
 * Rebuilt when list length / selection chrome changes.
 */
export class HitMap {
  private regions: HitRegion[] = [];

  clear(): void {
    this.regions = [];
  }

  add(region: HitRegion): void {
    this.regions.push(region);
  }

  /** Register N list rows starting at startRow (1-based), full content width. */
  addListRows(options: {
    idPrefix: string;
    count: number;
    startRow: number;
    col0: number;
    col1: number;
    /** Optional data action for each row (e.g. activate). */
    action?: string;
  }): void {
    const { idPrefix, count, startRow, col0, col1, action } = options;
    for (let i = 0; i < count; i++) {
      const row = startRow + i;
      this.add({
        id: `${idPrefix}:${i}`,
        row0: row,
        row1: row,
        col0,
        col1,
        data: {
          index: i,
          ...(action !== undefined ? { action } : {}),
        },
      });
    }
  }

  /**
   * Register a single button (game UI control).
   * row/col are 1-based inclusive terminal cells.
   */
  addButton(options: {
    id: string;
    row: number;
    col0: number;
    col1: number;
    action: string;
    index?: number;
  }): void {
    this.add({
      id: options.id,
      row0: options.row,
      row1: options.row,
      col0: options.col0,
      col1: options.col1,
      data: {
        action: options.action,
        ...(options.index !== undefined ? { index: options.index } : {}),
      },
    });
  }

  /**
   * Register horizontal chips on one row (lane tabs).
   * Each chip has equal width from col0.
   */
  addChipRow(options: {
    idPrefix: string;
    count: number;
    row: number;
    col0: number;
    chipWidth: number;
    actions: string[];
  }): void {
    const { idPrefix, count, row, col0, chipWidth, actions } = options;
    for (let i = 0; i < count; i++) {
      const c0 = col0 + i * chipWidth;
      this.add({
        id: `${idPrefix}:${i}`,
        row0: row,
        row1: row,
        col0: c0,
        col1: c0 + chipWidth - 1,
        data: {
          index: i,
          action: actions[i] ?? `${idPrefix}-select`,
        },
      });
    }
  }

  hit(x: number, y: number): HitRegion | null {
    // Topmost last-added wins if overlap
    for (let i = this.regions.length - 1; i >= 0; i--) {
      const r = this.regions[i]!;
      if (y >= r.row0 && y <= r.row1 && x >= r.col0 && x <= r.col1) {
        return r;
      }
    }
    return null;
  }
}
