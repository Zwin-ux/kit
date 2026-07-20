import { describe, expect, it } from "vitest";
import { getPlaceholderFrames } from "../src/mascot/placeholderFrames.js";
import { renderFrame } from "../src/mascot/renderBitmap.js";
import {
  FRAME_COUNT,
  SCAN_FRAME_COUNT,
  SUCCESS_FRAME_COUNT,
} from "../src/mascot/types.js";
import {
  OFFICIAL_PACK_ICONS,
  PACK_ICON_SIZE,
  getPackIconAnimBitmap,
  getPackIconBitmap,
  renderPackIconLines,
} from "../src/mascot/packIcons.js";
import { layoutScaleFromTerminal } from "../src/mascot/layoutScale.js";
import {
  STATUS_ICON_IDS,
  STATUS_ICON_SIZE,
  getStatusIconBitmap,
  levelToStatusIcon,
  harnessToStatusIcon,
} from "../src/mascot/statusIcons.js";
import { PNG } from "pngjs";
import { pngToFrame } from "../src/mascot/loadFrames.js";

describe("placeholder mascot frames", () => {
  it("provides six silhouette frames", () => {
    const frames = getPlaceholderFrames();
    expect(frames).toHaveLength(FRAME_COUNT);
    expect(FRAME_COUNT).toBe(6);
    for (const frame of frames) {
      expect(frame.width).toBe(16);
      expect(frame.height).toBe(16);
      expect(frame.pixels).toHaveLength(16 * 16);
      expect(frame.source).toBe("placeholder");
      expect(frame.pixels.some(Boolean)).toBe(true);
    }
  });

  it("provides scan and success variants", () => {
    const scan = getPlaceholderFrames("scan");
    const success = getPlaceholderFrames("success");
    expect(scan).toHaveLength(SCAN_FRAME_COUNT);
    expect(success).toHaveLength(SUCCESS_FRAME_COUNT);
    for (const frame of [...scan, ...success]) {
      expect(frame.pixels.some(Boolean)).toBe(true);
      expect(frame.pixels).toHaveLength(16 * 16);
    }
  });

  it("renders only block characters and spaces", () => {
    const frame = getPlaceholderFrames()[0]!;
    const text = renderFrame(frame);
    expect(text.split("\n")).toHaveLength(16);
    expect(text).toMatch(/█/);
    expect(text).not.toMatch(/[a-zA-Z]/);
  });

  it("tight crop keeps content with padding (no edge clip)", () => {
    const frame = getPlaceholderFrames()[0]!;
    const lines = renderFrame(frame, { tight: true, pad: 1 }).split("\n");
    expect(lines.length).toBeGreaterThan(2);
    expect(lines.some((l) => l.includes("█"))).toBe(true);
    // First/last rows should be padding (empty) after tight+pad
    expect(lines[0]?.trim()).toBe("");
    expect(lines[lines.length - 1]?.trim()).toBe("");
  });

  it("letterbox fit keeps every frame the same terminal size", () => {
    const frames = getPlaceholderFrames();
    const fit = { width: 12, height: 12 };
    const sizes = frames.map((frame) => {
      const lines = renderFrame(frame, {
        tight: true,
        pad: 0,
        fit,
        cell: "█",
        empty: " ",
      }).split("\n");
      return { h: lines.length, w: lines[0]?.length ?? 0 };
    });
    for (const s of sizes) {
      expect(s.h).toBe(12);
      expect(s.w).toBe(12);
    }
  });
});

