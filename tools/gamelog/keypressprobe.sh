#!/bin/zsh
# keypressprobe.sh N TMPDIR — does a synthetic keystroke reach a bound action?
#
# `longtrace.sh`'s DRIVER hook, and the second half of run51's experiment.
#
# run51 settled that the profile's `<KEYS>` **is** read: with
# `OPTION_AUTO_EXPLORE` rebound to F9 by `bindkey.py`, the button's own tooltip
# stopped saying "(Hotkey: CTRL + E)" and started saying "(Hotkey: F9)". So
# `bindkey.py`'s element is right and `KeyMap::load`'s String overload does run,
# whatever the export shows about its callers.
#
# That leaves run50's failure unexplained rather than explained. Its
# `FORM_E_RIGHT` bind would have loaded the same way, so the formation not
# changing means either the keystroke never arrived or the action needs a
# context the scenario lacked. **Those are different bugs and this separates
# them**, using the one action whose state is legible: Auto Explore prints
# ON or OFF in its own tooltip.
#
#   hover -> OFF, press F9, hover -> ON   the keystroke arrives; run50's
#                                         problem is FORM_E_RIGHT's context
#   hover -> OFF, press F9, hover -> OFF  the keystroke does not arrive, and
#                                         every driven capture needs the
#                                         mouse rather than the keyboard
#
# `docs/ORACLE.md`'s recipe says keys reach the game through `osascript ...
# keystroke` while System Events *clicks* do not, so this is worth checking
# rather than assuming: a function key goes through `key code`, not
# `keystroke`, and that is a different path.
set -e
N=${1:-52}
T=${2:-/tmp/ron-runs}
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

BX=${BUTTON_X:-32}
BY=${BUTTON_Y:-1017}
FKEY=${FKEY:-101}     # macOS key code for F9

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
  osascript -e 'tell application "System Events" to set frontmost of process "riseofnations_trace.exe" to true' >/dev/null 2>&1
  sleep 0.4
}

hover_shot() {
  focus
  cliclick m:960,300
  sleep 1
  cliclick m:$BX,$BY
  sleep 3
  screencapture -x -R0,600,1400,500 "$T/r$N-$1.png" 2>/dev/null || true
  echo "$(date +%H:%M:%S) shot $1 at frame $(frame_now)"
}

wait_frame 195
hover_shot "a-before"

# The press, with the pointer off the card so a hover cannot be confused for
# a click, and the units still selected.
focus
cliclick m:960,300
sleep 0.5
osascript -e "tell application \"System Events\" to key code $FKEY" >/dev/null 2>&1
echo "$(date +%H:%M:%S) pressed key code $FKEY at frame $(frame_now)"
sleep 2

wait_frame 220
hover_shot "b-after"
cliclick m:960,300
echo "probe done"
