import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { cartLine } from './src/cart.mjs';
import { invoiceTotal } from './src/invoice.mjs';
import { formatPrice } from './src/money.mjs';
assert.equal(formatPrice(1999), '$19.99');
assert.equal(cartLine('Tea', 450), 'Tea: $4.50');
assert.equal(invoiceTotal(100), 'Total: $1.00');
for (const f of ['src/cart.mjs', 'src/invoice.mjs']) assert.ok(!readFileSync(f, 'utf8').includes('toFixed'), `${f} still formats prices itself`);
