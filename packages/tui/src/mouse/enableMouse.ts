/**
 * DECSET mouse tracking for click-to-select.
 * Always pair enable with disable on exit so the shell is not left broken.
 */

const ENABLE = "\x1b[?1000h\x1b[?1002h\x1b[?1006h";
const DISABLE = "\x1b[?1006l\x1b[?1002l\x1b[?1000l";

let enabled = false;

export function mouseAllowed(): boolean {
  if (process.env.KIT_NO_MOUSE === "1") return false;
  if (!process.stdin.isTTY || !process.stdout.isTTY) return false;
  return true;
}

export function enableMouse(): void {
  if (!mouseAllowed() || enabled) return;
  try {
    process.stdout.write(ENABLE);
    enabled = true;
  } catch {
    // ignore
  }
}

export function disableMouse(): void {
  if (!enabled) return;
  try {
    process.stdout.write(DISABLE);
  } catch {
    // ignore
  }
  enabled = false;
}

/** Register process exit hooks once. */
export function installMouseCleanup(): void {
  if (!mouseAllowed()) return;
  const clean = () => disableMouse();
  process.on("exit", clean);
  process.on("SIGINT", () => {
    clean();
    process.exit(130);
  });
  process.on("SIGTERM", () => {
    clean();
    process.exit(143);
  });
}
