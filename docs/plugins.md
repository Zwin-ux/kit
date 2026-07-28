# Local CLI plugins

Kit can register a CLI from a checkout on your computer.

Kit stores the checkout path and a SHA-256 digest of `kit.plugin.json`. Kit
stops a run when the manifest digest changes. Review the change and register
the plugin again.

Kit starts the executable with an argument array. Kit does not use a shell.
Kit does not copy the executable. The plugin owns its domain rules and
credentials.

## Commands

```bash
kit plugin add <path>
kit plugin add <path> --write
kit plugin list
kit plugin doctor <name>
kit plugin run <name> -- <arguments>
kit plugin remove <name> --write
```

Add and remove use a dry-run unless you pass `--write`.

## Manifest

Put `kit.plugin.json` in the plugin root.

```json
{
  "schemaVersion": 1,
  "name": "trenchwire",
  "displayName": "Trenchwire",
  "description": "Find Solana market facts from the terminal.",
  "version": "1.0.0",
  "command": "trenchwire",
  "localExecutables": {
    "win32": "target/release/trenchwire.exe",
    "darwin": "target/release/trenchwire",
    "linux": "target/release/trenchwire"
  },
  "versionArgs": ["--version"],
  "healthArgs": ["check", "--json"],
  "safety": {
    "summary": "Phantom owns trade approval.",
    "confirmationToken": "SEND"
  }
}
```

`name` uses lowercase letters, numbers, and hyphens. A local executable path
must stay inside the plugin root. Kit uses `command` from `PATH` when the local
executable does not exist.

`versionArgs`, `healthArgs`, and `safety` document the plugin contract. Kit
shows these fields in `plugin doctor`. Kit does not claim authority over them.

## Trust boundary

A plugin is code that runs on your computer. Review the checkout before you
register it.

Kit protects the launch boundary:

- Registration requires `--write`.
- Kit records the manifest digest.
- Kit blocks a changed manifest.
- Kit passes arguments without a shell.
- Local executable paths stay inside the plugin root.

The plugin must protect its own network, credential, and write actions.
