import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const got = readFileSync('ANSWER.txt', 'utf8').trim();
assert.equal(got, '4');
