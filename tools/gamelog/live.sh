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
L="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
P=riseofnations_trace.exe
source "$W/tools/gamelog/lobby.sh"
lobby_init || exit 1

echo "--- rontrace.cfg ---"; cat "$G/rontrace.cfg"
echo "--- rontrace.cmd ---"; cat "$G/rontrace.cmd"
# The lane is taken before anything it shares is written (parked 974).
source "$W/tools/gamelog/winelaunch.sh"
ron_lane_take || exit 75

rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
source "$W/tools/gamelog/winelaunch.sh"
ron_wine "$T/wine$N.log" "$G/$P" -config check.ini -automation
echo "launched pid $RON_WINE_PID"

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
lobby_click solo 4 "$T/r$N-solo.png"
lobby_click quick 8 "$T/r$N-quick.png"
# The islands lobby's Map Style. Only the wide desktop has it measured; on
# any other, set the style in `PlayerProfile/Player.dat`'s `<MULTI>` block
# and `check.ini` with the game closed, which is what roadcapture.sh does.
lobby_click combo 3 "$T/r$N-combo.png"
lobby_click indies 3 "$T/r$N-indies.png"
lobby_start 20 "$T/r$N-"
echo "in game; screenshots in $T/r$N-*.png"
