#!/usr/bin/env node
/**
 * Package integrity smoke test — proves the published artifacts work.
 *
 * 1. `pnpm pack` every publishable package (workspace:* deps become pinned
 *    versions inside each tarball; the source tree is never mutated).
 * 2. Inspect tarball contents: dist present, no src/test leaks, no oversized
 *    or private files, catalog ships packs/ + skills/.
 * 3. `npm install` all five tarballs into a fresh temp project so inter-deps
 *    resolve from the local tarballs, then verify no @mzwin/* package was
 *    silently fetched from the registry instead.
 * 4. Run the installed CLI (`kit --version/ready/status/doctor` and a full
 *    `ready --write`) with HOME/USERPROFILE/KIT_HOME sandboxed to temp dirs.
 *
 * Usage: pnpm build && node scripts/pack-smoke.mjs
 * Exit 0 = every check passed. On failure the temp dir is kept for debugging.
 */
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGES = ["shared", "core", "tui", "catalog", "cli"];
const NPM_NAMES = {
  shared: "@mzwin/kit-shared",
  core: "@mzwin/kit-core",
  tui: "@mzwin/kit-tui",
  catalog: "@mzwin/kit-catalog",
  cli: "@mzwin/kit",
};
const MAX_FILE_BYTES = 5 * 1024 * 1024;
const WIN = process.platform === "win32";

let failures = 0;
let work = null;
function pass(msg) {
  console.log(`  ok  ${msg}`);
}
function fail(msg) {
  failures += 1;
  console.error(`  FAIL  ${msg}`);
}

