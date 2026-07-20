/**
 * Smoke tests for CLI argv normalization.
 * Run: node packages/cli/tests/argv.test.mjs
 * (Does not import bin.js — that would start the CLI.)
 */
import assert from "node:assert/strict";
import { normalizeArgv } from "../dist/argv.js";

function check(name, input, expected) {
  const got = normalizeArgv(input);
  assert.deepEqual(got, expected, name);
  console.log(`ok  ${name}`);
}

check("passthrough tui", ["tui"], ["tui"]);
check("strip leading --", ["--", "tui"], ["tui"]);
check("keep flags after command", ["unify", "--write", "--link"], [
  "unify",
  "--write",
  "--link",
]);
check("strip only first --", ["--", "unify", "--write"], [
  "unify",
  "--write",
]);
check("empty", [], []);
check("help after --", ["--", "--help"], ["--help"]);
check("version after --", ["--", "-v"], ["-v"]);

console.log("all argv tests passed");
