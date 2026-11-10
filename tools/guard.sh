#!/bin/zsh
# The reflex lane: the paperwork guards, in debug, in seconds — run this
# between edits. `guard.sh [filter …]` also runs the named tests (the
# current item's), in `sim` and `rondata` both, and names a filter neither
# ran. The full `--release` diff suite stays the gate: before a commit and
# after a document rewrite. This script is the loop, not the definition of
# done.
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
# `--built` (the gate's guard step, the thirty-fifth steer): the guards on the
# release profile, the binary the release step then runs, instead of a debug
# build. Since Loop 4070 the gate runs this step before the release step, so
# it is where the release test binaries are built.
PROFILE=()
if [ "$1" = "--built" ]; then PROFILE=(--release); shift; fi
(cd tools/explore && python3 -m unittest -q test_brief test_queueledger test_sweep_gen.FamilyRegistrations)
# The structure pin, with nothing built (Loop 4070): 4063's builder ended a
# production file in an inline test tail, this reflex reported nothing on
# `diff::start`, and a whole gate went red on the pin at minute 49. Under a
# second; `diff::floors::tests::the_pinned_shape_is_the_tree_s` stays the
# authority and runs below and in the release suite.
python3 tools/shape.py --check
if [ -n "$RON_LANE" ]; then
  # The journal's gate quote cannot exist before the lane's gate has run (3277),
  # and its Loop line is the commander's to file at the merge (3328).
  cargo test -q "${PROFILE[@]}" -p sim docs_guard -- --skip an_item_number_is_minted_once_and_in_its_file_s_form --skip a_journal_quotes_its_lane_verdict_and_gate_minutes --skip a_journal_s_loop_line_is_filed_or_says_none
else
  cargo test -q "${PROFILE[@]}" -p sim docs_guard
fi
cargo test -q "${PROFILE[@]}" -p sim no_float
# The blind list (Loop 3449, the fiftieth pass): `blind::` reads every
# `name@address` citation under docs/, a journal's included, and 3442's
# booking gate was red on it alone — its journal, written after its lane
# gate, quoted the earlier red by `name@address`, and this reflex, the one
# a lane runs after writing the quote, never ran it. It skips itself
# without the Ghidra export or the kept traces; with them it is ~2 s in
# debug, so `RON_DIFF_DEBUG` lets it past the debug door (`testenv::dump`).
RON_DIFF_DEBUG=1 cargo test -q "${PROFILE[@]}" -p rondata blind::tests::the_blind_list_is_the_pinned_residue
# The rest of the pins that read the tree, not a game (Loop 4070): the
# structure pin itself, the field-writer guards and the widening ledger
# (`release_gate.py`'s `PAPERWORK_GUARDS`), ten tests in ~6 s debug on the
# library the blind list just built. They read source, never a kept dump, so
# no `RON_DIFF_DEBUG`: one that ever reaches a dump refuses here, loudly.
cargo test -q "${PROFILE[@]}" -p rondata --lib -- writers:: ledger:: diff::floors::tests::the_pinned_shape_is_the_tree_s
if [ -z "$RON_LANE" ]; then cargo test -q "${PROFILE[@]}" -p rondata the_handoff; fi
python3 tools/queueledger.py
# The nations census round-trips (Loop 3972): `docs/NATIONS.md`'s generated
# tables are what `nations_census.py` emits on this tree, so a lane that runs
# `--write` to re-pin its row carries no churn it did not cause. ~3 s; it
# skips, exit 0, without the decompile export or the install's rules.
python3 -I tools/nations_census.py --check
# The item's filters, each against both crates (Loop 4040, the seventy-sixth
# pass): this ran `-p sim` alone, so a filter whose tests live in `rondata` —
# every render and diff item's — ran nothing and the reflex exited green. A
# filter no test in either crate matched is named and the reflex exits 1, as
# `release_gate.py`'s `unmatched_filters` names one; a test counts when it
# passed or failed, never when it was ignored. A `rondata` filter that reaches
# a kept dump refuses in debug by design (`testenv::dump`): run it with
# `--built` after a release build, or `zsh tools/check.sh test --release`.
setopt pipefail
unmatched=()
for f in "$@"; do
  log=$(mktemp -t guard-filter)
  cargo test -q "${PROFILE[@]}" -p sim -p rondata "$f" 2>&1 | tee "$log"
  ran=$(awk '/^test result:/ { for (i = 1; i < NF; i++) if ($(i+1) == "passed;" || $(i+1) == "failed;") n += $i } END { print n + 0 }' "$log")
  rm -f "$log"
  if [ "$ran" -eq 0 ]; then unmatched+=("$f"); fi
done
if [ ${#unmatched} -gt 0 ]; then
  echo "guard.sh: the filter(s) matched no test in sim or rondata: ${unmatched[*]}" >&2
  exit 1
fi
