import { spawn, type ChildProcess } from "node:child_process";
import { access, mkdir, readFile, stat, unlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { getKitHome } from "../library/paths.js";
import type {
  LocalModel,
  OllamaDiscoveryOptions,
  OllamaPullOptions,
  OllamaServiceReport,
  OllamaServeOptions,
  OllamaServeState,
  WorkbenchResult,
} from "./types.js";
import { listOllamaModels } from "./workbench.js";

const DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434";
const DEFAULT_START_TIMEOUT_MS = 20_000;
const DEFAULT_POLL_MS = 400;

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

function runtimeDir(kitHome = getKitHome()): string {
  return path.join(kitHome, "runtime", "ollama");
}

function pidPath(kitHome = getKitHome()): string {
  return path.join(runtimeDir(kitHome), "serve.pid");
}

function metaPath(kitHome = getKitHome()): string {
  return path.join(runtimeDir(kitHome), "serve.json");
}

async function isFile(candidate: string): Promise<boolean> {
  try {
    return (await stat(candidate)).isFile();
  } catch {
    return false;
  }
}

/** Resolve `ollama` on PATH (Windows .exe aware). */
export async function findOllamaExecutable(): Promise<string | null> {
  const command = "ollama";
  const directories = (process.env.PATH ?? "")
    .split(path.delimiter)
    .filter(Boolean);
  const extensions =
    process.platform === "win32" ? [".EXE", ".CMD", ".BAT", ""] : [""];
  for (const directory of directories) {
    for (const extension of extensions) {
      const candidate = path.join(directory, `${command}${extension}`);
      try {
        await access(candidate);
        if (await isFile(candidate)) return candidate;
      } catch {
        // continue
      }
    }
  }
  return null;
}

async function readKitServeMeta(
  kitHome = getKitHome(),
): Promise<{ pid: number; host: string; startedAt: string } | null> {
  try {
    const raw = await readFile(metaPath(kitHome), "utf8");
    const data = JSON.parse(raw) as {
      pid?: unknown;
      host?: unknown;
      startedAt?: unknown;
    };
    if (
      typeof data.pid === "number" &&
      typeof data.host === "string" &&
      typeof data.startedAt === "string"
    ) {
      return { pid: data.pid, host: data.host, startedAt: data.startedAt };
    }
    return null;
  } catch {
    return null;
  }
}

function processAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

async function writeServeMeta(
  pid: number,
  host: string,
  kitHome = getKitHome(),
): Promise<void> {
  await mkdir(runtimeDir(kitHome), { recursive: true });
  const startedAt = new Date().toISOString();
  await writeFile(pidPath(kitHome), String(pid), "utf8");
  await writeFile(
    metaPath(kitHome),
    JSON.stringify({ pid, host, startedAt }, null, 2),
    "utf8",
  );
}

async function clearServeMeta(kitHome = getKitHome()): Promise<void> {
  for (const file of [pidPath(kitHome), metaPath(kitHome)]) {
    try {
      await unlink(file);
    } catch {
      // ignore
    }
  }
}

/**
 * Probe the local Ollama HTTP service + installed models.
 * Does not start anything.
 */
export async function probeOllamaService(
  options: OllamaDiscoveryOptions = {},
): Promise<OllamaServiceReport> {
  const base = ollamaBaseUrl(options.baseUrl);
  const executable = await findOllamaExecutable();
  const kitMeta = await readKitServeMeta();
  const kitOwned =
    kitMeta !== null &&
    processAlive(kitMeta.pid) &&
    (base.ok ? kitMeta.host === base.value : true);

  if (!base.ok) {
    return {
      state: "error",
      host: process.env.KIT_OLLAMA_HOST ?? DEFAULT_OLLAMA_URL,
      executable,
      kitOwned: false,
      models: [],
      detail: base.error,
    };
  }

  if (!executable) {
    return {
      state: "missing",
      host: base.value,
      executable: null,
      kitOwned: false,
      models: [],
      detail: "Ollama CLI not on PATH. Install from https://ollama.com",
    };
  }

  const models = await listOllamaModels(options);
  if (!models.ok) {
    return {
      state: "offline",
      host: base.value,
      executable,
      kitOwned: false,
      models: [],
      detail: models.error,
    };
  }

  return {
    state: "online" satisfies OllamaServeState,
    host: base.value,
    executable,
    kitOwned,
    models: models.value,
    detail:
      models.value.length === 0
        ? "Online · no models pulled yet"
        : `Online · ${models.value.length} model${models.value.length === 1 ? "" : "s"}`,
  };
}

/**
 * Start `ollama serve` in the background if the HTTP API is offline.
 * Tracks PID under ~/.kit/runtime/ollama so Kit can stop what it started.
 */
export async function startOllamaServe(
  options: OllamaServeOptions = {},
): Promise<WorkbenchResult<OllamaServiceReport>> {
  const timeoutMs = options.timeoutMs ?? DEFAULT_START_TIMEOUT_MS;
  const base = ollamaBaseUrl(options.baseUrl);
  if (!base.ok) return base;

  const already = await probeOllamaService(options);
  if (already.state === "online") {
    return {
      ok: true,
      value: {
        ...already,
        detail: already.detail.startsWith("Online")
          ? `${already.detail} (already running)`
          : already.detail,
      },
    };
  }
  if (already.state === "missing") {
    return { ok: false, error: already.detail };
  }

  const executable = already.executable ?? (await findOllamaExecutable());
  if (!executable) {
    return {
      ok: false,
      error: "Ollama CLI not on PATH. Install from https://ollama.com",
    };
  }

  options.onProgress?.("Starting ollama serve…");

  let child: ChildProcess;
  try {
    child = spawn(executable, ["serve"], {
      shell: false,
      windowsHide: true,
      detached: true,
      stdio: "ignore",
      env: {
        ...process.env,
        OLLAMA_HOST: base.value.replace(/^https?:\/\//, ""),
      },
    });
    child.unref();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot start Ollama: ${message}` };
  }

  if (typeof child.pid !== "number") {
    return { ok: false, error: "Ollama serve started without a PID." };
  }

  await writeServeMeta(child.pid, base.value);
  options.onProgress?.(`Ollama PID ${child.pid} · waiting for API…`);

  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    await sleep(DEFAULT_POLL_MS);
    const report = await probeOllamaService(options);
    if (report.state === "online") {
      options.onProgress?.("Ollama is online.");
      return {
        ok: true,
        value: {
          ...report,
          kitOwned: true,
          detail: `${report.detail} · kit-managed`,
        },
      };
    }
  }

  return {
    ok: false,
    error: `Ollama did not become ready within ${Math.round(timeoutMs / 1000)}s. Check ollama serve logs.`,
  };
}

/**
 * Stop a Kit-managed Ollama serve process only.
 * Will not kill a user-started Ollama instance.
 */
export async function stopOllamaServe(
  options: OllamaServeOptions = {},
): Promise<WorkbenchResult<OllamaServiceReport>> {
  const meta = await readKitServeMeta();
  if (!meta) {
    const report = await probeOllamaService(options);
    return {
      ok: false,
      error:
        report.state === "online"
          ? "Ollama is online but not Kit-managed. Stop it outside Kit."
          : "No Kit-managed Ollama process to stop.",
    };
  }

  if (!processAlive(meta.pid)) {
    await clearServeMeta();
    const report = await probeOllamaService(options);
    return {
      ok: true,
      value: {
        ...report,
        detail: "Cleared stale Kit Ollama PID.",
      },
    };
  }

  options.onProgress?.(`Stopping Ollama PID ${meta.pid}…`);
  try {
    if (process.platform === "win32") {
      spawn("taskkill", ["/PID", String(meta.pid), "/T", "/F"], {
        shell: false,
        windowsHide: true,
        stdio: "ignore",
      });
    } else {
      process.kill(meta.pid, "SIGTERM");
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot stop Ollama: ${message}` };
  }

  await sleep(500);
  if (processAlive(meta.pid) && process.platform !== "win32") {
    try {
      process.kill(meta.pid, "SIGKILL");
    } catch {
      // ignore
    }
  }

  await clearServeMeta();
  // Give the port a moment to free
  await sleep(300);
  const report = await probeOllamaService(options);
  return {
    ok: true,
    value: {
      ...report,
      kitOwned: false,
      detail:
        report.state === "offline"
          ? "Ollama stopped (Kit-managed)."
          : report.detail,
    },
  };
}

/**
 * Pull a model via `ollama pull`. Streams text progress to the caller.
 */
export async function pullOllamaModel(
  model: string,
  options: OllamaPullOptions = {},
): Promise<WorkbenchResult<{ model: string; models: LocalModel[] }>> {
  const name = model.trim();
  if (!name) {
    return { ok: false, error: "Model name is required (e.g. llama3.2)." };
  }
  const executable = await findOllamaExecutable();
  if (!executable) {
    return {
      ok: false,
      error: "Ollama CLI not on PATH. Install from https://ollama.com",
    };
  }

  // Ensure service is up before pull
  const probe = await probeOllamaService(options);
  if (probe.state === "offline" || probe.state === "error") {
    const started = await startOllamaServe(options);
    if (!started.ok) return started;
  } else if (probe.state === "missing") {
    return { ok: false, error: probe.detail };
  }

  options.onProgress?.(`Pulling ${name}…`);

  return await new Promise((resolve) => {
    const child = spawn(executable, ["pull", name], {
      shell: false,
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"],
      env: process.env,
    });
    let settled = false;
    const finish = (
      result: WorkbenchResult<{ model: string; models: LocalModel[] }>,
    ): void => {
      if (settled) return;
      settled = true;
      resolve(result);
    };

    const onAbort = () => {
      child.kill();
      finish({ ok: false, error: "Pull cancelled." });
    };
    if (options.signal?.aborted) {
      onAbort();
      return;
    }
    options.signal?.addEventListener("abort", onAbort, { once: true });

    const feed = (chunk: Buffer) => {
      const text = chunk.toString("utf8");
      if (text.trim()) options.onOutput?.(text);
    };
    child.stdout?.on("data", feed);
    child.stderr?.on("data", feed);

    child.on("error", (error) => {
      options.signal?.removeEventListener("abort", onAbort);
      finish({ ok: false, error: `Cannot pull model: ${error.message}` });
    });
    child.on("close", async (code) => {
      options.signal?.removeEventListener("abort", onAbort);
      if (code !== 0) {
        finish({
          ok: false,
          error: `ollama pull exited with code ${code ?? 1}.`,
        });
        return;
      }
      const models = await listOllamaModels(options);
      finish({
        ok: true,
        value: {
          model: name,
          models: models.ok ? models.value : [],
        },
      });
    });
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    const t = setTimeout(resolve, ms);
    t.unref?.();
  });
}
