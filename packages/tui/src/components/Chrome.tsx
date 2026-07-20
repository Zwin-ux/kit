import React from "react";
import { Box, Text } from "ink";
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

export function Header(props: {
  screen: string;
  detail?: string;
}): React.ReactElement {
  return (
    <Box>
      <Text bold>Kit</Text>
      <Text dimColor>
        {" "}
        · {props.screen} · v{KIT_PACKAGE_VERSION}
        {props.detail ? ` · ${props.detail}` : ""}
      </Text>
    </Box>
  );
}

export function Footer(props: { keys: string }): React.ReactElement {
  return (
    <Box marginTop={1}>
      <Text dimColor>{props.keys}</Text>
    </Box>
  );
}

export function StatusLine(props: {
  skillCount: number;
  packCount: number;
  message?: string;
}): React.ReactElement {
  return (
    <Box marginTop={1} flexDirection="column">
      <Text dimColor>
        offline · {props.skillCount} skill(s) · {props.packCount} pack(s)
      </Text>
      {props.message ? <Text>{props.message}</Text> : null}
    </Box>
  );
}
