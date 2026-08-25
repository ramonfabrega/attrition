#!/bin/zsh
# runwin.sh N LO HI TAG — a DUMP_ALL window run [LO, HI) of run24's game (the
# islands lobby with human hoplites dropped beside the AI capital at 12000):
# stage with window.py, re-add the hoplite lines, launch, drive the lobby,
# wait for the dump to settle after !quit, archive as run<N>-<TAG>.
N=$1; LO=$2; HI=$3; TAG=$4
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
P=riseofnations_trace.exe

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
screencapture -x -R760,152,1920,1108 "$T/r$N-end.png"; sips -Z 900 "$T/r$N-end.png" --out "$T/r$N-ends.png" >/dev/null
pkill -f $P; sleep 4
mv "$L/gamelog.txt" "$L/gamelog-run$N-islands-$TAG.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-islands-$TAG.txt" "$L/rontrace-run$N.log"
grep -a -n -m1 "MAP_STYLE" "$L/gamelog-run$N-islands-$TAG.txt"
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-islands-$TAG.txt"
echo "run$N archived"
