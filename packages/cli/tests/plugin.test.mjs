import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const bin = path.resolve(here, "../dist/bin.js");
const tempRoot = mkdtempSync(path.join(os.tmpdir(), "kit-plugin-cli-"));
const pluginRoot = path.join(tempRoot, "plugin");
const kitHome = path.join(tempRoot, "home");

function run(args) {
  const result = spawnSync(process.execPath, [bin, ...args], {
    cwd: pluginRoot,
    encoding: "utf8",
    env: { ...process.env, KIT_HOME: kitHome },
    shell: false,
  });
  assert.equal(
    result.status,
    0,
    `${args.join(" ")} failed:\n${result.stderr}`,
  );
  return result.stdout;
}

try {
  mkdirSync(pluginRoot, { recursive: true });
  writeFileSync(
    path.join(pluginRoot, "kit.plugin.json"),
    `${JSON.stringify(
      {
        schemaVersion: 1,
        name: "proof-cli",
        displayName: "Proof CLI",
        description: "Runs a controlled local command.",
        version: "1.0.0",
        command: "node",
        versionArgs: ["--version"],
        healthArgs: ["--version"],
        safety: {
          summary: "The test command writes no external state.",
        },
      },
      null,
      2,
    )}\n`,
  );

  const dryRun = run(["plugin", "add", pluginRoot]);
  assert.match(dryRun, /Plugin plan \(dry-run\)/);
  assert.throws(() => readFileSync(path.join(kitHome, "plugins.json")));

  const added = run(["plugin", "add", pluginRoot, "--write"]);
  assert.match(added, /Plugin registered/);

  const doctor = run(["plugin", "doctor", "proof-cli"]);
  assert.match(doctor, /status:\s+ready/);
  assert.match(doctor, /manifest:\s+unchanged/);

  const version = run(["plugin", "run", "proof-cli", "--", "--version"]);
  assert.match(version.trim(), /^v\d+/);

  const removed = run(["plugin", "remove", "proof-cli", "--write"]);
  assert.match(removed, /Removed plugin proof-cli/);

  console.log("all plugin CLI tests passed");
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
}
