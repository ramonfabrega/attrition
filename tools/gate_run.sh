#!/bin/zsh
# gate_run.sh <sha> <log> [gate flags…] — the gate on a commit, in the gate's
# own tree (docs/DECISIONS.md entry 117 §1). It lived in the driver's job
# temp dir until 2026-11-12, where it died with the job; the queue's opener
# names this file. The gate tree is the `gate` worktree of whichever checkout
# this script is run from, and the install is RON_INSTALL or the root
# checkout's `game/`.
#
# After a green gate, the two disk steps (`tools/disk.py`, the steering pass
# of 2026-11-12, after the disk filled to 0 GB mid-run): every merged builder
# tree is reaped, tree and branch (a dirty one loses only its `target/`), and
# the oracle generations no build reads go. Nothing runs before the gate, and
# a red gate runs neither: a just-merged builder may still be sent its
# failing test to fix in its own tree. Their lines go to the log above its
# last line, which stays `gate exit <code>`; a failed step never changes it.
export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH
if [ $# -lt 2 ]; then echo "usage: gate_run.sh <sha> <log> [gate flags…]" >&2; exit 64; fi
sha="$1"; log="$2"; shift 2
case "$log" in /*) ;; *) log="$PWD/$log" ;; esac
common="$(cd "$(dirname "$0")" && cd "$(git rev-parse --git-common-dir)" && pwd)" || exit 98
root="$(dirname "$common")"
install="${RON_INSTALL:-$root/game}"
# One driver row at most (docs/DECISIONS.md entries 130, 131 §8): a second
# one resting in the tree is the row a notification wakes cold. Asked from
# the caller's tree, before the gate tree is touched.
#
# And the driver's two lines (entry 135): a scoped gate is refused, exit 93,
# once more landings stand since the last pass than the backstop and the
# seats, and any gate, exit 94, from a row past its context ceiling. The
# whole gate is the way out of the first and `handover.py to commander
# --ungated` of the second.
scoped=""; for a in "$@"; do [ "$a" = "--scope" ] && scoped="--scoped"; done
rows="$(cd "$(dirname "$0")/.." && python3 tools/handover.py check $scoped --tip "$sha" 2>&1)"
case $? in
  0) ;;
  3) printf '%s\ngate exit 93 (the backstop: gate whole and hand over to steer)\n' "$rows" > "$log"; exit 93 ;;
  4) printf '%s\ngate exit 94 (the driver is past its ceiling)\n' "$rows" > "$log"; exit 94 ;;
  *) printf '%s\ngate exit 95 (two driver rows)\n' "$rows" > "$log"; exit 95 ;;
esac
cd "$root/.claude/worktrees/gate" || { echo "gate exit 97 (no gate tree)" > "$log"; exit 97; }
git checkout -q --detach "$sha" || { echo "gate exit 96 (checkout $sha refused)" > "$log"; exit 96; }
echo "Gate tree: $(pwd) at $(git rev-parse HEAD)" > "$log"
# When it began, for `tools/gates.py`: `>` keeps the birth time of a log
# name written before, and two such read 443 and 481 minutes (2026-11-16).
echo "Gate start: $(date +%s) $(date '+%F %T %z')" >> "$log"
export ATTRITION_GATE_RUN=1
python3 tools/release_gate.py "$install" --test-threads 8 "$@" >> "$log" 2>&1
code=$?
echo "$rows" >> "$log"
if [ $code -eq 0 ]; then
  python3 tools/disk.py --sha "$sha" >> "$log" 2>&1 || echo "Disk: step failed: the builders step exited $?" >> "$log"
  python3 tools/disk.py --gc "$install" >> "$log" 2>&1 || echo "Disk: step failed: the oracle step exited $?" >> "$log"
fi
echo "gate exit $code" >> "$log"
exit $code
