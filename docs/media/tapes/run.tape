# Proof: one agent, one task, its own worktree, your repo's checks,
# then a receipt and a branch you can merge. Uses the real agent on PATH.
Output "out/run.gif"
Set Width 1180
Set Height 680
Source "kit-theme.tape"
Source "env.tape"

Hide
Type "cd ~/code/shop && clear" Enter
Type "last(){ ls -t ~/.kit/runs | head -1; }" Enter
Type "clear" Enter
Show

Sleep 600ms
Type "kit init" Sleep 300ms Enter
Wait+Screen@60s /kit.toml/
Sleep 2.5s
Hide
Type "git add kit.toml && git commit -qm 'kit gate' && clear" Enter
Show
Type `kit run "add a greet script that prints hello and a test for it"` Sleep 400ms Enter
Wait+Screen@600s /next +kit land/
Sleep 3.5s
Screenshot "out/run.png"
Sleep 1s
Type "kit land $(last)" Sleep 300ms Enter
Sleep 3.5s
