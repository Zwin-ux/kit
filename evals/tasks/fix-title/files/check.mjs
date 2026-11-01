import assert from 'node:assert/strict';
import { titleCase } from './src/title.mjs';
assert.equal(titleCase('hello WORLD'), 'Hello World');
