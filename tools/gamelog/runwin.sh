#!/bin/zsh
# runwin.sh N LO HI TAG — a DUMP_ALL window run [LO, HI) of run24's game (the
# islands lobby with human hoplites dropped beside the AI capital at 12000):
# stage with window.py, re-add the hoplite lines, launch, drive the lobby,
# wait for the dump to settle after !quit, archive as run<N>-<TAG>.
N=$1; LO=$2; HI=$3; TAG=$4
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
P=riseofnations_trace.exe
source "$W/tools/gamelog/lobby.sh"
lobby_init || exit 1

python3 "$W/tools/gamelog/window.py" stage $LO $HI
Q=$((HI + 1))
cat > "$G/rontrace.cmd" <<EOF
# run$N - run24's game (islands, hoplites beside the AI capital at 12000),
# DUMP_ALL window [$LO, $HI): $TAG. Dump off until the window.
5 !ffwd 30
12000 add hoplite who=0 203,207
12002 add hoplite who=0 205,207
12004 add hoplite who=0 203,210
16000 add hoplite who=0 203,207
16002 add hoplite who=0 205,207
16004 add hoplite who=0 203,210
16006 add hoplite who=0 206,210
$Q !quit
EOF
echo "staged run$N [$LO, $HI)"; cat "$G/rontrace.cfg"

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

# wait for the dump to settle: gamelog.txt unchanged for 90 s once past 100 MB
last=0; still=0
for i in {1..160}; do
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
mv "$L/gamelog.txt" "$L/gamelog-run$N-islands-$TAG.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-islands-$TAG.txt" "$L/rontrace-run$N.log"
grep -a -n -m1 "MAP_STYLE" "$L/gamelog-run$N-islands-$TAG.txt"
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-islands-$TAG.txt"
echo "run$N archived"
