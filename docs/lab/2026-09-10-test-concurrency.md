# Revisit test concurrency after reader memory reductions

## Scope and safeguards

The normal release gate remains pinned to two threads by `.cargo/config.toml`
and `diff::floors::tests::the_gate_is_pinned_to_two_threads`. Earlier wider
runs crossed 20 GiB; nothing in this experiment edits that policy or claims
the old measurement was wrong.

Recent bounded readers materially changed capture retention. This experiment
screens three and then four test threads on the current branch, with explicit
RON_INSTALL, the same 20 GiB process-tree monitor, and fixture-request auditing.
Only the width-policy assertion is filtered from the benchmark. It is not a
release gate, and all other rondata library tests remain selected.

Each benchmark uses this command shape, with a fresh audit directory and
one process at a time (substitute the owned install and requested width):

```sh
RON_INSTALL="$install" RON_FIXTURE_AUDIT_DIR="$audit_dir" \
  /usr/bin/time -l zsh tools/memcap.sh 20 \
  cargo test -p rondata --release --lib -- \
  --test-threads=3 \
  --skip diff::floors::tests::the_gate_is_pinned_to_two_threads
```

The width assertion's omission is intentional and visible in the result:
274 passed, one ignored, one filtered, rather than the normal 275 passed.
The audit's `release_completed` indicates that the benchmark cargo child
completed, not that this was a policy-compliant full gate. The monitor stays
active; its sampled ceiling is not a hard allocation limit between samples.

## Three-thread screening result

At tip `23f18ea`, three threads passed all selected tests in 162.92 test
seconds (163.92 process wall seconds). Sampled tree peak was 10,998 MiB.
All 550 observed fixture requests were present, covering 117 unique fixtures.
This is materially below the 20 GiB cap, but one sample is not a worst-case
memory bound. Recent two-thread rondata runs were about 240 seconds.

Evidence: `/tmp/attrition-three-thread-20260910/benchmark.log` and its
`fixture-coverage.json`. CPU/user totals from the outer timing process include
children; hardware-counter lines for the monitor are not whole-suite counters.

The successful three-thread screen justified a four-thread experiment under
the same cap. The normal gate and its width guard remain unchanged. No live
game runs, simulation rules, or fidelity scores are involved.

## Four-thread screening result

Four threads passed in 118.65 test seconds (120.13 process wall seconds),
with a sampled tree peak of 13,662 MiB. All 550 fixture requests were present.
Evidence: `/tmp/attrition-four-thread-20260910/benchmark.log` and its
`fixture-coverage.json`. This is approximately half the recent two-thread
run time. It does not establish that every future capture set fits this width,
or that another worker can run a heavy gate concurrently.

A second four-thread run and an unchanged two-thread full gate follow as
repeatability and control checks. No default is changed by this report.

## Four-thread repeat

The second four-thread run passed in 118.10 test seconds (120.14 process
wall seconds), with a sampled tree peak of 12,596 MiB and all 550 fixture
requests present. Evidence:
`/tmp/attrition-four-thread-repeat-20260910/benchmark.log` and its
`fixture-coverage.json`. No other heavy task was deliberately launched
alongside either four-thread run.

The two measured peaks leave 6,818–7,884 MiB below the 20 GiB sampled cap.
That is observed margin, not a guarantee against brief between-sample peaks
or a changed corpus. A larger width is not tested here.

## Adoption proposal

Four threads are a strong candidate for a monitored release lane on this
branch's current corpus: both repeated runs complete in about two minutes.
Keep a conservative two-thread default for ordinary direct cargo invocations;
a future explicit monitored-gate option can carry a separately reviewed width
policy and retain fixture auditing. The current width assertion must be
updated deliberately for such a lane, not silently skipped in production.
The benchmark command above is diagnostic only.

Before adopting, steering should reproduce against its selected commits and
corpus, keep the 20 GiB monitor, and schedule heavy runs sequentially. These
results do not license the historical default of sixteen threads, and do not
prove another machine or expanded capture corpus has the same margin. No
change to `.cargo/config.toml`, the width guard, QUEUE, or DECISIONS is made.

## Fresh two-thread control and conclusion

The unchanged full gate passed: 27 Python tests, 275 rondata tests (one
ignored), 824 sim, 13 fixed and three doctests, plus survey, clippy, formatting
and guards. Rondata test time was 240.43 seconds; sampled tree peak was
9,214 MiB. The 550 fixture request records match the benchmark records exactly,
including filenames, requesting tests, and present/missing counts.

| Run | Rondata test seconds | Sampled tree peak, MiB | Width assertion |
| --- | ---: | ---: | --- |
| Three-thread benchmark | 162.92 | 10,998 | Filtered |
| Four-thread benchmark | 118.65 | 13,662 | Filtered |
| Four-thread repeat | 118.10 | 12,596 | Filtered |
| Two-thread full-gate control | 240.43 | 9,214 | Passed |

Four threads delivered approximately twice the rondata test throughput in
these runs, for roughly 3.3–4.3 GiB more sampled peak memory than the control.
The comparison is not perfectly identical work: the control includes the
width assertion, and its outer gate includes other stages. Only rondata's
reported test interval is compared above. The claimed gain is not a halving
of every surrounding survey/build/lint stage.

The result supports an opt-in monitored four-thread gate proposal, with the
normal two-thread default retained. This turn lands evidence and a proposal,
not a changed execution policy. The experiment's entire code baseline remains
`23f18ea`; only lab documents change.

Control log: `/tmp/concurrency-control-final-gate.log`; fixture report:
`/tmp/attrition-gate-concurrency-control-20260910/fixture-coverage.json`.

The subsequent [monitored wide-gate implementation](2026-09-10-monitored-wide-gate.md)
adds this opt-in lane with the width guard included and preserves the direct
two-thread default. This report remains the earlier benchmark evidence.
