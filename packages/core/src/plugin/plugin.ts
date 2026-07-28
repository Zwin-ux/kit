import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  access,
  mkdir,
  readFile,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import { getKitHome } from "../library/paths.js";
import { getPluginsIndexPath } from "./paths.js";
import type {
  AddPluginOptions,
  AddPluginReport,
  KitPluginManifest,
  KitPluginTask,
  PluginDoctorReport,
  PluginRegistry,
  PluginRegistryEntry,
  PluginResult,
  PluginRunReport,
  PluginTaskRunReport,
  RegisteredPlugin,
  RunPluginOptions,
} from "./types.js";

const MANIFEST_NAME = "kit.plugin.json";
const NAME_PATTERN = /^[a-z0-9][a-z0-9-]{1,62}$/;
const COMMAND_PATTERN = /^[A-Za-z0-9._-]+$/;
const TASK_TIMEOUT_MS = 30_000;

function digest(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function inside(root: string, candidate: string): boolean {
  const relative = path.relative(root, candidate);
  return (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) &&
      relative !== ".." &&
      !path.isAbsolute(relative))
  );
}

function stringArray(value: unknown): string[] | undefined {
  if (value === undefined) return undefined;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    return undefined;
  }
  return value;
}

function validateTasks(value: unknown): PluginResult<KitPluginTask[]> {
  if (!Array.isArray(value)) {
    return { ok: false, error: "Plugin tasks must be an array." };
  }
  const tasks: KitPluginTask[] = [];
  const names = new Set<string>();
  for (const item of value) {
    if (!item || typeof item !== "object" || Array.isArray(item)) {
      return { ok: false, error: "Each plugin task must be an object." };
    }
    const task = item as Record<string, unknown>;
    if (typeof task.name !== "string" || !NAME_PATTERN.test(task.name)) {
      return {
        ok: false,
        error: "Plugin task names must use lowercase letters, numbers, and hyphens.",
      };
    }
    if (names.has(task.name)) {
      return { ok: false, error: `Plugin task ${task.name} is duplicated.` };
    }
    if (typeof task.description !== "string" || !task.description.trim()) {
      return {
        ok: false,
        error: `Plugin task ${task.name} needs a description.`,
      };
    }
    const args = stringArray(task.args);
    if (!args || args.length === 0) {
      return {
        ok: false,
        error: `Plugin task ${task.name} needs at least one argument.`,
      };
    }
    if (task.access !== "read-only") {
      return {
        ok: false,
        error: `Plugin task ${task.name} access must be read-only.`,
      };
    }
    names.add(task.name);
    tasks.push({
      name: task.name,
      description: task.description.trim(),
      args,
      access: "read-only",
    });
  }
  return { ok: true, value: tasks };
}

