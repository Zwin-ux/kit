import assert from 'node:assert/strict';
import { readJson } from './src/json.mjs';
assert.deepEqual(await readJson('data.json'), { ok: true });
await assert.rejects(readJson('missing.json'));
