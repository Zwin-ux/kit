# Agent logos

`claude.png`, `codex.png` and `grok.png` are 64×64 tiles that `kit setup`
draws next to each agent in terminals that can show images (kitty, Ghostty,
WezTerm, iTerm2). Everywhere else Kit shows a text mark in the brand colour
(`src/brand.rs`).

The marks come from [lobehub/lobe-icons](https://github.com/lobehub/lobe-icons)
(`@lobehub/icons-static-svg` 1.95.1, MIT): `claudecode-color`, `codex-color`
and `grok`, rendered onto rounded tiles. The logos are trademarks of their
owners (Anthropic, OpenAI, xAI). Kit uses them only to name the agent it
sets up, and does not suggest endorsement.

The icon set's MIT licence is in [`LICENSE-lobe-icons`](LICENSE-lobe-icons),
and ships with every Kit download in the root `README.md` and
`THIRD-PARTY-NOTICES.md`.
