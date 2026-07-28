import { spawn } from "node:child_process";
import { access, stat } from "node:fs/promises";
import path from "node:path";

import type {
  CodingInvocation,
  CodingJobReport,
  CodingJobRequest,
  CodingRunnerId,
  CodingRunnerStatus,
  RunnerCommand,
  RunCodingJobOptions,
  WorkbenchResult,
} from "./types.js";

const DEFAULT_TIMEOUT_MS = 10 * 60_000;
const DEFAULT_MAX_OUTPUT_BYTES = 256 * 1024;
const MAX_PROMPT_LENGTH = 8_000;

const RUNNERS: Record<
  CodingRunnerId,
  { label: string; command: string }
> = {
  codex: { label: "Codex", command: "codex" },
  "claude-code": { label: "Claude Code", command: "claude" },
  "grok-build": { label: "Grok Build", command: "grok" },
};

async function isFile(candidate: string): Promise<boolean> {
  try {
    return (await stat(candidate)).isFile();
  } catch {
    return false;
  }
}

async function findExecutable(command: string): Promise<string | null> {
  const directories = (process.env.PATH ?? "")
    .split(path.delimiter)
    .filter(Boolean);
  const extensions =
    process.platform === "win32"
      ? [".EXE", ".COM"]
      : [""];
  for (const directory of directories) {
    for (const extension of extensions) {
      const candidate = path.join(directory, `${command}${extension}`);
      try {
        await access(candidate);
        if (await isFile(candidate)) return candidate;
      } catch {
        // Continue.
      }
    }
  }
  return null;
}

async function resolveRunnerCommand(
  runner: CodingRunnerId,
): Promise<RunnerCommand | null> {
  const descriptor = RUNNERS[runner];
  const executable = await findExecutable(descriptor.command);
  if (executable) return { executable, prefixArgs: [] };

  if (process.platform === "win32" && runner === "codex") {
    for (const directory of (process.env.PATH ?? "")
      .split(path.delimiter)
      .filter(Boolean)) {
      const script = path.join(
        directory,
        "node_modules",
        "@openai",
        "codex",
        "bin",
        "codex.js",
      );
      if (await isFile(script)) {
        return { executable: process.execPath, prefixArgs: [script] };
      }
    }
  }
  return null;
}

export async function detectCodingRunners(): Promise<CodingRunnerStatus[]> {
  const rows: CodingRunnerStatus[] = [];
  for (const id of Object.keys(RUNNERS) as CodingRunnerId[]) {
    const command = await resolveRunnerCommand(id);
    rows.push({
      id,
      label: RUNNERS[id].label,
      available: command !== null,
      executable: command?.executable ?? null,
    });
  }
  return rows;
}

function runnerArgs(
  request: CodingJobRequest,
  command: RunnerCommand,
): string[] {
  const { mode, projectDir, prompt } = request;
  if (request.runner === "codex") {
    return [
      ...command.prefixArgs,
      "exec",
      "--cd",
      projectDir,
      "--sandbox",
      mode === "inspect" ? "read-only" : "workspace-write",
      "--ephemeral",
      "--color",
      "never",
      prompt,
    ];
  }
  if (request.runner === "claude-code") {
    return [
      ...command.prefixArgs,
      "--print",
      "--permission-mode",
      mode === "inspect" ? "plan" : "acceptEdits",
      "--output-format",
      "text",
      prompt,
    ];
  }
  return [
    ...command.prefixArgs,
    "--single",
    prompt,
    "--cwd",
    projectDir,
    "--permission-mode",
    mode === "inspect" ? "plan" : "acceptEdits",
    "--output-format",
    "plain",
    "--no-subagents",
    "--disable-web-search",
  ];
}

export async function planCodingJob(
  request: CodingJobRequest,
  commandOverride?: RunnerCommand,
): Promise<WorkbenchResult<CodingInvocation>> {
  const prompt = request.prompt.trim();
  if (!prompt) {
    return { ok: false, error: "Workbench needs a job prompt." };
  }
  if (prompt.length > MAX_PROMPT_LENGTH) {
    return {
      ok: false,
      error: `Workbench prompt must be ${MAX_PROMPT_LENGTH} characters or fewer.`,
    };
  }
  if (request.mode === "build" && request.confirmBuild !== true) {
    return {
      ok: false,
      error: "Build mode needs explicit confirmation.",
    };
  }
  try {
    if (!(await stat(request.projectDir)).isDirectory()) {
      return { ok: false, error: "Workbench project path is not a directory." };
    }
  } catch {
    return {
      ok: false,
      error: `Workbench project path does not exist: ${request.projectDir}`,
    };
  }
  const command =
    commandOverride ?? (await resolveRunnerCommand(request.runner));
  if (!command) {
    return {
      ok: false,
      error: `${RUNNERS[request.runner].label} is not installed or is not on PATH.`,
    };
  }
  const normalizedRequest = { ...request, prompt };
  return {
    ok: true,
    value: {
      runner: request.runner,
      mode: request.mode,
      executable: command.executable,
      args: runnerArgs(normalizedRequest, command),
      cwd: path.resolve(request.projectDir),
    },
  };
}

function appendBounded(
  parts: Buffer[],
  chunk: Buffer,
  state: { bytes: number; truncated: boolean },
  limit: number,
): void {
  if (state.bytes >= limit) {
    state.truncated = true;
    return;
  }
  const remaining = limit - state.bytes;
  if (chunk.length > remaining) {
    parts.push(chunk.subarray(0, remaining));
    state.bytes = limit;
    state.truncated = true;
    return;
  }
  parts.push(chunk);
  state.bytes += chunk.length;
}

export async function runCodingJob(
  request: CodingJobRequest,
  options: RunCodingJobOptions = {},
): Promise<WorkbenchResult<CodingJobReport>> {
  const planned = await planCodingJob(request, options.commandOverride);
  if (!planned.ok) return planned;

  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const maxOutputBytes =
    options.maxOutputBytes ?? DEFAULT_MAX_OUTPUT_BYTES;
  if (timeoutMs <= 0) {
    return { ok: false, error: "Workbench timeout must be greater than zero." };
  }
  if (maxOutputBytes <= 0) {
    return {
      ok: false,
      error: "Workbench output limit must be greater than zero.",
    };
  }

  return await new Promise((resolve) => {
    const started = Date.now();
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    const outputState = { bytes: 0, truncated: false };
    let timedOut = false;
    let settled = false;

    const finish = (result: WorkbenchResult<CodingJobReport>): void => {
      if (settled) return;
      settled = true;
      resolve(result);
    };

    const child = spawn(planned.value.executable, planned.value.args, {
      cwd: planned.value.cwd,
      shell: false,
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill();
    }, timeoutMs);
    timer.unref();

    child.stdout.on("data", (chunk: Buffer) => {
      appendBounded(stdout, chunk, outputState, maxOutputBytes);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      appendBounded(stderr, chunk, outputState, maxOutputBytes);
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      finish({
        ok: false,
        error: `Cannot start ${RUNNERS[request.runner].label}: ${error.message}`,
      });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      finish({
        ok: true,
        value: {
          ...planned.value,
          exitCode: code ?? 1,
          stdout: Buffer.concat(stdout).toString("utf8"),
          stderr: Buffer.concat(stderr).toString("utf8"),
          durationMs: Date.now() - started,
          timedOut,
          truncated: outputState.truncated,
        },
      });
    });
  });
}
