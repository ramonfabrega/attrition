#!/bin/zsh
# runqueue.sh [SCENARIO] [ITEM ...] — run the owed captures back to back.
#
# The capture lane's driver (docs/QUEUE.md item 90, docs/DECISIONS.md entry
# 27). Reads `captures.txt`, turns each stanza into `longtrace.sh`'s inputs
# and runs them **one at a time** — the screen, the `ron` bottle and the
# install's INIs are one global resource, and a second capture on top of a
# first is two games fighting over the same window.
#
#   zsh tools/gamelog/runqueue.sh                    every pending stanza
#   zsh tools/gamelog/runqueue.sh - 108              only item 108's
#   DRY=1 zsh tools/gamelog/runqueue.sh              print the plan, run nothing
#
# A stanza whose archive already exists is skipped, so this is re-runnable and
# an interrupted queue resumes where it stopped rather than at the top. Each
# capture's `check:` lines run straight afterwards and their verdicts are
# repeated in the summary; a failing check does not stop the queue, because
# the next capture is usually independent of it and the screen is better spent
# than idle.
set -e
# Both the capture and its checks are piped into `tee`, and without this the
# status the `||` reads is **tee's**, which is always 0 — a failed capture
# and a failed check would both be summarised as fine. This is the same trap
# that `[ -n "$X" ] && cmd` sets under `set -e`, one pipe further along.
set -o pipefail

# **Killing this script must kill the capture it started.** `longtrace.sh`
# runs as a child, and its last acts are to `mv` the live `gamelog.txt` onto
# the archive name and to restore the INIs. An orphan that outlives its
# parent therefore renames whatever `gamelog.txt` happens to be there *now*
# over a finished archive — which nearly cost run45 its 508 MB after a
# `pkill -f runqueue.sh` matched only the wrapper. Trap and take the whole
# process group down.
cleanup() {
  trap - EXIT INT TERM
  pkill -P $$ 2>/dev/null
  pkill -f "gamelog/longtrace.sh" 2>/dev/null
  exit
}
trap cleanup INT TERM

