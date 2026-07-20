import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@mzwin/kit-core";
import type { ToolkitRecommendation } from "@mzwin/kit-core";
import { PackIcon } from "../mascot/PackIcon.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { SelectPulse } from "../motion/index.js";

export interface ToolkitPickerProps {
  packs: PackListItem[];
  selectedIndex: number;
  selectTick?: number;
  recommended?: ToolkitRecommendation[];
  filter?: string;
  appliedNames?: Set<string>;
  dense?: boolean;
  /** Show tiny detail silhouette under selection (layout-gated). */
  showSelectedIcon?: boolean;
}

/**
 * Pack list: animated mini glyphs. Optional 4×4 detail under selection only.
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
  const scale = useLayoutScale();
  const top = recommended[0]?.packName;
  const scoreByName = new Map(
    recommended.map((r) => [r.packName, r] as const),
  );
  const showDetail =
    showSelectedIcon && scale.showPackDetail && scale.packDetailSize > 0;

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
            <Text wrap="truncate">
              <SelectPulse selected={selected} tick={selectTick} />{" "}
              <PackIcon packName={pack.name} size="mini" animate />{" "}
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
              <Box marginLeft={2} flexDirection="row">
                {showDetail ? (
                  <Box
                    marginRight={1}
                    flexShrink={0}
                    width={scale.packDetailSize}
                    height={scale.packDetailSize}
                  >
                    <PackIcon
                      packName={pack.name}
                      size="detail"
                      detailEdge={scale.packDetailSize}
                      animate
                    />
                  </Box>
                ) : null}
                <Box flexDirection="column" flexGrow={1} flexShrink={1}>
                  <Text dimColor wrap="truncate">
                    {pack.description.length > scale.contentSoftMax
                      ? `${pack.description.slice(0, scale.contentSoftMax - 1)}…`
                      : pack.description}
                  </Text>
                  {rec && rec.reasons.length > 0 ? (
                    <Text dimColor wrap="truncate">
                      {rec.reasons[0]}
                    </Text>
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
