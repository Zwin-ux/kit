import { describe, expect, it } from "vitest";
import { HitMap } from "../src/mouse/HitMap.js";
import {
  isPrimaryClick,
  parseSgrMouseChunk,
} from "../src/mouse/parseSgrMouse.js";

describe("parseSgrMouseChunk", () => {
  it("parses left click press", () => {
    const ev = parseSgrMouseChunk("\x1b[<0;12;8M");
    expect(ev).not.toBeNull();
    expect(ev!.x).toBe(12);
    expect(ev!.y).toBe(8);
    expect(isPrimaryClick(ev!)).toBe(true);
  });

  it("ignores release", () => {
    const ev = parseSgrMouseChunk("\x1b[<0;12;8m");
    expect(ev).not.toBeNull();
    expect(isPrimaryClick(ev!)).toBe(false);
  });
});

describe("HitMap", () => {
  it("hits list rows by y", () => {
    const map = new HitMap();
    map.addListRows({
      idPrefix: "pack",
      count: 3,
      startRow: 10,
      col0: 5,
      col1: 40,
    });
    expect(map.hit(10, 10)?.data?.index).toBe(0);
    expect(map.hit(20, 11)?.data?.index).toBe(1);
    expect(map.hit(20, 12)?.data?.index).toBe(2);
    expect(map.hit(20, 9)).toBeNull();
  });
});
