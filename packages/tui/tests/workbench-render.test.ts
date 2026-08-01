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

const basePacks = [
  {
    name: "essentials",
    title: "Essentials",
    description: "Core skills",
    version: "0.1.0",
    skillCount: 4,
    tags: [] as string[],
    projectTypes: [] as string[],
    extends: [] as string[],
  },
  {
    name: "web-app",
    title: "Web App",
    description: "Web stack",
    version: "0.1.0",
    skillCount: 6,
    tags: [] as string[],
    projectTypes: [] as string[],
    extends: [] as string[],
  },
];

const baseRunners = [
  {
    id: "codex" as const,
    label: "Codex",
    available: true,
    executable: "codex",
  },
  {
    id: "claude-code" as const,
    label: "Claude Code",
    available: true,
    executable: "claude",
  },
  {
    id: "grok-build" as const,
    label: "Grok Build",
    available: true,
    executable: "grok",
  },
  {
    id: "ollama" as const,
    label: "Ollama · Codex",
    available: true,
    executable: "codex",
    detail: "2 local models",
    models: [
      { name: "gemma4:e2b", size: 7_200_000_000 },
      { name: "llama3.1:latest", size: 4_900_000_000 },
    ],
  },
];

const baseServices = [
  {
    plugin: "trenchwire",
    displayName: "Trenchwire",
    task: "health",
    description: "Check local service health.",
    status: "ready" as const,
  },
  {
    plugin: "trenchwire",
    displayName: "Trenchwire",
    task: "market",
    description: "Read recorded market data.",
    status: "ready" as const,
  },
];

describe("Action Terminal rendered frames", () => {
  it.each([
    [60, 18],
    [80, 24],
    [120, 32],
  ] as const)("agents lane fits inside %ix%i", async (columns, rows) => {
    const terminal = terminalStream(columns, rows);
    const instance = render(
      React.createElement(Workbench, {
        projectDir: "C:\\work\\kit",
        packs: basePacks,
        selectedPackIndex: 0,
        runners: baseRunners,
        serviceTasks: baseServices,
        lane: "agents",
        selectedRunnerIndex: 3,
        selectedTaskIndex: 0,
        selectedModelIndex: 0,
        selectedOpsIndex: 0,
        mode: "inspect",
        prompt: "Inspect the workbench tests.",
        runStatus: "succeeded",
        runLabel: "Ollama · Codex / gemma4:e2b",
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
    expect(lines.length).toBeLessThanOrEqual(rows + 2);
    expect(clean).toMatch(/KIT|PACKS|AGENTS|SETUP/);
    expect(clean).toContain("Ollama");
    expect(clean).toMatch(/AGENTS|PACKS/);
    expect(clean).toMatch(/LOG|DONE|reading files|Install|Run/);
  });

  it.each([
    [60, 18],
    [80, 24],
  ] as const)(
    "services lane maps to selected task at %ix%i",
    async (columns, rows) => {
      const terminal = terminalStream(columns, rows);
      const instance = render(
        React.createElement(Workbench, {
          projectDir: "C:\\work\\kit",
          packs: basePacks,
          selectedPackIndex: 0,
          runners: baseRunners.slice(0, 1),
          serviceTasks: [
            {
              plugin: "trenchwire",
              displayName: "Trenchwire",
              task: "market",
              description: "Read recorded market data.",
              status: "ready",
            },
          ],
          lane: "services",
          selectedRunnerIndex: 0,
          selectedTaskIndex: 0,
          selectedModelIndex: 0,
          selectedOpsIndex: 0,
          mode: "inspect",
          prompt: "",
          runStatus: "idle",
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
      expect(clean).toMatch(/TOOLS|SERVICES|TASKS/);
      expect(clean).toContain("Trenchwire");
      expect(clean).toContain("market");
      expect(clean).toMatch(/Trenchwire|market|Run/);
    },
  );

  it("skills and ops lanes render product chrome", async () => {
    const terminal = terminalStream(100, 30);
    const instance = render(
      React.createElement(Workbench, {
        projectDir: "C:\\work\\kit",
        packs: basePacks,
        selectedPackIndex: 1,
        runners: baseRunners,
        serviceTasks: baseServices,
        lane: "skills",
        selectedRunnerIndex: 0,
        selectedTaskIndex: 0,
        selectedModelIndex: 0,
        selectedOpsIndex: 0,
        mode: "inspect",
        prompt: "",
        runStatus: "idle",
        storyTitle: "Make this repo agent-ready",
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
    const clean = terminal
      .read()
      .replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "");
    instance.unmount();
    expect(clean).toMatch(/PACKS|SKILLS/);
    expect(clean).toContain("Web App");
    expect(clean).toMatch(/Install|SETUP|KIT/);
  });

  it("keeps edit controls visible with reduced motion", async () => {
    const previous = process.env.KIT_REDUCED_MOTION;
    process.env.KIT_REDUCED_MOTION = "1";
    const terminal = terminalStream(60, 18);
    const instance = render(
      React.createElement(Workbench, {
        projectDir: "C:\\work\\kit",
        packs: basePacks,
        selectedPackIndex: 0,
        runners: baseRunners.slice(0, 1),
        serviceTasks: [],
        lane: "agents",
        selectedRunnerIndex: 0,
        selectedTaskIndex: 0,
        selectedModelIndex: 0,
        selectedOpsIndex: 0,
        mode: "inspect",
        prompt: "Explain q behavior",
        editingPrompt: true,
        runStatus: "idle",
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
    if (previous === undefined) delete process.env.KIT_REDUCED_MOTION;
    else process.env.KIT_REDUCED_MOTION = previous;

    const clean = frame.replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "");
    expect(clean).toMatch(/Explain q behavior|edit|job|>/i);
    expect(clean).toMatch(/Run|Enter|AGENTS/);
  });
});
