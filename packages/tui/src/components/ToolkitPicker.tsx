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
  /** One-line description only; skip extra chrome. */
  dense?: boolean;
  /**
   * @deprecated Detail █ silhouettes are a11y-hostile on dark terminals.
   * Kept for API compat; ignored — detail is always text-only.
   */
  showSelectedIcon?: boolean;
}

/**
 * Menu-first pack list: geometry stable on ↑↓; density follows layout mode.
 *
 * A11y (visual + cognitive):
 * - No solid █ detail silhouettes (white blobs on dark WT)
 * - Selection = inverse + bold + fixed ASCII cursor (not color-only)
 * - Focus readout N/M always visible above the windowed list
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
  showSelectedIcon: _showSelectedIcon = true,
}: ToolkitPickerProps): React.ReactElement {
  void _showSelectedIcon;
  const scale = useLayoutScale();
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
  // Dense = 1 desc line; normal = 2 + optional reason (still no █ art)
  const descLines = dense ? 1 : 2;
  const detailText = fixedLines(
    selectedPack?.description ?? "",
    descLines,
    metaCols,
  );
  const reasonLine = dense
    ? ""
    : fixedLine(selectedRec?.reasons[0] ?? "", metaCols);
  const focusIdx =
    filtered.findIndex((p) => originalIndex(p) === selectedIndex) + 1;

  return (
    <Box flexDirection="column">
      {/* A11y: stable selection readout — not color-only */}
      <Text bold>
        {focusIdx > 0
          ? `focus ${focusIdx}/${filtered.length} ${selectedPack?.title ?? ""}`
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

      {/* Text-only detail — never solid █ bitmaps (dark-terminal a11y) */}
      <Box flexDirection="column" flexShrink={0} marginTop={dense ? 0 : 1}>
        {detailText.map((line, i) => (
          <Text key={`d${i}`} dimColor>
            {line}
          </Text>
        ))}
        {!dense && reasonLine.trim() ? (
          <Text dimColor>{reasonLine}</Text>
        ) : null}
      </Box>
    </Box>
  );
}
