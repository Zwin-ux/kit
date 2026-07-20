import { getConfig } from "./config.js";

const GITHUB_API = "https://api.github.com";
const GITHUB_LOGIN = "https://github.com";

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface TokenResponse {
  access_token?: string;
  token_type?: string;
  scope?: string;
  error?: string;
  error_description?: string;
  error_uri?: string;
  interval?: number;
}

export interface GitHubUser {
  id: number;
  login: string;
  name: string | null;
  avatar_url: string;
  html_url: string;
}

function requireOAuthConfig(): { clientId: string; clientSecret: string } {
  const { githubClientId, githubClientSecret } = getConfig();
  if (!githubClientId || !githubClientSecret) {
    throw new Error(
      "GitHub OAuth is not configured. Set GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET.",
    );
  }
  return { clientId: githubClientId, clientSecret: githubClientSecret };
}

/**
 * Start GitHub App device flow (for CLI / TUI).
 * https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
 */
export async function startDeviceFlow(): Promise<DeviceCodeResponse> {
  const { clientId } = requireOAuthConfig();
  const body = new URLSearchParams({
    client_id: clientId,
    // Identity scopes for sign-in. Repo scopes can be added later for publish.
    scope: "read:user user:email",
  });

  const res = await fetch(`${GITHUB_LOGIN}/login/device/code`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GitHub device code failed (${res.status}): ${text}`);
  }

  return (await res.json()) as DeviceCodeResponse;
}

/**
 * Poll device flow token endpoint.
 */
export async function pollDeviceToken(
  deviceCode: string,
): Promise<TokenResponse> {
  const { clientId } = requireOAuthConfig();
  const body = new URLSearchParams({
    client_id: clientId,
    device_code: deviceCode,
    grant_type: "urn:ietf:params:oauth:grant-type:device_code",
  });

  const res = await fetch(`${GITHUB_LOGIN}/login/oauth/access_token`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GitHub token poll failed (${res.status}): ${text}`);
  }

  return (await res.json()) as TokenResponse;
}

/**
 * Exchange web OAuth code for access token.
 */
export async function exchangeCodeForToken(code: string): Promise<TokenResponse> {
  const { clientId, clientSecret } = requireOAuthConfig();
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    code,
  });

  const res = await fetch(`${GITHUB_LOGIN}/login/oauth/access_token`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GitHub code exchange failed (${res.status}): ${text}`);
  }

  return (await res.json()) as TokenResponse;
}

export async function fetchGitHubUser(
  accessToken: string,
): Promise<GitHubUser> {
  const res = await fetch(`${GITHUB_API}/user`, {
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${accessToken}`,
      "User-Agent": "kit-registry-api",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GitHub user fetch failed (${res.status}): ${text}`);
  }

  return (await res.json()) as GitHubUser;
}

export function buildAuthorizeUrl(state: string): string {
  const { githubClientId } = getConfig();
  const { publicBaseUrl } = getConfig();
  const params = new URLSearchParams({
    client_id: githubClientId,
    redirect_uri: `${publicBaseUrl}/auth/github/callback`,
    scope: "read:user user:email",
    state,
  });
  return `${GITHUB_LOGIN}/login/oauth/authorize?${params.toString()}`;
}