W=$(cd "$(dirname "$0")/../.." && pwd)
SCEN=${1:--}
if [ "$SCEN" = "-" ]; then SCEN="$W/tools/gamelog/captures.txt"; fi
if [ $# -gt 0 ]; then shift; fi
WANT=("$@")
B="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations"
L="$B/Logs"
T=${RON_TMP:-/tmp/ron-runs}
mkdir -p "$T"
LOG="$T/runqueue-$(date +%Y%m%d-%H%M%S).log"
echo "scenario $SCEN -> $LOG"

# --- parse. zsh has no arrays of dicts; a stanza is accumulated into scalars
# and flushed by the blank line, or by EOF.
typeset -a summary cmds checks covers
summary=()
run=""; tag=""; item=""; why=""; frames=""; mapstyle=""; cfg=""
endd=""; startt=""; dumpall=""; window=""; fwindow=""; driver=""; settle=""; ffwd=""
# `poll_max` was the one key not reset between stanzas, so a stanza after
# run53 inherited its 900 and the default was unreachable for the rest of the
# file — a leak that can only ever make a run wait longer, but a leak.
pollmax=""
cmds=(); checks=(); covers=()

reset_stanza() {
  run=""; tag=""; item=""; why=""; frames=""; mapstyle=""; cfg=""
  endd=""; startt=""; dumpall=""; window=""; fwindow=""; driver=""; settle=""; ffwd=""
  pollmax=""
  cmds=(); checks=(); covers=()
}

flush() {
  if [ -z "$run" ]; then return 0; fi
  local skip=0 w archive rc verdicts c
  if [ ${#WANT} -gt 0 ]; then
    skip=1
    for w in "${WANT[@]}"; do
      if [ "$w" = "$item" ]; then skip=0; fi
      if [ "$w" = "$run" ]; then skip=0; fi
    done
  fi
  archive="$L/gamelog-run$run-$tag.txt"
  if [ $skip -eq 0 ]; then
    if [ -f "$archive" ]; then
      echo "== item $item / run$run: already archived, skipping"
      summary+=("item $item run$run  SKIPPED (archive exists)")
      skip=1
    fi
  fi
  if [ $skip -eq 0 ]; then
    echo "== item $item / run$run ($tag): $why"
    echo "   frames=$frames mapstyle=$mapstyle cfg='${cfg:-default}' cmds=${#cmds}"
    # Only the keys the stanza actually set are exported: longtrace.sh's
    # defaults are run33/run39's, and an empty override is not the same thing
    # as an absent one.
    typeset -a pass
    pass=()
    if [ -n "$endd" ];    then pass+=("DETAIL_END=$endd"); fi
    if [ -n "$startt" ];  then pass+=("DETAIL_START=$startt"); fi
    if [ -n "$dumpall" ]; then pass+=("DUMP_ALL=$dumpall"); fi
    if [ -n "$window" ];  then pass+=("WINDOW=$window"); fi
    if [ -n "$fwindow" ]; then pass+=("FRAME_WINDOW=$fwindow"); fi
    # `rontrace.cfg` is a several-line file when the trace takes a window of
    # its own, so `cover:` is repeatable and the lines are joined in order.
    if [ ${#covers} -gt 0 ]; then pass+=("TRACE_COVER=${(pj:\n:)covers}"); fi
    if [ -n "$driver" ];  then pass+=("DRIVER=$W/$driver"); fi
    if [ -n "$settle" ];  then pass+=("SETTLE_MIN=$settle"); fi
    if [ -n "$pollmax" ]; then pass+=("POLL_MAX=$pollmax"); fi
    # `ffwd: -` means no fast-forward at all, which every driven stanza wants.
    if [ "$ffwd" = "-" ]; then
      pass+=("FFWD=")
    elif [ -n "$ffwd" ]; then
      pass+=("FFWD=$ffwd")
    fi
    # `cfg: -` is "no -config"; an absent cfg leaves longtrace.sh's own
    # per-mapstyle default, which is not the same thing.
    if [ "$cfg" = "-" ]; then
      pass+=("CFG=")
    elif [ -n "$cfg" ]; then
      pass+=("CFG=$cfg")
    fi
    # `(j:\n:)` joins with a **literal** backslash-n in zsh; `p` is what makes
    # it a newline. Without it every `cmd:` line of a stanza became one line of
    # `rontrace.cmd`, `parse_cmd` took the first and silently ignored the rest,
    # and the capture was of a scenario that half happened. `cmdsran.py` is
    # what caught it — 3 lines parsed where the stanza wrote 4.
    if [ ${#cmds} -gt 0 ]; then pass+=("CMD_EXTRA=${(pj:\n:)cmds}"); fi
    if [ -n "$DRY" ]; then
      echo "   would run: env ${pass} longtrace.sh $run $frames $tag $mapstyle"
      for c in "${checks[@]}"; do echo "   would check: $c"; done
      summary+=("item $item run$run  DRY")
    else
      rc=0
      env $pass zsh "$W/tools/gamelog/longtrace.sh" "$run" "$frames" "$tag" "$mapstyle" 2>&1 | tee -a "$LOG" || rc=$?
      if [ $rc -ne 0 ]; then
        summary+=("item $item run$run  CAPTURE FAILED (rc=$rc)")
      else
        verdicts=ok
        for c in "${checks[@]}"; do
          echo "-- check: $c" | tee -a "$LOG"
          ( cd "$W" && eval "$c" ) 2>&1 | tee -a "$LOG" || verdicts=FAILED
        done
        summary+=("item $item run$run  captured, checks $verdicts")
      fi
    fi
  fi
  reset_stanza
}

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    \#*) continue ;;
    "") flush; continue ;;
  esac
  key=${line%%:*}
  val=${line#*: }
  case "$key" in
    run) run=$val ;;
    tag) tag=$val ;;
    item) item=$val ;;
    why) why=$val ;;
    frames) frames=$val ;;
    mapstyle) mapstyle=$val ;;
    cfg) cfg=$val ;;
    end) endd=$val ;;
    start) startt=$val ;;
    dump_all) dumpall=$val ;;
    window) window=$val ;;
    frame_window) fwindow=$val ;;
    cover) covers+=("$val") ;;
    driver) driver=$val ;;
    settle_min) settle=$val ;;
    poll_max) pollmax=$val ;;
    ffwd) ffwd=$val ;;
    cmd) cmds+=("$val") ;;
    check) checks+=("$val") ;;
    *) echo "unknown key '$key' in $SCEN" >&2; exit 1 ;;
  esac
done < "$SCEN"
flush

echo
echo "=== the queue, as it went ==="
for s in "${summary[@]}"; do echo "  $s"; done
echo "full log: $LOG"
