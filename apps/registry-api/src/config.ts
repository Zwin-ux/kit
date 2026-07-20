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
    publicBaseUrl:
      process.env.PUBLIC_BASE_URL ??
      (process.env.RAILWAY_PUBLIC_DOMAIN
        ? `https://${process.env.RAILWAY_PUBLIC_DOMAIN}`
        : "https://kit-registry-production.up.railway.app"),
  };
}

export function authConfigured(): boolean {
  const c = getConfig();
  return Boolean(c.githubClientId && c.githubClientSecret);
}
