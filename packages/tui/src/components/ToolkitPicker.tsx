import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@mzwin/kit-core";
import type { ToolkitRecommendation } from "@mzwin/kit-core";
import { PackIcon } from "../mascot/PackIcon.js";
import { SelectPulse } from "../motion/index.js";

export interface ToolkitPickerProps {
  packs: PackListItem[];
  selectedIndex: number;
  selectTick?: number;
  recommended?: ToolkitRecommendation[];
  filter?: string;
  appliedNames?: Set<string>;
  dense?: boolean;
  /** Show full 16×16 silhouette under the selected row. Default true. */
  showSelectedIcon?: boolean;
}

/**
 * Shared pack/toolkit list with silhouette marks + recommendation badges.
 */
export function ToolkitPicker({
  packs,
  selectedIndex,
  selectTick = 0,
  recommended = [],
  filter,
  appliedNames,
  dense = false,
  showSelectedIcon = true,
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
        No match{filter ? ` for “${filter}”` : ""}. Esc clears filter.
      </Text>
    );
  }

  const originalIndex = (pack: PackListItem) =>
    packs.findIndex((p) => p.name === pack.name);

  return (
    <Box flexDirection="column">
      {filtered.map((pack) => {
        const oi = originalIndex(pack);
        const selected = oi === selectedIndex;
        const rec = scoreByName.get(pack.name);
        const isTop = pack.name === top;
        const applied = appliedNames?.has(pack.name);
        const extendsNote =
          pack.extends && pack.extends.length > 0
            ? ` · +${pack.extends.join("+")}`
            : "";

        return (
          <Box key={pack.name} flexDirection="column">
            <Text>
              <SelectPulse selected={selected} tick={selectTick} />{" "}
              <PackIcon packName={pack.name} size="mini" />{" "}
              <Text bold={selected}>{pack.title}</Text>
              <Text dimColor>
                {" "}
                · {pack.skillCount} skills
                {extendsNote}
              </Text>
              {isTop ? <Text> ★</Text> : null}
              {applied ? <Text dimColor> · on</Text> : null}
            </Text>
            {!dense && selected ? (
              <Box marginLeft={2} marginTop={0} flexDirection="row">
                {showSelectedIcon ? (
                  <Box marginRight={2} flexShrink={0}>
                    <PackIcon packName={pack.name} size="full" />
                  </Box>
                ) : null}
                <Box flexDirection="column">
                  <Text dimColor>{pack.description}</Text>
                  {rec && rec.reasons.length > 0 ? (
                    <Text dimColor>{rec.reasons[0]}</Text>
                  ) : null}
                </Box>
              </Box>
            ) : null}
          </Box>
        );
      })}
    </Box>
  );
}
