import assert from 'node:assert/strict';
import { slugify } from './src/slug.mjs';
assert.equal(slugify('Hello, World!'), 'hello-world');
assert.equal(slugify('  Kit 2  rocks '), 'kit-2-rocks');
