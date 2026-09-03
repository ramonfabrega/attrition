#!/bin/zsh
# viadriver.sh <script> [args...] — run a capture outside Claude Code's process
# tree, so its macOS permissions survive the next Claude Code update.
#
#   zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 197
#
# Everything the capture needs — `cliclick`'s synthetic clicks (Accessibility),
# `screencapture` (Screen Recording), `osascript`'s window queries (Automation)
# — is granted to the *responsible process*, and under Claude Code that is
# `~/.local/share/claude/versions/<VERSION>`: a path that changes with every
# update and takes all three grants with it. `~/bin/RonDriver.app` is a fixed
# path holding the same three permanently; this hands it the job.
#
# Build the bundle once with `zsh tools/gamelog/rondriver/build.sh`, then grant
# it Accessibility by hand (System Settings → + → ⇧⌘G → the path below).
#
# The run is detached: `open` returns as soon as LaunchServices has the app, so
# poll the log this prints. That is the same shape as launching `runqueue.sh`
# with `nohup`, and the log is the only place its output goes.
set -e

APP=${RONDRIVER_APP:-$HOME/bin/RonDriver.app}
W=${0:A:h:h:h}

if [ ! -x "$APP/Contents/MacOS/RonDriver" ]; then
  echo "no $APP — build it first:" >&2
  echo "  zsh $W/tools/gamelog/rondriver/build.sh" >&2
  exit 1
fi
if [ $# -lt 1 ]; then
  echo "usage: viadriver.sh <script> [args...]" >&2
  exit 64
fi

script=$1; shift
[ -f "$script" ] || script="$W/$script"
[ -f "$script" ] || { echo "no such script: $1" >&2; exit 66 }

mkdir -p /tmp/ron-runs
log=/tmp/ron-runs/viadriver-$(date +%Y%m%d-%H%M%S).log
echo "log: $log"

# --args goes to RonDriver as <cwd> <logfile> <program> [args...]; `open`
# launches through LaunchServices, which is what makes the bundle — rather than
# whatever spawned this script — the responsible process.
open -a "$APP" --args "$W" "$log" /bin/zsh "$script" "$@"

echo "launched through $APP"
echo "tail -f $log"
