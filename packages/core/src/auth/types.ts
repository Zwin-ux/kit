export interface KitAuthUser {
  id: number;
  login: string;
  name: string | null;
  avatar_url: string;
  html_url: string;
}

export interface KitAuthSession {
  version: 1;
  accessToken: string;
  tokenType: string;
  scope: string;
  user: KitAuthUser;
  /** ISO timestamp when the session was saved. */
  loggedInAt: string;
  /** Registry base URL used for this login. */
  registryUrl: string;
}

export type AuthResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string };
