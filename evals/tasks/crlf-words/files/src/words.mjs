export function countWords(text) {
  return text.split(/[ \n]+/).filter(Boolean).length;
}
