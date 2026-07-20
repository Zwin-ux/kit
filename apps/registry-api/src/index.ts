import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { randomBytes } from "node:crypto";
import { SEED_PACKS, SEED_SKILLS } from "./catalog.js";
import {
  appCredentialsConfigured,
  authConfigured,
  getConfig,
} from "./config.js";
import {
  buildAuthorizeUrl,
  exchangeCodeForToken,
  fetchGitHubUser,
  pollDeviceToken,
  startDeviceFlow,
} from "./github.js";
import { verifyGitHubSignature } from "./webhook.js";

const app = new Hono();

// In-memory OAuth state for browser flow (MVP). Replace with Redis/DB later.
const oauthStates = new Map<string, number>();

app.use(
  "*",
  cors({
    origin: "*",
    allowMethods: ["GET", "HEAD", "POST", "OPTIONS"],
    allowHeaders: ["Content-Type", "Authorization", "X-Hub-Signature-256"],
  }),
);

app.get("/", (c) =>
  c.json({
    name: "kit-registry-api",
    version: "0.2.0",
    authConfigured: authConfigured(),
    appCredentialsConfigured: appCredentialsConfigured(),
    docs: {
      health: "GET /health",
      packs: "GET /v1/packs",
      pack: "GET /v1/packs/:name",
      skills: "GET /v1/skills",
      skill: "GET /v1/skills/:name",
      search: "GET /v1/search?q=",
      deviceStart: "POST /auth/github/device/start",
      devicePoll: "POST /auth/github/device/poll",
      browserLogin: "GET /auth/github/login",
      browserCallback: "GET /auth/github/callback",
      webhook: "POST /webhooks/github",
    },
    note: "Catalog is public. Auth uses GitHub App Kit-skills (device + browser).",
  }),
);

app.get("/health", (c) => {
  const cfg = getConfig();
  return c.json({
    ok: true,
    service: "kit-registry-api",
    time: new Date().toISOString(),
    packs: SEED_PACKS.length,
    skills: SEED_SKILLS.length,
    authConfigured: authConfigured(),
    appCredentialsConfigured: appCredentialsConfigured(),
    githubAppId: cfg.githubAppId || null,
    githubClientId: cfg.githubClientId
      ? `${cfg.githubClientId.slice(0, 8)}…`
      : null,
  });
});

// --- Catalog (public) ---

app.get("/v1/packs", (c) => {
  const tag = c.req.query("tag");
  const projectType = c.req.query("projectType");
  let packs = SEED_PACKS;
  if (tag) packs = packs.filter((p) => p.tags.includes(tag));
  if (projectType) {
    packs = packs.filter((p) => p.projectTypes.includes(projectType));
  }
  return c.json({ packs, count: packs.length });
});

app.get("/v1/packs/:name", (c) => {
  const name = c.req.param("name");
  const pack = SEED_PACKS.find((p) => p.name === name);
  if (!pack) return c.json({ error: `Pack not found: ${name}` }, 404);
  const skills = SEED_SKILLS.filter((s) => pack.skills.includes(s.name));
  return c.json({ pack, skills });
});

app.get("/v1/skills", (c) => {
  const agent = c.req.query("agent");
  let skills = SEED_SKILLS;
  if (agent) {
    skills = skills.filter((s) => s.compatibility.includes(agent));
  }
  return c.json({ skills, count: skills.length });
});

app.get("/v1/skills/:name", (c) => {
  const name = c.req.param("name");
  const skill = SEED_SKILLS.find((s) => s.name === name);
  if (!skill) return c.json({ error: `Skill not found: ${name}` }, 404);
  const packs = SEED_PACKS.filter((p) => p.skills.includes(skill.name)).map(
    (p) => p.name,
  );
  return c.json({ skill, packs });
});

app.get("/v1/search", (c) => {
  const q = (c.req.query("q") ?? "").trim().toLowerCase();
  if (!q) return c.json({ error: "Query param q is required." }, 400);
  const packs = SEED_PACKS.filter(
    (p) =>
      p.name.includes(q) ||
      p.title.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.tags.some((t) => t.includes(q)),
  );
  const skills = SEED_SKILLS.filter(
    (s) =>
      s.name.includes(q) || s.description.toLowerCase().includes(q),
  );
  return c.json({ q, packs, skills, count: packs.length + skills.length });
});

// --- GitHub auth (device + browser) ---

app.post("/auth/github/device/start", async (c) => {
  if (!authConfigured()) {
    return c.json(
      {
        error: "GitHub OAuth not configured",
        hint: "Set GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET on Railway",
      },
      503,
    );
  }
  try {
    const data = await startDeviceFlow();
    return c.json({
      device_code: data.device_code,
      user_code: data.user_code,
      verification_uri: data.verification_uri,
      expires_in: data.expires_in,
      interval: data.interval,
      message: `Open ${data.verification_uri} and enter code ${data.user_code}`,
    });
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return c.json({ error: detail }, 502);
  }
});

