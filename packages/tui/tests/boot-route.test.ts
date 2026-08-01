import { describe, expect, it, afterEach } from "vitest";
import {
  mascotAnimEnabled,
  mascotVisible,
  splashEnabled,
} from "../src/brand/mascotPolicy.js";

const keys = [
  "KIT_NO_MASCOT",
  "KIT_SHOW_MASCOT",
  "KIT_MASCOT_ANIM",
  "KIT_TUI_SPLASH",
] as const;

const saved: Partial<Record<(typeof keys)[number], string | undefined>> = {};

afterEach(() => {
  for (const key of keys) {
    if (saved[key] === undefined) delete process.env[key];
    else process.env[key] = saved[key];
    delete saved[key];
  }
});

function setEnv(key: (typeof keys)[number], value: string | undefined): void {
  if (!(key in saved)) saved[key] = process.env[key];
  if (value === undefined) delete process.env[key];
  else process.env[key] = value;
}

describe("agent-class boot policy", () => {
  it("hides mascot by default", () => {
    for (const key of keys) setEnv(key, undefined);
    expect(mascotVisible()).toBe(false);
    expect(mascotAnimEnabled()).toBe(false);
    expect(splashEnabled()).toBe(false);
  });

  it("allows opt-in static brand without animation", () => {
    for (const key of keys) setEnv(key, undefined);
    setEnv("KIT_SHOW_MASCOT", "1");
    expect(mascotVisible()).toBe(true);
    expect(mascotAnimEnabled()).toBe(false);
  });

  it("animates only with KIT_MASCOT_ANIM", () => {
    for (const key of keys) setEnv(key, undefined);
    setEnv("KIT_MASCOT_ANIM", "1");
    expect(mascotVisible()).toBe(true);
    expect(mascotAnimEnabled()).toBe(true);
  });

  it("splash is never default", () => {
    for (const key of keys) setEnv(key, undefined);
    expect(splashEnabled()).toBe(false);
    setEnv("KIT_TUI_SPLASH", "1");
    expect(splashEnabled()).toBe(true);
  });

  it("KIT_NO_MASCOT wins over show flags", () => {
    setEnv("KIT_SHOW_MASCOT", "1");
    setEnv("KIT_MASCOT_ANIM", "1");
    setEnv("KIT_NO_MASCOT", "1");
    expect(mascotVisible()).toBe(false);
    expect(mascotAnimEnabled()).toBe(false);
  });
});

describe("startTui defaults", () => {
  it("documents setup wizard as default entry", async () => {
    const src = await import("node:fs/promises").then((fs) =>
      fs.readFile(
        new URL("../src/start.tsx", import.meta.url),
        "utf8",
      ),
    );
    expect(src).toContain('options.initialScreen ?? "setup"');
    expect(src).toMatch(/Setup wizard|setup/i);
  });
});
