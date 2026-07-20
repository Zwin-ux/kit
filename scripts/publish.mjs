#!/usr/bin/env node
/**
 * Publish @kit-skills/* packages in dependency order (public).
 * Requires: clean tree recommended, npm login, org access for @kit-skills.
 */
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

run("node", ["scripts/sync-catalog-package.mjs"]);
run("pnpm", ["build"]);

for (const name of order) {
  const dir = path.join(root, "packages", name);
  run("pnpm", ["publish", "--access", "public", "--no-git-checks"], dir);
}

console.log("\nPublished:", order.map((n) => `@kit-skills/${n}`).join(", "));
