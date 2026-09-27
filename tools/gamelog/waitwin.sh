#!/bin/zsh
# wait for the RoN window to exist, then focus it and shoot a downscaled
# screenshot.
#
# The unix process is `riseofnations.exe` for the install's own executable and
# `riseofnations_trace.exe` for the traced copy (`tools/trace/`); match both,
# or a traced run waits forever (2026-08-25, run20). Under free WineHQ the
# unix side also carries a `start.exe /exec` parent, which matches the same
# pattern and is harmless here — we only use it to know a run is alive.
#
# **The window, though, is not found by that name.** System Events calls every
# free-Wine GUI process `wine`, so the window is matched by its title;
# `focus.sh` is the one place that knows how, and this waits on it.
# `$0` inside a zsh *function* is the function's name, not the file's, so the
# script's own directory is captured here at load time and used below.
#
# **A wait that cannot end is not a wait** (parked 937, the seventeenth
# pass). This loop had one way out — a window — so a game that died before
# its window held `longtrace.sh`, `runqueue.sh` and the lane until four pids
# were killed by hand (run314's first take). It has three more now, each its
# own exit so the caller's log says which:
#
#   3  the game was seen and is gone on two polls running: it died before
#      its window. Nothing to archive; the caller restores and stops.
#   4  no game process appeared at all in WAITWIN_START_MAX seconds (180).
#   5  the game is alive and no window answered in WAITWIN_MAX seconds (600):
#      the permission-shaped one. Ask the window through `viadriver.sh`
#      before blaming anything else (`docs/ORACLE.md`, "Off CrossOver").
#
# `WAITWIN_PATTERN`, `WAITWIN_FOCUS` and `WAITWIN_POLL` are the test's
# (`tools/explore/test_waitwin.py`): a fixture process under its own name, a
# focus script that never answers, and a poll short enough to run offline.
RON_TOOLS=${0:A:h}
pattern=${WAITWIN_PATTERN:-'riseofnations(_trace)?\.exe'}
focus=${WAITWIN_FOCUS:-$RON_TOOLS/focus.sh}
poll=${WAITWIN_POLL:-3}
start_max=${WAITWIN_START_MAX:-180}
max=${WAITWIN_MAX:-600}
proc=""; seen=""; gone=0
typeset -F SECONDS
t0=$SECONDS
while true; do
  proc=$(pgrep -fl "$pattern" | grep -oE "$pattern" | head -1)
  if [ -n "$proc" ]; then
    seen=$proc; gone=0
    w=$(zsh "$focus" --title 2>/dev/null)
    if [ -n "$w" ]; then break; fi
  elif [ -n "$seen" ]; then
    gone=$((gone + 1))
    if [ $gone -ge 2 ]; then
      print -u2 -- "waitwin: $seen died before its window — nothing to wait for"
      exit 3
    fi
  elif (( SECONDS - t0 > start_max )); then
    print -u2 -- "waitwin: no game process in ${start_max} s — the launch started none"
    exit 4
  fi
  if (( SECONDS - t0 > max )); then
    print -u2 -- "waitwin: ${seen:-the game} is alive and no window answered in ${max} s — ask the window through viadriver.sh"
    exit 5
  fi
  sleep $poll
done
sleep 8
zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1
sleep 2
out=${1:-/tmp/ron-runs/d1.png}
small=${out%.png}s.png
screencapture -x "$out"
sips -Z 900 "$out" --out "$small" >/dev/null
echo "ready ($proc, window: $w)"
