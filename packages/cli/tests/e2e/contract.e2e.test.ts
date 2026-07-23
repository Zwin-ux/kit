/**
 * E2E: stable JSON contract (schemaVersion "1") for doctor + ready.
 *
 * Golden guarantees pinned here:
 * - `--json` stdout is pure JSON (whole stdout parses).
 * - Envelope shape is exactly {schemaVersion, command, ok, data, warnings, errors}.
 * - Error codes are typed (E_GUARD_REFUSED, E_CHECK_FAILED, ...).
 * - Pre-envelope commands (status/unify --json) keep their raw report shape.
 */
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  assertRealHomeUntouched,
  createSandbox,
  expectExit,
  snapshotRealHome,
  type RealHomeSnapshot,
  type Sandbox,
} from "./harness.js";
import { makeWebProject } from "./fixtures.js";

const ENVELOPE_KEYS = [
  "command",
  "data",
  "errors",
  "ok",
  "schemaVersion",
  "warnings",
];

interface Envelope {
  schemaVersion: string;
  command: string;
  ok: boolean;
  data: unknown;
  warnings: string[];
  errors: Array<{ code: string; message: string }>;
}

function parseEnvelope(stdout: string, context: string): Envelope {
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch (error) {
    throw new Error(
      `${context}: stdout is not pure JSON (${String(error)})\n---\n${stdout}`,
    );
  }
  const envelope = parsed as Envelope;
  expect(Object.keys(envelope).sort(), `${context}: envelope keys`).toEqual(
    ENVELOPE_KEYS,
  );
  expect(envelope.schemaVersion, `${context}: schemaVersion`).toBe("1");
  return envelope;
}

let realHome: RealHomeSnapshot;
const sandboxes: Sandbox[] = [];

async function sandbox(): Promise<Sandbox> {
  const sb = await createSandbox();
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

describe("kit doctor --json", () => {
  it("emits the versioned envelope with doctor data", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["doctor", "--json"]);
    expectExit(result, 0);

    const envelope = parseEnvelope(result.stdout, "doctor --json");
    expect(envelope.command).toBe("doctor");
    expect(envelope.ok).toBe(true);
    expect(envelope.errors).toEqual([]);
    // Fresh sandbox always has warn-level checks (registry, auth, library).
    expect(envelope.warnings.length).toBeGreaterThan(0);

    const data = envelope.data as {
      ok: boolean;
      version: string;
      summary: { failed: number };
      checks: unknown[];
    };
    expect(data.ok).toBe(true);
    expect(data.summary.failed).toBe(0);
    expect(data.checks.length).toBeGreaterThan(0);
  });

  it("accepts --no-color anywhere on the command line", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["doctor", "--no-color", "--json"]);
    expectExit(result, 0);
    const envelope = parseEnvelope(result.stdout, "doctor --no-color --json");
    expect(envelope.command).toBe("doctor");
    // eslint-disable-next-line no-control-regex
    expect(result.stdout).not.toMatch(/\x1b\[/);
  });
});

describe("kit ready --json", () => {
  it("dry-run: ok envelope with the planned report", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit(["ready", "--json"]);
    expectExit(result, 0);

    const envelope = parseEnvelope(result.stdout, "ready --json");
    expect(envelope.command).toBe("ready");
    expect(envelope.ok).toBe(true);
    expect(envelope.errors).toEqual([]);

    const data = envelope.data as {
      dryRun: boolean;
      packName: string;
      steps: Array<{ id: string; status: string }>;
    };
    expect(data.dryRun).toBe(true);
    expect(data.steps.map((s) => s.id)).toEqual([
      "pack-install",
      "pack-apply",
      "unify",
      "link",
      "doctor",
    ]);
  });

  it("--write: ok envelope with complete=true", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit([
      "ready",
      "--write",
      "--pack",
      "essentials",
      "--json",
    ]);
    expectExit(result, 0);

    const envelope = parseEnvelope(result.stdout, "ready --write --json");
    expect(envelope.ok).toBe(true);
    const data = envelope.data as { dryRun: boolean; complete: boolean };
    expect(data.dryRun).toBe(false);
    expect(data.complete).toBe(true);
  });

  it("guard refusal: typed E_GUARD_REFUSED error, exit 1", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["ready", "--write", "--json"], {
      cwd: sb.home,
    });
    expectExit(result, 1);

    const envelope = parseEnvelope(result.stdout, "ready guard --json");
    expect(envelope.ok).toBe(false);
    expect(envelope.data).toBeNull();
    expect(envelope.errors).toHaveLength(1);
    expect(envelope.errors[0]?.code).toBe("E_GUARD_REFUSED");
  });
});

describe("pre-envelope --json commands stay unchanged", () => {
  it("status --json keeps its raw report shape (no envelope)", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["status", "--json"]);
    expectExit(result, 0);
    const report = JSON.parse(result.stdout) as Record<string, unknown>;
    expect(report).not.toHaveProperty("schemaVersion");
    expect(report).toHaveProperty("libraryCount");
    expect(report).toHaveProperty("rows");
  });

  it("unify --json keeps its raw report shape (no envelope)", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["unify", "--json"]);
    expectExit(result, 0);
    const report = JSON.parse(result.stdout) as Record<string, unknown>;
    expect(report).not.toHaveProperty("schemaVersion");
    expect(report).toHaveProperty("scanned");
    expect(report).toHaveProperty("keepers");
  });
});
