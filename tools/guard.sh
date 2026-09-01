#!/bin/zsh
# The reflex lane: the paperwork guards, in debug, in seconds — run this
# between edits. `guard.sh [filter …]` also runs the named sim tests (the
# current item's). The full `--release` diff suite stays the gate: before a
# commit and after a document rewrite. This script is the loop, not the
# definition of done.
set -e
cd "$(dirname "$0")/.."
cargo test -q -p sim docs_guard
cargo test -q -p sim no_float
cargo test -q -p rondata the_handoff
for f in "$@"; do
  cargo test -q -p sim "$f"
done
