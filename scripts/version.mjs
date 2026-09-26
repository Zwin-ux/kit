#!/usr/bin/env node
// The product version lives in `[workspace.package] version` of Cargo.toml.
// cargo publish also needs it on each internal dependency in
// `[workspace.dependencies]`. Those are exact pins (`=2.0.0`): the library
// crates carry no semver promise, so `cargo install kitctl` must get the
// libraries it was released with, not a newer compatible one.
//
//   node scripts/version.mjs                print the version
//   node scripts/version.mjs --check        exit 1 if any copy drifts
//   node scripts/version.mjs --set 2.0.0    rewrite every copy (then run
//                                           `cargo update -w` for Cargo.lock)
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const CARGO = path.join(ROOT, "Cargo.toml");
// SemVer 2.0: MAJOR.MINOR.PATCH with an optional dot-separated prerelease.
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$/;

function die(msg) {
  console.error(`version: ${msg}`);
  process.exit(1);
}

// Split Cargo.toml into its `[section]` blocks, keeping every byte, so a
// rewrite touches only the lines it means to.
function sections(toml) {
  const parts = toml.split(/^(?=\[)/m);
  return parts.map((text) => ({ name: text.match(/^\[([^\]]+)\]/)?.[1] ?? "", text }));
}

function read() {
  const toml = readFileSync(CARGO, "utf8");
  const blocks = sections(toml);
  const pkg = blocks.find((b) => b.name === "workspace.package");
  const version = pkg?.text.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) die("no version in [workspace.package] of Cargo.toml");
  const deps = blocks.find((b) => b.name === "workspace.dependencies");
  // Internal crates are the ones with a `path`; each needs the version too.
  const internal = [...(deps?.text.matchAll(/^([\w-]+)\s*=\s*\{[^}\n]*\bpath\s*=[^}\n]*\}/gm) ?? [])].map((m) => ({
    name: m[1],
    line: m[0],
    version: m[0].match(/\bversion\s*=\s*"=?([^"]+)"/)?.[1],
    exact: /\bversion\s*=\s*"=/.test(m[0]),
  }));
  return { toml, blocks, version, internal };
}

const args = process.argv.slice(2);
const { toml, blocks, version, internal } = read();

if (args[0] === "--check") {
  const bad = internal.filter((d) => d.version !== version || !d.exact);
  if (!SEMVER.test(version)) die(`"${version}" is not a SemVer version`);
  if (bad.length) {
    for (const d of bad) console.error(`version: ${d.name} has "${d.exact ? "=" : ""}${d.version ?? ""}", expected "=${version}"`);
    process.exit(1);
  }
  console.log(`${version} (Cargo.toml, ${internal.length} internal crates agree)`);
} else if (args[0] === "--set") {
  const next = args[1];
  if (!next || !SEMVER.test(next)) die(`usage: --set <semver>, got "${next ?? ""}"`);
  const out = blocks
    .map((b) => {
      if (b.name === "workspace.package") return b.text.replace(/^(version\s*=\s*)"[^"]+"/m, `$1"${next}"`);
      if (b.name === "workspace.dependencies")
        return b.text.replace(/^([\w-]+\s*=\s*\{[^}\n]*\bpath\s*=[^}\n]*)\}/gm, (line) =>
          /\bversion\s*=/.test(line)
            ? line.replace(/(\bversion\s*=\s*)"[^"]+"/, `$1"=${next}"`)
            : line.replace(/\s*\}$/, `, version = "=${next}" }`),
        );
      return b.text;
    })
    .join("");
  if (out === toml && version !== next) die("nothing rewritten");
  writeFileSync(CARGO, out);
  console.log(`${version} -> ${next}. Now run: cargo update -w`);
} else if (args.length === 0) {
  console.log(version);
} else {
  die(`unknown argument ${args[0]}`);
}