function validateManifest(
  raw: unknown,
  root: string,
): PluginResult<KitPluginManifest> {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return { ok: false, error: "Plugin manifest must be a JSON object." };
  }
  const row = raw as Record<string, unknown>;
  if (row.schemaVersion !== 1) {
    return { ok: false, error: "Plugin schemaVersion must be 1." };
  }
  if (typeof row.name !== "string" || !NAME_PATTERN.test(row.name)) {
    return {
      ok: false,
      error: "Plugin name must use lowercase letters, numbers, and hyphens.",
    };
  }
  for (const field of ["displayName", "description", "version"] as const) {
    if (typeof row[field] !== "string" || !row[field].trim()) {
      return { ok: false, error: `Plugin ${field} must be a non-empty string.` };
    }
  }
  if (
    typeof row.command !== "string" ||
    !COMMAND_PATTERN.test(row.command)
  ) {
    return {
      ok: false,
      error: "Plugin command must be one executable name without a path.",
    };
  }

  const defaultArgs = stringArray(row.defaultArgs);
  const versionArgs = stringArray(row.versionArgs);
  const healthArgs = stringArray(row.healthArgs);
  const tasks =
    row.tasks === undefined ? undefined : validateTasks(row.tasks);
  if (row.defaultArgs !== undefined && !defaultArgs) {
    return { ok: false, error: "Plugin defaultArgs must be a string array." };
  }
  if (row.versionArgs !== undefined && !versionArgs) {
    return { ok: false, error: "Plugin versionArgs must be a string array." };
  }
  if (row.healthArgs !== undefined && !healthArgs) {
    return { ok: false, error: "Plugin healthArgs must be a string array." };
  }
  if (tasks && !tasks.ok) return tasks;

  let localExecutables:
    | KitPluginManifest["localExecutables"]
    | undefined;
  if (row.localExecutables !== undefined) {
    if (
      !row.localExecutables ||
      typeof row.localExecutables !== "object" ||
      Array.isArray(row.localExecutables)
    ) {
      return {
        ok: false,
        error: "Plugin localExecutables must be an object.",
      };
    }
    localExecutables = {};
    for (const [platform, value] of Object.entries(
      row.localExecutables as Record<string, unknown>,
    )) {
      if (!["win32", "darwin", "linux", "default"].includes(platform)) {
        return {
          ok: false,
          error: `Plugin localExecutables has an unknown platform: ${platform}.`,
        };
      }
      if (typeof value !== "string" || !value.trim()) {
        return {
          ok: false,
          error: `Plugin executable for ${platform} must be a path.`,
        };
      }
      const resolved = path.resolve(root, value);
      if (path.isAbsolute(value) || !inside(root, resolved)) {
        return {
          ok: false,
          error: `Plugin executable for ${platform} must stay inside the plugin root.`,
        };
      }
      localExecutables[platform as keyof typeof localExecutables] = value;
    }
  }

  let safety: KitPluginManifest["safety"] | undefined;
  if (row.safety !== undefined) {
    if (!row.safety || typeof row.safety !== "object") {
      return { ok: false, error: "Plugin safety must be an object." };
    }
    const safetyRow = row.safety as Record<string, unknown>;
    if (typeof safetyRow.summary !== "string" || !safetyRow.summary.trim()) {
      return {
        ok: false,
        error: "Plugin safety summary must be a non-empty string.",
      };
    }
    safety = { summary: safetyRow.summary };
    if (
      safetyRow.confirmationToken !== undefined &&
      typeof safetyRow.confirmationToken !== "string"
    ) {
      return {
        ok: false,
        error: "Plugin confirmationToken must be a string.",
      };
    }
    if (typeof safetyRow.confirmationToken === "string") {
      safety.confirmationToken = safetyRow.confirmationToken;
    }
  }

  return {
    ok: true,
    value: {
      schemaVersion: 1,
      name: row.name,
      displayName: String(row.displayName).trim(),
      description: String(row.description).trim(),
      version: String(row.version).trim(),
      command: row.command,
      ...(defaultArgs ? { defaultArgs } : {}),
      ...(localExecutables ? { localExecutables } : {}),
      ...(versionArgs ? { versionArgs } : {}),
      ...(healthArgs ? { healthArgs } : {}),
      ...(tasks?.ok ? { tasks: tasks.value } : {}),
      ...(safety ? { safety } : {}),
    },
  };
}

async function readRegistry(kitHome: string): Promise<PluginRegistry> {
  try {
    const raw = JSON.parse(
      await readFile(getPluginsIndexPath(kitHome), "utf8"),
    ) as unknown;
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
      return { version: 1, plugins: {} };
    }
    const row = raw as Record<string, unknown>;
    if (
      row.version !== 1 ||
      !row.plugins ||
      typeof row.plugins !== "object" ||
      Array.isArray(row.plugins)
    ) {
      return { version: 1, plugins: {} };
    }
    return row as unknown as PluginRegistry;
  } catch (error) {
    const code =
      error && typeof error === "object" && "code" in error
        ? String((error as { code: unknown }).code)
        : undefined;
    if (code === "ENOENT") return { version: 1, plugins: {} };
    throw error;
  }
}

async function writeRegistry(
  registry: PluginRegistry,
  kitHome: string,
): Promise<void> {
  await mkdir(kitHome, { recursive: true });
  await writeFile(
    getPluginsIndexPath(kitHome),
    `${JSON.stringify(registry, null, 2)}\n`,
    "utf8",
  );
}

async function loadManifest(
  source: string,
): Promise<
  PluginResult<{
    root: string;
    path: string;
    raw: string;
    manifest: KitPluginManifest;
  }>
> {
  const root = path.resolve(source);
  const manifestPath = path.join(root, MANIFEST_NAME);
  try {
    const raw = await readFile(manifestPath, "utf8");
    const parsed = validateManifest(JSON.parse(raw) as unknown, root);
    if (!parsed.ok) return parsed;
    return {
      ok: true,
      value: { root, path: manifestPath, raw, manifest: parsed.value },
    };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      error: `Cannot read ${manifestPath}: ${detail}`,
    };
  }
}

async function fileExists(candidate: string): Promise<boolean> {
  try {
    return (await stat(candidate)).isFile();
  } catch {
    return false;
  }
}

async function findOnPath(command: string): Promise<string | null> {
  const directories = (process.env.PATH ?? "")
    .split(path.delimiter)
    .filter(Boolean);
  const extensions =
    process.platform === "win32"
      ? (process.env.PATHEXT ?? ".EXE;.CMD;.BAT")
          .split(";")
          .filter(Boolean)
      : [""];
  for (const directory of directories) {
    for (const extension of extensions) {
      const candidate = path.join(
        directory,
        process.platform === "win32" ? `${command}${extension}` : command,
      );
      try {
        await access(candidate);
        return candidate;
      } catch {
        // Continue to the next path entry.
      }
    }
  }
  return null;
}

