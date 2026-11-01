import assert from 'node:assert/strict';
import { loadConfig, DEFAULTS } from './src/config.mjs';
assert.deepEqual(loadConfig({ port: 9000 }), { port: 9000, host: 'localhost' });
assert.deepEqual(DEFAULTS, { port: 8080, host: 'localhost' });
