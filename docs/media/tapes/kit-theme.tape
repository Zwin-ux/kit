# Shared look for every Kit recording. See docs/media/README.md for why.
# Kit's palette (crates/kit-tui/src/theme.rs): bg #0B0E12, fg #F0F1E3,
# muted #6B7280, accent #00E6CC, success #39FF9E, danger #FF3B4E, warn #FFBA3D.
# Blue and magenta are not Kit colours; they only keep the shell off stock ones.
Set Shell bash
Set FontFamily "DejaVu Sans Mono"
Set FontSize 16
Set LineHeight 1.0
Set Padding 24
Set Framerate 30
Set PlaybackSpeed 1.0
Set TypingSpeed 45ms
Set CursorBlink false
Set Theme { "name": "Kit", "background": "#0B0E12", "foreground": "#F0F1E3", "cursor": "#00E6CC", "selection": "#1F2A33", "black": "#1A1F26", "red": "#FF3B4E", "green": "#39FF9E", "yellow": "#FFBA3D", "blue": "#5B9CFF", "magenta": "#C792EA", "cyan": "#00E6CC", "white": "#B8BCB0", "brightBlack": "#6B7280", "brightRed": "#FF3B4E", "brightGreen": "#39FF9E", "brightYellow": "#FFBA3D", "brightBlue": "#8AB8FF", "brightMagenta": "#D8B4FE", "brightCyan": "#00E6CC", "brightWhite": "#F0F1E3" }
# VHS leaves COLORTERM empty; without it Kit uses its 16-colour fallback.
Env COLORTERM "truecolor"
Env PS1 "\[\e[38;2;107;114;128m\]~/code/shop\[\e[0m\] $ "
