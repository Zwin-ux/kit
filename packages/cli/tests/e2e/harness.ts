/**
 * E2E harness for the compiled Kit CLI.
 *
 * Every test runs `node packages/cli/dist/bin.js` inside an isolated sandbox:
 * HOME, USERPROFILE, and KIT_HOME all point into a fresh temp directory, so
 * no command can ever touch the developer's real home or agent config dirs.
 *
 * On assertion failure, expectExit() reports the full command, exit code,
 * stdout, stderr, and sandbox path (per tests/e2e/README.md).
 */
import { execFile } from "node:child_process";
import { access, mkdir, mkdtemp, readdir, realpath, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));

/** Monorepo root (tests/e2e → tests → cli → packages → root). */
export const REPO_ROOT = path.resolve(HERE, "..", "..", "..", "..");

/** The compiled CLI under test. E2E must never import CLI source. */
export const CLI_BIN = path.join(REPO_ROOT, "packages", "cli", "dist", "bin.js");

/**
 * Registry URL used inside sandboxes: an unroutable local port so
 * `doctor` gets a fast connection-refused warning instead of real network.
 */
export const SANDBOX_REGISTRY_URL = "http://127.0.0.1:1";

const RUN_TIMEOUT_MS = 90_000;

export interface RunResult {
  /** Full command line for diagnostics. */
  command: string;
  args: string[];
  cwd: string;
  code: number;
  stdout: string;
  stderr: string;
  sandboxRoot: string;
  timedOut: boolean;
  /** Human-readable failure report: command, exit code, stdout, stderr, sandbox path. */
  describe(): string;
}

export interface RunOptions {
  /** Working directory. Default: sandbox.projectDir. */
  cwd?: string;
  /** Extra env overrides on top of the sandbox env. */
  env?: Record<string, string | undefined>;
}

export interface Sandbox {
  /** Temp root that owns everything below. */
  root: string;
  /** Fake user home (HOME / USERPROFILE for spawned CLI). */
  home: string;
  /** KIT_HOME inside the fake home. */
  kitHome: string;
  /** Default project directory (cwd for spawned CLI). */
  projectDir: string;
  /** Personal-scope harness skill roots inside the fake home. */
  claudeSkills: string;
  codexSkills: string;
  grokSkills: string;
  /** Env the CLI child processes run with. */
  env: NodeJS.ProcessEnv;
  runKit(args: string[], options?: RunOptions): Promise<RunResult>;
  cleanup(): Promise<void>;
}

export interface SandboxOptions {
  /** Name for the project directory (e.g. one with spaces/unicode). */
  projectDirName?: string;
}

/** Create a fresh isolated sandbox under the OS temp dir. */
export async function createSandbox(
  options: SandboxOptions = {},
): Promise<Sandbox> {
  await assertCliBuilt();

  // realpath: macOS os.tmpdir() is a symlink (/var/folders -> /private/var),
  // and the CLI resolves cwd to the real path, so HOME must match it or the
  // home-guard comparison never fires.
  const root = await realpath(await mkdtemp(path.join(os.tmpdir(), "kit-e2e-")));
  const home = path.join(root, "home");
  const kitHome = path.join(home, ".kit");
  const projectDir = path.join(root, options.projectDirName ?? "project");

  await mkdir(home, { recursive: true });
  await mkdir(projectDir, { recursive: true });

  const env = sandboxEnv({ home, kitHome });

  const sandbox: Sandbox = {
    root,
    home,
    kitHome,
    projectDir,
    claudeSkills: path.join(home, ".claude", "skills"),
    codexSkills: path.join(home, ".codex", "skills"),
    grokSkills: path.join(home, ".grok", "skills"),
    env,
    runKit: (args, runOptions) =>
      runKit({
        args,
        cwd: runOptions?.cwd ?? projectDir,
        env: { ...env, ...runOptions?.env },
        sandboxRoot: root,
      }),
    cleanup: async () => {
      await rm(root, { recursive: true, force: true, maxRetries: 5 });
    },
  };

  return sandbox;
}

function sandboxEnv(input: { home: string; kitHome: string }): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };

  // Kill every route back to the real user profile / kit state.
  delete env.HOMEDRIVE;
  delete env.HOMEPATH;
  delete env.KIT_PACKS;
  delete env.KIT_SKILLS;
  delete env.KIT_ASSETS;
  delete env.KIT_REGISTRY_URL;

  env.HOME = input.home; // POSIX os.homedir()
  env.USERPROFILE = input.home; // Windows os.homedir()
  env.KIT_HOME = input.kitHome;
  env.KIT_REGISTRY_URL = SANDBOX_REGISTRY_URL;
  env.NO_COLOR = "1";
  return env;
}

