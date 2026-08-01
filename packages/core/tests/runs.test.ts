import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { listRuns, loadRun, saveRun } from "../src/workbench/runs.js";

const dirs: string[] = [];

afterEach(async () => {
  while (dirs.length) {
    const d = dirs.pop();
    if (d) await rm(d, { recursive: true, force: true });
  }
});

async function tempHome(): Promise<string> {
  const d = await mkdtemp(path.join(os.tmpdir(), "kit-runs-"));
  dirs.push(d);
  return d;
}

describe("job proof vault", () => {
  it("saves a run and reloads the transcript", async () => {
    const kitHome = await tempHome();
    const saved = await saveRun({
      kitHome,
      kind: "coding",
      label: "Codex / inspect",
      projectDir: "/work/demo",
      status: "succeeded",
      transcript: "line one\nline two\n",
      runner: "codex",
      mode: "inspect",
      prompt: "Inspect the tests.",
      exitCode: 0,
      durationMs: 42,
    });
    expect(saved.ok).toBe(true);
    if (!saved.ok) return;

    const listed = await listRuns({ kitHome, limit: 5 });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value[0]?.id).toBe(saved.value.id);
    expect(listed.value[0]?.label).toContain("Codex");

    const loaded = await loadRun(saved.value.id, { kitHome });
    expect(loaded.ok).toBe(true);
    if (!loaded.ok) return;
    expect(loaded.value.transcript).toContain("line one");
    expect(loaded.value.prompt).toBe("Inspect the tests.");
    expect(loaded.value.logPath).toContain(saved.value.id);
  });

  it("keeps newest runs first and caps index", async () => {
    const kitHome = await tempHome();
    for (let i = 0; i < 3; i++) {
      const r = await saveRun({
        kitHome,
        kind: "ops",
        label: `ops-${i}`,
        projectDir: "/p",
        status: "succeeded",
        transcript: `t${i}`,
      });
      expect(r.ok).toBe(true);
    }
    const listed = await listRuns({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value[0]?.label).toBe("ops-2");
    expect(listed.value).toHaveLength(3);
  });
});
