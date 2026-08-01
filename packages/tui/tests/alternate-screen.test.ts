import { describe, expect, it } from "vitest";

import {
  alternateScreenCodes,
  enterAlternateScreen,
} from "../src/terminal/alternateScreen.js";

describe("alternate terminal screen", () => {
  it("enters once and restores once", () => {
    const writes: string[] = [];
    const leave = enterAlternateScreen({
      isTTY: true,
      write(value) {
        writes.push(value);
      },
    });
    leave();
    leave();
    expect(writes).toEqual([
      alternateScreenCodes.enter,
      alternateScreenCodes.leave,
    ]);
  });

  it("does nothing outside a TTY", () => {
    const writes: string[] = [];
    const leave = enterAlternateScreen({
      isTTY: false,
      write(value) {
        writes.push(value);
      },
    });
    leave();
    expect(writes).toEqual([]);
  });
});
