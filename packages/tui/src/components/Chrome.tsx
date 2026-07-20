import React from "react";
import { Box, Text } from "ink";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { FadeSteps } from "../motion/index.js";

export function Header(props: {
  screen: string;
  detail?: string;
}): React.ReactElement {
  const scale = useLayoutScale();
  const layoutTag =
    scale.mode === "stack" ? "stack" : scale.mode === "wide" ? "wide" : "split";

  return (
    <Box flexDirection="column">
      <Box>
        <Text bold>Kit</Text>
        <Text> · </Text>
        <FadeSteps text={props.screen} triggerKey={props.screen} />
        <Text dimColor>
          {" "}
          · v{KIT_PACKAGE_VERSION}
          {props.detail ? ` · ${props.detail}` : ""}
        </Text>
      </Box>
      {/* A11y: always show layout + size so users know density mode */}
      <Text dimColor>
        {scale.columns}x{scale.rows} · {layoutTag}
        {scale.mascotPlacement === "hidden" ? " · menu-only" : ""}
      </Text>
    </Box>
  );
}

export function Footer(props: { keys: string }): React.ReactElement {
  const scale = useLayoutScale();
  // On stack/narrow, keep footer short and high-signal
  const keys =
    scale.mode === "stack" && props.keys.length > 56
      ? `${props.keys.slice(0, 53)}...`
      : props.keys;

  return (
    <Box marginTop={1} flexDirection="column">
      <Text bold wrap="truncate">
        {keys}
      </Text>
      {scale.mode === "stack" ? (
        <Text dimColor>tip: widen terminal for side mascot</Text>
      ) : null}
    </Box>
  );
}

export function StatusLine(props: {
  skillCount: number;
  packCount: number;
  message?: string;
  /** e.g. "3/7 web-app" for selection a11y — sticky, not color-only */
  focus?: string;
}): React.ReactElement {
  return (
    <Box marginTop={1} flexDirection="column">
      {props.focus ? (
        <Text bold inverse>
          {" "}
          sel {props.focus}{" "}
        </Text>
      ) : null}
      <Text dimColor>
        {props.skillCount} skills · {props.packCount} packs
      </Text>
      {props.message ? <Text>{props.message}</Text> : null}
    </Box>
  );
}
