import assert from 'node:assert/strict';
import { escapeHtml } from './src/html.mjs';
assert.equal(escapeHtml(`<a href="x">'&'</a>`), '&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;');
