import assert from 'node:assert/strict';
import { loadAll } from './src/load.mjs';
const out = await loadAll([async () => 1, async () => 2]);
assert.deepEqual(out, [1, 2]);
