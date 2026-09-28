#!/bin/zsh
# startcapture.sh [N] [MAPSTYLE] [TAG] — the `DUMP_ALL` start of a game.
#
# The companion to `longtrace.sh`. A long capture at the `[End Frame]`
# thresholds gives the frames; it does **not** give the start-of-game full
# dump — `master_land_heights`, the cells at every level, the fog grids, the
# regions and the herds — which the harness's `build_sim` needs and which
# only `DUMP_ALL=1` with `InitialDump=1` writes (`docs/ORACLE.md`, "run20
# and run21"). On the Great Lakes map the harness borrows that table from
# run12/run13; a game on any other map has no sibling to borrow from, so it
# needs this run of its own before its long trace is worth anything.
#
# Two `DUMP_ALL` frame blocks and out: `LogStartFrame=0 LogEndFrame=2`,
# `rontrace.cfg` `cover=1 window=0-1`, and `2 !quit`. Minutes, not seconds —
# the start dump alone is several hundred megabytes — and the end-of-game
# dump keeps writing after the quit, so the poll waits for the file to stop
# growing rather than for the process to end.
set -e
N=${1:-34}
MAPSTYLE=${2:-18}
TAG=${3:-start}
# See `longtrace.sh`: `-config check.ini` pins the map style to the default
# 14 and no file can move it, so a run on any other map drops it and takes
# the profile's lobby, which `mapstyle.py` has just written.
if [ -z "${CFG+set}" ]; then
  if [ "$MAPSTYLE" = 14 ]; then CFG="-config check.ini"; else CFG=""; fi
fi
W=$(cd "$(dirname "$0")/../.." && pwd)
G=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
B="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$B/Logs"
T=${RON_TMP:-/tmp/ron-runs}
P=riseofnations_trace.exe
mkdir -p "$T"
# The lane is taken before anything it shares is written (parked 974).
source "$W/tools/gamelog/winelaunch.sh"
ron_lane_take || exit 75

# --- the probe. Three permissions, each failing differently, and two
# earlier versions of this block passed while Accessibility was off —
# `tools/gamelog/probe.sh` carries why, and tests by posting an event and
# reading back where the cursor went. Then the lobby's own table, because
# the buttons are not in the same place on both of this machine's desktops
# (`tools/gamelog/lobby.sh`).
source "$W/tools/gamelog/probe.sh"
perm_probe "$T/permprobe$N.png" || exit 1
source "$W/tools/gamelog/lobby.sh"
lobby_init "$T/probe$N.png" || exit 1

python3 "$W/tools/gamelog/mapstyle.py" "$MAPSTYLE"
python3 "$W/tools/fuzz/seedini.py" 12345
python3 "$W/tools/gamelog/window.py" stage 0 2

rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
source "$W/tools/gamelog/winelaunch.sh"
ron_wine "$T/wine$N.log" "$G/$P" ${=CFG} -automation
echo "launched pid $RON_WINE_PID"

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
lobby_click solo 4 "$T/r$N-solo.png"
lobby_click quick 8 "$T/r$N-quick.png"
lobby_start 20 "$T/r$N-"

last=0; still=0
for i in {1..90}; do
  sleep 20
  sz=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)
  if [ "$sz" = "$last" ]; then still=$((still + 1)); else still=0; fi
  last=$sz
  echo "$(date +%H:%M:%S) gamelog=$sz still=$still"
  if [ $still -ge 4 ] && [ "$sz" -gt 10000000 ]; then echo settled; break; fi
done
lobby_shot "$T/r$N-end.png"
pkill -f $P || true
sleep 4
mv "$L/gamelog.txt" "$L/gamelog-run$N-$TAG.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-$TAG.txt" "$L/rontrace-run$N.log"
grep -a -m1 "MAP_STYLE" "$L/gamelog-run$N-$TAG.txt"
grep -a -m1 "(int)seed" "$L/gamelog-run$N-$TAG.txt"
echo "run$N archived — expect MAP_STYLE $MAPSTYLE and seed 12345"
