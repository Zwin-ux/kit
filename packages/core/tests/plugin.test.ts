import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";

import {
  addPlugin,
  doctorPlugin,
  getPluginsIndexPath,
  listPlugins,
  removePlugin,
  runPlugin,
} from "../src/plugin/mod.js";


const tempDirs: string[] = [];

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

async function writeManifest(
  root: string,
  overrides: Record<string, unknown> = {},
): Promise<void> {
  await writeFile(
    path.join(root, "kit.plugin.json"),
    `${JSON.stringify(
      {
        schemaVersion: 1,
        name: "test-cli",
        displayName: "Test CLI",
        description: "Runs a controlled test command.",
        version: "1.0.0",
        command: "node",
        versionArgs: ["--version"],
        healthArgs: ["--version"],
        safety: {
          summary: "The test command writes no external state.",
        },
        ...overrides,
      },
      null,
      2,
    )}\n`,
    "utf8",
  );
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("Kit CLI plugins", () => {
  it("keeps add as a dry-run until write is explicit", async () => {
    const kitHome = await tempDir("kit-plugin-home-");
    const pluginRoot = await tempDir("kit-plugin-source-");
    await writeManifest(pluginRoot);

    const result = await addPlugin(pluginRoot, { kitHome });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.dryRun).toBe(true);
    await expect(readFile(getPluginsIndexPath(kitHome), "utf8")).rejects.toThrow();
  });

  it("registers, checks, and runs a CLI without a shell", async () => {
    const kitHome = await tempDir("kit-plugin-home-");
    const pluginRoot = await tempDir("kit-plugin-source-");
    await writeManifest(pluginRoot);

    const added = await addPlugin(pluginRoot, { kitHome, write: true });
    expect(added.ok).toBe(true);

    const listed = await listPlugins(kitHome);
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value.map((plugin) => plugin.manifest.name)).toEqual([
      "test-cli",
    ]);

    const doctor = await doctorPlugin("test-cli", kitHome);
    expect(doctor.ok).toBe(true);
    if (!doctor.ok) return;
    expect(doctor.value.ready).toBe(true);
    expect(doctor.value.executableSource).toBe("path");
    expect(doctor.value.manifestChanged).toBe(false);

    const run = await runPlugin("test-cli", ["--version"], {
      kitHome,
      stdio: "pipe",
    });
    expect(run.ok).toBe(true);
    if (!run.ok) return;
    expect(run.value.exitCode).toBe(0);
    expect(run.value.stdout.trim()).toMatch(/^v\d+/);
    expect(run.value.stderr).toBe("");
  });

  it("reports a changed manifest and removes only with write", async () => {
    const kitHome = await tempDir("kit-plugin-home-");
    const pluginRoot = await tempDir("kit-plugin-source-");
    await writeManifest(pluginRoot);
    await addPlugin(pluginRoot, { kitHome, write: true });

    await writeManifest(pluginRoot, {
      description: "The description changed after registration.",
    });
    const doctor = await doctorPlugin("test-cli", kitHome);
    expect(doctor.ok && doctor.value.manifestChanged).toBe(true);

    const blockedRun = await runPlugin("test-cli", ["--version"], {
      kitHome,
      stdio: "pipe",
    });
    expect(blockedRun.ok).toBe(false);
    if (blockedRun.ok) return;
    expect(blockedRun.error).toContain("changed after registration");

    const planned = await removePlugin("test-cli", { kitHome });
    expect(planned.ok && planned.value.dryRun).toBe(true);
    const beforeWrite = await listPlugins(kitHome);
    expect(beforeWrite.ok && beforeWrite.value).toHaveLength(1);

    const removed = await removePlugin("test-cli", {
      kitHome,
      write: true,
    });
    expect(removed.ok && removed.value.dryRun).toBe(false);
    const afterWrite = await listPlugins(kitHome);
    expect(afterWrite.ok && afterWrite.value).toHaveLength(0);
  });

  it("rejects a local executable outside the plugin root", async () => {
    const kitHome = await tempDir("kit-plugin-home-");
    const pluginRoot = await tempDir("kit-plugin-source-");
    await writeManifest(pluginRoot, {
      localExecutables: {
        default: "../outside",
      },
    });

    const result = await addPlugin(pluginRoot, { kitHome, write: true });
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.error).toContain("inside the plugin root");
  });
});
