import { describe, expect, it } from "vitest";
import { parseSkillMd } from "../src/parse/skillMd.js";
import { parseAndValidateSkillMd } from "../src/loadSkill.js";

describe("parseSkillMd", () => {
  it("parses YAML front matter and body", () => {
    const result = parseSkillMd(`---
name: demo
description: A demo skill.
version: 0.0.1
compatibility:
  - codex
---

# Hello

Body text.
`);

    expect(result.ok).toBe(true);
    if (!result.ok) return;

    expect(result.value.frontMatter.name).toBe("demo");
    expect(result.value.body).toContain("# Hello");
    expect(result.value.body).toContain("Body text.");
  });

  it("rejects content without front matter markers", () => {
    const result = parseSkillMd("# No front matter\n");
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.issues[0]?.field).toBe("frontMatter");
  });

  it("rejects empty file", () => {
    const result = parseSkillMd("   \n");
    expect(result.ok).toBe(false);
  });
});

describe("parseAndValidateSkillMd", () => {
  it("returns a skill when content is valid", () => {
    const result = parseAndValidateSkillMd(`---
name: add-readme
description: Create a clear README for a new project.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
---

# Instructions

Do the work.
`);

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.skill.name).toBe("add-readme");
    expect(result.skill.compatibility).toEqual(["claude-code", "grok-build"]);
    expect(result.skill.body).toContain("Do the work.");
  });
});
