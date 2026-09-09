# Select replay setup during capture indexing

The indexed reader now records retained setup byte ranges during its existing
index scan. This removes a complete source scan from replay initialization.
On the paired run69 measurement, source loading plus a 3,000-record replay falls
from 6.044 to 5.141 seconds; a 30-record prefix falls from 3.754 to 2.927 seconds.
Complete reports are unchanged. No queue score or simulation rule changes.

## Profile and implementation

The isolated phase probe measured the previous reader as follows: index 0.606 s,
setup selection 0.895 s, complete initialization (including setup selection)
3.114 s, and decoding all 3,001 indexed records 2.255 s. The setup text is only
10,765,643 bytes of a 467,769,059-byte capture. Re-scanning every source line to
select that text was avoidable because index construction already visits them.
The complete-initialization time overlaps selection; these phases must not be
added as independent costs.

`SetupRanges` applies the existing selection rules to lines already decoded by
index construction. Adjacent retained lines coalesce into a range. The reader
seeks and reads those ranges, inserting the existing empty frame marker after
the prefix. Late first setup children, GAME-level fields and non-GAME roots
retain the previous semantics. No capture-derived strings enter the index cache.
Both frame and setup range storage count against its existing eight-MiB budget.
Source length/mtime checks and unsupported-shape refusal remain in force.

Afterward, the same phase probe measured index 0.643 s and setup selection
0.001 s, retaining exactly the same byte count; complete initialization was
2.323 s. These single observations locate the removed work. The committed
`indexed_profile` example reproduces the stages without loading an install or
running a simulation. It requires release mode and decodes every indexed record,
so its 3,001-record decode pass differs from the 3,000-record replay below.

## Paired replay and concurrency measurements

A saved executable from `0caf4f8` and the new executable ran sequentially on the
same main run69 capture and install, without siblings, recording or trace.
The ordinary release profile and `/usr/bin/time -l` supplied process peak RSS.
The replay timer includes source loading/index construction, setup and replay;
it excludes install loading and buffered report serialization.

| indexed reader | records | load + replay seconds | peak RSS bytes |
| --- | ---: | ---: | ---: |
| previous | 30 | 3.754 | 85,344,256 |
| selected ranges | 30 | 2.927 | 80,969,728 |
| previous | 3,000 | 6.044 | 121,044,992 |
| selected ranges | 3,000 | 5.141 | 110,002,176 |

That is about 22% less elapsed time for the prefix and 15% for the full replay.
These are single paired measurements, not confidence intervals. Memory remains
far below the resident-source reader; the small differences between indexed
versions are not treated as an independent memory optimization.

A separate four-process batch compared the current in-memory and indexed
readers, each replaying 3,000 records. Batch wall time includes install loading
and writing four complete reports: **5.092 s memory, 6.879 s indexed**. Individual
process peaks were 566–569 MB and 115–118 MB respectively. The shared watchdog
sampled 2,081 MiB maximum over the entire sequential/parallel experiment under
its 20 GiB ceiling. Summing process high-water marks would not establish a
simultaneous tree peak. Filesystem caches were uncontrolled; the memory batch
ran first. This is a bounded concurrency check, not a full-suite benchmark or a
memory-pressure experiment.

The four-worker case does not demonstrate a throughput win from lower memory.
Keep the default reader unchanged. Under tighter memory limits the useful
concurrency may differ, but this experiment does not establish that threshold.
Remaining costs include parsing correction observations across the entire
capture and parsing compared frames. This probe produces zero clock frames on
run69, yet the observation accumulator still decodes full unit records before
filtering out units without clocks. The subsequent `2026-09-09-selective-clock-observations.md` implements and
measures that selection, preserving seeds and animation lengths even when no
clock rows survive. Raising concurrency solely because RSS is lower is
not supported by these measurements.

## Verification and reproduction

Complete reports match the previous executable at both limits. All eight
parallel reports match the sequential 3,000-record report. Six real complete
setup comparisons cover run6, run13, run20, run69, run82 and run906. The synthetic
range test asserts exact output text and retained byte count across CRLF, blank
lines in omitted frames, late fields, duplicate labels and a final root without
a newline. It also checks cached range reuse and cache byte accounting. Existing
setup-equivalence and source-mutation tests remain applicable. The canonical
four-sibling viewer export also matches all 30 full frame records and notes,
with 450 paired positions.

Build and run the standalone phase probe:

```sh
cargo run --release -p rondata --example indexed_profile -- "$CAPTURE"
```

Use the `replay_memory` example documented in `2026-09-09-indexed-replay.md`
for complete report comparisons. Every report and viewer artifact stays outside
git. A byte range is a source-selection optimization, not a cached oracle result.

Validation: the full release gate passed 268 rondata tests (one ignored),
821 sim, 13 fixed and three doctests. Rondata took 248.11 s; sampled tree peak
was 9,195 MiB under the 20 GiB ceiling. Clippy with warnings denied, formatting,
seven paperwork guards and the real-install survey passed. This suite run is a
correctness gate, not an isolated performance comparison; setup verification
and viewer export also ran while it was active.
