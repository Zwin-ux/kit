# Scan rules (hand fallback)

Use only when `node scripts/inventory.mjs` cannot run. Match the script.

## Files

1. Git repo with a non-empty source diff → those files only (`mode: diff`).
2. Else walk `src/`, `lib/`, `crates/*/src/`, `app/`, `apps/*/src/` (`mode: tree`).
3. Extensions: `.ts` `.tsx` `.js` `.jsx` `.mjs` `.cjs` `.rs` `.py`.
4. Skip tests: `*.test.*`, `*.spec.*`, `*_test.rs`, `*_test.py`, `test_*.py`, paths under `tests/`, `__tests__/`, `benches/`, `examples/`.

## Public symbols

- JS/TS: `export function`, `export async function`, `export const/let name =` function/arrow, `export class` methods (not `#`/`_`), `exports.name` / `module.exports.name`. Skip `export type`, `export interface`, `export {`.
- Rust: `pub fn` / `pub async fn`. Skip `pub(crate)`, `tests/`, `#[cfg(test)]` modules.
- Python: module-level `def` / `async def` and class methods. Skip names starting with `_` and all dunders.

## Stub

Body (≤30 lines) is only: empty; `todo!()` / `unimplemented!()`; `pass`; `raise NotImplementedError`; sole `throw new Error(...)`; sole `return null` / `undefined` / `None` / `return;`; TODO/FIXME as the only content.

## Tested

Exact identifier as a word in any test file. False positives are fine.

## Status

`stub` if slop. Else `ok` if mentioned in tests, else `untested`.

## Verdict

| Verdict | When |
|---|---|
| `done` | public > 0, stub = 0, untested = 0 |
| `partial` | stub = 0, untested > 0 |
| `not_done` | stub > 0 |
| `empty` | public = 0 — never call this done |