function run(cmd, args, options = {}) {
  const result = spawnSync(cmd, args, {
    encoding: "utf8",
    shell: WIN && (cmd === "pnpm" || cmd === "npm"),
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
  return {
    code: result.status ?? -1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function must(step, result) {
  if (result.code !== 0) {
    fail(`${step} (exit ${result.code})`);
    console.error(`--- stdout ---\n${result.stdout}\n--- stderr ---\n${result.stderr}`);
    finish();
  }
  return result;
}

// --- minimal tar reader (ustar + prefix + pax path override) ---------------
function listTarball(tgzPath) {
  const buf = zlib.gunzipSync(readFileSync(tgzPath));
  const entries = [];
  let offset = 0;
  let paxPath = null;
  while (offset + 512 <= buf.length) {
    const header = buf.subarray(offset, offset + 512);
    if (header.every((b) => b === 0)) break;
    const rawName = cstr(header.subarray(0, 100));
    const prefix = cstr(header.subarray(345, 500));
    const size = parseInt(cstr(header.subarray(124, 136)).trim(), 8) || 0;
    const type = String.fromCharCode(header[156]);
    const dataStart = offset + 512;
    if (type === "x" || type === "g") {
      const body = buf.subarray(dataStart, dataStart + size).toString("utf8");
      const match = body.match(/\d+ path=([^\n]+)\n/);
      if (type === "x" && match) paxPath = match[1];
    } else {
      const name = paxPath ?? (prefix ? `${prefix}/${rawName}` : rawName);
      paxPath = null;
      entries.push({
        name,
        size,
        data: () => buf.subarray(dataStart, dataStart + size),
      });
    }
    offset = dataStart + Math.ceil(size / 512) * 512;
  }
  return entries;
}
function cstr(buf) {
  return buf.toString("utf8").replace(/\0[\s\S]*$/, "");
}

// --- sandboxed CLI runner --------------------------------------------------
function makeSandboxEnv(homeDir, kitHome) {
  const env = { ...process.env };
  delete env.HOMEDRIVE;
  delete env.HOMEPATH;
  delete env.KIT_PACKS;
  delete env.KIT_SKILLS;
  delete env.KIT_ASSETS;
  env.HOME = homeDir;
  env.USERPROFILE = homeDir;
  env.KIT_HOME = kitHome;
  env.KIT_REGISTRY_URL = "http://127.0.0.1:1";
  env.NO_COLOR = "1";
  return env;
}

// ---------------------------------------------------------------------------
console.log("pack-smoke: package integrity check\n");

const cliPkg = JSON.parse(
  readFileSync(path.join(ROOT, "packages", "cli", "package.json"), "utf8"),
);
const expectedVersion = cliPkg.version;
console.log(`expected version: ${expectedVersion}`);

if (!existsSync(path.join(ROOT, "packages", "cli", "dist", "bin.js"))) {
  fail("packages/cli/dist/bin.js missing — run `pnpm build` first");
  finish();
}

work = mkdtempSync(path.join(os.tmpdir(), "kit-pack-smoke-"));
const tarballDir = path.join(work, "tarballs");
mkdirSync(tarballDir, { recursive: true });
console.log(`workdir: ${work}\n`);

// 1) pack every publishable package -----------------------------------------
console.log("[1/4] pnpm pack");
const tarballs = {};
for (const name of PACKAGES) {
  const pkgDir = path.join(ROOT, "packages", name);
  must(
    `pnpm pack ${name}`,
    run("pnpm", ["pack", "--pack-destination", tarballDir], { cwd: pkgDir }),
  );
  // Exact name — a bare prefix like "mzwin-kit-" would also match
  // mzwin-kit-catalog-*.tgz.
  const base = NPM_NAMES[name].replace("@", "").replace("/", "-");
  const file = readdirSync(tarballDir).find(
    (f) => f === `${base}-${expectedVersion}.tgz`,
  );
  if (!file) {
    fail(`tarball ${base}-${expectedVersion}.tgz not found in ${tarballDir}`);
    finish();
  }
  tarballs[name] = path.join(tarballDir, file);
  pass(`${NPM_NAMES[name]} → ${file}`);
}

// 2) inspect tarball contents ------------------------------------------------
console.log("\n[2/4] tarball contents");
for (const name of PACKAGES) {
  const entries = listTarball(tarballs[name]);
  const names = entries.map((e) => e.name);

  const manifestEntry = entries.find((e) => e.name === "package/package.json");
  if (!manifestEntry) {
    fail(`${name}: package/package.json missing from tarball`);
    continue;
  }
  const manifest = JSON.parse(manifestEntry.data().toString("utf8"));

  if (manifest.version !== expectedVersion) {
    fail(
      `${name}: tarball version ${manifest.version} != expected ${expectedVersion}`,
    );
  } else {
    pass(`${name}: version ${manifest.version}`);
  }

  const workspaceDeps = Object.entries({
    ...manifest.dependencies,
    ...manifest.devDependencies,
  }).filter(([, range]) => String(range).includes("workspace:"));
  if (workspaceDeps.length > 0) {
    fail(`${name}: workspace:* leaked into tarball manifest: ${workspaceDeps}`);
  } else {
    pass(`${name}: no workspace:* in manifest`);
  }

  const leaks = names.filter(
    (n) =>
      n.startsWith("package/src/") ||
      n.startsWith("package/tests/") ||
      n.endsWith(".tsbuildinfo") ||
      /(^|\/)\.env(\.|$)/.test(n) ||
      n.endsWith("auth.json"),
  );
  if (leaks.length > 0) {
    fail(`${name}: unexpected files in tarball: ${leaks.join(", ")}`);
  } else {
    pass(`${name}: no src/test/private leaks`);
  }

  const oversized = entries.filter((e) => e.size > MAX_FILE_BYTES);
  if (oversized.length > 0) {
    fail(
      `${name}: oversized files: ${oversized.map((e) => `${e.name} (${e.size}B)`).join(", ")}`,
    );
  } else {
    pass(`${name}: all files < ${MAX_FILE_BYTES / 1024 / 1024}MB`);
  }

  if (name === "cli" || name === "core" || name === "shared" || name === "tui") {
    if (!names.some((n) => n.startsWith("package/dist/"))) {
      fail(`${name}: dist/ missing from tarball`);
    } else {
      pass(`${name}: dist/ present`);
    }
  }
  if (name === "cli" && !names.includes("package/dist/bin.js")) {
    fail("cli: package/dist/bin.js missing");
  }
  if (name === "catalog") {
    const hasPacks = names.some((n) => n.startsWith("package/packs/"));
    const hasSkills = names.some((n) => n.startsWith("package/skills/"));
    if (!hasPacks || !hasSkills) {
      fail(`catalog: packs/skills content missing (packs=${hasPacks}, skills=${hasSkills})`);
    } else {
      pass("catalog: ships packs/ and skills/");
    }
  }
}
if (failures > 0) finish();

// 3) install all tarballs into a fresh project -------------------------------
console.log("\n[3/4] npm install from tarballs");
const installDir = path.join(work, "install");
mkdirSync(installDir, { recursive: true });
writeFileSync(
  path.join(installDir, "package.json"),
  JSON.stringify({ name: "kit-pack-smoke", version: "0.0.0", private: true }, null, 2),
);
must(
  "npm install <tarballs>",
  run(
    "npm",
    [
      "install",
      "--no-audit",
      "--no-fund",
      ...PACKAGES.map((name) => tarballs[name]),
    ],
    { cwd: installDir },
  ),
);
pass("npm install completed");

// Inter-deps must have resolved from the local tarballs, not the registry.
const lockPath = path.join(installDir, "package-lock.json");
const lock = JSON.parse(readFileSync(lockPath, "utf8"));
const fromRegistry = Object.entries(lock.packages ?? {}).filter(
  ([pkgPath, meta]) =>
    pkgPath.includes("@mzwin") &&
    String(meta.resolved ?? "").includes("registry.npmjs.org"),
);
if (fromRegistry.length > 0) {
  fail(
    `@mzwin/* resolved from registry instead of local tarballs: ${fromRegistry
      .map(([p]) => p)
      .join(", ")}`,
  );
} else {
  pass("all @mzwin/* packages resolved from local tarballs");
}

const binDir = path.join(installDir, "node_modules", ".bin");
const binWired = readdirSync(binDir).some((f) => f === "kit" || f.startsWith("kit."));
if (!binWired) {
  fail("npm did not wire a `kit` entry in node_modules/.bin");
} else {
  pass("bin wiring: node_modules/.bin/kit present");
}

// 4) run the installed CLI in a sandbox --------------------------------------
console.log("\n[4/4] run installed CLI (sandboxed HOME/KIT_HOME)");
const sandboxHome = path.join(work, "home");
const projectDir = path.join(work, "project");
mkdirSync(sandboxHome, { recursive: true });
mkdirSync(projectDir, { recursive: true });
writeFileSync(
  path.join(projectDir, "package.json"),
  JSON.stringify({ name: "smoke-project", version: "0.0.0", private: true }, null, 2),
);
const env = makeSandboxEnv(sandboxHome, path.join(sandboxHome, ".kit"));
const installedBin = path.join(
  installDir,
  "node_modules",
  "@mzwin",
  "kit",
  "dist",
  "bin.js",
);

function kit(args, step, { expectCode = 0, expectOut } = {}) {
  const result = run(process.execPath, [installedBin, ...args], {
    cwd: projectDir,
    env,
  });
  if (result.code !== expectCode) {
    fail(`${step}: exit ${result.code} (wanted ${expectCode})`);
    console.error(`--- stdout ---\n${result.stdout}\n--- stderr ---\n${result.stderr}`);
  } else if (expectOut && !result.stdout.includes(expectOut)) {
    fail(`${step}: stdout missing ${JSON.stringify(expectOut)}`);
    console.error(`--- stdout ---\n${result.stdout}`);
  } else {
    pass(step);
  }
  return result;
}

const versionOut = kit(["--version"], "kit --version exits 0");
if (versionOut.stdout.trim() !== expectedVersion) {
  fail(
    `kit --version printed "${versionOut.stdout.trim()}" but package.json says ${expectedVersion}`,
  );
} else {
  pass(`kit --version == package.json version (${expectedVersion})`);
}

kit(["ready"], "kit ready (dry-run)", { expectOut: "READY  (dry-run)" });
kit(["status"], "kit status");
kit(["doctor"], "kit doctor", { expectOut: "Doctor OK" });
kit(["ready", "--write", "--pack", "essentials"], "kit ready --write --pack essentials", {
  expectOut: "READY",
});
if (
  existsSync(
    path.join(projectDir, ".claude", "skills", "add-readme", "SKILL.md"),
  )
) {
  pass("ready --write linked add-readme into project .claude/skills");
} else {
  fail("ready --write did not produce .claude/skills/add-readme/SKILL.md");
}

finish();

function finish() {
  console.log("");
  if (failures > 0) {
    console.error(`pack-smoke: ${failures} check(s) FAILED`);
    console.error(`workdir kept for debugging: ${work ?? "(none)"}`);
    process.exit(1);
  }
  try {
    rmSync(work, { recursive: true, force: true, maxRetries: 5 });
  } catch {
    // temp cleanup is best-effort
  }
  console.log("pack-smoke: all checks passed");
  process.exit(0);
}
