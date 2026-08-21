#!/bin/zsh
# cheat.sh X Y "rest of cheat line"   -- aim the mouse at X,Y then send "cheat <rest>"
# cheat.sh - - "rest"                 -- leave the mouse where it is
#
# Focus first: System Events keystrokes go to whatever is frontmost, and a
# screenshot or another app's dialog stealing focus makes the whole line vanish
# with no error — indistinguishable from a refused cheat, so read the state
# back afterwards regardless.
#
# Aim with a jiggle. `cliclick m:` to the point the pointer is already on emits
# no motion event, the game's cursor tile stays where it was, and an `add` at
# the cursor lands nowhere at all. Always move somewhere else first.
x=$1; y=$2; shift 2
osascript -e 'tell application "System Events" to set frontmost of process "riseofnations.exe" to true' >/dev/null 2>&1
sleep 0.6
osascript -e 'tell application "System Events" to keystroke return' >/dev/null 2>&1
sleep 0.8
osascript -e "tell application \"System Events\" to keystroke \"cheat $*\"" >/dev/null 2>&1
sleep 0.4
if [ "$x" != "-" ]; then
  cliclick m:$((x + 40)),$((y + 30))
  sleep 0.2
  cliclick m:$x,$y
  sleep 0.4
fi
osascript -e 'tell application "System Events" to keystroke return' >/dev/null 2>&1
sleep 1.6
