#!/bin/sh
# A scripted stand-in for an agent CLI, for recordings only (the 16-way
# fleet tape, where 16 real agent runs would be slow and costly). Symlink it
# as claude, codex, grok and ollama in a throwaway PATH. It answers the
# probes kit makes (version, login, mcp, model list), then edits the
# worktree it is started in. Never put it on a real PATH.
name=$(basename "$0")
case "$1" in
  --version|-V|version)
    case $name in
      claude) echo "2.1.283 (Claude Code)" ;;
      codex) echo "codex-cli 0.155.0" ;;
      grok) echo "grok 1.0.34" ;;
      ollama) echo "ollama version is 0.12.3" ;;
    esac
    exit 0 ;;
  auth) echo '{"loggedIn": true}'; exit 0 ;;
  login) echo "Logged in using ChatGPT" >&2; exit 0 ;;
  mcp) echo "Added stdio MCP server $4 to user config"; exit 0 ;;
  list) echo "NAME ID SIZE MODIFIED"; echo "llama3.2:latest a80c4f17acd5 2.0GB now"; exit 0 ;;
esac
[ -t 0 ] || cat >/dev/null
d=${KIT_FAKE_SECS:-6}
echo "$name: reading the repo"; sleep $((d / 3))
echo "$name: editing src/greet.sh"; sleep $((d / 3))
mkdir -p src
printf '#!/bin/sh\necho "hello, $1"\n' > src/greet.sh
echo "$name: done, 1 file changed"; sleep $((d / 3))
