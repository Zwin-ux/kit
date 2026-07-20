import { createHmac, timingSafeEqual } from "node:crypto";
import { getConfig } from "./config.js";

/**
 * Verify GitHub webhook signature (sha256).
 * https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries
 */
export function verifyGitHubSignature(
  rawBody: string,
  signatureHeader: string | undefined,
): boolean {
  const secret = getConfig().githubWebhookSecret;
  if (!secret) {
    return false;
  }
  if (!signatureHeader || !signatureHeader.startsWith("sha256=")) {
    return false;
  }

  const expected = createHmac("sha256", secret).update(rawBody).digest("hex");
  const provided = signatureHeader.slice("sha256=".length);

  try {
    const a = Buffer.from(expected, "utf8");
    const b = Buffer.from(provided, "utf8");
    if (a.length !== b.length) return false;
    return timingSafeEqual(a, b);
  } catch {
    return false;
  }
}
