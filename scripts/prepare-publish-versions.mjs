#!/usr/bin/env node
/**
 * Rewrite package.json versions/deps for npm publish (replace workspace:*).
 * Usage: node scripts/prepare-publish-versions.mjs 0.1.1
 */
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const ver = process.argv[2] || "0.1.1";

const files = {
  shared: "packages/shared/package.json",
  core: "packages/core/package.json",
  tui: "packages/tui/package.json",
  catalog: "packages/catalog/package.json",
  cli: "packages/cli/package.json",
};

const names = {
  shared: "@mzwin/kit-shared",
  core: "@mzwin/kit-core",
  tui: "@mzwin/kit-tui",
  catalog: "@mzwin/kit-catalog",
  cli: "@mzwin/kit",
};

function load(rel) {
  const raw = readFileSync(path.join(root, rel), "utf8").replace(/^\uFEFF/, "");
  return JSON.parse(raw);
}
function save(rel, j) {
  writeFileSync(path.join(root, rel), JSON.stringify(j, null, 2) + "\n");
}

const shared = load(files.shared);
shared.name = names.shared;
shared.version = ver;
shared.private = false;
shared.publishConfig = { access: "public" };
save(files.shared, shared);

const core = load(files.core);
core.name = names.core;
core.version = ver;
core.private = false;
core.publishConfig = { access: "public" };
core.dependencies = {
  "@mzwin/kit-shared": ver,
  semver: core.dependencies?.semver ?? "^7.7.2",
  yaml: core.dependencies?.yaml ?? "^2.8.0",
};
save(files.core, core);

const tui = load(files.tui);
tui.name = names.tui;
tui.version = ver;
tui.private = false;
tui.publishConfig = { access: "public" };
tui.bin = { "kit-tui": "dist/bin.js" };
tui.dependencies = {
  "@mzwin/kit-core": ver,
  "@mzwin/kit-shared": ver,
  ink: tui.dependencies?.ink ?? "^5.2.1",
  pngjs: tui.dependencies?.pngjs ?? "^7.0.0",
  react: tui.dependencies?.react ?? "^18.3.1",
};
save(files.tui, tui);

const catalog = load(files.catalog);
catalog.name = names.catalog;
catalog.version = ver;
catalog.private = false;
catalog.publishConfig = { access: "public" };
save(files.catalog, catalog);

const cli = load(files.cli);
cli.name = names.cli;
cli.version = ver;
cli.private = false;
cli.publishConfig = { access: "public" };
cli.bin = { kit: "dist/bin.js" };
cli.dependencies = {
  "@mzwin/kit-catalog": ver,
  "@mzwin/kit-core": ver,
  "@mzwin/kit-shared": ver,
  "@mzwin/kit-tui": ver,
};
save(files.cli, cli);

// version constant
const sharedSrc = path.join(root, "packages/shared/src/index.ts");
let src = readFileSync(sharedSrc, "utf8");
src = src.replace(
  /KIT_PACKAGE_VERSION = "[^"]+"/,
  `KIT_PACKAGE_VERSION = "${ver}"`,
);
writeFileSync(sharedSrc, src);

console.log("prepared publish versions", ver);
