import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@mzwin/kit-core";
import type { ToolkitRecommendation } from "@mzwin/kit-core";
import { PackIcon } from "../mascot/PackIcon.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { fixedLine, fixedLines } from "../motion/fixedLines.js";
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
  showSelectedIcon?: boolean;
}

/**
 * Pack list: one line per row, fixed detail panel, no list timers.
 * ↑↓ must not change frame geometry.
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
  // Detail bitmaps only when tall; always reserve 2 text lines under list
  const showDetailIcon =
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
        No match{filter ? ` for “${filter}”` : ""}. Esc clear filter.
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

  const metaCols = Math.max(24, Math.min(scale.contentSoftMax, 56));
  const detailText = fixedLines(
    selectedPack?.description ?? "",
    2,
    metaCols,
  );
  const reasonLine = fixedLine(selectedRec?.reasons[0] ?? "", metaCols);

  return (
    <Box flexDirection="column">
      {filtered.map((pack) => {
        const oi = originalIndex(pack);
        const selected = oi === selectedIndex;
        const isTop = pack.name === top;
        const applied = appliedNames?.has(pack.name);
        const extendsNote =
          pack.extends && pack.extends.length > 0
            ? `+${pack.extends.join("+")}`
            : "";
        // Fixed-ish meta: avoid mounting optional Text nodes that change width
        const meta = fixedLine(
          `· ${pack.skillCount}sk${extendsNote ? ` ${extendsNote}` : ""}${isTop ? " *" : ""}${applied ? " on" : ""}`,
          Math.max(12, metaCols - 16),
        );

        return (
          <Text key={pack.name} wrap="truncate">
            <SelectPulse
              selected={selected}
              tick={selectTick}
              direction={selectDirection}
            />
            <PackIcon packName={pack.name} size="mini" animate={false} />{" "}
            <Text bold={selected}>{pack.title}</Text>{" "}
            <Text dimColor>{meta.trimEnd()}</Text>
          </Text>
        );
      })}

      {/* Always 2 description lines + optional icon; height constant */}
      {!dense ? (
        <Box marginTop={1} flexDirection="row" flexShrink={0}>
          {showDetailIcon && selectedPack ? (
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
                animate={false}
              />
            </Box>
          ) : null}
          <Box flexDirection="column" flexShrink={0}>
            <Text dimColor>{detailText[0]}</Text>
            <Text dimColor>{detailText[1]}</Text>
            <Text dimColor>{reasonLine}</Text>
          </Box>
        </Box>
      ) : null}
    </Box>
  );
}
