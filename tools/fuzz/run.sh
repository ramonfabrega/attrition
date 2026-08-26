#!/bin/zsh
# run.sh SEED [LO HI] — one fuzzed seed, end to end, unattended.
#
# Writes the seed into `rise.ini`, the scenario into `rontrace.cmd`, stages the
# dump window, launches, clicks the lobby, waits for the channel's own `!quit`,
# archives the dump, diffs it and appends a ledger row.
#
# **Why the seed goes in `rise.ini` and not through `restart`.** Item 13's plan
# was many scenarios per launch via the console's `restart <seed>`. That does
# not work: the channel fires at `Game::do_frame` entry, and `restart`'s
# `Game::close` / `Game::init` tear down the game whose tick it is in --
# `parse_cmd` never returns, the screen goes black and the process wedges with
# no further frames (gate run, 2026-08-26; the trace's last `INFO cmd` is the
# line before it). `rise.ini`'s `Seed (0 for random)` costs one launch per seed
# instead, needs no re-entrancy, and still varies the generated map, which is
# the whole point -- every capture before this one used the same lobby.
#
# The lobby clicks are fixed coordinates for the 3440x1440 display the window
# opens on at (760, 152); they are the same six `runwin.sh` uses. If the window
# moves, this is what breaks first, and the screenshots under $T are how you
# see that rather than guess it.
set -e
SEED=${1:?usage: run.sh SEED [LO HI]}
LO=${2:-3000}
HI=${3:-3020}
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
R="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$R/Logs"
P=riseofnations_trace.exe

python3 "$W/tools/fuzz/seedini.py" "$SEED"
python3 "$W/tools/gamelog/window.py" stage "$LO" "$HI"
printf 'cover=1\nwindow=%d-%d\n' "$LO" "$((HI - 1))" > "$G/rontrace.cfg"
RON_INSTALL=$G python3 "$W/tools/fuzz/scenario.py" "$SEED" --lo "$LO" --hi "$HI" > "$G/rontrace.cmd"
echo "=== seed $SEED, window [$LO, $HI) ==="; cat "$G/rontrace.cmd"

rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
nohup /Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine \
  --bottle ron --workdir "$G" --wait-children \
  "$G/$P" -config check.ini -automation > "$T/wine-$SEED.log" 2>&1 &

click() {
  osascript -e "tell application \"System Events\" to set frontmost of process \"$P\" to true" >/dev/null 2>&1
  sleep 0.5
  cliclick m:$1,$2 w:400 c:$1,$2
  sleep ${4:-3}
  screencapture -x -R760,152,1920,1108 "$T/s$SEED-$3.png" 2>/dev/null
  echo "$(date +%H:%M:%S) clicked $3"
}

zsh "$W/tools/gamelog/waitwin.sh" "$T/s$SEED-menu.png"
sleep 4
click 1715 744 solo 4
click 1715 672 quick 8
click 2392 291 combo 3
click 2262 551 indies 3
click 1061 1176 start1 3
click 1061 1176 start2 20

# The channel quits at HI+1, so wait for the process rather than for a size.
for i in $(seq 1 90); do
  sleep 10
  pgrep -f $P >/dev/null || { echo "quit cleanly"; break; }
  echo "$(date +%H:%M:%S) gamelog=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)"
done
screencapture -x -R760,152,1920,1108 "$T/s$SEED-end.png" 2>/dev/null
pkill -f $P 2>/dev/null || true
sleep 3

DUMP="$L/gamelog-fuzz-$SEED.txt"
mv "$L/gamelog.txt" "$DUMP"
cp "$G/rontrace.log" "$L/rontrace-fuzz-$SEED.log"
python3 "$W/tools/gamelog/window.py" restore

cargo run --quiet --manifest-path "$W/Cargo.toml" -p rondata -- "$G" \
  --gamelog "$DUMP" --diff > "$T/diff-$SEED.txt" 2>&1 || true
tail -20 "$T/diff-$SEED.txt"
python3 "$W/tools/fuzz/ledger.py" append "$SEED" --lo "$LO" \
  --diff "$T/diff-$SEED.txt" --trace "$L/rontrace-fuzz-$SEED.log" \
  --note "window $LO-$HI"
echo "seed $SEED done"
