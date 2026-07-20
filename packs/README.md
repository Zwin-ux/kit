# Starter packs

Seven official kits. Stack packs **extend** `essentials` so base skills always install.

| Key | Pack | Best for |
|-----|------|----------|
| 1 | `essentials` | Any project (default) |
| 2 | `web-app` | Apps and sites |
| 3 | `library` | Packages / SDKs |
| 4 | `cli-tool` | Developer CLIs |
| 5 | `api-service` | HTTP backends |
| 6 | `full-stack` | UI + API products |
| 7 | `data-ml` | Data / ML work |

Guide: [docs/packs.md](../docs/packs.md)

```sh
pnpm kit -- pack list
pnpm kit -- pack install essentials
pnpm kit -- pack apply web-app --dir .
pnpm kit -- recommend --dir .
```
