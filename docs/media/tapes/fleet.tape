# Many agents at once: 4 agents x 4 roles = 16 runs, 8 at a time, the rest
# queued. Agents here are scripted stand-ins (docs/media/fakeagent.sh);
# the Control Room, worktrees, checks and receipts are real.
Output "out/fleet.gif"
Set Width 1290
Set Height 420
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 500ms
Type "kit" Sleep 300ms Enter
Sleep 1.5s
Type "d" Sleep 800ms
Tab Sleep 300ms
Down Sleep 250ms Space Sleep 250ms Down Sleep 250ms Space Sleep 250ms Down Sleep 250ms Space Sleep 250ms
Tab Sleep 300ms
Down Sleep 250ms Space Sleep 250ms Down Sleep 250ms Space Sleep 250ms Down Sleep 250ms Space Sleep 250ms
Sleep 800ms
Tab Sleep 300ms
Type@35ms "add a greet script" Sleep 500ms
Enter
Sleep 2.5s
Screenshot "out/fleet.png"
Down@120ms 12
Sleep 1.5s
Sleep 8s
Type "q" Sleep 500ms
