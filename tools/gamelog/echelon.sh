#!/bin/zsh
# echelon.sh N TMPDIR — put a group in an Echelon, and march it.
#
# `longtrace.sh`'s DRIVER hook. Item 23's second half: `docs/GROUPS.md` §6.4's
# slot table reads `reverse` on the **Echelon** rows only — Refused is
# `Y = Y0 - |X|`, with no `reverse` in it — so a Line, which is every group in
# every capture on disk, cannot show what the mirror does to a position.
#
# There is no console verb for a formation and no button on the card. What
# there is, is a **bindable action with no default binding**:
# `data/playerprofile.xml` is the keymap `KeyMap::init@007d5a90` loads, and its
# `FORM_*` entries carry no `<INPUT>` child. `tools/gamelog/bindkey.py` writes
# one into the profile, so the formation becomes a keystroke.
#
# The order of business matters. A formation is a property of a **group**, and
# a group exists once the units have been given a move order together, so this
# presses the key once with only the selection (in case it takes there) and
# again after the first march order. Then it marches the group back and
# forth the way run46 did, so the mirror toggles inside an Echelon rather than
# inside a Line.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
set -e
N=${1:-50}
T=${2:-/tmp/ron-runs}
L="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

CX=${CLICK_X:-960}
CY=${CLICK_Y:-445}
# macOS key code for F9, which `bindkey.py` binds to FORM_E_RIGHT (VK 120).
FKEY=${FKEY:-101}

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

focus() {
  zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
  sleep 0.4
}

press_form() {
  focus
  osascript -e "tell application \"System Events\" to key code $FKEY" >/dev/null 2>&1
  sleep 1
  echo "$(date +%H:%M:%S) pressed the formation key at frame $(frame_now)"
}

rclick() {
  focus
  cliclick m:$CX,$CY
  sleep 0.5
  cliclick rc:$CX,$CY
  sleep 1
  echo "$(date +%H:%M:%S) right-click at frame $(frame_now)"
}

# 1. the key with only a selection
wait_frame 195
press_form
screencapture -x "$T/r$N-a-after-key.png" 2>/dev/null || true

# 2. the first march, which is what makes a group
wait_frame 212
rclick
screencapture -x "$T/r$N-b-first-march.png" 2>/dev/null || true

# 3. the key again, now that a group exists
wait_frame 250
press_form
screencapture -x "$T/r$N-c-key-with-group.png" 2>/dev/null || true

# 4. and march it back and forth, so the mirror toggles inside whatever
#    formation is now set.
wait_frame 332
rclick
wait_frame 452
rclick
screencapture -x "$T/r$N-d-third-march.png" 2>/dev/null || true
echo "driver: done"
