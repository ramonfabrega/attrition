#!/bin/zsh
# con.sh X Y "command"   -- aim the mouse at X,Y then type a line into the `~` console
# con.sh - - "command"   -- leave the mouse alone
#
# The console is the OTHER input path (docs/ORACLE.md). It takes the command
# with NO `cheat ` prefix — with one it answers "Unknown Command" — and it
# reaches run_cmd's first switch as well as the second, which the chat box
# cannot. `StartConsole=1` in rise2.ini opens it at game start; it is invisible
# until it has printed something, so a game where the chat box never opens is
# usually a game where the console has the keyboard.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
x=$1; y=$2; shift 2
zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
sleep 0.6
osascript -e "tell application \"System Events\" to keystroke \"$*\"" >/dev/null 2>&1
sleep 0.4
if [ "$x" != "-" ]; then
  cliclick m:$((x + 40)),$((y + 30))
  sleep 0.2
  cliclick m:$x,$y
  sleep 0.4
fi
osascript -e 'tell application "System Events" to keystroke return' >/dev/null 2>&1
sleep 1.6
