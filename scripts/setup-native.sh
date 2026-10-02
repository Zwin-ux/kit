#!/usr/bin/env bash
# Project dependency setup; does not install global tools or configure Claude.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
fail() { printf 'setup-native: %s\n' "$*" >&2; exit 1; }
[[ $# -eq 0 ]] || fail 'Usage: bash scripts/setup-native.sh'
for tool in node cargo rustc; do
  command -v "$tool" >/dev/null || fail "Missing $tool. Provision Node >=20 and Rust >=1.90 in your environment, then rerun."
done
node -e 'if (+process.versions.node.split(".")[0] < 20) process.exit(1)' || fail 'Node >=20 is required.'
rust_version=$(rustc --version)
node -e 'const m=process.argv[1].match(/^rustc (\d+)\.(\d+)\./); if (!m || +m[1]<1 || (+m[1]===1 && +m[2]<90)) process.exit(1)' "$rust_version" || fail 'Rust >=1.90 is required; select a compatible existing toolchain.'
cargo fmt --version >/dev/null || fail 'rustfmt is missing. Provision rustfmt for the selected toolchain (rustup users: rustup component add rustfmt).'
cargo clippy --version >/dev/null || fail 'Clippy is missing. Provision Clippy for the selected toolchain (rustup users: rustup component add clippy).'
if command -v pnpm >/dev/null && [[ $(pnpm --version) == 10.12.4 ]]; then
  pnpm_cmd=(pnpm)
elif command -v corepack >/dev/null; then
  pnpm_cmd=(corepack pnpm)
else
  fail 'pnpm 10.12.4 or Corepack is required. Provision it in the environment; no global installer is run here.'
fi
[[ $("${pnpm_cmd[@]}" --version) == 10.12.4 ]] || fail 'Expected pnpm 10.12.4 from package.json; correct the environment version.'
printf 'Node %s; %s; pnpm 10.12.4\n' "$(node --version)" "$rust_version"
# Ignore lifecycle scripts: native Mod verification needs no dependency build scripts.
"${pnpm_cmd[@]}" install --frozen-lockfile --ignore-scripts
cargo fetch --locked
printf 'Dependencies prepared. Run bash scripts/check-native.sh after integration is complete.\n'
