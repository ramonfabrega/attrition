#!/bin/zsh
# The reflex lane: the paperwork guards, in debug, in seconds — run this
# between edits. `guard.sh [filter …]` also runs the named sim tests (the
# current item's). The full `--release` diff suite stays the gate: before a
# commit and after a document rewrite. This script is the loop, not the
# definition of done.
#
# `RON_LANE=1` is a worker's: the tests that read the queue's handoff
# against a pin are the commander's to turn green at the merge
# (`tools/release_gate.py`, `COMMANDERS_LINES`), so a lane skips them here
# and runs everything else. `release_gate.py --lane` sets it.
set -e
cd "$(dirname "$0")/.."
if [ -n "$RON_LANE" ]; then
  cargo test -q -p sim docs_guard -- --skip an_item_number_is_minted_once_and_in_its_file_s_form
else
  cargo test -q -p sim docs_guard
fi
cargo test -q -p sim no_float
if [ -z "$RON_LANE" ]; then cargo test -q -p rondata the_handoff; fi
python3 tools/queueledger.py
for f in "$@"; do
  cargo test -q -p sim "$f"
done
