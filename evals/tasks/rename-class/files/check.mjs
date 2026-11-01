import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Basket } from './src/cart.mjs';
import { newCart } from './src/shop.mjs';
assert.ok(newCart() instanceof Basket);
assert.equal(new Basket().add(1).items.length, 1);
for (const f of ['src/cart.mjs', 'src/shop.mjs']) assert.ok(!/\bCart\b/.test(readFileSync(f, 'utf8')), `${f} still says Cart`);
