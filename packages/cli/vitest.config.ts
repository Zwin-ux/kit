import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/e2e/**/*.e2e.test.ts"],
    environment: "node",
    // Each test spawns the compiled CLI several times; keep generous budgets
    // so slow CI runners (Windows especially) never flake on timing.
    testTimeout: 120_000,
    hookTimeout: 60_000,
  },
});
