import assert from 'node:assert/strict';
import { sortNumbers } from './src/sort.mjs';
assert.deepEqual(sortNumbers([10, 9, 1]), [1, 9, 10]);
assert.deepEqual(sortNumbers([3, -2, 20]), [-2, 3, 20]);
