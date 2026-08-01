import React from "react";
import { Box, Text } from "ink";
import {
  menuIconGlyph,
  type MenuIconId,
} from "../mascot/menuIcons.js";
import { menuMotionEnabled } from "../brand/mascotPolicy.js";
import { useIntervalFrame } from "../motion/useIntervalFrame.js";
import { theme } from "../theme.js";

export interface MenuButtonProps {
  icon: MenuIconId | string;
  label: string;
  meta?: string;
  selected?: boolean;
  primary?: boolean;
  disabled?: boolean;
  pressed?: boolean;
  hotkey?: string;
  variant?: "list" | "chip" | "bar";
}

/**
 * Controls — reverse video + fox-orange on focus.
 * Cursor is always 2 cells so lists never jiggle (selection-stability).
 */
export function MenuButton({
  icon,
  label,
  meta,
  selected = false,
  primary = false,
  disabled = false,
  pressed = false,
  hotkey,
  variant = "list",
}: MenuButtonProps): React.ReactElement {
  const pulse = useIntervalFrame(
    2,
    520,
    Boolean(selected && !disabled && !pressed && menuMotionEnabled()),
  );
  const cursor =
    selected && !disabled ? (pulse === 0 ? "> " : "› ") : "  ";
  const glyph = menuIconGlyph(icon);
  const lit = pressed || selected;

  if (variant === "chip") {
    return (
      <Text
        bold={lit || primary}
        inverse={pressed || selected}
        {...(selected || pressed ? { color: theme.accent } : {})}
        dimColor={disabled && !lit}
        wrap="truncate"
      >
        {selected || pressed ? " " : "  "}
        {label}
        {hotkey ? ` ${hotkey}` : ""}
        {selected || pressed ? " " : " "}
      </Text>
    );
  }

  if (variant === "bar") {
    const barLit = pressed || selected || primary;
    return (
      <Text
        bold={barLit}
        inverse={barLit}
        {...(barLit ? { color: theme.accent } : {})}
        dimColor={disabled && !lit}
        wrap="truncate"
      >
        {" "}
        {pressed ? "* " : ""}
        {label}
        {hotkey ? ` [${hotkey}]` : ""}{" "}
      </Text>
    );
  }

  return (
    <Text
      bold={lit}
      inverse={pressed || selected}
      {...(lit ? { color: theme.accent } : {})}
      dimColor={disabled && !lit}
      wrap="truncate"
    >
      {pressed ? "* " : cursor}
      <Text>{glyph}</Text>
      {" "}
      {label}
      {meta ? (
        <Text dimColor={!lit}>{`  ${meta}`}</Text>
      ) : null}
      {hotkey && !selected ? (
        <Text dimColor>{`  ${hotkey}`}</Text>
      ) : null}
    </Text>
  );
}

export interface MenuButtonRowProps {
  children: React.ReactNode;
}

export function MenuButtonRow({
  children,
}: MenuButtonRowProps): React.ReactElement {
  return (
    <Box flexDirection="row" flexWrap="wrap" gap={1}>
      {children}
    </Box>
  );
}
