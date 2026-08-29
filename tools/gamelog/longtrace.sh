#!/bin/zsh
# longtrace.sh [N] [FRAMES] [TAG] [MAPSTYLE] — the long traced capture (item 38).
#
# One run that is both a **dump at run10's own detail** and a **draw-site
# trace**, over enough frames to measure the whole of run10 rather than its
# first 284. run14 — the only trace of this game — stops at 284 of run10's
# 1,772 frames, and since queue item 66 its word matches for all of them, so
# nothing on disk says where the simulation next parts by *site*.
#
# The game is run10–14's by default: seed 12345, map style 14 (Great Lakes),
# the profile's lobby with `-config check.ini -automation`, no input. Pass a
# `MAPSTYLE` index from `data/rules.xml`'s `mapstyles` category list — 14 is
# Great Lakes, 18 East Indies — and the same capture is taken on that map
# instead: the lobby reads the style from the **profile**, never from
# `check.ini`, so both are written with the game closed rather than clicked
# (`docs/ORACLE.md`, "The lobby is a file"). The settings are run10's
# exactly —
#
#   [Start Game] WORLD=6 TERRAIN=2 GOODS=3 UNITS=3 BUILDS=7 CITIES=5
#                GUYS=2 LEADERS=9 DEATHS=1
#   [End Frame]  UNITS=3 BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1
#
# — so the log is a drop-in longer sibling of `gamelog-run10-world6-long.txt`
# and the two can be diffed frame for frame. That diff is also the proof that
# the traced executable and `!ffwd` leave the stream alone over 1,800 frames
# rather than only over the four run18a checked.
#
# `!ffwd 30` sets fast_forward_frame to 27,000, past the end of the run, so
# the wall-clock cap never binds; the dump is then the only speed floor
# (~3 frames a second at UNITS=3), which is what run10 ran at too.
#
# Needs all three macOS permissions — Screen Recording, Automation and
# **Accessibility** (`cliclick p` must answer a real cursor position, not
# 0,0). Restores gamelog.ini/rise.ini/rise2.ini at the end.
set -e
N=${1:-33}
FRAMES=${2:-1850}
TAG=${3:-longtrace}
MAPSTYLE=${4:-14}
# **`-config check.ini` pins the map style and no file can move it.** The
# lobby it builds is a default `GameInfo` with the rules half of the file
# applied on top, and `mapstyles=` is one of the four combos that never
# take (`docs/ORACLE.md`, "The lobby is a file"), so the style stays the
# default 14 — Great Lakes — whatever the profile says. Without `-config`
# the lobby is the **profile's**, and `mapstyle.py`'s write does take.
# So: the Great Lakes captures keep `-config`, because run10–14's game is
# that lobby; a run on any other map drops it and takes the profile's,
# recording its own lobby in its own `GAME INFO` block either way.
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

# --- the probe. Three permissions, and they come back separately; then the
# lobby's own table, because the buttons are not in the same place on both of
# this machine's desktops (`tools/gamelog/lobby.sh`).
osascript -e 'tell application "System Events" to get name of first process' >/dev/null \
  || { echo "automation is off"; exit 1; }
pos=$(cliclick p 2>/dev/null | tail -1)
if [ "$pos" = "0,0" ]; then
  echo "Accessibility is off (cliclick p answered $pos) — System Settings ->"
  echo "Privacy & Security -> Accessibility, for Claude Code."
  exit 1
fi
source "$W/tools/gamelog/lobby.sh"
lobby_init "$T/probe$N.png" || exit 1
echo "probe ok (cursor $pos)"

# --- stage. The map style lives in two files and neither is the lobby's
# combo: `check.ini`'s `mapstyles=` and the profile's `<MULTI>` block.
python3 "$W/tools/gamelog/mapstyle.py" "$MAPSTYLE"
python3 "$W/tools/fuzz/seedini.py" 12345
python3 "$W/tools/gamelog/setlog.py" 0 \
  end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=1 \
  start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1
python3 "$W/tools/gamelog/window.py" frames 0 $((FRAMES + 50))
printf 'cover=1\n' > "$G/rontrace.cfg"
{
  echo "# longtrace.sh — run10's game, traced, to frame $FRAMES."
  echo "5 !ffwd 30"
  echo "$FRAMES !quit"
} > "$G/rontrace.cmd"

# --- run
rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
nohup /Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine \
  --bottle ron --workdir "$G" --wait-children \
  "$G/$P" ${=CFG} -automation > "$T/wine$N.log" 2>&1 &
echo "launched pid $!"

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
lobby_click solo 4 "$T/r$N-solo.png"
lobby_click quick 8 "$T/r$N-quick.png"
lobby_start 20 "$T/r$N-"

last=0; still=0
for i in {1..160}; do
  sleep 20
  sz=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)
  fr=$(tail -c 4000000 "$L/gamelog.txt" 2>/dev/null | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1)
  if [ "$sz" = "$last" ]; then still=$((still + 1)); else still=0; fi
  last=$sz
  echo "$(date +%H:%M:%S) gamelog=$sz last='$fr' still=$still"
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
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-$TAG.txt"
echo "run$N archived — expect MAP_STYLE $MAPSTYLE, seed 12345 and ~$FRAMES frame blocks"
