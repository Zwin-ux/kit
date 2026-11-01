import assert from 'node:assert/strict';
import { shout } from './src/cli.mjs';
assert.equal(shout(['--upper', 'hi', 'there']), 'HI THERE');
assert.equal(shout(['hi']), 'hi');
