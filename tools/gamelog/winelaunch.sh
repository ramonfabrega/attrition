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
# **The prefix is the lane's** (parked 1139, item 1567): `RON_CAPTURE_LANE=2`
# is `~/wine-ron-2`, its own lock beside it; unset is lane 1, `~/wine-ron`.
# `lanes.sh` has the table.
source "${${(%):-%x}:A:h}/lanes.sh" || return 64
RON_WINEPREFIX=${RON_WINEPREFIX:-$RON_LANE_PREFIX}

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

# **The lane is taken before anything it shares is written** (parked 974,
# the eighteenth pass, 2026-09-28). The lock was looked at only here, at
# the launch, and every capture script stages first: `longtrace.sh` rewrote
# the profile and removed `Logs/gamelog.txt` and `rontrace.log` before it
# was refused, so a second lane's launch cleared the running game's log.
# `ron_lane_take` is the same lock taken by the capture *script*, on its
# own pid, before its first write; `ron_wine` then writes the game's pid
# above it. The file is three lines now — the game's pid (or the script's,
# until there is a game), the holder and the time, the script's pid — and
# the lane is held while **either** pid lives, which also covers the
# minute after the game's exit in which its script is still moving the
# log. A holder that is this shell is no refusal. `tools/explore/
# test_lane_lock.py` fails on a capture script that writes before it takes.
#
# **A pid is a number the kernel hands out again** (parked 1180, the
# twentieth pass, 2026-09-29). run426's lock named pid 15593; its game had
# exited and an unrelated `next-server` held the number, so `kill -0` said
# the lane was held and run428 was refused with no wine running. The lock
# carries each pid's start time, as `stamp <pid> <ps -o lstart=>` lines
# under the three it had, and a pid that lives under another start time is
# dead for the lane. A lock with no stamp is read as it always was.
# `ron_lane_state` says what a reader of the file cannot: `free`, `stale`
# with the reason, or `held by` with the holder — a lock whose pids were
# both dead read as a busy lane six times in one tranche.
_ron_pid_started () {
  ps -o lstart= -p "$1" 2>/dev/null | sed 's/^ *//;s/ *$//'
}

_ron_pid_holds () {
  # 0 when $1 is a live pid and the one the lock stamped; 1 when it is
  # dead; 2 when it lives under another start time (recycled).
  local pid=$1 stamped
  kill -0 "$pid" 2>/dev/null || return 1
  stamped=$(sed -n "s/^stamp $pid //p" "$RON_LANE_LOCK" | sed -n 1p)
  [[ -z "$stamped" || "$stamped" == "$(_ron_pid_started "$pid")" ]] || return 2
  return 0
}

_ron_lane_holder () {
  # Prints the live pid that holds the lane, and returns 0; returns 1 when
  # the lane is free, stale, or held by this shell — or by the pid this
  # shell's runner took it for (`RON_LANE_TAKEN`): the click-free lane
  # launches through a child shell, whose `$$` is not the taker's. With
  # `--any` nothing is exempt: a reader asks who holds the lane, not
  # whether it may launch.
  [[ -r "$RON_LANE_LOCK" ]] || return 1
  local pid mine=$$ taken=${RON_LANE_TAKEN:-}
  if [[ "${1:-}" == --any ]]; then mine=; taken=; fi
  for pid in "$(sed -n 1p "$RON_LANE_LOCK")" "$(sed -n 3p "$RON_LANE_LOCK")"; do
    if [[ "$pid" == <-> && "$pid" != "$mine" && "$pid" != "$taken" ]] && _ron_pid_holds "$pid"; then
      print -r -- "$pid"
      return 0
    fi
  done
  return 1
}

ron_lane_state () {
  # One line for a reader: `free`, `stale: …` or `held by … (pid N, …)`.
  # The exit is the answer too (parked 1488, the twenty-fifth pass): 0 when
  # the next launch would go — free, or stale and taken over — and 1 when
  # the lane is held, so `ron_lane_state && launch` launches only then;
  # item 1481's chained launch went regardless and the flock refused it.
  if [[ ! -r "$RON_LANE_LOCK" ]]; then
    print -r -- "free"
    return 0
  fi
  local pid held why=""
  if held=$(_ron_lane_holder --any); then
    print -r -- "held by $(sed -n 2p "$RON_LANE_LOCK") (pid $held, $(ps -o comm= -p "$held" 2>/dev/null))"
    return 1
  fi
  for pid in "$(sed -n 1p "$RON_LANE_LOCK")" "$(sed -n 3p "$RON_LANE_LOCK")"; do
    [[ "$pid" == <-> ]] || continue
    _ron_pid_holds "$pid"
    case $? in
      0) why+=" pid $pid is this shell;" ;;
      1) why+=" pid $pid is dead;" ;;
      2) why+=" pid $pid is recycled ($(ps -o comm= -p "$pid" 2>/dev/null), started $(_ron_pid_started "$pid"));" ;;
    esac
  done
  print -r -- "stale:${why} last held by $(sed -n 2p "$RON_LANE_LOCK"); the next launch takes it over"
}

