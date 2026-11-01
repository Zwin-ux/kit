import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { canInvite, canRegister } from './src/users.mjs';
assert.ok(canInvite('a@b'));
assert.ok(!canInvite('a@'));
assert.ok(canRegister('a@b', 13));
assert.ok(!canRegister('a@b', 12));
assert.equal(readFileSync('src/users.mjs', 'utf8').split("includes('@')").length - 1, 1, 'the check should exist once');
