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
 * Menu-first pack list: geometry stable on ↑↓; density follows layout mode.
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
        No match{filter ? ` for "${filter}"` : ""}. Esc clear filter.
      </Text>
    );
  }

  const originalIndex = (pack: PackListItem) =>
    packs.findIndex((p) => p.name === pack.name);

  // Window list around selection so small screens still show the focused row
  const maxShow = scale.packListMax > 0 ? scale.packListMax : filtered.length;
  let windowed = filtered;
  let windowOffset = 0;
  if (filtered.length > maxShow) {
    const selInFiltered = Math.max(
      0,
      filtered.findIndex((p) => originalIndex(p) === selectedIndex),
    );
    const half = Math.floor(maxShow / 2);
    windowOffset = Math.max(
      0,
      Math.min(selInFiltered - half, filtered.length - maxShow),
    );
    windowed = filtered.slice(windowOffset, windowOffset + maxShow);
  }

  const selectedPack =
    packs[selectedIndex] ??
    filtered.find((p) => originalIndex(p) === selectedIndex);
  const selectedRec = selectedPack
    ? scoreByName.get(selectedPack.name)
    : undefined;

  const metaCols = Math.max(20, Math.min(scale.contentSoftMax, 56));
  const detailText = fixedLines(selectedPack?.description ?? "", 2, metaCols);
  const reasonLine = fixedLine(selectedRec?.reasons[0] ?? "", metaCols);
  const focusIdx =
    filtered.findIndex((p) => originalIndex(p) === selectedIndex) + 1;

  return (
    <Box flexDirection="column">
      {/* A11y: stable selection readout */}
      <Text bold>
        {focusIdx > 0
          ? `${focusIdx}/${filtered.length} ${selectedPack?.title ?? ""}`
          : `${filtered.length} packs`}
      </Text>

      {windowOffset > 0 ? (
        <Text dimColor>  ^ {windowOffset} more above</Text>
      ) : null}

      {windowed.map((pack) => {
        const oi = originalIndex(pack);
        const selected = oi === selectedIndex;
        const isTop = pack.name === top;
        const applied = appliedNames?.has(pack.name);
        const extendsNote =
          pack.extends && pack.extends.length > 0
            ? `+${pack.extends.join("+")}`
            : "";
        const meta = fixedLine(
          `${pack.skillCount}sk${extendsNote ? ` ${extendsNote}` : ""}${isTop ? " *" : ""}${applied ? " on" : ""}`,
          Math.max(10, metaCols - 18),
        );

        return (
          <Text key={pack.name} wrap="truncate" inverse={selected}>
            <SelectPulse
              selected={selected}
              tick={selectTick}
              direction={selectDirection}
            />
            <PackIcon packName={pack.name} size="mini" animate={false} />{" "}
            <Text bold={selected}>{pack.title}</Text>{" "}
            <Text dimColor={!selected}>{meta.trimEnd()}</Text>
          </Text>
        );
      })}

      {windowOffset + windowed.length < filtered.length ? (
        <Text dimColor>
          {"  "}v {filtered.length - windowOffset - windowed.length} more below
        </Text>
      ) : null}

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
