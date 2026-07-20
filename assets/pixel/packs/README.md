# Pack silhouettes (v1)

Pure black 16×16 icons for each official starter pack.
Same rules as kit-idle: no gray, no anti-aliasing, readable at TUI size.

| File | Pack |
|------|------|
| `essentials.png` | Kit mark |
| `web-app.png` | Browser |
| `library.png` | Book |
| `cli-tool.png` | Terminal prompt |
| `api-service.png` | Linked nodes |
| `full-stack.png` | Layers |
| `data-ml.png` | Chart bars |

Regenerate after editing `packages/tui/src/mascot/packIcons.ts`:

```sh
pnpm --filter @kit-skills/tui build
pnpm --filter @kit-skills/tui icons:write
```

TUI loads the bitmaps from source (not PNG) for speed; PNGs are for GitHub / marketing.
