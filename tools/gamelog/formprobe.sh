#!/bin/zsh
# formprobe.sh N TMPDIR — where the formation buttons are.
#
# `longtrace.sh`'s DRIVER hook, a diagnostic. Item 23's remaining half needs a
# group in a **Refused or an Echelon**: `docs/GROUPS.md` §6.4's slot table only
# reads `reverse` on the Echelon rows, so a Line — which is every group in
# every capture on disk — cannot show what the mirror does to a position.
#
# The console cannot do it. Its 102 commands, re-derived from the install by
# `tools/gamelog/console.py`, have no formation verb, and the profile's
# `<KEYS/>` is empty so the game is on its built-in bindings, which are not in
# any data file. `data/playerprofile.xml` does name the actions —
# `FORM_LINE`, `FORM_REFUSED`, `FORM_ENVELOP`, `FORM_E_RIGHT`, `FORM_E_LEFT` —
# so they exist as bindable commands; what is unknown is the key and the
# button.
#
# The command card, read off run47's screenshot with hoplites selected, is a
# 2-wide grid in the bottom-left: move and attack on the top row, then a
# **compass ringed with arrows** and a ship, then stop and garrison. The
# compass is the candidate: in this game a card button can swap the card for a
# sub-card, and a formation chooser is exactly the shape of thing that would.
#
# So: photograph the card, click the compass, photograph it again. If the card
# changes, the second shot names the buttons and a second run can press one.
# Nothing here issues an order, so a wrong guess costs a screenshot.
set -e
N=${1:-48}
T=${2:-/tmp/ron-runs}
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

# The card's buttons, measured off run47's 1920x1080 shot: the grid starts at
# about x 5 and each cell is ~57 wide and ~38 tall, the rows at y 916, 954,
# 1000 and 1035.
COMPASS_X=${COMPASS_X:-32}
COMPASS_Y=${COMPASS_Y:-1017}

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

shot() {
  screencapture -x "$T/r$N-$1.png" 2>/dev/null || true
  echo "$(date +%H:%M:%S) shot $1 at frame $(frame_now)"
}

focus() {
  osascript -e 'tell application "System Events" to set frontmost of process "riseofnations_trace.exe" to true' >/dev/null 2>&1
  sleep 0.4
}

wait_frame 190
shot "a-card-before"

focus
echo "$(date +%H:%M:%S) clicking the compass at $COMPASS_X,$COMPASS_Y"
cliclick m:$COMPASS_X,$COMPASS_Y
sleep 0.5
cliclick c:$COMPASS_X,$COMPASS_Y
sleep 1.5
shot "b-card-after-compass"

# Hover the top-left card cell so any tooltip names it.
focus
cliclick m:32,935
sleep 1.5
shot "c-hover-first"

wait_frame 240
shot "d-later"
echo "probe done"
