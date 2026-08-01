import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { getKitHome, getRunsDir } from "../library/paths.js";
import type {
  CodingJobMode,
  CodingRunnerId,
  SavedRunRecord,
  SavedRunSummary,
  WorkbenchResult,
} from "./types.js";

const INDEX_NAME = "index.json";
const MAX_INDEX = 50;
const MAX_LOG_CHARS = 256 * 1024;

export interface SaveRunInput {
  kind: "coding" | "service" | "ops";
  label: string;
  projectDir: string;
  status: SavedRunRecord["status"];
  transcript: string;
  runner?: CodingRunnerId;
  mode?: CodingJobMode;
  model?: string;
  prompt?: string;
  plugin?: string;
  task?: string;
  exitCode?: number;
  durationMs?: number;
  error?: string;
  kitHome?: string;
}

function newRunId(): string {
  const t = new Date().toISOString().replace(/[:.]/g, "-");
  const r = Math.random().toString(36).slice(2, 8);
  return `${t}-${r}`;
}

async function ensureRunsDir(kitHome: string): Promise<string> {
  const dir = getRunsDir(kitHome);
  await mkdir(dir, { recursive: true });
  return dir;
}

async function readIndex(
  kitHome: string,
): Promise<SavedRunSummary[]> {
  try {
    const raw = await readFile(
      path.join(getRunsDir(kitHome), INDEX_NAME),
      "utf8",
    );
    const data = JSON.parse(raw) as { runs?: SavedRunSummary[] };
    return Array.isArray(data.runs) ? data.runs : [];
  } catch {
    return [];
  }
}

async function writeIndex(
  kitHome: string,
  runs: SavedRunSummary[],
): Promise<void> {
  const dir = await ensureRunsDir(kitHome);
  await writeFile(
    path.join(dir, INDEX_NAME),
    JSON.stringify({ version: 1, runs }, null, 2),
    "utf8",
  );
}

/**
 * Save a finished job proof under ~/.kit/runs/.
 * Keeps an index of the last 50 runs. Does not store API keys.
 */
export async function saveRun(
  input: SaveRunInput,
): Promise<WorkbenchResult<SavedRunRecord>> {
  const kitHome = input.kitHome ?? getKitHome();
  const dir = await ensureRunsDir(kitHome);
  const id = newRunId();
  const createdAt = new Date().toISOString();
  const transcript = input.transcript.slice(-MAX_LOG_CHARS);
  const logPath = path.join(dir, `${id}.log`);
  const metaPath = path.join(dir, `${id}.json`);

  const record: SavedRunRecord = {
    id,
    kind: input.kind,
    label: input.label,
    projectDir: path.resolve(input.projectDir),
    status: input.status,
    createdAt,
    logPath,
    metaPath,
    ...(input.runner !== undefined ? { runner: input.runner } : {}),
    ...(input.mode !== undefined ? { mode: input.mode } : {}),
    ...(input.model !== undefined ? { model: input.model } : {}),
    ...(input.prompt !== undefined
      ? { prompt: input.prompt.slice(0, 8_000) }
      : {}),
    ...(input.plugin !== undefined ? { plugin: input.plugin } : {}),
    ...(input.task !== undefined ? { task: input.task } : {}),
    ...(input.exitCode !== undefined ? { exitCode: input.exitCode } : {}),
    ...(input.durationMs !== undefined
      ? { durationMs: input.durationMs }
      : {}),
    ...(input.error !== undefined ? { error: input.error } : {}),
    transcript,
  };

  try {
    await writeFile(logPath, transcript, "utf8");
    await writeFile(metaPath, JSON.stringify(record, null, 2), "utf8");

    const summary: SavedRunSummary = {
      id,
      kind: record.kind,
      label: record.label,
      projectDir: record.projectDir,
      status: record.status,
      createdAt,
      logPath,
    };
    const prev = await readIndex(kitHome);
    const next = [summary, ...prev.filter((r) => r.id !== id)].slice(
      0,
      MAX_INDEX,
    );
    await writeIndex(kitHome, next);
    return { ok: true, value: record };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot save run: ${message}` };
  }
}

/** List recent run proofs (newest first). */
export async function listRuns(
  options: { kitHome?: string; limit?: number } = {},
): Promise<WorkbenchResult<SavedRunSummary[]>> {
  const kitHome = options.kitHome ?? getKitHome();
  const limit = options.limit ?? 20;
  try {
    const runs = await readIndex(kitHome);
    return { ok: true, value: runs.slice(0, limit) };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot list runs: ${message}` };
  }
}

/** Load one run (metadata + transcript). */
export async function loadRun(
  id: string,
  options: { kitHome?: string } = {},
): Promise<WorkbenchResult<SavedRunRecord>> {
  const kitHome = options.kitHome ?? getKitHome();
  const safe = id.replace(/[^a-zA-Z0-9._-]/g, "");
  if (!safe || safe !== id) {
    return { ok: false, error: "Invalid run id." };
  }
  const metaPath = path.join(getRunsDir(kitHome), `${safe}.json`);
  try {
    const raw = await readFile(metaPath, "utf8");
    const record = JSON.parse(raw) as SavedRunRecord;
    try {
      record.transcript = await readFile(
        record.logPath ?? path.join(getRunsDir(kitHome), `${safe}.log`),
        "utf8",
      );
    } catch {
      record.transcript = record.transcript ?? "";
    }
    return { ok: true, value: record };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot load run: ${message}` };
  }
}

/** Best-effort cleanup helper for tests. */
export async function listRunFiles(
  kitHome: string = getKitHome(),
): Promise<string[]> {
  try {
    return await readdir(getRunsDir(kitHome));
  } catch {
    return [];
  }
}
