import { spawn } from "node:child_process";
import { access, mkdir, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import type {
  CodingInvocation,
  CodingJobReport,
  CodingJobRequest,
  CodingRunnerId,
  CodingRunnerStatus,
  LocalModel,
  OllamaDiscoveryOptions,
  RunnerCommand,
  RunCodingJobOptions,
  WorkbenchResult,
} from "./types.js";

const DEFAULT_TIMEOUT_MS = 10 * 60_000;
const DEFAULT_MAX_OUTPUT_BYTES = 256 * 1024;
const MAX_PROMPT_LENGTH = 8_000;
const DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_TIMEOUT_MS = 800;

const RUNNERS: Record<
  CodingRunnerId,
  { label: string; command: string }
> = {
  codex: { label: "Codex", command: "codex" },
  "claude-code": { label: "Claude Code", command: "claude" },
  "grok-build": { label: "Grok Build", command: "grok" },
  ollama: { label: "Ollama · Codex", command: "ollama" },
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

function ollamaBaseUrl(value?: string): WorkbenchResult<string> {
  try {
    const parsed = new URL(
      value ?? process.env.KIT_OLLAMA_HOST ?? DEFAULT_OLLAMA_URL,
    );
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return { ok: false, error: "Ollama host must use HTTP or HTTPS." };
    }
    if (parsed.username || parsed.password) {
      return { ok: false, error: "Ollama host must not include credentials." };
    }
    return { ok: true, value: parsed.origin };
  } catch {
    return { ok: false, error: "Ollama host is not a valid URL." };
  }
}

export async function listOllamaModels(
  options: OllamaDiscoveryOptions = {},
): Promise<WorkbenchResult<LocalModel[]>> {
  const baseUrl = ollamaBaseUrl(options.baseUrl);
  if (!baseUrl.ok) return baseUrl;

  const controller = new AbortController();
  const timeout = setTimeout(
    () => controller.abort(),
    options.timeoutMs ?? DEFAULT_OLLAMA_TIMEOUT_MS,
  );
  timeout.unref();
  try {
    const response = await (options.fetchImpl ?? fetch)(
      `${baseUrl.value}/api/tags`,
      {
        method: "GET",
        headers: { accept: "application/json" },
        signal: controller.signal,
      },
    );
    if (!response.ok) {
      return {
        ok: false,
        error: `Ollama returned HTTP ${response.status}.`,
      };
    }
    const data = (await response.json()) as {
      models?: Array<{
        name?: unknown;
        size?: unknown;
        details?: {
          parameter_size?: unknown;
          quantization_level?: unknown;
        };
      }>;
    };
    const models = (data.models ?? [])
      .filter(
        (model): model is {
          name: string;
          size: number;
          details?: {
            parameter_size?: unknown;
            quantization_level?: unknown;
          };
        } => typeof model.name === "string" && typeof model.size === "number",
      )
      .map((model) => ({
        name: model.name,
        size: model.size,
        ...(typeof model.details?.parameter_size === "string"
          ? { parameterSize: model.details.parameter_size }
          : {}),
        ...(typeof model.details?.quantization_level === "string"
          ? { quantization: model.details.quantization_level }
          : {}),
      }));
    return { ok: true, value: models };
  } catch (error) {
    const message =
      error instanceof Error && error.name !== "AbortError"
        ? error.message
        : "connection timed out";
    return { ok: false, error: `Cannot reach Ollama: ${message}.` };
  } finally {
    clearTimeout(timeout);
  }
}

export async function detectCodingRunners(
  ollamaOptions: OllamaDiscoveryOptions = {},
): Promise<CodingRunnerStatus[]> {
  const rows: CodingRunnerStatus[] = [];
  for (const id of ["codex", "claude-code", "grok-build"] as const) {
    const command = await resolveRunnerCommand(id);
    rows.push({
      id,
      label: RUNNERS[id].label,
      available: command !== null,
      executable: command?.executable ?? null,
    });
  }
  const [command, codexCommand, models] = await Promise.all([
    resolveRunnerCommand("ollama"),
    resolveRunnerCommand("codex"),
    listOllamaModels(ollamaOptions),
  ]);
  // Ollama runner is "available" when CLI + Codex exist and the service is up.
  // Models can be empty (user still needs to pull); kit can start serve for them.
  const serviceUp = models.ok;
  const available =
    command !== null && codexCommand !== null && serviceUp;
  rows.push({
    id: "ollama",
    label: RUNNERS.ollama.label,
    available,
    executable: command?.executable ?? null,
    detail:
      command === null
        ? "needs Ollama CLI"
        : codexCommand === null
          ? "needs Codex"
          : !models.ok
            ? "offline · press o to start"
            : models.value.length === 0
              ? "online · no models · pull one"
              : `${models.value.length} local model${models.value.length === 1 ? "" : "s"}`,
    ...(models.ok ? { models: models.value } : {}),
  });
  return rows;
}

function runnerArgs(
  request: CodingJobRequest,
  command: RunnerCommand,
): string[] {
  const { mode, projectDir, prompt } = request;
  if (request.runner === "ollama") {
    return [
      ...command.prefixArgs,
      "launch",
      "codex",
      "--model",
      request.model ?? "",
      "--yes",
      "--",
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
  if (request.runner === "ollama" && !request.model?.trim()) {
    return {
      ok: false,
      error: "Choose an installed Ollama model before the run.",
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
      ...(request.model ? { model: request.model } : {}),
    },
  };
}

function appendBounded(
  parts: Buffer[],
  chunk: Buffer,
  state: { bytes: number; truncated: boolean },
  limit: number,
): Buffer | null {
  if (state.bytes >= limit) {
    state.truncated = true;
    return null;
  }
  const remaining = limit - state.bytes;
  if (chunk.length > remaining) {
    const accepted = chunk.subarray(0, remaining);
    parts.push(accepted);
    state.bytes = limit;
    state.truncated = true;
    return accepted;
  }
  parts.push(chunk);
  state.bytes += chunk.length;
  return chunk;
}

async function isolatedOllamaCodexHome(): Promise<string> {
  const kitHome = process.env.KIT_HOME
    ? path.resolve(process.env.KIT_HOME)
    : path.join(os.homedir(), ".kit");
  const directory =
    process.env.KIT_OLLAMA_CODEX_HOME ??
    path.join(kitHome, "runtime", "codex-ollama");
  await mkdir(directory, { recursive: true });
  return directory;
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
  let childEnvironment = process.env;
  if (request.runner === "ollama") {
    const host = ollamaBaseUrl();
    if (!host.ok) return host;
    childEnvironment = {
      ...process.env,
      CODEX_HOME: await isolatedOllamaCodexHome(),
      OLLAMA_HOST: host.value,
    };
  }

  return await new Promise((resolve) => {
    const started = Date.now();
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    const outputState = { bytes: 0, truncated: false };
    let timedOut = false;
    let cancelled = false;
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
      env: childEnvironment,
    });
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill();
    }, timeoutMs);
    timer.unref();
    const onAbort = () => {
      cancelled = true;
      child.kill();
    };
    if (options.signal?.aborted) {
      onAbort();
    } else {
      options.signal?.addEventListener("abort", onAbort, { once: true });
    }

    child.stdout.on("data", (chunk: Buffer) => {
      const accepted = appendBounded(
        stdout,
        chunk,
        outputState,
        maxOutputBytes,
      );
      if (accepted) options.onOutput?.(accepted.toString("utf8"), "stdout");
    });
    child.stderr.on("data", (chunk: Buffer) => {
      const accepted = appendBounded(
        stderr,
        chunk,
        outputState,
        maxOutputBytes,
      );
      if (accepted) options.onOutput?.(accepted.toString("utf8"), "stderr");
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      options.signal?.removeEventListener("abort", onAbort);
      finish({
        ok: false,
        error: `Cannot start ${RUNNERS[request.runner].label}: ${error.message}`,
      });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      options.signal?.removeEventListener("abort", onAbort);
      finish({
        ok: true,
        value: {
          ...planned.value,
          exitCode: code ?? 1,
          stdout: Buffer.concat(stdout).toString("utf8"),
          stderr: Buffer.concat(stderr).toString("utf8"),
          durationMs: Date.now() - started,
          timedOut,
          cancelled,
          truncated: outputState.truncated,
        },
      });
    });
  });
}
