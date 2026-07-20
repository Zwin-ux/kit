/**
 * Strip leading `--` so npm/pnpm pass-through works:
 *   pnpm kit -- tui   →  ["--", "tui"]  →  ["tui"]
 *   pnpm kit tui      →  ["tui"]
 * Does NOT strip later `--` (e.g. kit unify --write --link).
 */
export function normalizeArgv(argv: string[]): string[] {
  if (argv[0] === "--") return argv.slice(1);
  return argv;
}
