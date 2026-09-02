#!/bin/zsh
# roadcapture.sh [N] — the road path oracle (docs/ROADS.md §7, queue item 55).
#
# Stages and runs one capture, end to end: run10-14's game (seed 12345, map
# style 14 "Great Lakes"), two enhancers dropped on fresh, un-roaded ground
# inside the human city's radius at sim-frame 100, and a `DUMP_ALL` window
# over the frames on which each replans its road.
#
#   Granary at tile (6, 171)   — object 2007, replans on sim-frame 105
#   Smelter at tile (33, 161)  — object 2008, replans on sim-frame 104
#
# `add` without the `NEW` token calls `Build::activate`, which flags the city
# through `City::regen_roads`; `Build::process` fires a building on
# `(frame + o) % 16 == 0`. A block `FRAME n` is the end of sim-frame n-1, so
# the window [104, 109) holds the state before both roads, both roads, and
# the Market's and old Library's replans behind them.
#
# What it produces, beside the dumps: the trace's per-draw *seed*, which is
# what lets the harness reproduce the search's jitters exactly
# (`sim.rng.seed`) and diff the laid road tile by tile.
#
# Needs macOS **Accessibility** for the lobby clicks (`cliclick p` must
# answer a real cursor position, not 0,0), on top of Screen Recording and
# Automation. Restores gamelog.ini/rise.ini/rise2.ini at the end; check.ini
# and Player.dat are left on map style 14 deliberately -- that is run10-14's
# game and every dump on disk is of it.
set -e
N=${1:-32}
TAG=roadpath
W=$(cd "$(dirname "$0")/../.." && pwd)
G=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
B="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$B/Logs"
T=${RON_TMP:-/tmp/ron-runs}
P=riseofnations_trace.exe
mkdir -p "$T"

# --- the probe. Three permissions, and they come back separately; then the
# lobby's own table, because the buttons are not in the same place on both of
# this machine's desktops (`tools/gamelog/lobby.sh`).
osascript -e 'tell application "System Events" to get name of first process' >/dev/null \
  || { echo "automation is off"; exit 1; }
pos=$(cliclick p 2>/dev/null | tail -1)
if [ "$pos" = "0,0" ]; then
  echo "Accessibility is off (cliclick p answered $pos) — System Settings ->"
  echo "Privacy & Security -> Accessibility, for the terminal this runs in."
  exit 1
fi
source "$W/tools/gamelog/lobby.sh"
lobby_init "$T/probe$N.png" || exit 1
echo "probe ok (cursor $pos)"

# --- stage
python3 - "$G" "$B" <<'PY'
import re, sys
G, B = sys.argv[1], sys.argv[2]
p = G + "/check.ini"
t = open(p, "rb").read().decode("utf-8", "replace")
open(p, "wb").write(re.sub(r"mapstyles=.*", "mapstyles=#ICON98Great Lakes", t).encode())
p = B + "/PlayerProfile/Player.dat"
t = open(p, "rb").read().decode("utf-8", "replace")
i = t.index("<MULTI>")
open(p, "wb").write((t[:i] + t[i:].replace('<MAP_STYLE value="18"/>',
                                           '<MAP_STYLE value="14"/>', 1)).encode())
print("map style 14 (Great Lakes) in check.ini and the profile")
PY
python3 "$W/tools/fuzz/seedini.py" 12345
python3 "$W/tools/gamelog/window.py" stage 104 109
# `RON_CALLWIN=99-101 roadcapture.sh 62` adds the three road proxies
# (`tools/trace/README.md`, "The call proxies"): every candidate's
# coordinate, every node's price, and the bracket that delimits the two
# searches. Without it the capture is run32's instrument exactly.
printf 'cover=1\nwindow=103-108\n' > "$G/rontrace.cfg"
if [ -n "$RON_CALLWIN" ]; then
  printf 'callwin=%s\n' "$RON_CALLWIN" >> "$G/rontrace.cfg"
fi
cat > "$G/rontrace.cmd" <<'EOF'
# roadcapture.sh - two enhancers on fresh ground; see docs/ROADS.md §7.
5 !ffwd 30
100 add granary who=0 6,171
100 add smelter who=0 33,161
110 !quit
EOF

# --- run
rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
nohup /Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine \
  --bottle ron --workdir "$G" --wait-children \
  "$G/$P" -config check.ini -automation > "$T/wine$N.log" 2>&1 &
echo "launched pid $!"

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
lobby_click solo 4 "$T/r$N-solo.png"
lobby_click quick 8 "$T/r$N-quick.png"
lobby_start 20 "$T/r$N-"

last=0; still=0
for i in {1..120}; do
  sleep 15
  sz=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)
  fr=$(grep -a -o 'BEGIN FRAME [0-9]*' "$L/gamelog.txt" 2>/dev/null | tail -1)
  if [ "$sz" = "$last" ]; then still=$((still+1)); else still=0; fi
  last=$sz
  echo "$(date +%H:%M:%S) gamelog=$sz last='$fr' still=$still"
  if [ $still -ge 6 ] && [ "$sz" -gt 100000000 ]; then echo settled; break; fi
done
lobby_shot "$T/r$N-end.png"
pkill -f $P; sleep 4
mv "$L/gamelog.txt" "$L/gamelog-run$N-$TAG.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-$TAG.txt" "$L/rontrace-run$N.log"
grep -a -m1 "MAP_STYLE" "$L/gamelog-run$N-$TAG.txt"
grep -a -m1 "game_random seed" "$L/gamelog-run$N-$TAG.txt"
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-$TAG.txt"
echo "run$N archived — expect MAP_STYLE 14, seed 12345 and 5 frame blocks"
