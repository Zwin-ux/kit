// node --test scripts/loc-budget.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { countCodeLines, measure } from "./loc-budget.mjs";

test("blank lines and comments are not code", () => {
  const src = "// one\n\n/* block\n still block */\nfn a() {}\n/// doc\n//! inner\n";
  assert.equal(countCodeLines(src), 1);
});

test("CRLF sources count the same as LF", () => {
  assert.equal(countCodeLines("fn a() {}\r\n\r\nfn b() {}\r\n"), 2);
});

test("a #[cfg(test)] module is not counted", () => {
  const src = [
    "fn real() -> u8 {",
    "    1",
    "}",
    "#[cfg(test)]",
    "mod tests {",
    "    #[test]",
    "    fn t() { assert_eq!(super::real(), 1); }",
    "}",
    "fn after() {}",
  ].join("\n");
  assert.equal(countCodeLines(src), 4);
});

test("a one-line #[cfg(test)] item ends at its semicolon", () => {
  assert.equal(countCodeLines("#[cfg(test)]\nuse std::fs;\nfn a() {}\n"), 1);
});

function tree(files) {
  const dir = mkdtempSync(path.join(tmpdir(), "loc-"));
  for (const [rel, body] of Object.entries(files)) {
    const full = path.join(dir, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, body);
  }
  return dir;
}

test("absent crates are skipped, over-budget crates fail", () => {
  const dir = tree({
    "crates/a/src/lib.rs": "fn a() {}\nfn b() {}\nfn c() {}\n",
    "crates/a/tests/t.rs": "fn not_counted() {}\n",
  });
  try {
    const r = measure(dir, [
      { crate: "crates/a", total: 2 },
      { crate: "crates/missing", total: 10 },
    ]);
    assert.equal(r[0].status, "over");
    assert.equal(r[0].total, 3);
    assert.equal(r[1].status, "absent");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("per-file budgets exempt the shared modules but cap them together", () => {
  const big = "fn x() {}\n".repeat(5);
  const dir = tree({
    "crates/p/src/lib.rs": big,
    "crates/p/src/anthropic.rs": big,
  });
  try {
    const [r] = measure(dir, [{ crate: "crates/p", perFile: 4, shared: ["lib.rs"], sharedTotal: 10 }]);
    assert.equal(r.status, "over");
    assert.deepEqual(r.over, ["src/anthropic.rs: 5 lines > 4"]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
