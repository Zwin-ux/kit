import { PassThrough } from "node:stream";
import React from "react";
import { render } from "ink";
import { describe, expect, it } from "vitest";

import { Workbench } from "../src/screens/Workbench.js";

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

describe("Workbench rendered terminal frames", () => {
  it.each([
    [60, 18],
    [80, 24],
    [120, 32],
  ] as const)("fits inside %ix%i", async (columns, rows) => {
    const terminal = terminalStream(columns, rows);
    const instance = render(
      React.createElement(Workbench, {
        projectDir: "C:\\work\\kit",
        runners: [
          {
            id: "codex",
            label: "Codex",
            available: true,
            executable: "codex",
          },
          {
            id: "claude-code",
            label: "Claude Code",
            available: true,
            executable: "claude",
          },
          {
            id: "grok-build",
            label: "Grok Build",
            available: true,
            executable: "grok",
          },
          {
            id: "ollama",
            label: "Ollama · Codex",
            available: true,
            executable: "codex",
            detail: "2 local models",
            models: [
              { name: "gemma4:e2b", size: 7_200_000_000 },
              { name: "llama3.1:latest", size: 4_900_000_000 },
            ],
          },
        ],
        serviceTasks: [
          {
            plugin: "trenchwire",
            displayName: "Trenchwire",
            task: "health",
            description: "Check local service health.",
            status: "ready",
          },
          {
            plugin: "trenchwire",
            displayName: "Trenchwire",
            task: "market",
            description: "Read recorded market data.",
            status: "ready",
          },
        ],
        lane: "runner",
        selectedRunnerIndex: 3,
        selectedTaskIndex: 0,
        selectedModelIndex: 0,
        mode: "inspect",
        prompt: "Inspect the workbench tests.",
        output: "reading files\nrunning tests\nall checks passed",
      }),
      {
        stdout: terminal.stream,
        stdin: inputStream(),
        exitOnCtrlC: false,
        patchConsole: false,
        debug: true,
      },
    );
    await new Promise<void>((resolve) => setImmediate(resolve));
    const frame = terminal.read();
    instance.unmount();

    const clean = frame.replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "");
    const lines = clean.split(/\r?\n/);
    expect(lines.length).toBeLessThanOrEqual(rows);
    expect(Math.max(...lines.map((line) => [...line].length))).toBeLessThanOrEqual(
      columns,
    );
    expect(clean).toContain("KIT / WORKBENCH");
    expect(clean).toContain("Ollama");
    expect(clean).toContain("LIVE OUTPUT");
  });
});
