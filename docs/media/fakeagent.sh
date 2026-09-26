#!/bin/sh
# A scripted stand-in for an agent CLI, for recordings only: the 12-way
# fleet tape, and recordings made where a real agent cannot run (inside a
# Claude Code session). Symlink it as claude, codex, grok and ollama in a
# throwaway PATH. It answers the probes kit makes (version, login, mcp,
# model list), then writes a small greet script and its test in the
# worktree it is started in. Never put it on a real PATH.
name=$(basename "$0")
case "$1" in
  --version|-V|version)
    case $name in
      claude) echo "2.1.283 (Claude Code)" ;;
      codex) echo "codex-cli 0.155.0" ;;
      grok) echo "grok 1.0.34" ;;
      ollama) echo "ollama version is 0.12.3" ;;
    esac
    exit 0 ;;
  auth) echo '{"loggedIn": true}'; exit 0 ;;
  login) echo "Logged in using ChatGPT" >&2; exit 0 ;;
  mcp) echo "Added stdio MCP server $4 to user config"; exit 0 ;;
  list) echo "NAME ID SIZE MODIFIED"; echo "llama3.2:latest a80c4f17acd5 2.0GB now"; exit 0 ;;
esac
[ -t 0 ] || cat >/dev/null
d=${KIT_FAKE_SECS:-6}
echo "Reading package.json and the test setup."; sleep $((d / 3))
mkdir -p src test
cat > src/greet.js <<'JS'
export function greet(name = "world") {
  return `hello, ${name}`;
}

if (import.meta.url === `file://${process.argv[1]}`) console.log(greet(process.argv[2]));
JS
cat > test/greet.test.js <<'JS'
import { test } from "node:test";
import assert from "node:assert/strict";
import { greet } from "../src/greet.js";

test("greets the world by default", () => assert.equal(greet(), "hello, world"));
test("greets a name", () => assert.equal(greet("Ada"), "hello, Ada"));
JS
echo "Added src/greet.js and test/greet.test.js."; sleep $((d / 3))
echo "Done: 2 files changed."; sleep $((d / 3))
