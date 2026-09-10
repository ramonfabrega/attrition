# Owned in-memory replay checkpoints

## Contract

`diff::ReplaySession` owns the existing private replay state machine and an
optional recording `Stream`. Cloning the session creates an in-memory checkpoint
at a source-record boundary. It retains the built simulation, object and type
links, correction observations, RNG accounting, report prefix, player count,
last processed tick, recording commands/cursor/selection, and next source index.
`Built` and `Stream` now derive Clone; Sim already did. The ordinary replay
entry points and comparator are unchanged.

`new` consumes an already-enriched Initial during construction; callers must
apply their required sibling and trace corrections first. `push(index, frame)`
requires exactly the next source index and a nondecreasing frame label before
mutating state. Repeated labels and forward tick gaps preserve the underlying
harness semantics. `finish` returns the entire accumulated report, including
the checkpointed prefix. `built` exposes read-only diagnostic state.

This is not a serialized, portable or compact snapshot. The clone retains full
history and correction arrays, and clones recording commands rather than
sharing them. Capture files and install data are not bundled. The caller must
supply the correct future frames: indices and monotonic labels do not bind a
session to a capture's content. There is no new source-identity certificate,
random seek, cancellation rollback, or asynchronous checkpoint mechanism.

## Differential continuation checks

Two retained-fixture regressions establish the first boundary:

- Run69 is built with its trace and canonical four siblings, then advanced
  through source index 1898. The checkpoint is restored twice; each branch
  replays the next three source records and agrees on the complete Report and
  final Built debug representation. Seed and figure-clock observations must be
  populated. Invalid source indices and backwards labels fail without advancing
  the cursor; repeated frame labels still compare without an extra tick.
- Run7 includes its actual recording. The session is cloned halfway through
  the 1,732-record capture; uninterrupted and restored sessions both equal the
  existing `run_with` entry point's complete Report, including nine applied
  move orders. A negative branch restores the replay but resets Stream to the
  start: its report differs, demonstrating why the stream belongs in a snapshot.

The debug-state comparison supplements future frame/report agreement; it is not
a canonical hash or proof of every possible hidden-state interaction. These
checks cover two fixtures and a short run69 suffix, not universal checkpoint
correctness, arbitrary order streams, all maps, or cross-process restoration.
No gameplay rule or fidelity score changes. Paused native drafts are untouched.

## Isolated measurements

A clean release test, explicit RON_INSTALL and one test thread measured run69:

| Stage | Measurement |
| --- | ---: |
| Enriched setup plus prefix replay | 4.4526 s |
| Initial checkpoint clone | 1.304 ms |
| Restore clone + three records + equality checks, first branch | 12.659 ms |
| Restore clone + three records + equality checks, second branch | 12.662 ms |
| Whole isolated test wall time | 5.33 s |
| Whole isolated process peak RSS | 609,665,024 bytes |

Prefix timing starts after install loading and opening/indexing the primary
capture. Suffix timing includes clone, frame reads, comparisons and final debug/
report equality checks. The process peak is not the snapshot's retained size.
An earlier focused test reported 4.505 s, 1.411 ms and about 12.2–12.4 ms for
these stages. This demonstrates cheap repeated local continuation after paying
for a prefix; it is not a full-suite speedup or a portable-resume benchmark.

Reproduce with the existing captures:

```sh
RON_INSTALL="$RON_INSTALL" cargo test -p rondata --release checkpoint \
  -- --test-threads=1 --nocapture
```

Evidence: `/tmp/replay-checkpoint-tests.log` and
`/tmp/replay-checkpoint-measure.log`. Both tests use existing fixtures; eight
additional observed lookups cover two primary logs, the trace, four siblings
and the recording.

## Next boundary

Integrate the owned session with focused diagnostics so repeated nearby windows
reuse a checkpoint. Before introducing a portable format or minimizing state,
measure retained snapshot components and separate immutable input tables from
mutable state. Bind future record identity and order/correction provenance in
that higher-level driver. A diagnostic HTML window is still not a checkpoint.

## Full validation

The explicit-install monitored four-thread gate passed 282 rondata tests
(one ignored, zero filtered), 824 sim tests, 13 fixed tests, three doctests
and 36 Python tests, plus survey, clippy, fmt and paperwork guards. Rondata
took 122.56 seconds; sampled tree peak was 13,574 MiB under 20 GiB. All 565
observed fixture requests were present. This includes eight additional requests
from the two continuation tests. The gate is validation, not a checkpoint
throughput benchmark. Evidence: `/tmp/replay-checkpoint-gate.log` and
`/tmp/attrition-gate-replay-checkpoint-20260910/fixture-coverage.json`.
