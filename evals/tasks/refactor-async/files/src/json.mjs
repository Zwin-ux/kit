import { readFile } from 'node:fs';

export function readJson(path, cb) {
  readFile(path, 'utf8', (err, text) => (err ? cb(err) : cb(null, JSON.parse(text))));
}
