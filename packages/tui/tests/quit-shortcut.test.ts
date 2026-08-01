import { describe, expect, it } from "vitest";

import { shouldQuitWithQ } from "../src/input/quitShortcut.js";

describe("Q quit shortcut", () => {
  it("quits from a stable navigation state", () => {
    expect(
      shouldQuitWithQ({
        input: "q",
        busy: false,
        enteringText: false,
        awaitingChoice: false,
      }),
    ).toBe(true);
  });

  it.each([
    { busy: true, enteringText: false, awaitingChoice: false },
    { busy: false, enteringText: true, awaitingChoice: false },
    { busy: false, enteringText: false, awaitingChoice: true },
  ])("does not steal q from active work: %o", (state) => {
    expect(shouldQuitWithQ({ input: "q", ...state })).toBe(false);
  });
});
