export function invoiceTotal(cents) {
  return `Total: $${(cents / 100).toFixed(2)}`;
}
