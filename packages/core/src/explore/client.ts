import { getRegistryUrl } from "../auth/login.js";

export type ExploreResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string };

export interface RegistryPackSummary {
  name: string;
  title: string;
  description: string;
  version: string;
  tags: string[];
  projectTypes: string[];
  skillCount: number;
  skills: string[];
  publisher: string;
}

export interface RegistrySkillSummary {
  name: string;
  description: string;
  version: string;
  compatibility: string[];
}

export interface ExploreOptions {
  registryUrl?: string;
}

function baseUrl(options?: ExploreOptions): string {
  return (options?.registryUrl ?? getRegistryUrl()).replace(/\/$/, "");
}

async function getJson<T>(
  path: string,
  options?: ExploreOptions,
): Promise<ExploreResult<T>> {
  const url = `${baseUrl(options)}${path}`;
  try {
    const res = await fetch(url, {
      headers: { Accept: "application/json" },
    });
    const data = (await res.json()) as T & { error?: string };
    if (!res.ok) {
      return {
        ok: false,
        error:
          typeof data === "object" && data && "error" in data && data.error
            ? String(data.error)
            : `Registry request failed (${res.status}) ${url}`,
      };
    }
    return { ok: true, value: data };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot reach registry: ${detail}` };
  }
}

export async function exploreListPacks(
  options?: ExploreOptions & { tag?: string; projectType?: string },
): Promise<
  ExploreResult<{ packs: RegistryPackSummary[]; count: number }>
> {
  const params = new URLSearchParams();
  if (options?.tag) params.set("tag", options.tag);
  if (options?.projectType) params.set("projectType", options.projectType);
  const q = params.toString();
  return getJson(`/v1/packs${q ? `?${q}` : ""}`, options);
}

export async function exploreShowPack(
  name: string,
  options?: ExploreOptions,
): Promise<
  ExploreResult<{
    pack: RegistryPackSummary;
    skills: RegistrySkillSummary[];
  }>
> {
  return getJson(`/v1/packs/${encodeURIComponent(name)}`, options);
}

export async function exploreListSkills(
  options?: ExploreOptions & { agent?: string },
): Promise<
  ExploreResult<{ skills: RegistrySkillSummary[]; count: number }>
> {
  const params = new URLSearchParams();
  if (options?.agent) params.set("agent", options.agent);
  const q = params.toString();
  return getJson(`/v1/skills${q ? `?${q}` : ""}`, options);
}

export async function exploreSearch(
  query: string,
  options?: ExploreOptions,
): Promise<
  ExploreResult<{
    q: string;
    packs: RegistryPackSummary[];
    skills: RegistrySkillSummary[];
    count: number;
  }>
> {
  const q = query.trim();
  if (!q) {
    return { ok: false, error: "Search query is required." };
  }
  return getJson(`/v1/search?q=${encodeURIComponent(q)}`, options);
}
