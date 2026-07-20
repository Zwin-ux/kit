import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  getLoggedInUser,
  logout,
  readAuthSession,
  writeAuthSession,
} from "../src/auth/mod.js";

const tempDirs: string[] = [];

async function tempHome(): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "kit-auth-"));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("auth session store", () => {
  it("returns not logged in when no session", async () => {
    const kitHome = await tempHome();
    const result = await getLoggedInUser(kitHome);
    expect(result.ok).toBe(false);
  });

  it("persists and clears a session", async () => {
    const kitHome = await tempHome();
    await writeAuthSession(
      {
        version: 1,
        accessToken: "test-token",
        tokenType: "bearer",
        scope: "read:user",
        user: {
          id: 1,
          login: "demo-user",
          name: "Demo",
          avatar_url: "https://example.com/a.png",
          html_url: "https://github.com/demo-user",
        },
        loggedInAt: new Date().toISOString(),
        registryUrl: "https://example.com",
      },
      kitHome,
    );

    const session = await readAuthSession(kitHome);
    expect(session?.user.login).toBe("demo-user");

    const who = await getLoggedInUser(kitHome);
    expect(who.ok).toBe(true);
    if (!who.ok) return;
    expect(who.value.user.login).toBe("demo-user");

    await logout(kitHome);
    expect(await readAuthSession(kitHome)).toBeNull();
  });
});
