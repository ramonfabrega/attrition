#!/bin/zsh
# clickdriver.sh N TMPDIR — the right-clicks a capture cannot script.
#
# `longtrace.sh`'s `DRIVER` hook. The cheat channel issues **no orders at
# all** (`docs/ORACLE.md`, "The channel's vocabulary"): `move` is a teleport
# through `Unit::set_new_location`, and nothing in the 102-command table
# touches an order list. So `GroupMoveOrder` — the record queue item 23 needs
# — exists in exactly one capture on disk, run31, and that one was driven by
# hand. Counted over nine archives: run31 has 945, and run13, run20, run22,
# run25, run26, run27, run29 and run45 have none, three of those being
# `DUMP_ALL` windows taken during the AI's own fighting.
#
# **The trick that avoids a projection fit.** `cheat camera X,Y` puts tile
# `(X, Y)` at the viewport centre, so an order to a tile is "camera there,
# right-click the centre" — one constant to know instead of the four of
# `aim.py`'s linear fit, and it survives a window that moves between
# launches. The stanza's `cmd:` lines do the camera moves on their own sim
# frames; this waits for each to have happened and clicks.
#
# **Why the destinations alternate.** The mirror flag toggles when the
# group's leader is set to a heading 90 degrees or more off its current one
# (`Unit::set_angle@00605400`, the `reversing` window). Marching the group
# back and forth between two tiles on opposite sides of it makes every order
# after the first a ~180 degree turn, so the toggle is not left to luck.
#
# It is deliberately blind: it clicks on a frame and screenshots the result
# rather than trying to read the game's state. What the click did is read
# afterwards, out of the dump, by `groupfacing.py`.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
set -e
N=${1:-46}
T=${2:-/tmp/ron-runs}
W=$(cd "$(dirname "$0")/../.." && pwd)
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

# The viewport centre. On the 1920-wide desktop the game is full-screen at
# (0, 0); the map's centre sits above the command bar, not at the screen's
# middle. `docs/ORACLE.md` measured the wide desktop's at window-relative
# (960, 422) and then at (960, 468) — "the window moves between launches, so
# re-probe rather than trust either number" — so this takes the midpoint and
# the run's own screenshots are the re-probe.
CX=${CLICK_X:-960}
CY=${CLICK_Y:-445}

# The frames the stanza's `camera` lines run on. A click must land *after*
# the camera has moved: a `.cmd` line runs at the top of its sim frame, and
# `BEGIN FRAME n` is written at the end of sim-frame n-1, so seeing `n+2`
# means the camera of frame `n` has certainly happened.
CLICK_FRAMES=${CLICK_FRAMES:-"210 330 450"}

frame_now() {
  tail -c 2000000 "$L/gamelog.txt" 2>/dev/null \
    | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1 | awk '{print $3}'
}

echo "driver: clicking at ($CX, $CY) on frames $CLICK_FRAMES"
for target in ${=CLICK_FRAMES}; do
  want=$((target + 2))
  # Wait for the game to pass the camera's frame, with a wall-clock ceiling
  # so a stalled run does not hang the capture.
  for i in {1..400}; do
    f=$(frame_now)
    if [ -n "$f" ] && [ "$f" -ge "$want" ]; then break; fi
    sleep 2
  done
  f=$(frame_now)
  echo "$(date +%H:%M:%S) frame $f — right-click at $CX,$CY (camera was frame $target)"
  zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
  sleep 0.4
  cliclick m:$CX,$CY
  sleep 0.5
  cliclick rc:$CX,$CY
  sleep 1
  screencapture -x "$T/r$N-click$target.png" 2>/dev/null || true
  sips -Z 900 "$T/r$N-click$target.png" --out "$T/r$N-click${target}s.png" >/dev/null 2>&1 || true
done
echo "driver: done"
