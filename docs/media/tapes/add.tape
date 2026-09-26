# The plan is the trust surface: every path, every pin and licence,
# what runs code, and nothing written until yes.
Output "out/add.gif"
Set Width 1180
Set Height 680
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 600ms
Type "kit add backend-engineer" Sleep 400ms Enter
Wait+Screen@120s /Continue/
Sleep 4s
Screenshot "out/add-plan.png"
Type "y" Sleep 300ms Enter
Wait+Screen@60s /undo/
Sleep 2s
Type "kit list" Sleep 300ms Enter
Sleep 3s
