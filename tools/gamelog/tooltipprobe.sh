#!/bin/zsh
# tooltipprobe.sh N TMPDIR — is the profile's <KEYS> read at all?
#
# `longtrace.sh`'s DRIVER hook. The one question item 23's Echelon half is
# waiting on. `KeyMap::save_entry@007d4220` writes a binding into the
# **profile** for any entry whose `dependent` is negative, so the profile is
# meant to hold the player's own keys — but the reader that would take them
# back, `KeyMap::load`'s `String` overload at `007d39a0`, has no caller the
# export shows outside unwind funclets. run50 bound `FORM_E_RIGHT` to F9 and
# the formation never changed, which is consistent with either "the profile is
# not read" or "the bind was wrong in some detail", and cannot tell them
# apart: a formation that does not happen looks the same both ways.
#
# So bind something whose key is **visible**. `OPTION_AUTO_EXPLORE` prints its
# own binding in its tooltip — run48 photographed "Auto Explore: ON - Click to
# turn off Auto Explore. (Hotkey: CTRL + E)". Rebind it to F9 and hover the
# button:
#
#   tooltip says F9      -> the profile IS read, and run50's bind was wrong
#   tooltip says CTRL+E  -> the profile is NOT read, and the <INPUT> has to go
#                           into the shipped data/playerprofile.xml instead
#
# Either answer closes the question from the behavioural side, which is why
# this is worth five minutes when a decompile read is worth an hour.
#
# The button is the compass ringed with arrows, bottom-left cell of the card's
# third row, at (32, 1017) on the 1920x1080 desktop — run48 identified it by
# its own tooltip. **The dwell matters**: run48 got a tooltip after 1.5 s and
# run49 got only a highlight after 1.2 s, so this waits 3 s and shoots twice.
set -e
N=${1:-51}
T=${2:-/tmp/ron-runs}
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

BX=${BUTTON_X:-32}
BY=${BUTTON_Y:-1017}

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
  zsh "${0:A:h}/focus.sh" >/dev/null 2>&1
  sleep 0.4
}

hover_shot() {
  focus
  # Park away first, so the tooltip is raised by *this* hover rather than
  # lingering from the last one.
  cliclick m:960,300
  sleep 1
  cliclick m:$BX,$BY
  sleep 3
  screencapture -x -R0,600,1400,500 "$T/r$N-$1.png" 2>/dev/null || true
  echo "$(date +%H:%M:%S) hovered the Auto Explore button, shot $1, frame $(frame_now)"
}

wait_frame 195
hover_shot "a"
wait_frame 215
hover_shot "b"
cliclick m:960,300
echo "probe done"
