import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { loadSkill, formatIssues } from "../src/loadSkill.js";
import { parseAndValidateSkillMd } from "../src/loadSkill.js";
import { validateSkill } from "../src/validate/skill.js";

const fixturesDir = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "fixtures",
);

describe("validateSkill", () => {
  it("accepts valid front matter", () => {
    const result = validateSkill(
      {
        name: "example-skill",
        description: "Does one useful thing.",
        version: "1.2.3",
        compatibility: ["claude-code", "codex"],
      },
      "# Body\n",
    );

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.skill.version).toBe("1.2.3");
  });

  it("rejects invalid name characters", () => {
    const result = validateSkill(
      {
        name: "Not_Valid",
        description: "Bad name case.",
        version: "0.1.0",
        compatibility: ["codex"],
      },
      "",
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues.some((i) => i.field === "name")).toBe(true);
  });

  it("rejects non-semver version", () => {
    const result = validateSkill(
      {
        name: "ok-name",
        description: "Bad version.",
        version: "latest",
        compatibility: ["grok-build"],
      },
      "",
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues.some((i) => i.field === "version")).toBe(true);
  });

  it("rejects empty compatibility", () => {
    const result = validateSkill(
      {
        name: "ok-name",
        description: "Empty list.",
        version: "0.1.0",
        compatibility: [],
      },
      "",
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues.some((i) => i.field === "compatibility")).toBe(true);
  });

  it("rejects unknown agents", () => {
    const result = validateSkill(
      {
        name: "ok-name",
        description: "Unknown agent id.",
        version: "0.1.0",
        compatibility: ["not-a-real-agent"],
      },
      "",
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues[0]?.message).toMatch(/unknown agent/i);
  });

  it("rejects descriptions with more than two sentences", () => {
    const result = validateSkill(
      {
        name: "ok-name",
        description: "One. Two. Three.",
        version: "0.1.0",
        compatibility: ["codex"],
      },
      "",
    );

    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues.some((i) => i.field === "description")).toBe(true);
  });

  it("collects multiple missing field errors", () => {
    const result = parseAndValidateSkillMd(`---
name: only-name
---

body
`);

    expect(result.ok).toBe(false);
    if (result.ok) return;
    const fields = result.issues.map((i) => i.field);
    expect(fields).toContain("description");
    expect(fields).toContain("version");
    expect(fields).toContain("compatibility");
    expect(formatIssues(result.issues)).toContain("description:");
  });
});

describe("loadSkill", () => {
  it("loads a valid skill directory", async () => {
    const result = await loadSkill(
      path.join(fixturesDir, "valid-add-readme"),
    );

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.skill.name).toBe("add-readme");
    expect(result.skill.rootDir).toBe(
      path.resolve(fixturesDir, "valid-add-readme"),
    );
  });

  it("fails when SKILL.md is missing", async () => {
    const result = await loadSkill(path.join(fixturesDir, "no-such-skill"));
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues[0]?.message).toMatch(/SKILL\.md is missing/);
  });

  it("fails invalid fixtures with clear reasons", async () => {
    const badName = await loadSkill(path.join(fixturesDir, "invalid-bad-name"));
    expect(badName.ok).toBe(false);

    const emptyCompat = await loadSkill(
      path.join(fixturesDir, "invalid-empty-compat"),
    );
    expect(emptyCompat.ok).toBe(false);

    const badVersion = await loadSkill(
      path.join(fixturesDir, "invalid-bad-version"),
    );
    expect(badVersion.ok).toBe(false);

    const missing = await loadSkill(
      path.join(fixturesDir, "invalid-missing-fields"),
    );
    expect(missing.ok).toBe(false);
  });
});
