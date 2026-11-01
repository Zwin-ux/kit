export function cartLine(name, cents) {
  return `${name}: $${(cents / 100).toFixed(2)}`;
}
