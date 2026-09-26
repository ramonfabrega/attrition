#!/bin/zsh
# winelaunch.sh — the one line that starts the original, sourced by every
# capture script. It was eight copies of CrossOver's `wine --bottle ron`
# until 2026-09-04, when the bottle's licence expired and the lane moved to
# free WineHQ Stable 11.0. `docs/ORACLE.md`, "Off CrossOver", carries why
# each piece below is there; the short version:
#
#   - **The prefix is `~/wine-ron`**, whose `AppData\Roaming\Microsoft Games`
#     is a symlink at the old bottle's, so `rise.ini`, `gamelog.ini` and
#     `Logs/` are the same files every other tool already knows.
#   - **`d3d11`, `dxgi` and `d3d10core` are native** — DXVK-macOS, installed
#     by `tools/gamelog/prefix.sh`. RoN:EE's renderer is `d3dgl.dll`, which
#     despite the name imports **d3d11** and asks for feature level 10_0;
#     Wine's own wined3d cannot serve that on macOS (winemac.drv refuses a
#     3.2+ GL context), and stock DXVK refuses Apple's GPU for want of
#     `geometryShader`. The macOS fork is the build that does neither.
#   - **`winedbg.exe` is left enabled.** A crash then prints its fault to the
#     run's log instead of parking a dialog on the screen — and with
#     `ShowCrashDialog=0` in the prefix it does not even do that.
#   - **MoltenVK's banner goes to stderr** and is a hundred lines a run, so
#     every reader of these logs filters `^\[mvk` or `^info:`.
#
# Usage, from a script that has already `cd`-ed to the install:
#
#   source "$W/tools/gamelog/winelaunch.sh"
#   ron_wine "$T/wine$N.log" "$G/$P" ${=CFG} -automation
#   echo "launched pid $RON_WINE_PID"

RON_WINE_BIN=${RON_WINE_BIN:-/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine}
RON_WINEPREFIX=${RON_WINEPREFIX:-$HOME/wine-ron}

# **The lane lock** (parked 446, the eighth pass, 2026-09-21). The prefix,
# the window and `Logs/` are singletons, and until now nothing stopped one
# launch landing inside another's game — the one case the human protocol
# (astra asks, Ramon relays, the commander holds) does not cover. So
# `ron_wine` refuses to launch while the last game it launched is alive.
# The lock is one file, `$RON_WINEPREFIX/.lane.lock`: the launched wine's
# pid on the first line, the holder and the time on the second. It is stale
# the moment that pid is dead — the game's own exit releases it and nothing
# has to remember to — and a stale lock is taken over silently. A live one
# refuses with exit 75 and names the holder. `RON_LANE_HOLDER` names this
# launch in the file (default: the sourcing script); `RON_LANE_FORCE=1`
# ignores a live lock, for the human who knows the other game is theirs to
# kill; `RON_LANE_WAIT=<seconds>` waits that long for a live holder to exit
# before refusing (parked 758, the fifteenth pass; `tools/explore/
# test_lane_lock.py`). A launch that does not go through this function is
# not covered, which is the same limit the protocol had.
RON_LANE_LOCK=${RON_LANE_LOCK:-$RON_WINEPREFIX/.lane.lock}

ron_wine () {
  local log=$1; shift
  if [[ -r "$RON_LANE_LOCK" && -z "${RON_LANE_FORCE:-}" ]]; then
    local held_pid held_by
    held_pid=$(sed -n 1p "$RON_LANE_LOCK")
    held_by=$(sed -n 2p "$RON_LANE_LOCK")
    if [[ "$held_pid" == <-> ]] && kill -0 "$held_pid" 2>/dev/null; then
      # `RON_LANE_WAIT=<seconds>` waits for the holder to exit rather than
      # refusing (parked 758): a capture queued behind another lane's game
      # was waited for with a hand-rolled `kill -0` loop, once.
      local wait_for=${RON_LANE_WAIT:-0} waited=0
      if (( wait_for > 0 )); then
        print -u2 "ron_wine: the lane is held by $held_by (pid $held_pid); waiting up to ${wait_for}s for it (RON_LANE_WAIT)."
        while (( waited < wait_for )) && kill -0 "$held_pid" 2>/dev/null; do
          sleep 1
          (( waited += 1 ))
        done
      fi
      if kill -0 "$held_pid" 2>/dev/null; then
        print -u2 "ron_wine: the lane is held by $held_by (pid $held_pid, $RON_LANE_LOCK); refusing to launch a second game. RON_LANE_FORCE=1 overrides; RON_LANE_WAIT=<seconds> waits."
        return 75
      fi
    fi
  fi
  export WINEPREFIX="$RON_WINEPREFIX"
  export WINEDLLOVERRIDES="mscoree,mshtml=d;d3d11,dxgi,d3d10core=n"
  export WINEDEBUG=${WINEDEBUG:--all}
  export DXVK_LOG_LEVEL=${DXVK_LOG_LEVEL:-info}
  export DXVK_LOG_PATH=${DXVK_LOG_PATH:-none}
  nohup "$RON_WINE_BIN" "$@" > "$log" 2>&1 &
  RON_WINE_PID=$!
  print -r -- "$RON_WINE_PID" > "$RON_LANE_LOCK"
  print -r -- "${RON_LANE_HOLDER:-${ZSH_ARGZERO:-$0}} since $(date '+%Y-%m-%dT%H:%M:%S%z')" >> "$RON_LANE_LOCK"
}
