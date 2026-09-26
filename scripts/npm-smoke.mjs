#!/usr/bin/env node
/**
 * Prove the Kit 1.x npm packages work as installed, not as checked out.
 *
 * 1. Stage the launcher + this host's platform package (scripts/npm-stage.mjs).
 * 2. `npm pack` both and check each tarball holds exactly the expected files.
 * 3. Install both tarballs into a fresh temp project and run `kit` through the
 *    npm bin shim: --version, doctor --json, exit-code and argv passthrough.
 * 4. Install the launcher alone and check it stops with a clear error.
 *
 * Usage: cargo build --release -p kit-cli && node scripts/npm-smoke.mjs [--binary <path>]
 * Exit 0 = every check passed. On failure the temp dir is kept for debugging.
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const WIN = process.platform === "win32";
const PLATFORMS = JSON.parse(readFileSync(path.join(ROOT, "npm/platforms.json"), "utf8"));

let failures = 0;
const pass = (msg) => console.log(`  ok    ${msg}`);
const fail = (msg) => {
  failures += 1;
  console.log(`  FAIL  ${msg}`);
};
const check = (cond, msg, detail = "") => (cond ? pass(msg) : fail(detail ? `${msg}\n        ${detail}` : msg));

function run(cmd, args, opts = {}) {
  const res = spawnSync(cmd, args, { encoding: "utf8", shell: WIN && !cmd.endsWith(".exe"), ...opts });
  if (res.error) throw res.error;
  return res;
}

function must(res, what) {
  if (res.status !== 0) {
    console.error(`${what} failed (${res.status}):\n${res.stdout}\n${res.stderr}`);
    process.exit(1);
  }
  return res;
}

function argValue(name) {
  const i = process.argv.indexOf(name);
  return i === -1 ? undefined : process.argv[i + 1];
}

const host = PLATFORMS.find((p) => p.os === process.platform && p.cpu === process.arch);
if (!host) {
  console.error(`npm-smoke: no platform entry for ${process.platform}-${process.arch}`);
  process.exit(1);
}
const binary = path.resolve(argValue("--binary") ?? path.join(ROOT, "target/release", host.bin));
const version = (await import("./npm-stage.mjs")).cargoVersion();
const work = mkdtempSync(path.join(os.tmpdir(), "kit-npm-smoke-"));
const staged = path.join(work, "staged");
const tarballs = path.join(work, "tarballs");

console.log(`npm smoke: @mzwin/kit ${version} on ${host.package}`);
console.log(`  work  ${work}`);

// 1. Stage
must(
  run(process.execPath, [path.join(ROOT, "scripts/npm-stage.mjs"), "--binary", binary, "--target", host.target, "--out", staged]),
  "stage",
);

// 2. Pack + inspect
function pack(dir) {
  const res = must(run("npm", ["pack", "--json", "--pack-destination", tarballs], { cwd: dir }), `npm pack ${dir}`);
  // npm <=11 prints an array; npm 12 prints an object keyed by package name.
  const parsed = JSON.parse(res.stdout);
  const info = Array.isArray(parsed) ? parsed[0] : Object.values(parsed)[0];
  return { tarball: path.join(tarballs, info.filename), files: info.files.map((f) => f.path).sort(), info };
}
run(process.execPath, ["-e", `require("fs").mkdirSync(${JSON.stringify(tarballs)},{recursive:true})`]);

const launcher = pack(path.join(staged, "kit"));
check(
  JSON.stringify(launcher.files) === JSON.stringify(["LICENSE", "README.md", "bin/kit.js", "package.json", "platforms.json"]),
  "launcher tarball holds only launcher files",
  launcher.files.join(", "),
);
check(launcher.info.size < 20 * 1024, `launcher tarball is small (${launcher.info.size} bytes)`);

const launcherPkg = JSON.parse(readFileSync(path.join(staged, "kit/package.json"), "utf8"));
const optional = launcherPkg.optionalDependencies ?? {};
check(
  PLATFORMS.every((p) => optional[p.package] === version) && Object.keys(optional).length === PLATFORMS.length,
  `launcher pins all ${PLATFORMS.length} platform packages to ${version}`,
  JSON.stringify(optional),
);
check(!launcherPkg.dependencies, "launcher has no runtime dependencies");

const platform = pack(path.join(staged, host.package.replace("@mzwin/", "")));
check(
  JSON.stringify(platform.files) === JSON.stringify(["LICENSE", "README.md", `bin/${host.bin}`, "package.json"]),
  `${host.package} tarball holds only the binary`,
  platform.files.join(", "),
);

// 3. Install the pair and run through the bin shim
function project(name, specs) {
  const dir = path.join(work, name);
  run(process.execPath, ["-e", `require("fs").mkdirSync(${JSON.stringify(dir)},{recursive:true})`]);
  writeFileSync(path.join(dir, "package.json"), JSON.stringify({ name, private: true }));
  // --omit=optional keeps npm from fetching the other platforms from the
  // registry; the host package is installed directly from its tarball.
  must(
    run("npm", ["install", "--omit=optional", "--no-audit", "--no-fund", "--loglevel=error", ...specs], { cwd: dir }),
    `npm install (${name})`,
  );
  return dir;
}

const kitHome = path.join(work, "kit-home");
const env = { ...process.env, KIT_HOME: kitHome, NO_COLOR: "1" };
delete env.KIT_BINARY_PATH;

const full = project("full", [launcher.tarball, platform.tarball]);
const shim = path.join(full, "node_modules/.bin", WIN ? "kit.cmd" : "kit");
const kit = (args, extra = {}) => run(shim, args, { cwd: full, env: { ...env, ...extra } });

const ver = kit(["--version"]);
check(ver.status === 0 && ver.stdout.trim() === `kit ${version}`, "kit --version through npm shim", `${ver.status} ${ver.stdout}${ver.stderr}`);

const doctor = kit(["doctor", "--json"]);
let envelope;
try {
  envelope = JSON.parse(doctor.stdout);
} catch {
  envelope = undefined;
}
check(envelope?.schemaVersion === 1 && envelope?.command === "doctor", "kit doctor --json returns the v1 envelope", doctor.stdout.slice(0, 200));
check(envelope?.data?.install === "npm", "doctor reports the npm install", String(envelope?.data?.install));
const shimDir = path.dirname(shim).toLowerCase();
check(
  (envelope?.data?.pathCollisions ?? []).every((p) => path.dirname(p).toLowerCase() !== shimDir),
  "doctor does not flag its own npm shim as another kit",
  JSON.stringify(envelope?.data?.pathCollisions),
);

// Call the installed launcher without a shell: on Windows, spawnSync's
// shell mode joins args unquoted, which would test cmd.exe, not the launcher.
const installed = path.join(full, "node_modules/@mzwin/kit/bin/kit.js");
const probe = ["-e", "process.stdout.write(JSON.stringify(process.argv.slice(1)));process.exit(7)"];
const passthrough = run(process.execPath, [installed, ...probe, "--", "a b", "--json"], {
  cwd: full,
  env: { ...env, KIT_BINARY_PATH: process.execPath },
});
check(passthrough.status === 7, "exit code passes through the launcher", `got ${passthrough.status}`);
check(
  passthrough.stdout.includes('"a b"') && passthrough.stdout.includes('"--json"'),
  "argv with spaces passes through intact",
  passthrough.stdout,
);

const badOverride = kit(["--version"], { KIT_BINARY_PATH: path.join(work, "nope") });
check(badOverride.status === 1 && badOverride.stderr.includes("KIT_BINARY_PATH does not exist"), "bad KIT_BINARY_PATH is a clear error", badOverride.stderr);

// 4. Launcher without its platform package
const bare = project("bare", [launcher.tarball]);
const missing = run(path.join(bare, "node_modules/.bin", WIN ? "kit.cmd" : "kit"), ["--version"], { cwd: bare, env });
check(
  missing.status === 1 && missing.stderr.includes(`${host.package} is not installed`) && missing.stderr.includes("--omit=optional"),
  "missing platform package is a clear error",
  missing.stderr,
);

if (failures) {
  console.log(`\n${failures} check(s) failed. Kept ${work}`);
  process.exit(1);
}
rmSync(work, { recursive: true, force: true });
console.log("\nall npm smoke checks passed");
