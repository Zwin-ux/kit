#!/usr/bin/env node
/**
 * Publish @mzwin/* packages in dependency order (public).
 * Requires: npm login; clean tree recommended.
 *
 * Always runs prepare-publish-versions and refuses workspace:* leftovers.
 * Usage: node scripts/publish.mjs [version]
 * Default version: from packages/shared/package.json or 0.1.5
 */
import { readFileSync, readdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const order = ["shared", "core", "tui", "catalog", "cli"];

function run(cmd, args, cwd = root) {
  console.log(`\n> ${cmd} ${args.join(" ")}`);
  const r = spawnSync(cmd, args, { cwd, stdio: "inherit", shell: true });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

function readJson(rel) {
  return JSON.parse(readFileSync(path.join(root, rel), "utf8"));
}

const sharedVer = readJson("packages/shared/package.json").version;
const ver = process.argv[2] || sharedVer || "0.1.5";

console.log(`Publishing @mzwin/* at ${ver}`);

// 1) Rewrite workspace:* → pinned versions
run("node", ["scripts/prepare-publish-versions.mjs", ver]);

// 2) Gate: no workspace protocol left
for (const name of order) {
  const pkg = readJson(`packages/${name}/package.json`);
  const deps = { ...pkg.dependencies, ...pkg.devDependencies };
  for (const [dep, range] of Object.entries(deps || {})) {
    if (String(range).includes("workspace:")) {
      console.error(
        `REFUSE: packages/${name}/package.json still has ${dep}: ${range}`,
      );
      process.exit(1);
    }
  }
}
console.log("workspace:* gate: clean");

// 3) Sync catalog + build
run("node", ["scripts/sync-catalog-package.mjs"]);
run("pnpm", ["build"]);

// 4) Pack dry-run on cli
run("npm", ["pack", "--dry-run"], path.join(root, "packages/cli"));

// 5) Publish in order
for (const name of order) {
  const dir = path.join(root, "packages", name);
  run("pnpm", ["publish", "--access", "public", "--no-git-checks"], dir);
}

console.log("\nPublished @mzwin packages in order:", order.join(", "));
console.log("Verify: npm view @mzwin/kit version  → should be", ver);
