#!/bin/zsh
# archive.sh N TAG — end a live.sh run: kill the process, name the log
# gamelog-run<N>-<TAG>.txt beside its trace, restore the ini window, and print
# the map style and the frame count so the run is identified at a glance.
N=$1; TAG=$2
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"

screencapture -x -R760,152,1920,1108 "$T/r$N-end.png"
sips -Z 900 "$T/r$N-end.png" --out "$T/r$N-ends.png" >/dev/null
pkill -f riseofnations_trace.exe; sleep 4
mv "$L/gamelog.txt" "$L/gamelog-run$N-$TAG.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-$TAG.txt" "$L/rontrace-run$N.log"
grep -a -n -m1 "MAP_STYLE" "$L/gamelog-run$N-$TAG.txt"
grep -a -c "BEGIN FRAME" "$L/gamelog-run$N-$TAG.txt"
echo "run$N archived"
