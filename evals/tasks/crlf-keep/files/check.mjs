import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { settings } from './src/settings.mjs';
assert.equal(settings.timeout, 60);
const raw = readFileSync('src/settings.mjs', 'utf8');
assert.ok(!/[^\r]\n/.test(raw), 'every line must still end in CRLF');
