export function canInvite(email) {
  return email.includes('@') && !email.endsWith('@');
}

export function canRegister(email, age) {
  return email.includes('@') && !email.endsWith('@') && age >= 13;
}
