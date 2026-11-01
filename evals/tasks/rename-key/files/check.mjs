import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { orders } from './src/orders.mjs';
import { totalItems } from './src/total.mjs';
assert.equal(totalItems(), 3);
assert.ok(orders.every((o) => 'quantity' in o && !('qty' in o)));
assert.ok(!readFileSync('src/total.mjs', 'utf8').includes('qty'));
