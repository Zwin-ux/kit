# Starter Packs (v1)

Seven official kits. Stack packs **extend** `essentials` so dependency skills always install.

| Key | Pack | Use for | Silhouette |
|-----|------|---------|------------|
| 1 | `essentials` | Any project (default) | Kit mark |
| 2 | `web-app` | Apps and sites | Browser |
| 3 | `library` | Packages / SDKs | Book |
| 4 | `cli-tool` | Developer CLIs | Prompt |
| 5 | `api-service` | HTTP / backends | Nodes |
| 6 | `full-stack` | UI + API products | Layers |
| 7 | `data-ml` | Data / ML work | Chart |

Icons (pure black 16×16) live in `assets/pixel/packs/` and render in the TUI pack picker.

See [docs/STARTER_PACKS.md](../docs/STARTER_PACKS.md).

```sh
kit pack list
kit pack install essentials
kit pack apply web-app --dir .
kit recommend --dir .
```
