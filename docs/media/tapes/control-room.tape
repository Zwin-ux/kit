# The hero: every run in one table. The failed run leads with its first
# error line; running and gating rows breathe on the one animation clock.
# --demo starts no agent. Never press r or k here (see README).
Output "out/control-room.gif"
Set Width 1290
Set Height 380
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 500ms
Type "kit --demo" Sleep 300ms Enter
Sleep 1.2s
Screenshot "out/control-room.png"
Sleep 1s
# Open the failed run, then its gate log and diff.
Enter Sleep 1.8s
Type "2" Sleep 2.5s
Type "3" Sleep 2.2s
Escape Sleep 800ms
Up Sleep 600ms Up Sleep 1.2s
Type "?" Sleep 2.2s Type "?" Sleep 600ms
Type "q" Sleep 500ms
