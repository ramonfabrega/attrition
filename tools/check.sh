#!/bin/zsh
# Standalone Cargo work participates in the same host budget as release gates.
# Compiler jobs are bounded; the reservation is cooperative, not an OS CPU quota.
set -e
cd "$(dirname "$0")/.."
case "$1" in
  test|build|check|clippy|run) ;;
  *) print -u2 'usage: zsh tools/check.sh {test|build|check|clippy|run} [Cargo arguments]'; exit 2 ;;
esac
CHECK_COMMAND="$1"
shift
export CARGO_BUILD_JOBS=8
export RUST_TEST_THREADS=2
# `test` scales the execution timeout to the timed tests it selects, 1800 s the
# floor (item 3974: the whole rondata lib outran the fixed 1800 s).
SCALE=()
[[ "$CHECK_COMMAND" == test ]] && SCALE=(--test-count-timeout)
exec python3 tools/resource_budget.py --cpu 8 --memory-mib 32768 "${SCALE[@]}" -- \
  cargo "$CHECK_COMMAND" --jobs 8 "$@"
