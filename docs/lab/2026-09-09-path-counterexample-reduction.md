# From a semantic mismatch to an independent regression

## What was established

The persistent production Rust path worker agrees with the original on 4,096
deterministic semantic cases from the retained two-waypoint capsule. **No new
production defect was found.** A deliberate test mutation changed the existing
FINAL bit test into exact equality. That faulty worker diverged on search case 9.
The source was restored and the correct worker rebuilt before subsequent work.
The faulty executable remains outside git at `/tmp/path-worker-equality-mutant`.

The reducer preserved the survivor-count mismatch while removing payload bits.
In 24 predicate evaluations, it reduced 98 set bits to two:

| Stack entry, bottom first | x | y | tolerance | flags |
| --- | --- | --- | --- | --- |
| Earlier segment | 0 | 0 | 0 | 0 |
| Current segment boundary | 0 | 0 | 0 | FINAL \| SIDESTEP (3) |

The original keeps the earlier entry; the equality mutant removes both. Either
waypoint deletion, zeroing the remaining flag field, or clearing either flag bit
eliminates this mismatch. Those five neighbors and the counterexample were
checked again with fresh original engines and a separate Rust process for each
case. That prevents the certification sequence from depending on a warm worker.

The authored case is now the install-independent Rust test
`reduced_path_counterexample_stops_at_a_combined_final_flag`. It was run against
the injected source mutation and **failed**, then passed after the correct source
was restored. No executable bytes, original pointers, captured coordinates or
original assets enter the test. The regression invokes the same unit operation
that production order teardown uses.

## Reduction contract

`tools/explore/path_counterexample.py` has no emulator or game dependency. It
operates on tuples of `(x, y, tolerance, flags)` unsigned words and provides:

- chunk deletion, then single-waypoint deletion;
- whole-field zeroing and set-bit clearing, high bits first;
- a fixed-point pass, cached predicate results and a finite evaluation budget;
- an explicit final neighborhood certificate.

This establishes local minimality under those edits, **not a globally smallest
input** or a unique minimum. A different bit-clearing order can preserve a
different extra flag bit. The final rule clears high bits first, yielding the
named SIDESTEP bit for this fixture. Deleting chunks helps cross cases where
single deletion alone cannot preserve the predicate. Oracle exceptions and
budget exhaustion abort rather than return a successful reduction.

The driver preserves the original mismatch class: survivor count, x, y,
tolerance or flags. A candidate changing only the failure class is rejected.
Native access/ABI/write-contract failures, worker crashes, timeouts, malformed
or oversized responses, and unsolicited output are infrastructure errors. They
are not treated as semantic disagreements and never become counterexamples.

## Small persistent worker, bounded native calls

`crates/rondata/examples/path_capsule_worker.rs` accepts one counted waypoint
sequence per line, applies `Unit::discard_current_path_segment`, and flushes the
complete surviving sequence. It constructs a unit, not a world or simulation.
The subprocess stays alive through search/reduction, avoiding launch per query.
Pipe writes and reads have a shared deadline; responses have a byte ceiling.
The driver closes or terminates only the subprocesses it created.

`minimize_path_capsule.py` first validates the retained natural capsule and
executable identities. Its native side uses the existing byte-bounded emulator,
with restored state on every query, read-only waypoint storage and explicit
scratch initialization. It verifies unrelated fields, complete waypoint bytes,
ABI preservation and the path-length store sequence before returning semantic
survivors. Candidate lengths cannot exceed the captured two-entry domain.
Padding and unused capacity are authored zeros for these reduction queries.

Reports record source/capsule/worker hashes, seed, original and reduced inputs,
both semantic outputs, mismatch class, query counts and certification scope.
Output is CREATE_NEW outside the repository. Worker identity is checked again
before returning a report. An interrupted or failed reduction produces no
successful report. Reduced data must be reviewed before promotion into source;
this run's zero-valued fixture was authored explicitly as a Rust test.

## Results, costs and artifacts

`/tmp/path-reducer-clean-final.json`: all 4,096 cases agree, one Rust process,
4,096 worker requests and native queries, 0.741 seconds.
`/tmp/path-reducer-certified-fresh.json`: mismatch on case 9, 24 reduction
predicate evaluations, five fresh certified neighbors, 39 native queries
(45 search/reduction native executions including fresh checks), 40 worker
requests across seven processes, 1.034 seconds. Query counters exclude the
initial capsule-validation replay and its negative controls; elapsed time
includes validation and process cleanup, excluding report-file writing.

These are single local measurements on this narrow path operation, with the
release suite running concurrently. They are not a comparison with whole-game
replay or a general speed claim. This still exercises only lengths zero to two
and no suspended search. The earlier natural seed was shutdown path deletion;
this work did not capture an in-match cancellation or expand native coverage.

`/tmp/path-reduced-regression-mutant.log` records the permanent regression's
intentional failure. The correct copied worker is `/tmp/path-worker-correct`.
No source fault remains in the branch. No parity score moved and no game launch
or settings modification was needed in this tranche.

## Reproduction

```sh
set -e
cargo build --release -p rondata --example path_capsule_worker
uv run tools/explore/minimize_path_capsule.py /path/to/install /tmp/path-capture \
  --worker target/release/examples/path_capsule_worker \
  --cases 4096 --budget 10000 --output /tmp/new-reduction-report.json
python3 tools/explore/test_path_counterexample.py
cargo test --release -p sim reduced_path_counterexample
```

The correct worker should produce `no_mismatch`; it does not fabricate a
counterexample to demonstrate the reducer. Fault injection was a separate,
controlled validation step, not a runtime switch in the production code or worker.
Nine install-independent reducer/IPC tests cover deterministic reduction,
chunk deletion, agreement, broken oracles, budget exhaustion, mismatch classes,
malformed protocol, persistent requests, crash/timeout/extra-output handling.

Validation: the full release gate passes: 269 rondata (one ignored), 822 sim,
13 fixed and three doctests; rondata 229.49 seconds, process-tree peak
8,663 MiB under a 20 GiB ceiling. Clippy with warnings denied, formatting,
documentation guards, the install survey and nine reducer/IPC tests pass.
