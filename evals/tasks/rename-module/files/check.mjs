import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { report } from './src/report.mjs';
import { count } from './src/count.mjs';
assert.ok(existsSync('src/accounts.mjs'));
assert.ok(!existsSync('src/users.mjs'));
assert.equal(report(), 'ana, sam');
assert.equal(count(), 2);