async function resolveExecutable(
  root: string,
  manifest: KitPluginManifest,
): Promise<{
  executable: string | null;
  source: "local" | "path" | "missing";
}> {
  const platform =
    process.platform === "win32" ||
    process.platform === "darwin" ||
    process.platform === "linux"
      ? process.platform
      : "default";
  const relative =
    manifest.localExecutables?.[platform] ??
    manifest.localExecutables?.default;
  if (relative) {
    const local = path.resolve(root, relative);
    if (await fileExists(local)) {
      return { executable: local, source: "local" };
    }
  }
  const fromPath = await findOnPath(manifest.command);
  if (fromPath) return { executable: fromPath, source: "path" };
  return { executable: null, source: "missing" };
}

export async function addPlugin(
  source: string,
  options: AddPluginOptions = {},
): Promise<PluginResult<AddPluginReport>> {
  const loaded = await loadManifest(source);
  if (!loaded.ok) return loaded;
  const kitHome = options.kitHome ?? getKitHome();
  const registry = await readRegistry(kitHome);
  const current = registry.plugins[loaded.value.manifest.name];
  if (
    current &&
    current.root !== loaded.value.root &&
    options.force !== true
  ) {
    return {
      ok: false,
      error: `Plugin ${loaded.value.manifest.name} already points to ${current.root}. Use --force to replace it.`,
    };
  }

  const resolved = await resolveExecutable(
    loaded.value.root,
    loaded.value.manifest,
  );
  if (options.write === true) {
    registry.plugins[loaded.value.manifest.name] = {
      name: loaded.value.manifest.name,
      root: loaded.value.root,
      manifestPath: loaded.value.path,
      manifestDigest: digest(loaded.value.raw),
      addedAt: new Date().toISOString(),
    };
    await writeRegistry(registry, kitHome);
  }

  return {
    ok: true,
    value: {
      dryRun: options.write !== true,
      manifest: loaded.value.manifest,
      root: loaded.value.root,
      executable: resolved.executable,
      executableSource: resolved.source,
    },
  };
}

export async function listPlugins(
  kitHome: string = getKitHome(),
): Promise<PluginResult<RegisteredPlugin[]>> {
  const registry = await readRegistry(kitHome);
  const plugins: RegisteredPlugin[] = [];
  for (const entry of Object.values(registry.plugins).sort((left, right) =>
    left.name.localeCompare(right.name),
  )) {
    const loaded = await loadManifest(entry.root);
    if (!loaded.ok) {
      return {
        ok: false,
        error: `Plugin ${entry.name} is invalid: ${loaded.error}`,
      };
    }
    plugins.push({ manifest: loaded.value.manifest, entry });
  }
  return { ok: true, value: plugins };
}

async function registeredPlugin(
  name: string,
  kitHome: string,
): Promise<
  PluginResult<{
    manifest: KitPluginManifest;
    entry: PluginRegistryEntry;
    raw: string;
  }>
> {
  const registry = await readRegistry(kitHome);
  const entry = registry.plugins[name];
  if (!entry) {
    return {
      ok: false,
      error: `Plugin ${name} is not registered. Run kit plugin add <path> --write.`,
    };
  }
  const loaded = await loadManifest(entry.root);
  if (!loaded.ok) return loaded;
  return {
    ok: true,
    value: { manifest: loaded.value.manifest, entry, raw: loaded.value.raw },
  };
}

export async function doctorPlugin(
  name: string,
  kitHome: string = getKitHome(),
): Promise<PluginResult<PluginDoctorReport>> {
  const plugin = await registeredPlugin(name, kitHome);
  if (!plugin.ok) return plugin;
  const resolved = await resolveExecutable(
    plugin.value.entry.root,
    plugin.value.manifest,
  );
  const manifestChanged =
    digest(plugin.value.raw) !== plugin.value.entry.manifestDigest;
  return {
    ok: true,
    value: {
      name,
      ready: resolved.executable !== null && !manifestChanged,
      manifestChanged,
      executable: resolved.executable,
      executableSource: resolved.source,
      healthArgs: plugin.value.manifest.healthArgs ?? [],
      tasks: plugin.value.manifest.tasks ?? [],
      ...(plugin.value.manifest.safety
        ? { safetySummary: plugin.value.manifest.safety.summary }
        : {}),
      ...(plugin.value.manifest.safety?.confirmationToken
        ? {
            confirmationToken:
              plugin.value.manifest.safety.confirmationToken,
          }
        : {}),
    },
  };
}

