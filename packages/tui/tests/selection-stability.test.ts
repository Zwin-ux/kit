import { describe, expect, it } from "vitest";
import {
  fixedLine,
  fixedLines,
  selectCursorGlyph,
} from "../src/motion/fixedLines.js";
import {
  OFFICIAL_PACK_ICONS,
  packIconGlyph,
} from "../src/mascot/packIcons.js";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

describe("fixedLines", () => {
  it("always returns exactly N lines of fixed width", () => {
    for (const text of ["", "short", "a ".repeat(200), "x".repeat(400)]) {
      const lines = fixedLines(text, 2, 40);
      expect(lines).toHaveLength(2);
      for (const line of lines) {
        expect(line).toHaveLength(40);
      }
    }
  });

  it("fixedLine pads to maxCols", () => {
    expect(fixedLine("hi", 10)).toHaveLength(10);
    expect(fixedLine("hello world this is long", 10)).toHaveLength(10);
  });
});

describe("selectCursorGlyph", () => {
  it("is always 2 ASCII cells", () => {
    expect(selectCursorGlyph(false)).toBe("  ");
    expect(selectCursorGlyph(true, "none")).toBe("> ");
    expect(selectCursorGlyph(true, "up")).toBe("^ ");
    expect(selectCursorGlyph(true, "down")).toBe("v ");
    for (const g of [
      selectCursorGlyph(false),
      selectCursorGlyph(true, "none"),
      selectCursorGlyph(true, "up"),
      selectCursorGlyph(true, "down"),
    ]) {
      expect(g).toHaveLength(2);
      expect([...g].every((c) => c.charCodeAt(0) < 128)).toBe(true);
    }
  });
});

describe("packIconGlyph", () => {
  it("is single ASCII cell for every pack", () => {
    for (const name of OFFICIAL_PACK_ICONS) {
      const g = packIconGlyph(name, 0);
      expect(g).toHaveLength(1);
      expect(g.charCodeAt(0)).toBeLessThan(128);
    }
  });
});

describe("screens forbid wrap under selection", () => {
  it("Library and Explore do not use wrap=wrap", () => {
    const dir = path.dirname(fileURLToPath(import.meta.url));
    const screens = ["Library.tsx", "Explore.tsx", "Home.tsx", "Packs.tsx"];
    for (const file of screens) {
      const src = readFileSync(
        path.join(dir, "../src/screens", file),
        "utf8",
      );
      expect(src.includes('wrap="wrap"')).toBe(false);
    }
  });
});
