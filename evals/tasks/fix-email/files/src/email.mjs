export function isEmail(s) {
  return /^[a-z0-9.]+@[a-z0-9.-]+\.[a-z]+$/i.test(s);
}
