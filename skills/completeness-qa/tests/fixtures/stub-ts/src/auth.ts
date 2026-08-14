export type User = { id: string };
export interface Session {
  id: string;
}

export function login() {
  throw new Error("not implemented");
}

export function logout() {
  return true;
}

export function sessionId() {
  return "s";
}
