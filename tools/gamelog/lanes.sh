#!/bin/zsh
# lanes.sh — which click-free capture lane this shell is: its prefix, its
# install and its profile. Sourced by `winelaunch.sh` (the prefix and the
# lock) and by `tools/explore/golden_capture.sh` (the install and the
# profile the runner is handed); `unattended_capture.py` asks it too.
#
# **The click-free lane has two copies** (parked 1139, item 1567). Every
# thing a capture writes is a singleton of its lane — the prefix's
# `.lane.lock` and wineserver, the profile's `rise.ini`, `rise2.ini`,
# `gamelog.ini` and `Player.dat`, and the install the run's directory
# links its data from — so a second lane is a second of each, and nothing
# else: the run's own directory, its tracer build and its logs were per
# capture already. `RON_CAPTURE_LANE=2` chooses it, and lane 1 is what an
# unset variable has always meant.
#
#   lane 1  ~/wine-ron      the repo's `game/`              ~/ron-data/…
#   lane 2  ~/wine-ron-2    ~/ron-capture-lane-2/game       ~/ron-capture-lane-2/…
#
# `tools/gamelog/lane2.sh` builds lane 2 (the prefix through `prefix.sh`,
# the install copied, the profile copied from lane 1's); it is idempotent.
# **The queue lane stays one** (`runqueue.sh`, `cliclick`): it owns the
# cursor and finds the game's window by title, so it runs on lane 1 and
# never beside a lane-2 game. Only the click-free runner doubles.
#
# `RON_WINEPREFIX` set by hand still wins over the lane's (the lock tests
# set it); `RON_INSTALL` and `RON_PROFILE` win only on lane 1, so an
# environment left over from a lane-1 shell cannot point lane 2 at lane
# 1's profile.

RON_CAPTURE_LANE=${RON_CAPTURE_LANE:-1}
case $RON_CAPTURE_LANE in
  1)
    RON_LANE_PREFIX=$HOME/wine-ron
    RON_LANE_INSTALL=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
    RON_LANE_PROFILE=${RON_PROFILE:-"$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations"}
    ;;
  2)
    RON_LANE_PREFIX=$HOME/wine-ron-2
    RON_LANE_INSTALL=$HOME/ron-capture-lane-2/game
    RON_LANE_PROFILE="$HOME/ron-capture-lane-2/AppData/Roaming/Microsoft Games/Rise of Nations"
    ;;
  *)
    print -u2 "lanes.sh: RON_CAPTURE_LANE=$RON_CAPTURE_LANE — the lanes are 1 and 2"
    return 64 2>/dev/null || exit 64
    ;;
esac
# Every lane's prefix, for a reader that must tell one lane's game from
# another's (`live_session.require_closed`).
RON_LANE_PREFIXES=($HOME/wine-ron $HOME/wine-ron-2)
