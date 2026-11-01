import assert from 'node:assert/strict';
import { countWords } from './src/words.mjs';
assert.equal(countWords('a b\r\nc'), 3);
assert.equal(countWords('a\r\n\r\nb'), 2);
