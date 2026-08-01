import { describe, expect, it } from "vitest";
import {
  formatReadyStatus,
  formatReadySteps,
  formatUnifyPreview,
  formatUnifyStatus,
} from "../src/product/formatReports.js";
import type { ReadyReport, UnifyReport, UserStory } from "@mzwin/kit-core";

const story: UserStory = {
  id: "new-repo-agents",
  title: "Make this repo agent-ready",
  who: "you",
  pain: "pain",
  win: "win",
  primary: "kit ready --write",
  next: [],
};

function ready(partial: Partial<ReadyReport>): ReadyReport {
  return {
    dryRun: true,
    projectDir: "/tmp/proj",
    story,
    packName: "web-app",
    recommendSummary: "looks like a web app",
    steps: [
      { id: "pack-install", status: "planned", detail: "Would install" },
      { id: "link", status: "planned", detail: "Would link" },
    ],
    doctorOk: null,
    notes: ["Dry-run. Pass --write to apply."],
    complete: false,
    ...partial,
  };
}

function unify(partial: Partial<UnifyReport>): UnifyReport {
  return {
    projectDir: "/tmp/proj",
    kitHome: "/tmp/.kit",
    dryRun: true,
    includeNoise: false,
    scanned: 40,
    unique: 22,
    noiseCount: 18,
    keeperCount: 4,
    alreadyInLibrary: 1,
    adoptReady: 3,
    adopted: 0,
    linked: 0,
    candidates: [],
    keepers: [
      {
        name: "code-review",
        score: 92,
        grade: "S",
        description: "Review PRs",
        sources: [],
        fixes: [],
        signals: [],
        kitReady: true,
        isNoise: false,
        noiseReasons: [],
        isKeeper: true,
        inLibrary: false,
        normalized: {
          name: "code-review",
          description: "Review PRs",
          version: "0.1.0",
          compatibility: ["*"],
          body: "",
          content: "",
          fixes: [],
          kitReady: true,
        },
        bestSourceDir: "/tmp/a",
      },
    ],
    noiseSample: [],
    adoptedNames: [],
    notes: ["Dry-run only."],
    ...partial,
  };
}

describe("formatReadyStatus", () => {
  it("asks for write after dry-run", () => {
    expect(formatReadyStatus(ready({}))).toContain("y write");
    expect(formatReadyStatus(ready({}))).toContain("web-app");
  });

  it("reports complete writes", () => {
    expect(
      formatReadyStatus(
        ready({ dryRun: false, complete: true, notes: [] }),
      ),
    ).toContain("complete");
  });

  it("lists steps", () => {
    const lines = formatReadySteps(ready({}));
    expect(lines[0]).toMatch(/^~/);
    expect(lines.some((l) => l.includes("Dry-run"))).toBe(true);
  });
});

describe("formatUnifyStatus", () => {
  it("summarizes plan counts", () => {
    const line = formatUnifyStatus(unify({}));
    expect(line).toContain("40 scanned");
    expect(line).toContain("y write");
  });

  it("previews keepers", () => {
    const lines = formatUnifyPreview(unify({}));
    expect(lines[0]).toContain("keepers");
    expect(lines.some((l) => l.includes("code-review"))).toBe(true);
  });
});
