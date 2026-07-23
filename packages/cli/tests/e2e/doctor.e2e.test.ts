/**
 * E2E: kit doctor + basic CLI dispatch, against the compiled dist/bin.js.
 * Everything runs inside an isolated sandbox (HOME/USERPROFILE/KIT_HOME).
 */
import { readFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  REPO_ROOT,
  assertRealHomeUntouched,
  createSandbox,
  expectExit,
  expectOutput,
  outputMentionsPath,
  snapshotRealHome,
  type RealHomeSnapshot,
  type Sandbox,
} from "./harness.js";
import { makeWebProject } from "./fixtures.js";

let realHome: RealHomeSnapshot;
const sandboxes: Sandbox[] = [];

async function sandbox(projectDirName?: string): Promise<Sandbox> {
  const sb = await createSandbox(
    projectDirName !== undefined ? { projectDirName } : {},
  );
  sandboxes.push(sb);
  return sb;
}

beforeAll(async () => {
  realHome = await snapshotRealHome();
});

afterAll(async () => {
  for (const sb of sandboxes) await sb.cleanup();
  await assertRealHomeUntouched(realHome);
});

describe("kit --version / dispatch", () => {
  it("prints the version from packages/cli/package.json", async () => {
    const sb = await sandbox();
    const pkg = JSON.parse(
      await readFile(
        path.join(REPO_ROOT, "packages", "cli", "package.json"),
        "utf8",
      ),
    ) as { version: string };

    const result = await sb.runKit(["--version"]);
    expectExit(result, 0);
    expect(result.stdout.trim()).toBe(pkg.version);
  });

  it("--help exits 0 and shows core commands", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["--help"]);
    expectExit(result, 0);
    expectOutput(result, "kit ready --write");
    expectOutput(result, "kit doctor");
  });

  it("unknown command exits 1 with guidance", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["frobnicate"]);
    expectExit(result, 1);
    expectOutput(result, "unknown command: frobnicate");
  });
});

describe("kit doctor", () => {
  it("exits 0 on a fresh sandbox and reports zero failures", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["doctor"]);
    expectExit(result, 0);
    expectOutput(result, "kit doctor  v");
    expect(result.stdout).toMatch(/·\s+0 fail/);
    expectOutput(result, "Doctor OK");
  });

  it("reports the sandbox KIT_HOME, never the real one", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["doctor"]);
    expectExit(result, 0);
    expect(
      outputMentionsPath(result.stdout, sb.kitHome),
      `doctor home line must point into the sandbox\n${result.describe()}`,
    ).toBe(true);
  });

  it("stays green after kit ready --write and sees the library", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const ready = await sb.runKit(["ready", "--write", "--pack", "essentials"]);
    expectExit(ready, 0);

    const result = await sb.runKit(["doctor"]);
    expectExit(result, 0);
    expect(result.stdout).toMatch(/Library has \d+ skill\(s\)/);
    expect(result.stdout).toMatch(/·\s+0 fail/);
  });

  it("honors --dir for the project context", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["doctor", "--dir", sb.projectDir], {
      cwd: sb.root,
    });
    expectExit(result, 0);
    expect(
      outputMentionsPath(result.stdout, sb.projectDir),
      `doctor project line must show --dir target\n${result.describe()}`,
    ).toBe(true);
  });
});
