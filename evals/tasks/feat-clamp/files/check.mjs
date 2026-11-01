import assert from 'node:assert/strict';
import { clamp, square } from './src/num.mjs';
assert.equal(clamp(5, 0, 3), 3);
assert.equal(clamp(-1, 0, 3), 0);
assert.equal(clamp(2, 0, 3), 2);
assert.equal(square(3), 9);
