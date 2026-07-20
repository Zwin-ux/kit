/**
 * Runtime config from environment.
 * Secrets must come from Railway variables — never from git.
 */

export function getConfig() {
  return {
    port: Number(process.env.PORT ?? 3000),
    githubAppId: process.env.GITHUB_APP_ID ?? "",
    githubClientId: process.env.GITHUB_CLIENT_ID ?? "",
    githubClientSecret: process.env.GITHUB_CLIENT_SECRET ?? "",
    githubWebhookSecret: process.env.GITHUB_WEBHOOK_SECRET ?? "",
    /**
     * PEM private key for GitHub App JWTs (installation tokens).
     * Prefer GITHUB_APP_PRIVATE_KEY_BASE64 on Railway (avoids multiline issues).
     * Raw PEM via GITHUB_APP_PRIVATE_KEY also works (use \n for newlines if needed).
     */
    githubAppPrivateKey: resolvePrivateKey(),
    publicBaseUrl:
      process.env.PUBLIC_BASE_URL ??
      (process.env.RAILWAY_PUBLIC_DOMAIN
        ? `https://${process.env.RAILWAY_PUBLIC_DOMAIN}`
        : "https://kit-registry-production.up.railway.app"),
  };
}

function resolvePrivateKey(): string {
  const b64 = process.env.GITHUB_APP_PRIVATE_KEY_BASE64?.trim();
  if (b64) {
    try {
      return Buffer.from(b64, "base64").toString("utf8");
    } catch {
      return "";
    }
  }
  const raw = process.env.GITHUB_APP_PRIVATE_KEY ?? "";
  if (!raw) return "";
  // Support single-line env values with escaped newlines.
  return raw.replace(/\\n/g, "\n");
}

export function authConfigured(): boolean {
  const c = getConfig();
  return Boolean(c.githubClientId && c.githubClientSecret);
}

/** True when App can mint installation tokens (App ID + private key). */
export function appCredentialsConfigured(): boolean {
  const c = getConfig();
  return Boolean(
    c.githubAppId &&
      c.githubAppPrivateKey.includes("BEGIN") &&
      c.githubAppPrivateKey.includes("PRIVATE KEY"),
  );
}
