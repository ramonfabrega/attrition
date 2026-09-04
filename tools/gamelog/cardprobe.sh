#!/bin/zsh
# cardprobe.sh N TMPDIR — name every button on the unit command card.
#
# `longtrace.sh`'s DRIVER hook, a diagnostic. Item 23's remaining half needs a
# group in an **Echelon**, and `data/options.xml` proves the button exists —
# the card's option table carries `Formation`, `Formation Back`, `Face Left`,
# `Face Right`, `About Face` and `Forward` — while run48 shows a card with
# only move, attack, auto-explore, board, stop and garrison on it. Something
# gates the rest.
#
# The likely gate is **Military research**: RoN unlocks formations with
# military tech, and every capture so far is an Ancient-age nation with none.
# The console can lift it (`military [level] [who]`, `tech`), so the stanza
# does that before selecting, and this maps the card afterwards.
#
# The mapping is by **tooltip**, which is the reliable instrument here: run48
# established that hovering a card button pops a panel naming the command and
# its key — "Auto Explore: ON - Click to turn off Auto Explore. (Hotkey: CTRL
# + E)". So hover each cell in turn and photograph it; the shots name the
# card. Nothing is clicked, so nothing is ordered and a wrong guess costs
# screenshots.
#
# The grid, measured off run47's 1920x1080 shot: five columns ~57 apart from
# x 32, four rows at y 935, 976, 1017 and 1058.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
set -e
N=${1:-49}
T=${2:-/tmp/ron-runs}
L="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

COLS=(32 89 146 203 260)
ROWS=(935 976 1017 1058)

frame_now() {
  tail -c 2000000 "$L/gamelog.txt" 2>/dev/null \
    | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1 | awk '{print $3}'
}

wait_frame() {
  local want=$1 i f
  for i in {1..400}; do
    f=$(frame_now)
    if [ -n "$f" ] && [ "$f" -ge "$want" ]; then return 0; fi
    sleep 2
  done
}

osa_focus() {
  zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
  sleep 0.3
}

wait_frame 190
osa_focus
screencapture -x "$T/r$N-card.png" 2>/dev/null || true
echo "$(date +%H:%M:%S) card shot at frame $(frame_now)"

r=0
for y in $ROWS; do
  r=$((r + 1))
  c=0
  for x in $COLS; do
    c=$((c + 1))
    cliclick m:$x,$y
    sleep 1.2
    # Only the strip above the card can hold the tooltip, so keep the shots
    # small: 1200x260 from y 640 is the whole popup and the card's top rows.
    screencapture -x -R0,640,1200,440 "$T/r$N-tip-r${r}c${c}.png" 2>/dev/null || true
    echo "$(date +%H:%M:%S) hovered row $r col $c at $x,$y"
  done
done

# Park the mouse off the card so nothing stays highlighted.
cliclick m:960,400
echo "probe done"
