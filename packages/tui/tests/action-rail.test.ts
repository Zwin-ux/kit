import { PassThrough } from "node:stream";
import React from "react";
import { render } from "ink";
import { describe, expect, it } from "vitest";
import { ActionRail } from "../src/components/ActionRail.js";
import { Help } from "../src/screens/Help.js";

function terminalStream(columns: number, rows: number): {
  stream: NodeJS.WriteStream;
  read: () => string;
} {
  let output = "";
  const stream = new PassThrough() as unknown as NodeJS.WriteStream;
  stream.on("data", (chunk) => {
    output += chunk.toString();
  });
  Object.assign(stream, {
    columns,
    rows,
    isTTY: true,
  });
  return { stream, read: () => output };
}

function inputStream(): NodeJS.ReadStream {
  const stream = new PassThrough() as unknown as NodeJS.ReadStream;
  Object.assign(stream, {
    isTTY: true,
    setRawMode() {},
  });
  return stream;
}

describe("ActionRail", () => {
  it("renders primary action and story", async () => {
    const terminal = terminalStream(100, 30);
    const instance = render(
      React.createElement(ActionRail, {
        story: "Make this repo agent-ready",
        items: [
          { key: "r", label: "Ready", primary: true },
          { key: "u", label: "Unify" },
          { key: "w", label: "Workbench" },
        ],
      }),
      { stdout: terminal.stream, stdin: inputStream() },
    );
    await new Promise((r) => setTimeout(r, 50));
    const text = terminal.read();
    expect(text).toContain("Ready");
    expect(text).toContain("Unify");
    expect(text).toContain("agent-ready");
    instance.unmount();
  });
});

describe("Help screen", () => {
  it("lists product keys", async () => {
    const terminal = terminalStream(100, 40);
    // Empty frames → hide mascot path (bitmap renderer needs real pixels).
    const instance = render(
      React.createElement(Help, {
        frames: [],
        fromScreen: "home",
      }),
      { stdout: terminal.stream, stdin: inputStream() },
    );
    await new Promise((r) => setTimeout(r, 50));
    const text = terminal.read();
    expect(text).toMatch(/Ready|ready/i);
    expect(text).toMatch(/Workbench|workbench/i);
    expect(text).toMatch(/Help|help/i);
    instance.unmount();
  });
});
