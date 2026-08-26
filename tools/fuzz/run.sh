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
# **Why the window is the cheap one, not `DUMP_ALL`.** The first version staged
# with `window.py stage`, which sets `DUMP_ALL=1` because `scene_at` wants a
# `WORLD` block with its 3600 cells. It does -- but `[Start Game] WORLD=6`
# writes those cells on its own with `DUMP_ALL=0` (run31's start dump has all
# 3600), and `run_traced` stands the sim up from the **start** dump and ticks
# forward, so no frame inside the window needs a `WORLD` block at all. Measured
# on the two captures: `DUMP_ALL` cost 49.9 MB a frame and bought 5 frames;
# the cheap window costs 0.73 MB a frame and bought 219. Two orders of
# magnitude, for an ini setting. `docs/ORACLE.md`, "The 300-frame window".
#
# The `[End Frame]` set is run31's, `DEATHS=0` included: `dump_deaths` ends by
# calling `WorldData::log_data` twice, and `GroupData::log_data` never sets its
# own type, so with `DEATHS` on the group pool is silently dropped.
#
# The lobby clicks are fixed coordinates for the 3440x1440 display the window
# opens on at (760, 152); they are the same six `runwin.sh` uses. If the window
# moves, this is what breaks first, and the screenshots under $T are how you
# see that rather than guess it.
#
# `FUZZ_STAGE=0` runs the control shape: an early window and no cheats at all,
# which is the only way `survived` measures fidelity rather than where the
# window was put. See `scenario.py --no-stage`.
set -e
SEED=${1:?usage: run.sh SEED [LO HI]}
LO=${2:-1000}
HI=${3:-1300}
STAGE=${FUZZ_STAGE:-1}
[ "$STAGE" = 0 ] && NOSTAGE=--no-stage || NOSTAGE=
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
R="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$R/Logs"
P=riseofnations_trace.exe

python3 "$W/tools/fuzz/seedini.py" "$SEED"
python3 "$W/tools/gamelog/setlog.py" 0 \
  'end:UNITS=9,GROUPS=9,GUYS=9,LEADERS=1,MISC=9' \
  'start:UNITS=3,GROUPS=1,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1,GOODS=3,TERRAIN=2,WORLD=6,MISC=1'
python3 "$W/tools/gamelog/window.py" frames "$LO" "$HI"
printf 'cover=1\nwindow=%d-%d\n' "$LO" "$((HI - 1))" > "$G/rontrace.cfg"
RON_INSTALL=$G python3 "$W/tools/fuzz/scenario.py" "$SEED" --lo "$LO" --hi "$HI" \
  ${NOSTAGE:+$NOSTAGE} > "$G/rontrace.cmd"
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

# `!quit` returns to the **main menu**; it does not exit the process (run16b,
# `docs/ORACLE.md`). So wait for the dump to settle -- unchanged for a minute
# after it has grown -- rather than for an exit that never comes. Waiting on
# the process cost seed 424242 a quarter of an hour of spinning.
last=0; still=0
for i in $(seq 1 120); do
  sleep 10
  sz=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)
  if [ "$sz" = "$last" ]; then still=$((still + 1)); else still=0; fi
  last=$sz
  echo "$(date +%H:%M:%S) gamelog=$sz still=$still"
  if [ "$still" -ge 6 ] && [ "$sz" -gt 1000000 ]; then echo "settled"; break; fi
  pgrep -f $P >/dev/null || { echo "process gone"; break; }
done
screencapture -x -R760,152,1920,1108 "$T/s$SEED-end.png" 2>/dev/null
pkill -f $P 2>/dev/null || true
sleep 3

# The archive name carries the *shape*, not just the seed. Two runs of one
# seed -- staged and control -- are two different captures, and the first
# version of this named both `-fuzz-$SEED` and silently clobbered the staged
# run's trace with the control's. That cost the one measurement that would
# have said whether the cheat staging buys any function coverage at all
# (`ledger.tsv` has 6872 funcs against 6684, and whether any of the 188 are
# on the blind list is now unanswerable without a re-run).
TAG="$SEED-$LO"; [ "$STAGE" = 0 ] && TAG="$TAG-nostage"
DUMP="$L/gamelog-fuzz-$TAG.txt"
mv "$L/gamelog.txt" "$DUMP"
cp "$G/rontrace.log" "$L/rontrace-fuzz-$TAG.log"
python3 "$W/tools/gamelog/window.py" restore

cargo run --quiet --manifest-path "$W/Cargo.toml" -p rondata -- "$G" \
  --gamelog "$DUMP" --diff > "$T/diff-$TAG.txt" 2>&1 || true
tail -20 "$T/diff-$TAG.txt"
python3 "$W/tools/fuzz/ledger.py" append "$SEED" --lo "$LO" \
  --diff "$T/diff-$TAG.txt" --trace "$L/rontrace-fuzz-$TAG.log" \
  --note "cheap window $LO-$HI$([ "$STAGE" = 0 ] && echo ', no staging')"
echo "seed $SEED done"
