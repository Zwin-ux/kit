# Bare `kit` before any run: the empty Control Room and its fox. The fox
# shows only when the table is empty and the terminal has about 19 rows or
# more, so this tape is 26 rows tall.
Output "out/control-room-empty.gif"
Set Width 1290
Set Height 620
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 500ms
Type "kit" Sleep 300ms Enter
Sleep 3s
Screenshot "out/control-room-empty.png"
Sleep 1.5s
Type "q" Sleep 500ms
