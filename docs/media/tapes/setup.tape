# The first five minutes: kit finds your agents, asks three questions,
# shows every file it will write, and installs only after a yes.
Output "out/setup.gif"
Set Width 1180
Set Height 920
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 600ms
Type "kit" Sleep 400ms Enter
Sleep 2.5s
# Agents: every agent found is ticked.
Enter Sleep 1.2s
# Focus: move to Frontend Design and tick it.
Down Sleep 400ms Space Sleep 700ms Enter Sleep 1.2s
# Scope: this repo.
Down Sleep 400ms Enter
Wait+Screen@120s /Continue/
Sleep 3.5s
Screenshot "out/setup-plan.png"
Type "y" Sleep 300ms Enter
Wait+Screen@120s /Try it/
Sleep 4s
Screenshot "out/setup-done.png"
