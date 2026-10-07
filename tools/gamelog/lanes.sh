#!/bin/zsh
# lanes.sh — the click-free capture lanes: each one's prefix, install and
# profile, and how many there may be. Sourced by `winelaunch.sh` (the
# prefix and the lock) and by `tools/explore/golden_capture.sh`;
# `unattended_capture.py` asks it too.
#
# **The click-free lane is a pool** (parked 1139; items 1568, 1569).
# Every thing a capture writes is a singleton of its lane — the prefix's
# `.lane.lock` and wineserver, the profile's `rise.ini`, `rise2.ini`,
# `gamelog.ini` and `Player.dat`, and the install the run's directory
# links its data from — so a lane is one of each, and nothing else: the
# run's own directory, its tracer build and its logs were per capture
# already.
#
#   lane 1  ~/wine-ron      the repo's `game/`             ~/ron-data/AppData/…
#   lane N  ~/wine-ron-N    ~/ron-capture-lane-N/game      ~/ron-capture-lane-N/AppData/…
#
# **No caller names a lane.** The click-free runner takes the first free
# one (`unattended_capture.py`, `pool_lane`), grows one with
# `tools/gamelog/lane.sh N` when none is free and fewer than
# `RON_LANES_MAX` exist, and waits at the cap. A lane is in the pool once
# its `admitted` file says its reproduction run matched run676's trace
# (lane 1 is the reference and needs none). `RON_CAPTURE_LANE=N` pins a
# lane — for a test, and for the admission run itself.
#
# **The queue lane** (`runqueue.sh`, `longtrace.sh`, `cliclick`) owns the
# cursor and finds the game's window by title, so it runs on lane 1 alone
# and never beside a pool game: `winelaunch.sh`'s `ron_lane_take` refuses
# it while any pool lane is held, and refuses a pool take while it holds
# lane 1.
#
# `RON_WINEPREFIX` set by hand still wins over the lane's (the lock tests
# set it); `RON_INSTALL` and `RON_PROFILE` win only on lane 1, so an
# environment left over from a lane-1 shell cannot point another lane at
# lane 1's files. `RON_LANES_ROOT` (default `$HOME`) moves every lane but
# lane 1's install, for tests.

# The cap, in one place. A pass moves it on `tools/gamelog/lanes.py`'s
# peak, waits and rate (item 1569).
RON_LANES_MAX=${RON_LANES_MAX:-3}
RON_LANES_ROOT=${RON_LANES_ROOT:-$HOME}

ron_lane_paths () {
  # `ron_lane_paths N`: sets RON_LANE_PREFIX, RON_LANE_INSTALL,
  # RON_LANE_PROFILE and RON_LANE_HOME (where lane N's own files live).
  local n=$1
  if [[ "$n" != <-> ]] || (( n < 1 || n > RON_LANES_MAX )); then
    print -u2 "lanes.sh: lane $n — the lanes are 1 to $RON_LANES_MAX (RON_LANES_MAX)"
    return 64
  fi
  if (( n == 1 )); then
    RON_LANE_PREFIX=$RON_LANES_ROOT/wine-ron
    RON_LANE_HOME=$RON_LANES_ROOT/wine-ron
    RON_LANE_INSTALL=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
    RON_LANE_PROFILE=${RON_PROFILE:-"$RON_LANES_ROOT/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations"}
  else
    RON_LANE_PREFIX=$RON_LANES_ROOT/wine-ron-$n
    RON_LANE_HOME=$RON_LANES_ROOT/ron-capture-lane-$n
    RON_LANE_INSTALL=$RON_LANE_HOME/game
    RON_LANE_PROFILE="$RON_LANE_HOME/AppData/Roaming/Microsoft Games/Rise of Nations"
  fi
}

RON_CAPTURE_LANE=${RON_CAPTURE_LANE:-1}
ron_lane_paths "$RON_CAPTURE_LANE" || { return 64 2>/dev/null || exit 64 }
# Every lane's prefix up to the cap, for a reader that must tell one
# lane's game from another's (`live_session.require_closed`) and for the
# queue lane's refusal.
RON_LANE_PREFIXES=($RON_LANES_ROOT/wine-ron)
for (( _n = 2; _n <= RON_LANES_MAX; _n++ )); do RON_LANE_PREFIXES+=($RON_LANES_ROOT/wine-ron-$_n); done
unset _n
