#!/usr/bin/env bash
# Baseline checks do not require Claude. --host explicitly requires its real runner.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
fail() { printf 'check-native: %s\n' "$*" >&2; exit 1; }
host=false
case "${1:-}" in
  '') [[ $# -eq 0 ]] || fail 'Usage: bash scripts/check-native.sh [--host]' ;;
  --host) [[ $# -eq 1 ]] || fail 'Usage: bash scripts/check-native.sh [--host]'; host=true ;;
  *) fail 'Usage: bash scripts/check-native.sh [--host]' ;;
esac
for tool in node cargo python3; do
  command -v "$tool" >/dev/null || fail "Missing $tool. Run setup-native.sh after provisioning its prerequisites."
done
plugin=crates/kit-cli/claude-plugin
if "$host"; then
  command -v claude >/dev/null || fail '--host requires Claude Code on PATH; provision it explicitly. Baseline checks need no Claude.'
  [[ -f "$plugin/.claude-plugin/types/tsconfig.json" ]] || fail 'Installed-Claude declarations are missing. Load this source Mod with the installed host to generate them; see docs/dev/NATIVE-ENVIRONMENT.md.'
  [[ -f node_modules/typescript/bin/tsc ]] || fail 'Project TypeScript is missing. Run bash scripts/setup-native.sh.'
fi
printf 'Packaging checks (Claude discovery excluded)\n'
node --test --test-name-pattern='packaged agent declarations' scripts/claude-plugin.test.mjs
python3 scripts/package-native-alpha.test.py
node --test scripts/native-lifecycle.test.mjs
cargo test --locked -p kitctl native::tests
cargo test --locked -p kitctl --test native_cli
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
if "$host"; then
  claude --version
  claude plugin validate "$plugin" --strict --json
  # Rollout-off, missing host or test errors fail this script; never turn them into skips.
  claude plugin test "$plugin"
  node node_modules/typescript/bin/tsc --noEmit -p "$plugin/tsconfig.json"
  KIT_TEST_CLAUDE="$(command -v claude)" node --test scripts/claude-plugin.test.mjs
  printf 'Host automated checks passed; interactive rendering and live agent execution remain separate QA.\n'
else
  printf 'Baseline checks passed. NOT RUN: Claude validation/tests/discovery, generated-type check, interactive QA.\n'
fi
