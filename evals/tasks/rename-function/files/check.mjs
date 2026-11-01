import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { calculateTotal } from './src/calc.mjs';
import { a } from './src/a.mjs';
import { b } from './src/b.mjs';
assert.equal(calculateTotal([2, 2]), 4);
assert.equal(a() + b(), 6);
for (const f of ['src/calc.mjs', 'src/a.mjs', 'src/b.mjs']) assert.ok(!/\bcalc\b(?!\.mjs)/.test(readFileSync(f, 'utf8')), `${f} still says calc`);
