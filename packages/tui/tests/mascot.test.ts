import { describe, expect, it } from "vitest";
import { getPlaceholderFrames } from "../src/mascot/placeholderFrames.js";
import { renderFrame } from "../src/mascot/renderBitmap.js";
import { FRAME_COUNT } from "../src/mascot/types.js";
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

  it("renders only block characters and spaces", () => {
    const frame = getPlaceholderFrames()[0]!;
    const text = renderFrame(frame);
    expect(text.split("\n")).toHaveLength(16);
    expect(text).toMatch(/█/);
    expect(text).not.toMatch(/[a-zA-Z]/);
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
