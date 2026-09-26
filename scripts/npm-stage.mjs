#!/usr/bin/env node
/**
 * Stage the Kit 1.x npm packages from built Rust binaries.
 *
 *   dist/npm/kit/              @mzwin/kit (launcher, no binary)
 *   dist/npm/kit-<platform>/   one package per entry in npm/platforms.json
 *
 * The version comes from [workspace.package] in Cargo.toml. The launcher pins
 * every platform package to that exact version.
 *
 * Usage:
 *   node scripts/npm-stage.mjs --binaries <dir>
 *       <dir>/<rust-target>/kit[.exe] must exist for EVERY platform (release).
 *   node scripts/npm-stage.mjs --binary target/release/kit.exe --target x86_64-pc-windows-msvc
 *       Stage the launcher and one platform (local smoke test).
 *   --out <dir>   Output root (default dist/npm). It is deleted first.
 */
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PLATFORMS = JSON.parse(readFileSync(path.join(ROOT, "npm/platforms.json"), "utf8"));
const REPO = "https://github.com/Zwin-ux/kit";

function arg(name) {
  const i = process.argv.indexOf(name);
  return i === -1 ? undefined : process.argv[i + 1];
}

function die(msg) {
  console.error(`npm-stage: ${msg}`);
  process.exit(1);
}

export function cargoVersion() {
  const toml = readFileSync(path.join(ROOT, "Cargo.toml"), "utf8");
  const section = toml.split(/^\[workspace\.package\]\s*$/m)[1];
  const match = section && section.match(/^version\s*=\s*"([^"]+)"/m);
  if (!match) die("no version in [workspace.package] of Cargo.toml");
  return match[1];
}

function writeJson(file, value) {
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

const common = (version) => ({
  version,
  license: "MIT",
  repository: { type: "git", url: `git+${REPO}.git` },
  homepage: `${REPO}#readme`,
  bugs: `${REPO}/issues`,
});

function stagePlatform(outRoot, platform, binary, version) {
  const dir = path.join(outRoot, platform.package.replace("@mzwin/", ""));
  mkdirSync(path.join(dir, "bin"), { recursive: true });
  const dest = path.join(dir, "bin", platform.bin);
  copyFileSync(binary, dest);
  chmodSync(dest, 0o755);
  copyFileSync(path.join(ROOT, "LICENSE"), path.join(dir, "LICENSE"));
  writeFileSync(
    path.join(dir, "README.md"),
    `# ${platform.package}\n\nThe \`kit\` binary for ${platform.os}-${platform.cpu}${platform.libc ? ` (${platform.libc})` : ""}. Install [\`@mzwin/kit\`](https://www.npmjs.com/package/@mzwin/kit) instead.\n`,
  );
  writeJson(path.join(dir, "package.json"), {
    name: platform.package,
    ...common(version),
    description: `Kit binary for ${platform.os}-${platform.cpu}`,
    os: [platform.os],
    cpu: [platform.cpu],
    ...(platform.libc ? { libc: [platform.libc] } : {}),
    files: ["bin/", "LICENSE", "README.md"],
    preferUnplugged: true,
  });
  return dir;
}

function stageLauncher(outRoot, version) {
  const dir = path.join(outRoot, "kit");
  mkdirSync(path.join(dir, "bin"), { recursive: true });
  copyFileSync(path.join(ROOT, "npm/kit/bin/kit.js"), path.join(dir, "bin/kit.js"));
  chmodSync(path.join(dir, "bin/kit.js"), 0o755);
  copyFileSync(path.join(ROOT, "npm/kit/README.md"), path.join(dir, "README.md"));
  copyFileSync(path.join(ROOT, "npm/platforms.json"), path.join(dir, "platforms.json"));
  copyFileSync(path.join(ROOT, "LICENSE"), path.join(dir, "LICENSE"));
  writeJson(path.join(dir, "package.json"), {
    name: "@mzwin/kit",
    ...common(version),
    description: "Dispatch many coding agents. Watch them in one place. Nothing ships unproven.",
    keywords: ["agents", "codex", "claude", "ollama", "worktree", "cli", "tui", "ci", "gate"],
    bin: { kit: "bin/kit.js" },
    files: ["bin/", "platforms.json", "LICENSE", "README.md"],
    engines: { node: ">=18" },
    optionalDependencies: Object.fromEntries(PLATFORMS.map((p) => [p.package, version])),
  });
  return dir;
}

function main() {
  const version = cargoVersion();
  const outRoot = path.resolve(ROOT, arg("--out") ?? "dist/npm");
  const binariesDir = arg("--binaries");
  const singleBinary = arg("--binary");

  const jobs = [];
  if (binariesDir) {
    for (const p of PLATFORMS) {
      const file = path.resolve(binariesDir, p.target, p.bin);
      if (!existsSync(file)) die(`missing binary for ${p.package}: ${file}`);
      jobs.push([p, file]);
    }
  } else if (singleBinary) {
    const target = arg("--target") ?? die("--binary needs --target <rust-target>");
    const p = PLATFORMS.find((x) => x.target === target) ?? die(`unknown target ${target}`);
    const file = path.resolve(singleBinary);
    if (!existsSync(file)) die(`binary not found: ${file}`);
    jobs.push([p, file]);
  } else {
    die("pass --binaries <dir> or --binary <file> --target <rust-target>");
  }

  rmSync(outRoot, { recursive: true, force: true });
  mkdirSync(outRoot, { recursive: true });
  const staged = [stageLauncher(outRoot, version)];
  for (const [p, file] of jobs) staged.push(stagePlatform(outRoot, p, file, version));

  console.log(`staged @mzwin/kit ${version}`);
  for (const dir of staged) console.log(`  ${path.relative(ROOT, dir)}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
