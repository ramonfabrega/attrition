#!/bin/zsh
# gate.sh N — the one run item 13's Tier 1 rests on.
#
# Tier 1 wants many scenarios per launch, which needs `restart <seed>` to work
# from the cheat channel: `run_cmd` case 0x5d sets `game->info.seed`, calls
# `Game::close` then `Game::init` in place, and leaves the game paused
# (`TurnControl::set_pause(1)`). Three things are unproven and one run settles
# all of them:
#
#   1. `!restart` runs *inside* `Game::do_frame` — the channel fires at do_frame
#      entry — so it closes and re-inits the game whose tick it is in. `game` is
#      a singleton, so `this` stays valid, but nothing says do_frame survives it.
#   2. `Game::init` clears `fast_forward_frame`, so game 2 needs its own `!ffwd`.
#      Both it and `!go` (the unpause) must ride in the same frame's batch,
#      because a paused game stops calling do_frame and the channel with it.
#   3. Whether the second game emits its own `[Start Game]` block — the cheap
#      proof that the seed took and the map is different.
#
# The dump is off (`LogStartFrame=LogEndFrame=999999`), so the evidence is the
# two ungated start blocks and the trace's `INFO cmd` records.
N=${1:-32}
T=${RON_TMP:-/tmp/ron-runs}; mkdir -p "$T"
G=/Users/rf-studio/code/fun/attrition/game
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
P=riseofnations_trace.exe
source "$W/tools/gamelog/lobby.sh"
lobby_init || exit 1
SEED=305419896   # 0x12345678, and nothing like the ini's 12345

python3 "$W/tools/gamelog/window.py" stage 999999 999999

cat > "$G/rontrace.cfg" <<EOF
cover=1
window=900-902
EOF

cat > "$G/rontrace.cmd" <<EOF
# gate run$N — does the channel's \`restart\` work in-tick and unattended?
5 !ffwd 30
600 add hoplite who=0 203,207
900 !restart $SEED
900 !go
900 !ffwd 30
1200 add hoplite who=0 203,207
1500 !quit
EOF
echo "staged gate run$N, seed $SEED"; cat "$G/rontrace.cmd"

rm -f "$G/rontrace.log" "$L/gamelog.txt"
cd "$G"
source "$W/tools/gamelog/winelaunch.sh"
ron_wine "$T/wine$N.log" "$G/$P" -config check.ini -automation
echo "launched pid $RON_WINE_PID"

zsh "$W/tools/gamelog/waitwin.sh" "$T/r$N-menu.png"
sleep 4
lobby_click solo 4 "$T/r$N-solo.png"
lobby_click quick 8 "$T/r$N-quick.png"
# The islands lobby's Map Style, measured on the wide desktop only;
# elsewhere set it in `PlayerProfile/Player.dat` with the game closed.
lobby_click combo 3 "$T/r$N-combo.png"
lobby_click indies 3 "$T/r$N-indies.png"
lobby_start 20 "$T/r$N-"

# the dump is off, so wait on the process rather than on the log growing
for i in {1..80}; do
  sleep 10
  sz=$(stat -f %z "$L/gamelog.txt" 2>/dev/null || echo 0)
  tr=$(stat -f %z "$G/rontrace.log" 2>/dev/null || echo 0)
  echo "$(date +%H:%M:%S) gamelog=$sz trace=$tr"
  pgrep -f $P >/dev/null || { echo "exited"; break; }
done
lobby_shot "$T/r$N-end.png"
pkill -f $P; sleep 4
mv "$L/gamelog.txt" "$L/gamelog-run$N-gate-restart.txt"
cp "$G/rontrace.log" "$L/rontrace-run$N.log"
python3 "$W/tools/gamelog/window.py" restore
ls -la "$L/gamelog-run$N-gate-restart.txt" "$L/rontrace-run$N.log"
echo "--- start blocks and their seeds ---"
grep -a -n "game_random seed" "$L/gamelog-run$N-gate-restart.txt"
echo "--- cheat lines that ran ---"
python3 "$W/tools/trace/report.py" "$L/rontrace-run$N.log" summary 2>/dev/null | grep -i "cmd" || true
echo "run$N archived"
