import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  completeFirstRun,
  getFirstRunStatus,
  readConfig,
  writeConfig,
} from "../src/config/mod.js";
import { installSkill } from "../src/library/library.js";
import { fileURLToPath } from "node:url";

const repoSkills = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../skills/add-readme",
);

const tempDirs: string[] = [];

async function tempHome(): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "kit-cfg-"));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("kit config + first-run", () => {
  it("returns defaults when config is missing", async () => {
    const kitHome = await tempHome();
    const config = await readConfig(kitHome);
    expect(config.firstRunCompleted).toBe(false);
    expect(config.preferredPack).toBe("essentials");
  });

  it("offers first-run when library is empty and not completed", async () => {
    const kitHome = await tempHome();
    const status = await getFirstRunStatus(kitHome);
    expect(status.shouldOffer).toBe(true);
    expect(status.reason).toBe("not-completed");
  });

  it("stops offering after completeFirstRun skip", async () => {
    const kitHome = await tempHome();
    await completeFirstRun("skipped", { kitHome });
    const status = await getFirstRunStatus(kitHome);
    expect(status.shouldOffer).toBe(false);
    expect(status.config.firstRunOutcome).toBe("skipped");
  });

  it("does not offer when library already has skills", async () => {
    const kitHome = await tempHome();
    const installed = await installSkill(repoSkills, { kitHome, force: true });
    expect(installed.ok).toBe(true);
    const status = await getFirstRunStatus(kitHome);
    expect(status.shouldOffer).toBe(false);
    expect(status.reason).toBe("library-already-used");
  });

  it("persists preferred pack on write", async () => {
    const kitHome = await tempHome();
    await writeConfig(
      {
        version: 1,
        firstRunCompleted: true,
        preferredPack: "web-app",
        firstRunOutcome: "installed",
      },
      kitHome,
    );
    const config = await readConfig(kitHome);
    expect(config.preferredPack).toBe("web-app");
    expect(config.firstRunCompleted).toBe(true);
  });
});
