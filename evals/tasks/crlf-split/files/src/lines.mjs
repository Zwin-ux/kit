export function parseLines(text) {
  return text.split('\n').filter((l) => l.length > 0);
}
