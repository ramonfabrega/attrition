#!/bin/zsh
# cheat.sh X Y "rest of cheat line"   -- aim the mouse at X,Y then send "cheat <rest>"
# cheat.sh - - "rest"                 -- leave the mouse where it is
x=$1; y=$2; shift 2
osascript -e 'tell application "System Events" to keystroke return' >/dev/null 2>&1
sleep 0.8
osascript -e "tell application \"System Events\" to keystroke \"cheat $*\"" >/dev/null 2>&1
sleep 0.4
if [ "$x" != "-" ]; then cliclick m:$x,$y; sleep 0.3; fi
osascript -e 'tell application "System Events" to keystroke return' >/dev/null 2>&1
sleep 1.6
