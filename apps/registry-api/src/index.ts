import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { SEED_PACKS, SEED_SKILLS } from "./catalog.js";

const app = new Hono();

app.use(
  "*",
  cors({
    origin: "*",
    allowMethods: ["GET", "HEAD", "OPTIONS"],
  }),
);

app.get("/", (c) =>
  c.json({
    name: "kit-registry-api",
    version: "0.1.0",
    docs: {
      health: "GET /health",
      packs: "GET /v1/packs",
      pack: "GET /v1/packs/:name",
      skills: "GET /v1/skills",
      skill: "GET /v1/skills/:name",
      search: "GET /v1/search?q=",
    },
    note: "Public read-only catalog MVP. Auth + publish come next.",
  }),
);

app.get("/health", (c) =>
  c.json({
    ok: true,
    service: "kit-registry-api",
    time: new Date().toISOString(),
    packs: SEED_PACKS.length,
    skills: SEED_SKILLS.length,
  }),
);

app.get("/v1/packs", (c) => {
  const tag = c.req.query("tag");
  const projectType = c.req.query("projectType");
  let packs = SEED_PACKS;
  if (tag) {
    packs = packs.filter((p) => p.tags.includes(tag));
  }
  if (projectType) {
    packs = packs.filter((p) => p.projectTypes.includes(projectType));
  }
  return c.json({
    packs,
    count: packs.length,
  });
});

app.get("/v1/packs/:name", (c) => {
  const name = c.req.param("name");
  const pack = SEED_PACKS.find((p) => p.name === name);
  if (!pack) {
    return c.json({ error: `Pack not found: ${name}` }, 404);
  }
  const skills = SEED_SKILLS.filter((s) => pack.skills.includes(s.name));
  return c.json({ pack, skills });
});

app.get("/v1/skills", (c) => {
  const agent = c.req.query("agent");
  let skills = SEED_SKILLS;
  if (agent) {
    skills = skills.filter((s) => s.compatibility.includes(agent));
  }
  return c.json({
    skills,
    count: skills.length,
  });
});

app.get("/v1/skills/:name", (c) => {
  const name = c.req.param("name");
  const skill = SEED_SKILLS.find((s) => s.name === name);
  if (!skill) {
    return c.json({ error: `Skill not found: ${name}` }, 404);
  }
  const packs = SEED_PACKS.filter((p) => p.skills.includes(skill.name)).map(
    (p) => p.name,
  );
  return c.json({ skill, packs });
});

app.get("/v1/search", (c) => {
  const q = (c.req.query("q") ?? "").trim().toLowerCase();
  if (!q) {
    return c.json({ error: "Query param q is required." }, 400);
  }
  const packs = SEED_PACKS.filter(
    (p) =>
      p.name.includes(q) ||
      p.title.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.tags.some((t) => t.includes(q)),
  );
  const skills = SEED_SKILLS.filter(
    (s) =>
      s.name.includes(q) ||
      s.description.toLowerCase().includes(q),
  );
  return c.json({
    q,
    packs,
    skills,
    count: packs.length + skills.length,
  });
});

const port = Number(process.env.PORT ?? 3000);

console.log(`kit-registry-api listening on :${port}`);

serve({
  fetch: app.fetch,
  port,
});
