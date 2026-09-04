#!/bin/zsh
# censuswindow.sh [N] [LO] [HI] [TAG] [MAPSTYLE] — run10's game with the
# **leader census** dumped over a frame window.
#
# `longtrace.sh` runs the whole game at run10's `[End Frame]` detail, where
# `LEADERS=1` prints five scalars and no resources at all. A `LEADERS=9`
# record is the census oracle — `resources`, the `MAKELIST`, the ten `SITES`,
# `production_step`, `script_step`, the `reg_*` arrays (`docs/ORACLE.md`,
# "`LEADERS=9` is the census oracle") — and it is ~10k lines a leader, so it
# is a *window* setting rather than a whole-run one.
#
# Everything else is `longtrace.sh`'s: seed 12345, map style 14, the
# `-config check.ini` lobby, `!ffwd 30`, no input. The trace is taken whole
# (`cover=1`, no window) so the run's word can be diffed against run33's
# frame for frame — which is the proof it is the same game, since the dump
# blocks now carry records run33's do not and `samegame.py` cannot compare
# them.
#
# Needs the three macOS permissions (see `longtrace.sh`).
set -e
N=${1:-40}
LO=${2:-560}
HI=${3:-600}
TAG=${4:-census}
MAPSTYLE=${5:-14}
if [ -z "${CFG+set}" ]; then
  if [ "$MAPSTYLE" = 14 ]; then CFG="-config check.ini"; else CFG=""; fi
fi
W=$(cd "$(dirname "$0")/../.." && pwd)
G=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
B="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$B/Logs"
T=${RON_TMP:-/tmp/ron-runs}
P=riseofnations_trace.exe
mkdir -p "$T"

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
python3 "$W/tools/gamelog/setlog.py" 0 \
  end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=9 \
  start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1
python3 "$W/tools/gamelog/window.py" frames "$LO" "$HI"
printf 'cover=1\n' > "$G/rontrace.cfg"
{
  echo "# censuswindow.sh — run10's game, LEADERS=9 over [$LO, $HI)."
  echo "5 !ffwd 30"
  echo "$HI !quit"
} > "$G/rontrace.cmd"

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
  fr=$(tail -c 4000000 "$L/gamelog.txt" 2>/dev/null | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1)
  if [ "$sz" = "$last" ]; then still=$((still + 1)); else still=0; fi
  last=$sz
  echo "$(date +%H:%M:%S) gamelog=$sz last='$fr' still=$still"
  if [ $still -ge 4 ] && [ "$sz" -gt 1000000 ]; then echo settled; break; fi
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
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-$TAG.txt"
echo "run$N archived — expect MAP_STYLE $MAPSTYLE, seed 12345 and ~$((HI - LO)) frame blocks"
