/**
 * Parse SGR mouse sequences: ESC [ < btn ; x ; y M/m
 * Coordinates are 1-based cell positions from the terminal.
 */

export interface MouseEvent {
  button: number;
  x: number;
  y: number;
  /** true on button release (m), false on press (M) */
  release: boolean;
  /** true if motion (button 32+) */
  motion: boolean;
}

/**
 * Extract the last complete SGR mouse event from a stdin chunk.
 * Returns null if none.
 */
export function parseSgrMouseChunk(data: string): MouseEvent | null {
  // Match all SGR mouse sequences; use last complete one
  const re = /\x1b\[<(\d+);(\d+);(\d+)([Mm])/g;
  let match: RegExpExecArray | null;
  let last: MouseEvent | null = null;
  while ((match = re.exec(data)) !== null) {
    const button = Number(match[1]);
    const x = Number(match[2]);
    const y = Number(match[3]);
    const release = match[4] === "m";
    if (!Number.isFinite(button) || !Number.isFinite(x) || !Number.isFinite(y)) {
      continue;
    }
    last = {
      button: button & 3,
      x,
      y,
      release,
      motion: button >= 32,
    };
  }
  return last;
}

/** Left button press (not release, not motion). */
export function isPrimaryClick(ev: MouseEvent): boolean {
  return !ev.release && !ev.motion && ev.button === 0;
}
