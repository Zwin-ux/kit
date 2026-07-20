import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@mzwin/kit-core";
import type { ToolkitRecommendation } from "@mzwin/kit-core";
import { PackIcon } from "../mascot/PackIcon.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { SelectPulse, type SelectDirection } from "../motion/index.js";

export interface ToolkitPickerProps {
  packs: PackListItem[];
  selectedIndex: number;
  selectTick?: number;
  selectDirection?: SelectDirection;
  recommended?: ToolkitRecommendation[];
  filter?: string;
  appliedNames?: Set<string>;
  dense?: boolean;
  /** Show detail panel under the list (fixed height — no in-row expand). */
  showSelectedIcon?: boolean;
}

/**
 * Pack list: one line per row (never expands under selection).
 * Detail lives in a sticky panel below so ↑↓ does not reflow the list.
 */
export function ToolkitPicker({
  packs,
  selectedIndex,
  selectTick = 0,
  selectDirection = "none",
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
    showSelectedIcon &&
    !dense &&
    scale.showPackDetail &&
    scale.packDetailSize > 0;

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

  const selectedPack =
    packs[selectedIndex] ??
    filtered.find((p) => originalIndex(p) === selectedIndex);
  const selectedRec = selectedPack
    ? scoreByName.get(selectedPack.name)
    : undefined;

  // Fixed panel height: icon rows + 2 text lines (or empty placeholders)
  const detailLines = showDetail ? scale.packDetailSize + 2 : 2;

  return (
    <Box flexDirection="column">
      {filtered.map((pack) => {
        const oi = originalIndex(pack);
        const selected = oi === selectedIndex;
        const isTop = pack.name === top;
        const applied = appliedNames?.has(pack.name);
        const extendsNote =
          pack.extends && pack.extends.length > 0
            ? ` · +${pack.extends.join("+")}`
            : "";

        return (
          <Text key={pack.name} wrap="truncate">
            <SelectPulse
              selected={selected}
              tick={selectTick}
              direction={selectDirection}
            />
            <PackIcon packName={pack.name} size="mini" animate />{" "}
            <Text bold={selected}>{pack.title}</Text>
            <Text dimColor={!selected}>
              {" "}
              · {pack.skillCount} skills
              {extendsNote}
            </Text>
            {isTop ? <Text> ★</Text> : null}
            {applied ? <Text dimColor> · on</Text> : null}
          </Text>
        );
      })}

      {/* Sticky detail panel — always same height so ↑↓ never inserts rows */}
      {!dense ? (
        <Box
          marginTop={1}
          flexDirection="row"
          height={detailLines}
          overflow="hidden"
        >
          {showDetail && selectedPack ? (
            <Box
              marginRight={1}
              flexShrink={0}
              width={scale.packDetailSize}
              height={scale.packDetailSize}
            >
              <PackIcon
                packName={selectedPack.name}
                size="detail"
                detailEdge={scale.packDetailSize}
                animate
              />
            </Box>
          ) : null}
          <Box flexDirection="column" flexGrow={1} flexShrink={1}>
            <Text dimColor wrap="truncate">
              {selectedPack
                ? selectedPack.description.length > scale.contentSoftMax
                  ? `${selectedPack.description.slice(0, scale.contentSoftMax - 1)}…`
                  : selectedPack.description
                : " "}
            </Text>
            <Text dimColor wrap="truncate">
              {selectedRec?.reasons[0] ?? " "}
            </Text>
          </Box>
        </Box>
      ) : null}
    </Box>
  );
}
