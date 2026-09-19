#!/bin/zsh
# golden_capture.sh — the click-free lane, launched through LaunchServices.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh \
#       <output> --map 14 --end-frame 2000 --cmd-file <chapter.cmd> ...
#
# `unattended_capture.py` needs no TCC grant and no human at the menu, but it
# does need a **window**: winemac.drv refuses one to a process with no GUI
# session, and a launch from inside Claude Code's process tree dies in 3.8 s
# with `nodrv_CreateWindow` ("The graphics driver is missing") and no frames.
# `viadriver.sh` is the lane's answer — LaunchServices makes `RonDriver.app`
# the responsible process — and it runs a zsh script, so this is the shim that
# hands it a Python runner. The install and the profile are the machine's, not
# the caller's; everything else is passed through.
set -e
W=${0:A:h:h:h}
# The lane's own default, as longtrace.sh has it: a worktree has no `game`.
INSTALL=${RON_INSTALL:-/Users/rf-studio/code/fun/attrition/game}
PROFILE=${RON_PROFILE:-$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations}
[ $# -ge 1 ] || { echo "usage: golden_capture.sh <output> [unattended_capture.py options...]" >&2; exit 64 }
OUT=$1; shift
echo "install: $INSTALL"
echo "profile: $PROFILE"
echo "output:  $OUT"
# **A relative `--cmd-file` is resolved against the repo root, not against
# `tools/explore`.** This script cds into the runner's directory before
# exec'ing it, so the recipe every golden run is written with —
# `--cmd-file tools/gamelog/golden/chapterN.cmd`, as `docs/RUNS.md`
# run101-run105 has it — reached `tools/explore/tools/gamelog/...` and the
# runner refused before the game was launched (run112's first attempt).
# Rewriting it here keeps the recipe the one a reader would type.
args=()
for a in "$@"; do
  case "$prev" in
    --cmd-file)
      case "$a" in
        /*) ;;
        *) [ -f "$W/$a" ] && a="$W/$a" ;;
      esac
      ;;
  esac
  args+=("$a")
  prev=$a
done
cd "$W/tools/explore"
exec python3 unattended_capture.py "$INSTALL" "$OUT" "$PROFILE" "${args[@]}"
