import assert from 'node:assert/strict';
import { compareVersions } from './src/version.mjs';
assert.equal(compareVersions('1.10.0', '1.9.0'), 1);
assert.equal(compareVersions('2.0.0', '2.0.0'), 0);
assert.equal(compareVersions('0.9.1', '0.10.0'), -1);
