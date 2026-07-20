import { mkdtemp, mkdir, writeFile, rm, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  installSkill,
  listSkills,
  removeSkill,
  getLibraryIndexPath,
  getSkillsDir,
} from "../src/library/mod.js";

const tempHomes: string[] = [];

async function makeKitHome(): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "kit-lib-"));
  tempHomes.push(dir);
  return dir;
}

async function writeSkill(
  root: string,
  folderName: string,
  skillMd: string,
): Promise<string> {
  const skillDir = path.join(root, folderName);
  await mkdir(skillDir, { recursive: true });
  await writeFile(path.join(skillDir, "SKILL.md"), skillMd, "utf8");
  return skillDir;
}

const validSkillMd = `---
name: add-readme
description: Create a clear README for a new project.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
---

# Instructions

Write a README.
`;

afterEach(async () => {
  while (tempHomes.length > 0) {
    const dir = tempHomes.pop();
    if (dir) {
      await rm(dir, { recursive: true, force: true });
    }
  }
});

describe("local skill library", () => {
  it("installs a skill from a folder", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(sourceRoot, "add-readme", validSkillMd);
    const result = await installSkill(source, { kitHome });

    expect(result.ok).toBe(true);
    if (!result.ok) return;

    expect(result.value.name).toBe("add-readme");
    expect(result.value.version).toBe("0.1.0");
    expect(result.value.installPath).toBe(
      path.join(getSkillsDir(kitHome), "add-readme"),
    );

    const skillFile = path.join(result.value.installPath, "SKILL.md");
    const copied = await readFile(skillFile, "utf8");
    expect(copied).toContain("name: add-readme");

    const indexRaw = await readFile(getLibraryIndexPath(kitHome), "utf8");
    const index = JSON.parse(indexRaw) as {
      skills: Record<string, { name: string }>;
    };
    expect(index.skills["add-readme"]?.name).toBe("add-readme");
  });

  it("lists installed skills", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(sourceRoot, "add-readme", validSkillMd);
    await installSkill(source, { kitHome });

    const listed = await listSkills({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;

    expect(listed.value).toHaveLength(1);
    expect(listed.value[0]?.name).toBe("add-readme");
  });

  it("removes an installed skill", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(sourceRoot, "add-readme", validSkillMd);
    await installSkill(source, { kitHome });

    const removed = await removeSkill("add-readme", { kitHome });
    expect(removed.ok).toBe(true);

    const listed = await listSkills({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value).toHaveLength(0);
  });

  it("rejects invalid skills on install", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(
      sourceRoot,
      "bad",
      `---
name: Bad_Name
description: Invalid.
version: 0.1.0
compatibility:
  - codex
---

# Body
`,
    );

    const result = await installSkill(source, { kitHome });
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.error).toMatch(/validation/i);
  });

  it("fails when skill already installed without force", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(sourceRoot, "add-readme", validSkillMd);
    await installSkill(source, { kitHome });
    const second = await installSkill(source, { kitHome });

    expect(second.ok).toBe(false);
    if (second.ok) return;
    expect(second.error).toMatch(/already installed/);
  });

  it("replaces install when force is true", async () => {
    const kitHome = await makeKitHome();
    const sourceRoot = await mkdtemp(path.join(os.tmpdir(), "kit-src-"));
    tempHomes.push(sourceRoot);

    const source = await writeSkill(sourceRoot, "add-readme", validSkillMd);
    await installSkill(source, { kitHome });

    const updatedMd = validSkillMd.replace("version: 0.1.0", "version: 0.2.0");
    await writeFile(path.join(source, "SKILL.md"), updatedMd, "utf8");

    const replaced = await installSkill(source, { kitHome, force: true });
    expect(replaced.ok).toBe(true);
    if (!replaced.ok) return;
    expect(replaced.value.version).toBe("0.2.0");
  });

  it("returns an error when removing a missing skill", async () => {
    const kitHome = await makeKitHome();
    const result = await removeSkill("does-not-exist", { kitHome });
    expect(result.ok).toBe(false);
  });
});
