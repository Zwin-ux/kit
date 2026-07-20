import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@kit-skills/core";
import type { ToolkitRecommendation } from "@kit-skills/core";

export interface ToolkitPickerProps {
  packs: PackListItem[];
  selectedIndex: number;
  /** Recommended pack names in score order. */
  recommended?: ToolkitRecommendation[];
  /** Filter: show only packs matching tag or project type substring. */
  filter?: string;
  appliedNames?: Set<string>;
  /** Show skill count and version row */
  dense?: boolean;
}

/**
 * Shared pack/toolkit list with recommendation badges.
 * Used on Home, Packs, and First-run.
 */
export function ToolkitPicker({
  packs,
  selectedIndex,
  recommended = [],
  filter,
  appliedNames,
  dense = false,
}: ToolkitPickerProps): React.ReactElement {
  const top = recommended[0]?.packName;
  const scoreByName = new Map(
    recommended.map((r) => [r.packName, r] as const),
  );

  const filtered = filter
    ? packs.filter((p) => {
        const f = filter.toLowerCase();
        return (
          p.name.includes(f) ||
          p.title.toLowerCase().includes(f) ||
          p.tags.some((t) => t.includes(f)) ||
          p.projectTypes.some((t) => t.includes(f)) ||
          p.description.toLowerCase().includes(f)
        );
      })
    : packs;

  if (filtered.length === 0) {
    return (
      <Text dimColor>
        No toolkits match{filter ? ` “${filter}”` : ""}. Clear filter with Esc.
      </Text>
    );
  }

  // Map filtered index → original packs index for selection highlight
  const originalIndex = (pack: PackListItem) =>
    packs.findIndex((p) => p.name === pack.name);

  return (
    <Box flexDirection="column">
      {filtered.map((pack) => {
        const oi = originalIndex(pack);
        const mark = oi === selectedIndex ? "›" : " ";
        const rec = scoreByName.get(pack.name);
        const isTop = pack.name === top;
        const applied = appliedNames?.has(pack.name);

        return (
          <Box key={pack.name} flexDirection="column" marginBottom={dense ? 0 : 0}>
            <Text>
              {mark}{" "}
              <Text bold={oi === selectedIndex}>{pack.title}</Text>
              <Text dimColor>
                {" "}
                ({pack.name} · {pack.skillCount} skills · v{pack.version})
              </Text>
              {isTop ? <Text> ★ recommended</Text> : null}
              {applied ? <Text dimColor> · applied</Text> : null}
            </Text>
            {!dense && oi === selectedIndex ? (
              <Box flexDirection="column" marginLeft={3}>
                <Text dimColor>{pack.description}</Text>
                {rec && rec.reasons.length > 0 ? (
                  <Text dimColor>
                    why: {rec.reasons.slice(0, 2).join("; ")}
                  </Text>
                ) : null}
                {pack.projectTypes.length > 0 ? (
                  <Text dimColor>
                    fits: {pack.projectTypes.join(", ")}
                  </Text>
                ) : null}
                {pack.tags.length > 0 ? (
                  <Text dimColor>tags: {pack.tags.join(", ")}</Text>
                ) : null}
              </Box>
            ) : null}
          </Box>
        );
      })}
    </Box>
  );
}
