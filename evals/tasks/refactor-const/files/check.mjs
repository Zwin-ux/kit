import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { withTax, taxOf, TAX_RATE } from './src/tax.mjs';
assert.equal(TAX_RATE, 0.2);
assert.equal(withTax(100), 120);
assert.equal(taxOf(50), 10);
assert.equal(readFileSync('src/tax.mjs', 'utf8').split('0.2').length - 1, 1, '0.2 should appear once, in TAX_RATE');
