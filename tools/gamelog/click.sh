#!/bin/zsh
# click.sh X Y  -- focus the game and left-click at desktop X,Y
#
# A fast `cliclick c:` often does not register: move first, then press and
# release as separate events with a gap. Clicks must go through cliclick and
# keys through osascript keystroke; the other way round does not reach the game.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
RON_TOOLS=${0:A:h}
zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
sleep 0.5
cliclick m:$1,$2
sleep 0.8
cliclick dd:$1,$2
sleep 0.4
cliclick du:$1,$2
sleep 1.5
