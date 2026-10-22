#!/bin/zsh
# poolshot_grab.sh <window id> <out.png> — the one `screencapture` call of
# `tools/explore/poolshot.py`, run through `tools/gamelog/viadriver.sh` so it
# runs under RonDriver.app's Screen Recording grant.
#
# **Window-only by construction** (Loop 3469, 3451): it takes a positive
# window id or refuses, and the capture is always `-l <id>`. There is no
# desktop arm to fall back to — a bare `screencapture -x` once photographed
# the whole desktop, other apps included (item 3444; the screen-capture
# hygiene note). `POOLSHOT_SCREENCAPTURE` replaces the binary for the tests.
id=${1:-}
out=${2:-}
if [[ "$id" != <-> ]] || (( id <= 0 )) || [[ -z "$out" ]]; then
  print -u2 "poolshot_grab.sh: refused — a window id and an output path, or nothing (got id='$id' out='$out')"
  exit 64
fi
exec ${POOLSHOT_SCREENCAPTURE:-/usr/sbin/screencapture} -x -o -l "$id" "$out"
