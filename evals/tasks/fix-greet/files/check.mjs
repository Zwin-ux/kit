import assert from 'node:assert/strict';
import { greet } from './src/greet.mjs';
assert.equal(greet(' Ana '), 'Hello, Ana!');
assert.equal(greet(), 'Hello, friend!');
