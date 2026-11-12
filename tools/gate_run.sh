#!/bin/zsh
# gate_run.sh <sha> <log> [gate flags…] — the gate on a commit, in the gate's
# own tree (docs/DECISIONS.md entry 117 §1). It lived in the driver's job
# temp dir until 2026-11-12, where it died with the job; the queue's opener
# names this file. The gate tree is the `gate` worktree of whichever checkout
# this script is run from, and the install is RON_INSTALL or the root
# checkout's `game/`.
export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH
if [ $# -lt 2 ]; then echo "usage: gate_run.sh <sha> <log> [gate flags…]" >&2; exit 64; fi
sha="$1"; log="$2"; shift 2
case "$log" in /*) ;; *) log="$PWD/$log" ;; esac
common="$(cd "$(dirname "$0")" && cd "$(git rev-parse --git-common-dir)" && pwd)" || exit 98
root="$(dirname "$common")"
install="${RON_INSTALL:-$root/game}"
cd "$root/.claude/worktrees/gate" || { echo "gate exit 97 (no gate tree)" > "$log"; exit 97; }
git checkout -q --detach "$sha" || { echo "gate exit 96 (checkout $sha refused)" > "$log"; exit 96; }
echo "Gate tree: $(pwd) at $(git rev-parse HEAD)" > "$log"
python3 tools/release_gate.py "$install" --test-threads 8 "$@" >> "$log" 2>&1
code=$?
echo "gate exit $code" >> "$log"
exit $code
