import React from "react";
import { Box, Text } from "ink";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { FadeSteps } from "../motion/index.js";

export function Header(props: {
  screen: string;
  detail?: string;
}): React.ReactElement {
  return (
    <Box>
      <Text bold>Kit</Text>
      <Text dimColor> · </Text>
      <FadeSteps text={props.screen} triggerKey={props.screen} />
      <Text dimColor>
        {" "}
        · v{KIT_PACKAGE_VERSION}
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
        {props.skillCount} skills · {props.packCount} packs
      </Text>
      {props.message ? <Text>{props.message}</Text> : null}
    </Box>
  );
}
