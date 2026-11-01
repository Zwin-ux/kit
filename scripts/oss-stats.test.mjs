import assert from "node:assert/strict";
import { test } from "node:test";
import { externalAuthors, windowStart } from "./oss-stats.mjs";

test("external authors skip the owner, bots, claude and exclusions, once each", () => {
  const logins = ["Zwin-ux", "alice", "dependabot[bot]", "claude", "bob", "alice", "mzwin-gif", "Bob"];
  assert.deepEqual(externalAuthors(logins, "zwin-ux", ["mzwin-gif"]), ["alice", "bob"]);
});

test("the window is twelve months back", () => {
  assert.equal(windowStart(new Date("2026-11-01T12:00:00Z")), "2025-11-01");
});
