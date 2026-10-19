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
# Standalone guards join the same budget as gates, with the reflex's own
# reservation: 2 slots and 4 GiB (3 GiB measured), a 16 GiB ceiling for the debug rebuild an
# edit can cost (DECISIONS 75 — the first cut reserved the whole host and a
# lane's reflex waited behind every gate on the box). Gate admission supplies
# the child-local marker to avoid acquiring a second lease while holding the
# first. This is a cooperative marker, not a security boundary.
export CARGO_BUILD_JOBS=8
export RUST_TEST_THREADS=2
if [ "$RON_GUARD_ADMITTED" != "1" ]; then
  exec python3 tools/resource_budget.py --cpu 2 --memory-mib 4096 --limit-mib 16384 -- \
    env RON_GUARD_ADMITTED=1 zsh tools/guard.sh "$@"
fi
# `--built` (the gate's step 6, the thirty-fifth steer): the guards on the
# release binary the suite just built, in seconds, instead of a debug build.
PROFILE=()
if [ "$1" = "--built" ]; then PROFILE=(--release); shift; fi
(cd tools/explore && python3 -m unittest -q test_brief test_queueledger test_sweep_gen.FamilyRegistrations)
if [ -n "$RON_LANE" ]; then
  # The journal's gate quote cannot exist before the lane's gate has run (3277),
  # and its Loop line is the commander's to file at the merge (3328).
  cargo test -q "${PROFILE[@]}" -p sim docs_guard -- --skip an_item_number_is_minted_once_and_in_its_file_s_form --skip a_journal_quotes_its_lane_verdict_and_gate_minutes --skip a_journal_s_loop_line_is_filed_or_says_none
else
  cargo test -q "${PROFILE[@]}" -p sim docs_guard
fi
cargo test -q "${PROFILE[@]}" -p sim no_float
if [ -z "$RON_LANE" ]; then cargo test -q "${PROFILE[@]}" -p rondata the_handoff; fi
python3 tools/queueledger.py
for f in "$@"; do
  cargo test -q "${PROFILE[@]}" -p sim "$f"
done
