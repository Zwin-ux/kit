import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { listSkills } from "../src/library/library.js";
import { loadSkill } from "../src/loadSkill.js";
import { normalizeSkillMd, writeNormalizedSkill } from "../src/unify/normalize.js";
import { scoreUnifyCandidate } from "../src/unify/score.js";
import { runUnify } from "../src/unify/unify.js";

const tempDirs: string[] = [];

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("normalizeSkillMd", () => {
  it("repairs Claude-style skills missing compatibility and passes validate", async () => {
    const raw = `---
name: My Cool Skill
description: Does a long thing that goes on and on and on and should be shortened into at most two clean sentences for the schema. Extra junk here is dropped.
---

# Instructions

1. Do the work.
2. Ship it.
`;
    const n = normalizeSkillMd(raw, { folderName: "my-cool-skill" });
    expect(n.name).toBe("my-cool-skill");
    expect(n.compatibility).toContain("claude-code");
    expect(n.kitReady).toBe(true);

    const dir = await tempDir("kit-norm-");
    const skillDir = await writeNormalizedSkill(dir, n);
    const loaded = await loadSkill(skillDir);
    expect(loaded.ok).toBe(true);
  });

  it("builds front matter for pure markdown", async () => {
    const n = normalizeSkillMd(
      "# Just markdown\n\nDo the thing carefully with clear steps.\n\n1. Read.\n2. Write.\n",
      { folderName: "pure-md" },
    );
    expect(n.name).toBe("pure-md");
    expect(n.kitReady).toBe(true);
    const dir = await tempDir("kit-pure-");
    const skillDir = await writeNormalizedSkill(dir, n);
    const loaded = await loadSkill(skillDir);
    expect(loaded.ok).toBe(true);
  });
});

describe("scoreUnifyCandidate", () => {
  it("filters automation bulk as noise", () => {
    const n = normalizeSkillMd(
      `---
name: ably-automation
description: Connect to Ably.
---

Use Ably.
`,
      { folderName: "ably-automation" },
    );
    const s = scoreUnifyCandidate({
      normalized: n,
      sources: [
        {
          sourceDir: "/x",
          folderName: "ably-automation",
          harness: "codex",
          scope: "personal",
        },
      ],
      rawBody: "thin",
      wasKitShaped: false,
    });
    expect(s.isNoise).toBe(true);
    expect(s.isKeeper).toBe(false);
    expect(s.grade).toBe("D");
  });

  it("boosts multi-agent structured skills into keepers", () => {
    const body = `# Review

1. Read the diff.
2. Call out risks.
3. Suggest tests.
`;
    const n = normalizeSkillMd(
      `---
name: careful
description: Safety guardrails for destructive commands and prod ops.
version: 0.1.0
compatibility:
  - claude-code
  - codex
---

${body}
`,
      { folderName: "careful" },
    );
    const s = scoreUnifyCandidate({
      normalized: n,
      sources: [
        {
          sourceDir: "/a",
          folderName: "careful",
          harness: "claude-code",
          scope: "personal",
        },
        {
          sourceDir: "/b",
          folderName: "careful",
          harness: "codex",
          scope: "personal",
        },
      ],
      rawBody: body,
      wasKitShaped: true,
    });
    expect(s.isNoise).toBe(false);
    expect(s.isKeeper).toBe(true);
    expect(s.score).toBeGreaterThanOrEqual(70);
    expect(["S", "A"]).toContain(s.grade);
  });
});

describe("runUnify", () => {
  it("ranks multi-source over stubs and adopts keepers only", async () => {
    const kitHome = await tempDir("kit-unify-home-");
    const homeDir = await tempDir("kit-unify-user-");
    const projectDir = await tempDir("kit-unify-proj-");

    // Keeper: multi-agent + structure
    const claude = path.join(homeDir, ".claude", "skills", "vibe-ship");
    await mkdir(claude, { recursive: true });
    await writeFile(
      path.join(claude, "SKILL.md"),
      `---
name: vibe-ship
description: Ship a small change with a clear checklist.
version: 0.1.0
compatibility:
  - claude-code
  - codex
---

# Ship

1. Diff the work.
2. Test the critical path.
3. Open a PR with risks called out.
`,
      "utf8",
    );

    const codex = path.join(homeDir, ".codex", "skills", "vibe-ship");
    await mkdir(codex, { recursive: true });
    await writeFile(
      path.join(codex, "SKILL.md"),
      `---
name: vibe-ship
description: Ship a small change with a clear checklist.
version: 0.1.0
compatibility:
  - codex
---

# Ship

1. Diff.
2. Test.
3. PR.
`,
      "utf8",
    );

    // Noise: automation bulk
    const auto = path.join(homeDir, ".codex", "skills", "ably-automation");
    await mkdir(auto, { recursive: true });
    await writeFile(
      path.join(auto, "SKILL.md"),
      `---
name: ably-automation
description: Ably connector.
---

Use Ably.
`,
      "utf8",
    );

    const dry = await runUnify({
      kitHome,
      homeDir,
      projectDir,
      write: false,
      includeLibrary: false,
    });
    expect(dry.ok).toBe(true);
    if (!dry.ok) return;

    expect(dry.value.scanned).toBeGreaterThanOrEqual(3);
    expect(dry.value.noiseCount).toBeGreaterThanOrEqual(1);
    expect(dry.value.keepers.some((k) => k.name === "vibe-ship")).toBe(true);
    // automation should not appear as keeper
    expect(dry.value.keepers.some((k) => k.name.includes("automation"))).toBe(
      false,
    );
    // default listing excludes noise
    expect(
      dry.value.candidates.some((c) => c.name === "ably-automation"),
    ).toBe(false);

    const written = await runUnify({
      kitHome,
      homeDir,
      projectDir,
      write: true,
      includeLibrary: false,
      minScore: 70,
      top: 25,
    });
    expect(written.ok).toBe(true);
    if (!written.ok) return;
    expect(written.value.adopted).toBeGreaterThanOrEqual(1);
    expect(written.value.adoptedNames).toContain("vibe-ship");
    expect(written.value.adoptedNames.some((n) => n.includes("automation"))).toBe(
      false,
    );

    const listed = await listSkills({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value.some((s) => s.name === "vibe-ship")).toBe(true);
    expect(listed.value.some((s) => s.name.includes("automation"))).toBe(false);
  });
});