describe("pack silhouette icons", () => {
  it("ships 7 official pack icons at 16×16", () => {
    expect(OFFICIAL_PACK_ICONS).toHaveLength(7);
    for (const name of OFFICIAL_PACK_ICONS) {
      const bmp = getPackIconBitmap(name);
      expect(bmp).toHaveLength(PACK_ICON_SIZE * PACK_ICON_SIZE);
      expect(bmp.some(Boolean)).toBe(true);
    }
  });

  it("detail render is smaller than mascot canvas (8 vs 12)", () => {
    for (const name of OFFICIAL_PACK_ICONS) {
      const lines = renderPackIconLines(name, { edge: 8, frame: 0 });
      expect(lines).toHaveLength(8);
      expect(lines[0]).toHaveLength(8);
    }
  });

  it("animation frames stay fixed size", () => {
    for (let f = 0; f < 4; f++) {
      const bmp = getPackIconAnimBitmap("essentials", f, 8);
      expect(bmp).toHaveLength(64);
    }
  });
});

describe("layout scale", () => {
  it("keeps pack detail smaller than mascot fit", () => {
    for (const [cols, rows] of [
      [60, 20],
      [80, 24],
      [120, 40],
    ] as const) {
      const s = layoutScaleFromTerminal(cols, rows);
      expect(s.packDetailSize).toBeLessThanOrEqual(s.mascotFit.height);
      expect(s.packDetailSize).toBeLessThanOrEqual(s.mascotFit.width);
    }
  });

  it("grows mascot on full-screen terminals", () => {
    const narrow = layoutScaleFromTerminal(60, 20);
    const wide = layoutScaleFromTerminal(120, 40);
    expect(wide.mascotFit.height).toBeGreaterThanOrEqual(
      narrow.mascotFit.height,
    );
    expect(wide.mode).toBe("wide");
    expect(narrow.mode).toBe("narrow");
  });
});

describe("status icons", () => {
  it("ships 8×8 bitmaps for every id", () => {
    expect(STATUS_ICON_IDS.length).toBeGreaterThanOrEqual(16);
    for (const id of STATUS_ICON_IDS) {
      const bmp = getStatusIconBitmap(id);
      expect(bmp).toHaveLength(STATUS_ICON_SIZE * STATUS_ICON_SIZE);
      expect(bmp.some(Boolean)).toBe(true);
    }
  });

  it("maps doctor levels and harnesses", () => {
    expect(levelToStatusIcon("pass")).toBe("ok");
    expect(levelToStatusIcon("fail")).toBe("fail");
    expect(levelToStatusIcon("warn")).toBe("warn");
    expect(harnessToStatusIcon("claude-code")).toBe("agent-claude");
    expect(harnessToStatusIcon("codex")).toBe("agent-codex");
    expect(harnessToStatusIcon("grok-build")).toBe("agent-grok");
  });
});

describe("pngToFrame", () => {
  it("maps dark opaque pixels to silhouette", () => {
    const png = new PNG({ width: 2, height: 2 });
    // black, white, transparent, dark gray
    const samples = [
      [0, 0, 0, 255],
      [255, 255, 255, 255],
      [0, 0, 0, 0],
      [30, 30, 30, 255],
    ];
    samples.forEach((rgba, i) => {
      const offset = i * 4;
      png.data[offset] = rgba[0]!;
      png.data[offset + 1] = rgba[1]!;
      png.data[offset + 2] = rgba[2]!;
      png.data[offset + 3] = rgba[3]!;
    });
    const buffer = PNG.sync.write(png);
    const frame = pngToFrame(buffer, 1, undefined, { maxHeight: 64 });
    expect(frame.pixels).toEqual([true, false, false, true]);
  });

  it("downscales tall frames for TUI", () => {
    const png = new PNG({ width: 4, height: 40 });
    for (let i = 0; i < png.data.length; i += 4) {
      png.data[i] = 0;
      png.data[i + 1] = 0;
      png.data[i + 2] = 0;
      png.data[i + 3] = 255;
    }
    const buffer = PNG.sync.write(png);
    const frame = pngToFrame(buffer, 1, undefined, { maxHeight: 10 });
    expect(frame.height).toBe(10);
    expect(frame.width).toBeGreaterThan(0);
    expect(frame.pixels.every(Boolean)).toBe(true);
  });
});