_ron_lane_free () {
  [[ -n "${RON_LANE_FORCE:-}" ]] && return 0
  local held_pid held_by
  held_pid=$(_ron_lane_holder) || return 0
  held_by=$(sed -n 2p "$RON_LANE_LOCK")
  # `RON_LANE_WAIT=<seconds>` waits for the holder to exit rather than
  # refusing (parked 758): a capture queued behind another lane's game
  # was waited for with a hand-rolled `kill -0` loop, once.
  local wait_for=${RON_LANE_WAIT:-0} waited=0
  if (( wait_for > 0 )); then
    print -u2 "ron_wine: the lane is held by $held_by (pid $held_pid); waiting up to ${wait_for}s for it (RON_LANE_WAIT)."
    while (( waited < wait_for )) && _ron_lane_holder > /dev/null; do
      sleep 1
      (( waited += 1 ))
    done
  fi
  if held_pid=$(_ron_lane_holder); then
    print -u2 "ron_wine: the lane is held by $held_by (pid $held_pid, $RON_LANE_LOCK); refusing to launch a second game. RON_LANE_FORCE=1 overrides; RON_LANE_WAIT=<seconds> waits."
    return 75
  fi
  return 0
}

_ron_lane_write () {
  # $1 the pid on the first line, $2 the script's pid or nothing.
  print -r -- "$1" > "$RON_LANE_LOCK"
  print -r -- "${RON_LANE_HOLDER:-${ZSH_ARGZERO:-$0}} since $(date '+%Y-%m-%dT%H:%M:%S%z')" >> "$RON_LANE_LOCK"
  if [[ -n "$2" ]]; then print -r -- "$2" >> "$RON_LANE_LOCK"; fi
  local pid
  for pid in "$1" "$2"; do
    if [[ "$pid" == <-> ]]; then
      print -r -- "stamp $pid $(_ron_pid_started "$pid")" >> "$RON_LANE_LOCK"
    fi
  done
}

ron_lane_take () {
  # `ron_lane_take [pid]`: the lane is taken for this shell, or for the pid
  # given — a runner that launches through a child shell takes it for its
  # own pid, so the lane is held through its restore and not only while
  # its game lives (parked 1234, the twenty-first pass: run467's game had
  # exited, the lane read `stale`, and another lane's long trace launched
  # while the runner's `finally` was still putting the profile back).
  _ron_lane_free || return 75
  RON_LANE_TAKEN=${1:-$$}
  _ron_lane_write "$RON_LANE_TAKEN" "$RON_LANE_TAKEN"
}

ron_lane_release () {
  # `ron_lane_release [pid]`: a taker lets the lane go — the lock is
  # removed when its first line is the given pid (this shell's by
  # default) and nothing else live holds it, so a waiter reads `free`
  # rather than `stale` once a runner has exited. Any other lock is left
  # exactly where it is: a stale lock is the next launch's to take over,
  # never a hand's to remove. Returns 1 when the lock is not the caller's.
  local mine=${1:-$$}
  [[ -r "$RON_LANE_LOCK" ]] || return 0
  # The taker's pid is the first line until its launch, and the third
  # after it (`ron_wine` puts the game's pid first): either is ours.
  [[ "$(sed -n 1p "$RON_LANE_LOCK")" == "$mine" || "$(sed -n 3p "$RON_LANE_LOCK")" == "$mine" ]] || return 1
  local pid
  for pid in "$(sed -n 1p "$RON_LANE_LOCK")" "$(sed -n 3p "$RON_LANE_LOCK")"; do
    if [[ "$pid" == <-> && "$pid" != "$mine" ]] && _ron_pid_holds "$pid"; then
      return 1
    fi
  done
  command rm -f -- "$RON_LANE_LOCK"
}

ron_wine () {
  local log=$1; shift
  _ron_lane_free || return 75
  export WINEPREFIX="$RON_WINEPREFIX"
  export WINEDLLOVERRIDES="mscoree,mshtml=d;d3d11,dxgi,d3d10core=n"
  export WINEDEBUG=${WINEDEBUG:--all}
  export DXVK_LOG_LEVEL=${DXVK_LOG_LEVEL:-info}
  export DXVK_LOG_PATH=${DXVK_LOG_PATH:-none}
  nohup "$RON_WINE_BIN" "$@" > "$log" 2>&1 &
  RON_WINE_PID=$!
  # The script's pid rides along only when the script took the lane: a
  # shell that launches and stays (a human's) would hold it for ever.
  _ron_lane_write "$RON_WINE_PID" "${RON_LANE_TAKEN:-}"
}
