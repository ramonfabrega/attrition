# Stream replay frames and retain only replay setup observations

A 3,000-frame run69 replay now peaks at **568 MB instead of 1.62 GB** in the
isolated comparison below, with the complete report unchanged. A 30-record prefix
peaks at **530 MB instead of 1.59 GB** and its replay timer falls from 3.27 s to
1.69 s. These measurements use the main capture alone, without siblings, a trace
or a recording; they are not a claim about autonomous fidelity or every test.

## What changed

`Log::visit_frame_states` uses the existing scratch-arena child scan and existing
record readers. It passes one owned `Frame` to its visitor at a time. A limit
counts ordinary frame records, including duplicate labels and empty frames; zero
returns before scanning children. Reaching the limit stops before decoding the
next child. `frame_states` remains the complete collecting API for callers that
need random access. It now collects through the visitor.

`run_traced_observed` compares each visited frame immediately. It no longer
allocates a complete `Vec<Frame>` before applying its limit. Setup is explicitly
dropped after `build_sim`, whose correction inputs are owned. Order application,
tick cadence, comparisons, observer placement and returned report are unchanged.

The crate-private `Log::replay_initial` retains every setup field, seed,
animation length and figure-clock correction, but omits `frame_bodies`: the
figure-position series consumed by an audit, not `build_sim`. It moves clocked
rows into the correction series rather than cloning them for an unused second
series. The public `initial` keeps its complete original behavior, including the
figure-position audit series. Existing audits and caller-owned sibling setup
values are not silently truncated.

This is a consumer boundary, not a new interpretation of the original format.
Tests compare complete `Initial` values with only `frame_bodies` cleared on both
a position-only capture (run6) and a capture carrying seeds and clocks (run13).
The latter asserts that all three observation series are nonempty before the
comparison, so it cannot pass by failing to exercise correction inputs.

## Measurements and equivalence

The saved baseline binary uses commit `a13abf6`'s replay implementation. The
intermediate binary streams frames and drops setup; the final binary also uses
`replay_initial`. Each runs in a separate sequential process on the same
467,769,059-byte run69 capture with the same install and ordinary release profile.
Compilation is outside the measurements. `time -l` supplies exact process peak
RSS; `tools/memcap.sh 20` monitors the process tree throughout.

| implementation | records | peak RSS bytes | MiB | replay seconds |
| --- | ---: | ---: | ---: | ---: |
| baseline | 30 | 1,586,479,104 | 1,513.0 | 3.273 |
| streamed comparisons + setup drop | 30 | 949,518,336 | 905.5 | 1.759 |
| replay-only observations | 30 | 530,497,536 | 505.9 | 1.691 |
| baseline | 3,000 | 1,623,834,624 | 1,548.6 | 3.312 |
| streamed comparisons + setup drop | 3,000 | 966,361,088 | 921.6 | 3.419 |
| replay-only observations | 3,000 | 567,803,904 | 541.5 | 3.272 |

These are single paired observations, not a statistical speed claim. Prefix
replay time is about 48% lower, and memory about 67% lower; full replay memory is
about 65% lower while its time is essentially unchanged. The timer covers
initialization and replay, excluding install/source loading and final report
serialization. The measurement probe originally wrote the report unbuffered,
which dominated process wall time for 3,000 records; that wall time is therefore
not reported as replay time. The committed probe buffers that output for future
use and explicitly flushes it.

All three implementations write identical complete reports at both limits:
1,001,894 bytes for 30 records and 120,840,611 bytes for 3,000. Byte comparisons
cover every `Report` field, including notes, correction draw counts and applied
orders. Synthetic visitor checks also compare against direct block traversal
with nested full dumps, city records, repeated labels, empty frames, zero limits
and limits past EOF. The full release suite remains the fidelity gate.

To repeat a measurement after building the probe:

```sh
cargo build -p rondata --release --example replay_memory
zsh tools/memcap.sh 20 /usr/bin/time -l \
  target/release/examples/replay_memory \
  "$RON_INSTALL" "$CAPTURE" 30 /tmp/new-replay-report.txt
```

Choose a new report path outside git. The report contains local capture-derived
values. Compare complete outputs from separately built revisions with `cmp`.

## Remaining boundary

The source string and child index remain resident. Initialization still scans
the whole capture for animation lengths, seeds and clock corrections, even for a
short prefix; no prefix-only setup claim is made. The visitor retains one frame
and the scratch arena's largest capacity, not constant total process memory.
The returned `Report` still grows with compared frames, and caller-owned sibling
`Initial` values can still contain audit observations. The next step should be
chosen from fresh memory measurements, distinguishing source text, correction
series and the reports that tests actually retain.

The existing canonical run6 viewer export was also regenerated with its four
setup siblings. Every full exported original/Rust unit record, typed difference
row and harness note matched the pre-change artifact across 30 records and 450
paired positions. This checks the read-only observer's delivered state as well
as the report-only comparison above.

Final gate: 264 rondata tests passed (one ignored), 821 sim, 13 fixed and three
doctests. Rondata took 240.61 s, with a sampled process-tree peak of 8,757 MiB
under the 20 GiB ceiling. Clippy with warnings denied, formatting, seven paperwork
guards and the install survey passed. The suite peak is sampled, unlike the
isolated `time -l` process high-water marks in the table.
