import React from "react";
import { Box, Text } from "ink";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { FadeSteps } from "../motion/index.js";
import { theme } from "../theme.js";

/**
 * Shared chrome — ink-console brand, matches marketing banners.
 * Inverse KIT mark + orange rule. No layout debug noise by default.
 */
export function Header(props: {
  screen: string;
  detail?: string;
  /** Hide terminal size / layout mode (default: hide — product not debug). */
  showLayoutMeta?: boolean;
}): React.ReactElement {
  const scale = useLayoutScale();
  const layoutTag =
    scale.mode === "stack" ? "stack" : scale.mode === "wide" ? "wide" : "split";
  const rule = "─".repeat(Math.max(8, Math.min(scale.columns - 4, 40)));

  return (
    <Box flexDirection="column">
      <Box justifyContent="space-between" width="100%">
        <Box>
          <Text bold inverse>
            {" "}
            KIT{" "}
          </Text>
          <Text> </Text>
          <FadeSteps text={props.screen} triggerKey={props.screen} />
          {props.detail ? (
            <Text dimColor> · {props.detail}</Text>
          ) : null}
        </Box>
        <Text dimColor>v{KIT_PACKAGE_VERSION}</Text>
      </Box>
      <Text color={theme.accent}>{rule}</Text>
      {props.showLayoutMeta ? (
        <Text dimColor>
          {scale.columns}x{scale.rows} · {layoutTag}
          {scale.mascotPlacement === "hidden" ? " · menu-only" : ""}
        </Text>
      ) : null}
    </Box>
  );
}

export function Footer(props: { keys: string }): React.ReactElement {
  const scale = useLayoutScale();
  const keys =
    scale.mode === "stack" && props.keys.length > 56
      ? `${props.keys.slice(0, 53)}...`
      : props.keys;

  return (
    <Box marginTop={1} flexDirection="column">
      <Text dimColor wrap="truncate">
        {keys}
      </Text>
    </Box>
  );
}

export function StatusLine(props: {
  skillCount: number;
  packCount: number;
  message?: string;
  /** e.g. "3/7 web-app" for selection a11y — sticky, not color-only */
  focus?: string;
  /** Agent wiring strip */
  agents?: string;
}): React.ReactElement {
  return (
    <Box marginTop={1} flexDirection="column">
      {props.focus ? (
        <Text bold inverse color={theme.accent}>
          {" "}
          SEL {props.focus}{" "}
        </Text>
      ) : null}
      <Text dimColor wrap="truncate">
        {props.skillCount} skills · {props.packCount} packs
        {props.agents ? ` · agents ${props.agents}` : ""}
      </Text>
      {props.message ? <Text wrap="truncate">{props.message}</Text> : null}
    </Box>
  );
}
