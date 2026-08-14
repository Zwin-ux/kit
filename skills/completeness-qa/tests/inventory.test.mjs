import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const skillRoot = path.resolve(here, "..");
const script = path.join(skillRoot, "scripts", "inventory.mjs");
const fixtures = path.join(here, "fixtures");

function run(args) {
  return spawnSync(process.execPath, [script, ...args], {
    encoding: "utf8",
    cwd: skillRoot,
  });
}

function parseJson(stdout) {
  return JSON.parse(stdout);
}

describe("completeness-qa inventory", () => {
  it("flags stub / untested / ok on the TS fixture", () => {
    const root = path.join(fixtures, "stub-ts");
    const result = run(["--root", root, "--json", "--no-write"]);
    assert.equal(result.status, 0, result.stderr);
    const report = parseJson(result.stdout);
    const names = report.symbols.map((s) => s.name).sort();
    assert.deepEqual(names, ["login", "logout", "sessionId"]);
    assert.equal(report.counts.public, 3);
    assert.equal(report.counts.stub, 1);
    assert.equal(report.counts.untested, 1);
    assert.equal(report.counts.ok, 1);
    assert.equal(report.verdict, "not_done");
    const login = report.symbols.find((s) => s.name === "login");
    const logout = report.symbols.find((s) => s.name === "logout");
    const session = report.symbols.find((s) => s.name === "sessionId");
    assert.equal(login.status, "stub");
    assert.equal(logout.status, "untested");
    assert.equal(session.status, "ok");
    assert.ok(
      report.skillLadder.some((row) => row.skill === "write-tests"),
      "write-tests on ladder",
    );
    assert.ok(report.skillLadder.length <= 4);
    assert.ok(report.testNext.length <= 10);
  });

  it("prints the tool voice on the TS fixture", () => {
    const root = path.join(fixtures, "stub-ts");
    const result = run(["--root", root, "--no-write"]);
    assert.equal(result.status, 0, result.stderr);
    const text = result.stdout;
    assert.match(text, /^KIT completeness-qa/m);
    assert.match(text, /Verdict\s+NOT DONE/);
    assert.match(text, /Test next/);
    const nextRows = text
      .split("Test next")[1]
      .split("Skill ladder")[0]
      .trim()
      .split("\n")
      .filter((l) => /^\s+\d+\./.test(l));
    assert.ok(nextRows.length <= 5);
    const ladderRows = text
      .split("Skill ladder")[1]
      .split("Next")[0]
      .trim()
      .split("\n")
      .filter((l) => /^\s+\d+\./.test(l));
    assert.ok(ladderRows.length <= 4);
  });

  it("marks a mentioned Rust pub fn done", () => {
    const root = path.join(fixtures, "clean-rs");
    const result = run(["--root", root, "--json", "--no-write"]);
    assert.equal(result.status, 0, result.stderr);
    const report = parseJson(result.stdout);
    assert.equal(report.counts.public, 1);
    assert.equal(report.counts.stub, 0);
    assert.equal(report.counts.untested, 0);
    assert.equal(report.verdict, "done");
    assert.equal(report.symbols[0].name, "add");
    assert.equal(report.symbols[0].status, "ok");
  });

  it("does not write JSON when --no-write", () => {
    const tmp = mkdtempSync(path.join(os.tmpdir(), "kit-cqa-"));
    try {
      const result = run([
        "--root",
        path.join(fixtures, "stub-ts"),
        "--json",
        "--no-write",
      ]);
      assert.equal(result.status, 0, result.stderr);
      assert.equal(existsSync(path.join(tmp, ".kit", "completeness.json")), false);
      assert.equal(
        existsSync(path.join(fixtures, "stub-ts", ".kit", "completeness.json")),
        false,
      );
    } finally {
      rmSync(tmp, { recursive: true, force: true });
    }
  });

  it("does not inventory test files as public symbols", () => {
    const root = path.join(fixtures, "stub-ts");
    const result = run(["--root", root, "--json", "--no-write"]);
    const report = parseJson(result.stdout);
    assert.ok(!report.symbols.some((s) => s.file.includes(".test.")));
    assert.ok(!report.symbols.some((s) => s.name === "test"));
  });

  it("ignores export type and export interface", () => {
    const root = path.join(fixtures, "stub-ts");
    const result = run(["--root", root, "--json", "--no-write"]);
    const report = parseJson(result.stdout);
    const names = report.symbols.map((s) => s.name);
    assert.ok(!names.includes("User"));
    assert.ok(!names.includes("Session"));
  });

  it("does not call an empty type-only scan done", () => {
    const root = path.join(fixtures, "empty-ts");
    const result = run(["--root", root, "--json", "--no-write"]);
    assert.equal(result.status, 0, result.stderr);
    const report = parseJson(result.stdout);
    assert.equal(report.counts.public, 0);
    assert.equal(report.verdict, "empty");
    assert.notEqual(report.verdict, "done");
  });

  it("validates SKILL.md front matter shape", () => {
    const text = readFileSync(path.join(skillRoot, "SKILL.md"), "utf8");
    const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    assert.ok(match, "front matter");
    const fm = match[1];
    assert.match(fm, /^name:\s*completeness-qa\s*$/m);
    assert.match(fm, /^version:\s*0\.1\.0\s*$/m);
    assert.match(fm, /claude-code/);
    assert.match(fm, /grok-build/);
    assert.match(fm, /codex/);
    const desc = fm.match(/description:\s*"([^"]+)"/);
    assert.ok(desc, "quoted description");
    const sentences = desc[1]
      .split(/(?<=[.!?])\s+/)
      .map((s) => s.trim())
      .filter(Boolean);
    assert.ok(sentences.length >= 1 && sentences.length <= 2);
  });
});
