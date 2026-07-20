import { getKitHome } from "../library/paths.js";
import {
  clearAuthSession,
  readAuthSession,
  writeAuthSession,
} from "./store.js";
import type { AuthResult, KitAuthSession, KitAuthUser } from "./types.js";

export const DEFAULT_REGISTRY_URL =
  "https://kit-registry-production.up.railway.app";

export function getRegistryUrl(): string {
  return (
    process.env.KIT_REGISTRY_URL?.trim() ||
    DEFAULT_REGISTRY_URL
  ).replace(/\/$/, "");
}

export interface DeviceStartPayload {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
  message?: string;
}

export interface LoginProgress {
  userCode: string;
  verificationUri: string;
  message: string;
  expiresIn: number;
}

export interface LoginOptions {
  kitHome?: string;
  registryUrl?: string;
  /** Called once after device codes are issued. */
  onPrompt?: (progress: LoginProgress) => void;
  /** Abort after this many ms (default 14 minutes). */
  timeoutMs?: number;
  /** Override poll interval (seconds). Default: server interval. */
  pollIntervalSeconds?: number;
}

/**
 * Full device-flow login against the Kit registry API.
 * Stores session at ~/.kit/auth.json on success.
 */
export async function loginWithDeviceFlow(
  options: LoginOptions = {},
): Promise<AuthResult<KitAuthSession>> {
  const kitHome = options.kitHome ?? getKitHome();
  const registryUrl = (options.registryUrl ?? getRegistryUrl()).replace(
    /\/$/,
    "",
  );
  const timeoutMs = options.timeoutMs ?? 14 * 60 * 1000;

  let start: DeviceStartPayload;
  try {
    const res = await fetch(`${registryUrl}/auth/github/device/start`, {
      method: "POST",
      headers: { Accept: "application/json" },
    });
    const data = (await res.json()) as DeviceStartPayload & {
      error?: string;
      hint?: string;
    };
    if (!res.ok) {
      return {
        ok: false,
        error: data.error
          ? `${data.error}${data.hint ? ` — ${data.hint}` : ""}`
          : `Device start failed (${res.status})`,
      };
    }
    if (!data.device_code || !data.user_code) {
      return { ok: false, error: "Device start response missing codes." };
    }
    start = data;
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot reach registry: ${detail}` };
  }

  options.onPrompt?.({
    userCode: start.user_code,
    verificationUri: start.verification_uri,
    message:
      start.message ??
      `Open ${start.verification_uri} and enter code ${start.user_code}`,
    expiresIn: start.expires_in,
  });

  const started = Date.now();
  let intervalMs =
    (options.pollIntervalSeconds ?? start.interval ?? 5) * 1000;
  // GitHub asks clients to wait at least the provided interval.
  if (intervalMs < 5000) intervalMs = 5000;

  while (Date.now() - started < timeoutMs) {
    await sleep(intervalMs);

    let poll: {
      status?: string;
      access_token?: string;
      token_type?: string;
      scope?: string;
      user?: KitAuthUser;
      error?: string;
      error_description?: string;
      interval?: number;
    };

    try {
      const res = await fetch(`${registryUrl}/auth/github/device/poll`, {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ device_code: start.device_code }),
      });
      poll = (await res.json()) as typeof poll;

      if (poll.interval && poll.interval * 1000 > intervalMs) {
        intervalMs = poll.interval * 1000;
      }

      if (poll.status === "complete" && poll.access_token && poll.user) {
        const session: KitAuthSession = {
          version: 1,
          accessToken: poll.access_token,
          tokenType: poll.token_type ?? "bearer",
          scope: poll.scope ?? "",
          user: poll.user,
          loggedInAt: new Date().toISOString(),
          registryUrl,
        };
        await writeAuthSession(session, kitHome);
        return { ok: true, value: session };
      }

      if (
        poll.status === "authorization_pending" ||
        poll.error === "authorization_pending"
      ) {
        continue;
      }

      if (poll.status === "slow_down" || poll.error === "slow_down") {
        intervalMs += 5000;
        continue;
      }

      if (poll.error || (poll.status && poll.status !== "complete")) {
        return {
          ok: false,
          error:
            poll.error_description ||
            poll.error ||
            poll.status ||
            "Login failed",
        };
      }
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      return { ok: false, error: `Poll failed: ${detail}` };
    }
  }

  return {
    ok: false,
    error: "Login timed out. Run kit login again and approve promptly.",
  };
}

export async function getLoggedInUser(
  kitHome: string = getKitHome(),
): Promise<AuthResult<KitAuthSession>> {
  const session = await readAuthSession(kitHome);
  if (!session) {
    return { ok: false, error: "Not logged in. Run: kit login" };
  }
  return { ok: true, value: session };
}

export async function logout(
  kitHome: string = getKitHome(),
): Promise<AuthResult<{ cleared: boolean }>> {
  return clearAuthSession(kitHome);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
