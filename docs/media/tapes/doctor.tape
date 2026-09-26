# One screen answers: which agents are ready, is this repo gated,
# and are the installed kits still exactly as installed.
Output "out/doctor.gif"
Set Width 1080
Set Height 680
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Show

Sleep 600ms
Type "kit doctor" Sleep 400ms Enter
Wait+Screen@60s /try:/
Sleep 4s
Screenshot "out/doctor.png"
