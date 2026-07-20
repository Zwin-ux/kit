#!/usr/bin/env node
/**
 * Copy monorepo skills/ + packs/ into packages/catalog for npm publish.
 */
import { cp, mkdir, rm, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dest = path.join(root, "packages", "catalog");

async function exists(p) {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}

async function syncDir(name) {
  const from = path.join(root, name);
  const to = path.join(dest, name);
  if (!(await exists(from))) {
    throw new Error(`missing source ${from}`);
  }
  await rm(to, { recursive: true, force: true });
  await mkdir(path.dirname(to), { recursive: true });
  await cp(from, to, {
    recursive: true,
    filter: (src) => {
      const base = path.basename(src);
      return base !== "node_modules" && base !== ".git" && base !== "dist";
    },
  });
  console.log("synced", name, "→", to);
}

await syncDir("skills");
await syncDir("packs");
console.log("catalog package ready");
