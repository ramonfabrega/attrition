#!/bin/zsh
# rclick.sh O X Y  -- cheat select O, wait a tick, then right-click at desktop X,Y
#
# The select travels in the order stream and lands a tick later, so the pause
# is load-bearing: click too early and you order whatever was selected before.
# A right-click on nothing orderable produces no order at all, which reads
# differently in the log from a move order onto terrain.
here=${0:a:h}
zsh "$here/cheat.sh" - - "select $1"
sleep 2
cliclick m:$2,$3
sleep 0.6
cliclick rc:$2,$3
sleep 2
