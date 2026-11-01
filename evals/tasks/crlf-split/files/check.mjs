import assert from 'node:assert/strict';
import { parseLines } from './src/lines.mjs';
assert.deepEqual(parseLines('a\r\nb\r\n'), ['a', 'b']);
assert.deepEqual(parseLines('a\nb\n'), ['a', 'b']);