async function runKit(input: {
  args: string[];
  cwd: string;
  env: NodeJS.ProcessEnv;
  sandboxRoot: string;
}): Promise<RunResult> {
  const { args, cwd, env, sandboxRoot } = input;

  return await new Promise<RunResult>((resolve) => {
    execFile(
      process.execPath,
      [CLI_BIN, ...args],
      {
        cwd,
        env,
        encoding: "utf8",
        timeout: RUN_TIMEOUT_MS,
        maxBuffer: 32 * 1024 * 1024,
        windowsHide: true,
      },
      (error, stdout, stderr) => {
        const timedOut = Boolean(
          error && "killed" in error && error.killed === true,
        );
        let code = 0;
        if (error) {
          const raw = (error as NodeJS.ErrnoException & { code?: unknown }).code;
          code = typeof raw === "number" ? raw : timedOut ? -1 : 1;
        }
        resolve(makeResult({ args, cwd, code, stdout, stderr, sandboxRoot, timedOut }));
      },
    );
  });
}

function makeResult(input: {
  args: string[];
  cwd: string;
  code: number;
  stdout: string;
  stderr: string;
  sandboxRoot: string;
  timedOut: boolean;
}): RunResult {
  const command = `node ${CLI_BIN} ${input.args.join(" ")}`.trim();
  return {
    command,
    args: input.args,
    cwd: input.cwd,
    code: input.code,
    stdout: input.stdout,
    stderr: input.stderr,
    sandboxRoot: input.sandboxRoot,
    timedOut: input.timedOut,
    describe(): string {
      return [
        `command:  ${command}`,
        `cwd:      ${input.cwd}`,
        `exit:     ${input.code}${input.timedOut ? " (timed out)" : ""}`,
        `sandbox:  ${input.sandboxRoot}`,
        `--- stdout ---`,
        input.stdout || "(empty)",
        `--- stderr ---`,
        input.stderr || "(empty)",
      ].join("\n");
    },
  };
}

/** Assert an exact exit code with full diagnostics on mismatch. */
export function expectExit(result: RunResult, expected: number): void {
  if (result.code !== expected) {
    throw new Error(
      `Expected exit code ${expected}, got ${result.code}\n${result.describe()}`,
    );
  }
}

/** Assert stdout+stderr contains a string, with full diagnostics on mismatch. */
export function expectOutput(result: RunResult, needle: string): void {
  const haystack = `${result.stdout}\n${result.stderr}`;
  if (!haystack.includes(needle)) {
    throw new Error(
      `Expected output to contain ${JSON.stringify(needle)}\n${result.describe()}`,
    );
  }
}

/**
 * Normalize a path for cross-platform comparisons:
 * forward slashes everywhere, lowercase on Windows (case-insensitive FS).
 */
export function normPath(p: string): string {
  const normalized = path.resolve(p).replace(/\\/g, "/");
  return process.platform === "win32" ? normalized.toLowerCase() : normalized;
}

/** True when CLI output mentions the given path (separator/case tolerant). */
export function outputMentionsPath(output: string, p: string): boolean {
  let haystack = output.replace(/\\/g, "/");
  if (process.platform === "win32") haystack = haystack.toLowerCase();
  return haystack.includes(normPath(p));
}

export async function pathExists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}

// ---------------------------------------------------------------------------
// Real-home sentinel: prove tests never create anything in the developer's
// actual home / agent directories. Read-only snapshot before, diff after.
// ---------------------------------------------------------------------------

const REAL_HOME = os.homedir();
const SENTINEL_DIRS = [
  path.join(REAL_HOME, ".kit", "skills"),
  path.join(REAL_HOME, ".claude", "skills"),
  path.join(REAL_HOME, ".codex", "skills"),
  path.join(REAL_HOME, ".grok", "skills"),
];

export type RealHomeSnapshot = Map<string, string[]>;

/** Snapshot entry names of the real home agent skill dirs (read-only). */
export async function snapshotRealHome(): Promise<RealHomeSnapshot> {
  const snapshot: RealHomeSnapshot = new Map();
  for (const dir of SENTINEL_DIRS) {
    try {
      const entries = await readdir(dir);
      snapshot.set(dir, entries.sort());
    } catch {
      snapshot.set(dir, []); // missing dir — must stay missing/empty
    }
  }
  return snapshot;
}

/** Throw if any NEW entry appeared in the real home agent dirs. */
export async function assertRealHomeUntouched(
  before: RealHomeSnapshot,
): Promise<void> {
  const after = await snapshotRealHome();
  const added: string[] = [];
  for (const [dir, entriesAfter] of after) {
    const entriesBefore = new Set(before.get(dir) ?? []);
    for (const entry of entriesAfter) {
      if (!entriesBefore.has(entry)) {
        added.push(path.join(dir, entry));
      }
    }
  }
  if (added.length > 0) {
    throw new Error(
      `E2E isolation breach — new entries in REAL home:\n  ${added.join("\n  ")}`,
    );
  }
}

async function assertCliBuilt(): Promise<void> {
  if (!(await pathExists(CLI_BIN))) {
    throw new Error(
      `Compiled CLI not found at ${CLI_BIN}. Run \`pnpm build\` before the e2e suite.`,
    );
  }
}
