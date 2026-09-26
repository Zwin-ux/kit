#!/usr/bin/env bash
# Record the README and release visuals from a real kit binary with VHS.
#
#   scripts/record-media.sh [--kit PATH] [--only NAME...] [--fake]
#
# Each tape in docs/media/tapes runs in a throwaway home (KIT_MEDIA_HOME,
# default /home/demo, deleted and recreated per tape) with a small git
# repo at ~/code/shop. Tapes use the agents on your PATH (real runs), except
# fleet, which always uses docs/media/fakeagent.sh. --fake uses the stand-in
# everywhere. Output: docs/media/*.gif and *.png.
#
# VHS starts from an empty environment. Real agents get only HOME (the
# throwaway one), PATH and ANTHROPIC_API_KEY / OPENAI_API_KEY / XAI_API_KEY
# when set, so log them in with an API key. From inside a Claude Code
# session, use --fake: a nested `claude` must never see the session's
# variables.
#
# Needs: vhs (with ttyd and ffmpeg), git, node + npm, and the font
# DejaVu Sans Mono. As root, VHS needs VHS_NO_SANDBOX=true (set here).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
kit="$root/target/release/kit"
fake=0
only=()
while [ $# -gt 0 ]; do
  case "$1" in
    --kit) kit=$(cd "$(dirname "$2")" && pwd)/$(basename "$2"); shift 2 ;;
    --fake) fake=1; shift ;;
    --only) shift; while [ $# -gt 0 ] && [ "${1#--}" = "$1" ]; do only+=("$1"); shift; done ;;
    -h|--help) sed -n '2,19p' "$0"; exit 0 ;;
    *) echo "record-media: unknown argument $1" >&2; exit 2 ;;
  esac
done
[ -x "$kit" ] || { echo "record-media: no kit binary at $kit (cargo build --release -p kitctl)" >&2; exit 1; }
command -v vhs >/dev/null || { echo "record-media: vhs is not installed" >&2; exit 1; }
if [ "$fake" = 0 ] && [ -n "${CLAUDECODE:-}${CLAUDE_CODE_REMOTE:-}" ]; then
  echo "record-media: inside a Claude Code session; use --fake" >&2; exit 2
fi

demo=${KIT_MEDIA_HOME:-/home/demo}
marker="$demo/.kit-media-home"
tapes="$root/docs/media/tapes"
out="$root/docs/media"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export VHS_NO_SANDBOX=true

# Only ever delete a home this script made.
reset_home() {
  if [ -e "$demo" ]; then
    [ -e "$marker" ] || { echo "record-media: $demo exists and was not made by this script; set KIT_MEDIA_HOME" >&2; exit 1; }
    rm -rf "$demo"
  fi
  mkdir -p "$demo/.cargo/bin" "$demo/agents" "$demo/code/shop"
  touch "$marker"
  cp "$kit" "$demo/.cargo/bin/kit"  # a copy: doctor shows ~/.cargo/bin, not the build folder
  (
    cd "$demo/code/shop"
    git init -q -b main
    cat > package.json <<'JSON'
{
  "name": "shop",
  "private": true,
  "type": "module",
  "scripts": { "test": "node --test" }
}
JSON
    printf 'node_modules/\n' > .gitignore
    git add -A
    GIT_AUTHOR_NAME=you GIT_AUTHOR_EMAIL=you@example.com GIT_COMMITTER_NAME=you GIT_COMMITTER_EMAIL=you@example.com \
      git commit -qm "first commit"
  )
}

# Stand-ins for the named agents (default: all four).
use_fakes() {
  cp "$root/docs/media/fakeagent.sh" "$demo/agents/fakeagent"
  local a names=("$@")
  [ $# -gt 0 ] || names=(claude codex grok ollama)
  for a in "${names[@]}"; do ln -s fakeagent "$demo/agents/$a"; done
}

# Bare `kit` runs setup until ~/.kit/config.toml exists.
configured() {
  mkdir -p "$demo/.kit"
  printf 'agents = ["claude"]\nkits = []\nscope = "global"\n' > "$demo/.kit/config.toml"
}

write_env() {
  local path="$demo/.cargo/bin:$demo/agents:$(dirname "$(command -v node)"):/usr/local/bin:/usr/bin:/bin"
  if [ "$1" = real ]; then
    for a in claude codex grok ollama; do
      p=$(command -v "$a" 2>/dev/null) && ln -sf "$p" "$demo/agents/$a"
    done
  fi
  cat > "$work/env.tape" <<EOT
Env HOME "$demo"
Env PATH "$path"
Env GIT_AUTHOR_NAME "you"
Env GIT_AUTHOR_EMAIL "you@example.com"
Env GIT_COMMITTER_NAME "you"
Env GIT_COMMITTER_EMAIL "you@example.com"
Env KIT_FAKE_SECS "12"
EOT
  # The stand-in grok counts as logged in only with a key in the environment.
  [ "$1" = fake ] && echo 'Env XAI_API_KEY "recording"' >> "$work/env.tape"
  return 0
}

record() {
  local name=$1 agents=$2
  reset_home
  if [ "$agents" = fake ]; then use_fakes; write_env fake
  # A --fake draft of a single-agent tape shows one agent, like a typical
  # machine with only Claude Code.
  elif [ "$fake" = 1 ]; then use_fakes claude; write_env fake
  else write_env real; fi
  case $name in
    control-room|control-room-empty|fleet) configured ;;
    doctor) HOME=$demo PATH="$demo/agents:$PATH" "$kit" -C "$demo/code/shop" add essentials --yes >/dev/null ;;
  esac
  echo "== $name"
  cp "$tapes/kit-theme.tape" "$tapes/$name.tape" "$work/"
  mkdir -p "$work/out"
  # Start from an empty environment: a real agent in the tape must not see
  # the recorder's identity, tokens or session variables (a nested `claude`
  # that inherits a parent Claude Code session's variables misbehaves).
  local keys=() k
  if [ "$fake" = 0 ]; then
    for k in ANTHROPIC_API_KEY OPENAI_API_KEY XAI_API_KEY; do
      [ -n "${!k:-}" ] && keys+=("$k=${!k}")
    done
  fi
  (cd "$work" && env -i HOME="$demo" PATH="$PATH" TERM=xterm-256color LANG=C.UTF-8 \
    ${VHS_NO_SANDBOX:+VHS_NO_SANDBOX="$VHS_NO_SANDBOX"} "${keys[@]}" vhs "$name.tape")
  # A tape that asks for a screenshot must produce it; a stale one would ship.
  local want
  for want in $(sed -n 's/^Screenshot "out\/\(.*\)"/\1/p' "$tapes/$name.tape"); do
    [ -s "$work/out/$want" ] || { echo "record-media: $name did not write $want" >&2; exit 1; }
  done
  cp "$work"/out/"$name"*.gif "$out/"
  cp "$work"/out/"$name"*.png "$out/" 2>/dev/null || true
}

all=(setup add doctor control-room control-room-empty run fleet)
[ ${#only[@]} -gt 0 ] && all=("${only[@]}")
for name in "${all[@]}"; do
  case $name in
    fleet) record fleet fake ;;
    *) record "$name" real ;;
  esac
done
rm -rf "$demo"
ls -la "$out"/*.gif "$out"/*.png
