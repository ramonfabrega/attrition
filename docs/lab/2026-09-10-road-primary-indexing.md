# Indexed primary setup for road regressions

## Change and fidelity

L45 bounded the sibling inputs and released construction inputs before ticking,
but still read each primary capture in full. The same two road tests now use
`IndexedCapture::with_replay_initial` for their primary run10/run71 input.
They retain all owned frame seeds, animation lengths and figure clocks needed
by replay, in addition to the borrowed setup data. Audit frame bodies are
omitted, as in the existing indexed replay path. Sibling precedence and the
market test's pasture correction are unchanged, as are every road assertion
and the simulated frame counts.

A new retained-corpus regression compares every Initial field with whole-text
parsing for both primary captures, excluding only audit frame bodies. It calls
the indexed path twice to exercise reuse and passed in 8.32 seconds. These two
fixture lookups add to the gate's observed request count without adding new
capture dependencies. Evidence: `/tmp/road-primary-equality.log`.

No parser, cache policy or sim implementation changes. The indexed reader
still scans frames for correction observations on a cache miss, so this is
bounded retention rather than setup-only I/O. It uses safe reads and the
existing finalized-source metadata contract, not memory mapping or content
hash binding. The comparison capture in the market test remains whole-text.

## Isolated measurements

The baseline is branch tip `3111c55`, already including scoped sibling reads.
Clean release binaries ran sequentially with explicit RON_INSTALL and one test
thread. Each test used before/after/after/before order in fresh processes;
build time is excluded and process caches begin cold. All eight runs passed,
with no fixture skips. RSS comes from `/usr/bin/time -l`.

| Test | Sample | Peak RSS bytes | Test seconds |
| --- | --- | ---: | ---: |
| Road rings | 1 before | 965,033,984 | 3.60 |
| Road rings | 2 after | 547,962,880 | 3.59 |
| Road rings | 3 after | 531,775,488 | 3.59 |
| Road rings | 4 before | 1,019,822,080 | 3.63 |
| Market road | 1 before | 1,950,842,880 | 6.19 |
| Market road | 2 after | 1,030,356,992 | 6.05 |
| Market road | 3 after | 1,028,210,688 | 6.00 |
| Market road | 4 before | 1,912,995,840 | 6.16 |

The market-road test drops a further 46–47% in peak RSS to about 1.03 GB,
with slightly lower observed test time. Road rings drop about 43–48% to
0.53–0.55 GB, with essentially unchanged time. These ranges compare isolated
samples, not statistical confidence intervals or universal gains. Relative
to the pre-L45 runs, the two steps together reduce each test's peak about 68–71%;
those earlier baselines were measured separately and are not a fresh combined
experiment. Logs: `/tmp/road-primary-run*-{1-before,2-after,3-after,4-before}.log`.

The remaining market-road comparison reads run72 in full for one WORLD block
and its height grid. A bounded frame comparison is a next candidate, provided
it preserves those height lookups and every existing tile assertion. No queue
item or fidelity score changed, and the paused runtime drafts are untouched.

## Full gate

The explicit-install monitored four-thread gate passed 279 rondata tests
(one ignored, zero filtered), 824 sim tests, 13 fixed tests, three doctests,
and 34 Python tests, plus install survey, clippy, fmt and paperwork guards.
Rondata took 123.49 seconds; sampled tree peak was 12,223 MiB under 20 GiB.
All 556 fixture requests were present, including the two added complete-input
comparisons. This single suite run is validation, not a controlled speedup
measurement. Evidence: `/tmp/road-primary-gate.log` and
`/tmp/attrition-gate-road-primary-20260910/fixture-coverage.json`.
