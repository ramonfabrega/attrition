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
#
# The two offline suites that read the queue run first, in under a second
# (parked 1188, the twentieth pass): a queue line `test_brief` refused was
# green here and red at the booking gate's first step.
set -e
cd "$(dirname "$0")/.."
# Standalone guards join the same budget as gates. Gate admission supplies the
# child-local marker to avoid acquiring a second lease while holding the first.
# This is a cooperative marker, not a security boundary.
export CARGO_BUILD_JOBS=8
export RUST_TEST_THREADS=2
if [ "$RON_GUARD_ADMITTED" != "1" ]; then
  exec python3 tools/resource_budget.py --cpu 8 --memory-mib 32768 -- \
    env RON_GUARD_ADMITTED=1 zsh tools/guard.sh "$@"
fi
# `--built` (the gate's step 6, the thirty-fifth steer): the guards on the
# release binary the suite just built, in seconds, instead of a debug build.
PROFILE=()
if [ "$1" = "--built" ]; then PROFILE=(--release); shift; fi
(cd tools/explore && python3 -m unittest -q test_brief test_queueledger)
if [ -n "$RON_LANE" ]; then
  cargo test -q "${PROFILE[@]}" -p sim docs_guard -- --skip an_item_number_is_minted_once_and_in_its_file_s_form
else
  cargo test -q "${PROFILE[@]}" -p sim docs_guard
fi
cargo test -q "${PROFILE[@]}" -p sim no_float
if [ -z "$RON_LANE" ]; then cargo test -q "${PROFILE[@]}" -p rondata the_handoff; fi
python3 tools/queueledger.py
for f in "$@"; do
  cargo test -q "${PROFILE[@]}" -p sim "$f"
done
