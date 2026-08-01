import React from "react";
import { Box, Text } from "ink";
import { useLayoutScale } from "../mascot/useLayoutScale.js";

export interface ActionItem {
  /** Single-key hint shown first. */
  key: string;
  /** Short verb. */
  label: string;
  /** Highlight as the situation-recommended move. */
  primary?: boolean;
  /** Dim unavailable actions. */
  disabled?: boolean;
}

export interface ActionRailProps {
  items: ActionItem[];
  /** Optional one-line story / context above the rail. */
  story?: string;
}

/**
 * Primary product actions — not a footer dump.
 * Situation story sits above; keys are scannable.
 */
export function ActionRail({
  items,
  story,
}: ActionRailProps): React.ReactElement {
  const scale = useLayoutScale();
  const compact = scale.mode === "stack" || scale.columns < 78;
  const visible = compact
    ? items.filter((i) => i.primary || !i.disabled).slice(0, 4)
    : items;

  return (
    <Box flexDirection="column" marginTop={1} flexShrink={0}>
      {story ? (
        <Text bold wrap="truncate">
          {story}
        </Text>
      ) : null}
      <Text wrap="truncate">
        {visible.map((item, index) => (
          <React.Fragment key={item.key}>
            {index > 0 ? <Text dimColor> · </Text> : null}
            <Text
              bold={Boolean(item.primary)}
              inverse={Boolean(item.primary)}
              dimColor={Boolean(item.disabled) && !item.primary}
            >
              {item.primary
                ? ` ${item.key} ${item.label} `
                : `${item.key} ${item.label}`}
            </Text>
          </React.Fragment>
        ))}
      </Text>
      {compact && items.length > visible.length ? (
        <Text dimColor>? all keys</Text>
      ) : null}
    </Box>
  );
}
