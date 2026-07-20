import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import { recommendToolkits } from "../src/recommend/mod.js";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);

const tempDirs: string[] = [];

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("recommendToolkits", () => {
  it("recommends web-app for react package.json", async () => {
    const dir = await tempDir("kit-rec-web-");
    await writeFile(
      path.join(dir, "package.json"),
      JSON.stringify({
        name: "demo-app",
        dependencies: { react: "^18.0.0", next: "^14.0.0" },
      }),
      "utf8",
    );

    const result = await recommendToolkits({
      projectDir: dir,
      packsRoot: path.join(repoRoot, "packs"),
      skillsRoot: path.join(repoRoot, "skills"),
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.topPick).toBe("web-app");
    expect(result.value.signals).toContain("web-framework");
  });

  it("recommends library for package-shaped ts project", async () => {
    const dir = await tempDir("kit-rec-lib-");
    await writeFile(
      path.join(dir, "package.json"),
      JSON.stringify({
        name: "@demo/sdk",
        devDependencies: { typescript: "^5.0.0" },
      }),
      "utf8",
    );

    const result = await recommendToolkits({
      projectDir: dir,
      packsRoot: path.join(repoRoot, "packs"),
      skillsRoot: path.join(repoRoot, "skills"),
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.topPick).toBe("library");
  });

  it("always includes essentials", async () => {
    const dir = await tempDir("kit-rec-empty-");
    await mkdir(dir, { recursive: true });
    const result = await recommendToolkits({
      projectDir: dir,
      packsRoot: path.join(repoRoot, "packs"),
      skillsRoot: path.join(repoRoot, "skills"),
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(
      result.value.recommendations.some((r) => r.packName === "essentials"),
    ).toBe(true);
  });
});
