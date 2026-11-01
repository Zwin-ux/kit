import assert from 'node:assert/strict';
import { isEmail } from './src/email.mjs';
assert.ok(isEmail('ana+news@example.com'));
assert.ok(isEmail('a.b@x.io'));
assert.ok(!isEmail('nope'));
