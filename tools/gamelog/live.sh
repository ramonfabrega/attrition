#!/bin/zsh
# live.sh N — launch the traced exe with whatever gamelog.ini / rise2.ini /
# rontrace.cfg / rontrace.cmd already say, drive the lobby's five clicks, and
# **return with the game running**.
#
# runwin.sh is the unattended form: it also waits for the dump to settle after
# a `!quit` and archives. This one stops at the point runwin.sh stops caring
# about, because the capture it exists for needs a *human-shaped* action in the
# middle — a right-click on a multi-unit selection, which the cheat channel
# cannot issue (docs/GROUPS.md §6.4). Stage with setlog.py and window.py, put
# the adds and the `select` lines in rontrace.cmd, run this, then drive with
# cliclick and archive with `archive.sh N TAG`.
N=$1
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
P=riseofnations_trace.exe

echo "--- rontrace.cfg ---"; cat "$G/rontrace.cfg"
echo "--- rontrace.cmd ---"; cat "$G/rontrace.cmd"

rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
nohup /Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine \
  --bottle ron --workdir "$G" --wait-children \
  "$G/$P" -config check.ini -automation > "$T/wine$N.log" 2>&1 &
echo "launched pid $!"

click() {
  osascript -e "tell application \"System Events\" to set frontmost of process \"$P\" to true" >/dev/null 2>&1
  sleep 0.5
  cliclick m:$1,$2 w:400 c:$1,$2
  sleep ${4:-3}
  screencapture -x -R760,152,1920,1108 "$T/r$N-$3.png"
  sips -Z 900 "$T/r$N-$3.png" --out "$T/r$N-$3s.png" >/dev/null
  echo "$(date +%H:%M:%S) clicked $3 at $1,$2"
}

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
click 1715 744 solo 4
click 1715 672 quick 8
click 2392 291 combo 3
click 2262 551 indies 3
click 1061 1176 start1 3
click 1061 1176 start2 20
echo "in game; screenshots in $T/r$N-*.png"
