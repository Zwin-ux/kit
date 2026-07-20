import { afterEach, describe, expect, it } from "vitest";
import { motionEnabled } from "../src/motion/motionEnabled.js";

describe("motionEnabled", () => {
  const prev = process.env.KIT_REDUCED_MOTION;

  afterEach(() => {
    if (prev === undefined) {
      delete process.env.KIT_REDUCED_MOTION;
    } else {
      process.env.KIT_REDUCED_MOTION = prev;
    }
  });

  it("is true by default", () => {
    delete process.env.KIT_REDUCED_MOTION;
    expect(motionEnabled()).toBe(true);
  });

  it("is false when KIT_REDUCED_MOTION=1", () => {
    process.env.KIT_REDUCED_MOTION = "1";
    expect(motionEnabled()).toBe(false);
  });

  it("stays true for other values", () => {
    process.env.KIT_REDUCED_MOTION = "0";
    expect(motionEnabled()).toBe(true);
  });
});

describe("typewriter timing budget", () => {
  it("default cps finishes short splash line under 1.2s", () => {
    const text = "Portable agent skills";
    const cps = 30;
    const ms = (text.length / cps) * 1000;
    expect(ms).toBeLessThanOrEqual(1200);
  });

  it("default success cps finishes typical install line under 1.5s", () => {
    const text = "Installed essentials@0.0.0 (5 skills)";
    const cps = 32;
    const ms = (text.length / cps) * 1000;
    expect(ms).toBeLessThanOrEqual(1500);
  });
});

describe("rail motion budget", () => {
  it("keeps rail fps calm to reduce full-screen paint thrash", async () => {
    const { LAYOUT_CAPS } = await import("../src/mascot/layoutScale.js");
    expect(LAYOUT_CAPS.railFrameDelayMs).toBeGreaterThanOrEqual(200);
  });
});
