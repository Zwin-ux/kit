import { mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import {
  detectCodingRunners,
  planCodingJob,
  runCodingJob,
} from "../src/workbench/mod.js";

const tempDirs: string[] = [];

async function tempDir(): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "kit-workbench-"));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("Kit Workbench", () => {
  it("detects each supported local coding runner", async () => {
    const runners = await detectCodingRunners();
    expect(runners.map((runner) => runner.id)).toEqual([
      "codex",
      "claude-code",
      "grok-build",
    ]);
  });

  it("maps inspect mode to provider read-only arguments", async () => {
    const projectDir = await tempDir();
    const codex = await planCodingJob(
      {
        runner: "codex",
        mode: "inspect",
        projectDir,
        prompt: "Inspect the tests.",
      },
      { executable: "codex-fixture", prefixArgs: [] },
    );
    expect(codex.ok).toBe(true);
    if (!codex.ok) return;
    expect(codex.value.args).toContain("read-only");
    expect(codex.value.args).toContain("--ephemeral");

    const claude = await planCodingJob(
      {
        runner: "claude-code",
        mode: "inspect",
        projectDir,
        prompt: "Inspect the tests.",
      },
      { executable: "claude-fixture", prefixArgs: [] },
    );
    expect(claude.ok && claude.value.args).toContain("plan");
  });

  it("blocks build mode until it is confirmed", async () => {
    const projectDir = await tempDir();
    const result = await planCodingJob(
      {
        runner: "grok-build",
        mode: "build",
        projectDir,
        prompt: "Add one test.",
      },
      { executable: "grok-fixture", prefixArgs: [] },
    );
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.error).toContain("explicit confirmation");
  });

  it("runs through a trusted shim and bounds captured output", async () => {
    const projectDir = await tempDir();
    const script = path.join(projectDir, "runner.mjs");
    await writeFile(script, "process.stdout.write('x'.repeat(200));\n", "utf8");
    const result = await runCodingJob(
      {
        runner: "claude-code",
        mode: "inspect",
        projectDir,
        prompt: "Inspect the tests.",
      },
      {
        commandOverride: {
          executable: process.execPath,
          prefixArgs: [script],
        },
        maxOutputBytes: 32,
        timeoutMs: 2_000,
      },
    );
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.exitCode).toBe(0);
    expect(Buffer.byteLength(result.value.stdout)).toBe(32);
    expect(result.value.truncated).toBe(true);
    expect(result.value.timedOut).toBe(false);
  });

  it("stops a job after its timeout", async () => {
    const projectDir = await tempDir();
    const script = path.join(projectDir, "runner.mjs");
    await writeFile(script, "setTimeout(() => {}, 10_000);\n", "utf8");
    const result = await runCodingJob(
      {
        runner: "claude-code",
        mode: "inspect",
        projectDir,
        prompt: "Wait.",
      },
      {
        commandOverride: {
          executable: process.execPath,
          prefixArgs: [script],
        },
        timeoutMs: 50,
      },
    );
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.timedOut).toBe(true);
  });
});