app.post("/auth/github/device/poll", async (c) => {
  if (!authConfigured()) {
    return c.json({ error: "GitHub OAuth not configured" }, 503);
  }
  let deviceCode: string | undefined;
  try {
    const body = await c.req.json<{ device_code?: string }>();
    deviceCode = body.device_code;
  } catch {
    return c.json({ error: "JSON body with device_code required" }, 400);
  }
  if (!deviceCode) {
    return c.json({ error: "device_code is required" }, 400);
  }

  try {
    const token = await pollDeviceToken(deviceCode);
    if (token.error) {
      // pending / slow_down / expired / access_denied
      return c.json(
        {
          status: token.error,
          error_description: token.error_description,
          interval: token.interval,
        },
        token.error === "authorization_pending" || token.error === "slow_down"
          ? 202
          : 400,
      );
    }
    if (!token.access_token) {
      return c.json({ error: "No access_token in response" }, 502);
    }
    const user = await fetchGitHubUser(token.access_token);
    return c.json({
      status: "complete",
      access_token: token.access_token,
      token_type: token.token_type ?? "bearer",
      scope: token.scope ?? "",
      user: {
        id: user.id,
        login: user.login,
        name: user.name,
        avatar_url: user.avatar_url,
        html_url: user.html_url,
      },
    });
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return c.json({ error: detail }, 502);
  }
});

app.get("/auth/github/login", (c) => {
  if (!authConfigured()) {
    return c.json({ error: "GitHub OAuth not configured" }, 503);
  }
  const state = randomBytes(16).toString("hex");
  oauthStates.set(state, Date.now() + 10 * 60 * 1000);
  // prune expired
  for (const [k, exp] of oauthStates) {
    if (exp < Date.now()) oauthStates.delete(k);
  }
  return c.redirect(buildAuthorizeUrl(state), 302);
});

app.get("/auth/github/callback", async (c) => {
  if (!authConfigured()) {
    return c.html("<h1>GitHub OAuth not configured</h1>", 503);
  }
  const code = c.req.query("code");
  const state = c.req.query("state");
  const err = c.req.query("error");
  if (err) {
    return c.html(`<h1>Authorization failed</h1><p>${err}</p>`, 400);
  }
  if (!code || !state) {
    return c.html("<h1>Missing code or state</h1>", 400);
  }
  const exp = oauthStates.get(state);
  oauthStates.delete(state);
  if (!exp || exp < Date.now()) {
    return c.html("<h1>Invalid or expired state</h1>", 400);
  }

  try {
    const token = await exchangeCodeForToken(code);
    if (token.error || !token.access_token) {
      return c.html(
        `<h1>Token exchange failed</h1><p>${token.error_description ?? token.error}</p>`,
        400,
      );
    }
    const user = await fetchGitHubUser(token.access_token);
    // MVP: show success page. Later: set session cookie / redirect to TUI deep link.
    return c.html(`<!doctype html>
<html><head><meta charset="utf-8"><title>Kit · signed in</title>
<style>
  body{font-family:system-ui,sans-serif;max-width:32rem;margin:3rem auto;padding:0 1rem}
  code{background:#f4f4f4;padding:.1rem .3rem;border-radius:4px}
</style></head>
<body>
  <h1>Signed in to Kit</h1>
  <p>GitHub user: <strong>${escapeHtml(user.login)}</strong></p>
  <p>You can close this tab and return to the Kit CLI or TUI.</p>
  <p><small>Device flow is preferred for terminal clients:
  <code>POST /auth/github/device/start</code></small></p>
</body></html>`);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return c.html(`<h1>Sign-in error</h1><p>${escapeHtml(detail)}</p>`, 502);
  }
});

// --- Webhooks ---

app.post("/webhooks/github", async (c) => {
  const raw = await c.req.text();
  const signature = c.req.header("x-hub-signature-256");
  const event = c.req.header("x-github-event") ?? "unknown";
  const delivery = c.req.header("x-github-delivery") ?? "";

  const secret = getConfig().githubWebhookSecret;
  if (secret) {
    if (!verifyGitHubSignature(raw, signature)) {
      return c.json({ error: "Invalid signature" }, 401);
    }
  } else {
    console.warn(
      "[webhook] GITHUB_WEBHOOK_SECRET not set — accepting without verify (dev only)",
    );
  }

  // Acknowledge ping and other events without side effects yet.
  console.log(
    `[webhook] event=${event} delivery=${delivery} bytes=${raw.length}`,
  );

  if (event === "ping") {
    return c.json({ ok: true, message: "pong from kit-registry-api" });
  }

  return c.json({ ok: true, received: event });
});

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

const port = getConfig().port;
console.log(
  `kit-registry-api listening on :${port} (authConfigured=${authConfigured()}, appKey=${appCredentialsConfigured()})`,
);

serve({
  fetch: app.fetch,
  port,
});