export async function removePlugin(
  name: string,
  options: { kitHome?: string; write?: boolean } = {},
): Promise<PluginResult<{ name: string; dryRun: boolean }>> {
  const kitHome = options.kitHome ?? getKitHome();
  const registry = await readRegistry(kitHome);
  if (!registry.plugins[name]) {
    return { ok: false, error: `Plugin ${name} is not registered.` };
  }
  if (options.write === true) {
    delete registry.plugins[name];
    await writeRegistry(registry, kitHome);
  }
  return { ok: true, value: { name, dryRun: options.write !== true } };
}

export async function runPlugin(
  name: string,
  args: string[],
  options: RunPluginOptions = {},
): Promise<PluginResult<PluginRunReport>> {
  const kitHome = options.kitHome ?? getKitHome();
  const plugin = await registeredPlugin(name, kitHome);
  if (!plugin.ok) return plugin;
  if (digest(plugin.value.raw) !== plugin.value.entry.manifestDigest) {
    return {
      ok: false,
      error: `Plugin ${name} changed after registration. Review it, then run kit plugin add ${plugin.value.entry.root} --write.`,
    };
  }
  const resolved = await resolveExecutable(
    plugin.value.entry.root,
    plugin.value.manifest,
  );
  if (!resolved.executable) {
    return {
      ok: false,
      error: `Plugin ${name} has no executable. Build it or add ${plugin.value.manifest.command} to PATH.`,
    };
  }

  const commandArgs = [
    ...(plugin.value.manifest.defaultArgs ?? []),
    ...args,
  ];
  return runResolvedPlugin(
    name,
    resolved.executable,
    plugin.value.entry.root,
    commandArgs,
    options,
  );
}

async function runResolvedPlugin(
  name: string,
  executable: string,
  root: string,
  args: string[],
  options: RunPluginOptions,
): Promise<PluginResult<PluginRunReport>> {
  const stdio = options.stdio ?? "inherit";
  return await new Promise((resolve) => {
    const child = spawn(executable, args, {
      cwd: root,
      shell: false,
      stdio,
      windowsHide: true,
    });
    let stdout = "";
    let stderr = "";
    let timedOut = false;
    const timer =
      options.timeoutMs === undefined
        ? undefined
        : setTimeout(() => {
            timedOut = true;
            child.kill();
          }, options.timeoutMs);
    timer?.unref();
    if (child.stdout) {
      child.stdout.setEncoding("utf8");
      child.stdout.on("data", (chunk: string) => {
        stdout += chunk;
      });
    }
    if (child.stderr) {
      child.stderr.setEncoding("utf8");
      child.stderr.on("data", (chunk: string) => {
        stderr += chunk;
      });
    }
    child.on("error", (error) => {
      if (timer) clearTimeout(timer);
      resolve({ ok: false, error: `Cannot run plugin ${name}: ${error.message}` });
    });
    child.on("close", (code) => {
      if (timer) clearTimeout(timer);
      if (timedOut) {
        resolve({
          ok: false,
          error: `Plugin ${name} timed out after ${options.timeoutMs} ms.`,
        });
        return;
      }
      resolve({
        ok: true,
        value: {
          name,
          command: executable,
          args,
          exitCode: code ?? 1,
          stdout,
          stderr,
        },
      });
    });
  });
}

export async function runPluginTask(
  name: string,
  taskName: string,
  options: RunPluginOptions = {},
): Promise<PluginResult<PluginTaskRunReport>> {
  const kitHome = options.kitHome ?? getKitHome();
  const plugin = await registeredPlugin(name, kitHome);
  if (!plugin.ok) return plugin;
  if (digest(plugin.value.raw) !== plugin.value.entry.manifestDigest) {
    return {
      ok: false,
      error: `Plugin ${name} changed after registration. Review it, then run kit plugin add ${plugin.value.entry.root} --write.`,
    };
  }
  const task = plugin.value.manifest.tasks?.find(
    (candidate) => candidate.name === taskName,
  );
  if (!task) {
    const names = plugin.value.manifest.tasks?.map((item) => item.name) ?? [];
    return {
      ok: false,
      error:
        names.length > 0
          ? `Plugin ${name} has no task named ${taskName}. Available tasks: ${names.join(", ")}.`
          : `Plugin ${name} has no fixed tasks.`,
    };
  }
  const resolved = await resolveExecutable(
    plugin.value.entry.root,
    plugin.value.manifest,
  );
  if (!resolved.executable) {
    return {
      ok: false,
      error: `Plugin ${name} has no executable. Build it or add ${plugin.value.manifest.command} to PATH.`,
    };
  }
  const run = await runResolvedPlugin(
    name,
    resolved.executable,
    plugin.value.entry.root,
    [...(plugin.value.manifest.defaultArgs ?? []), ...task.args],
    {
      ...options,
      timeoutMs: options.timeoutMs ?? TASK_TIMEOUT_MS,
    },
  );
  if (!run.ok) return run;
  return {
    ok: true,
    value: {
      ...run.value,
      task: task.name,
      access: task.access,
    },
  };
}
