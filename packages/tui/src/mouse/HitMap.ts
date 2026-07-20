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
  }): void {
    const { idPrefix, count, startRow, col0, col1 } = options;
    for (let i = 0; i < count; i++) {
      const row = startRow + i;
      this.add({
        id: `${idPrefix}:${i}`,
        row0: row,
        row1: row,
        col0,
        col1,
        data: { index: i },
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
