# File-backed differential replay

The optional indexed reader removes the resident primary capture string from
replay. It reduces full run69 peak RSS from 565 MB to 118 MB, with an identical
complete report. It is slower in this measurement, so the viewer keeps the
in-memory default and exposes `--reader indexed` as a memory-saving choice.
No simulation rule, acceptance floor or queue item changes.

## Implementation and boundaries

Both readers use the same private replay state machine for ticking, recorded
orders, comparisons, observers and report assembly. The indexed entry point is
`diff::run_indexed_observed`. Setup strings are borrowed only inside
`IndexedCapture::with_replay_initial`; the built simulation owns its inputs.

Setup cannot be reduced to the text before the first frame: existing parser
lookups can select a late first FULL DUMP, WORLD, CITIES or CONSTANTS child,
and flat GAME fields can occur after frames. `read_replay_setup` retains the
prefix, those first children, all GAME-level fields and all non-GAME roots.
An empty frame marker preserves the initial-record traversal boundary. This
reuses the existing parser semantics rather than asserting a new game format.

Whole-capture observations are still required. The reader scans one frame plus
its following siblings at a time, using the same observation accumulator as
`Log::initial`, retaining seeds, animation lengths and figure-clock corrections.
The public full initializer still retains its figure-position audit series;
replay still omits that unused series. Frame comparison then reads one indexed
frame at a time. Limits count records, not distinct labels.

This is not constant process memory. The compact setup, largest frame/sibling
segment, correction series, simulation and returned report still consume memory.
Caller-owned siblings are unchanged; the viewer still loads them in full. The
reader inherits the index's conventional single GAME / one-space direct FRAME
indentation requirement and refuses unsupported shapes. It checks source length
and modification time before/after reads and after the last observer. This is
ordinary mutation detection for finalized captures, not an atomic snapshot or
protection against same-size edits with restored timestamps.

## Measurements

Separate sequential release processes replayed the same 467,769,059-byte run69
primary capture, without siblings, trace or recording. `/usr/bin/time -l`
provided process peak RSS; `tools/memcap.sh 20` watched the process tree.
Compilation and install loading are outside the timer. The probe now times
**source loading/indexing plus setup and replay**, excluding buffered report
serialization. Its seconds cannot be compared directly with the narrower timer
in `2026-09-09-streaming-replay.md`.

| reader | records | peak RSS bytes | load + replay seconds |
| --- | ---: | ---: | ---: |
| memory | 30 | 530,497,536 | 2.006 |
| indexed | 30 | 79,413,248 | 3.705 |
| memory | 3,000 | 565,444,608 | 3.539 |
| indexed | 3,000 | 117,751,808 | 6.078 |

These single observations show approximately 85% and 79% less peak memory,
respectively, with extra scan/parsing cost. They do not establish higher test
throughput. At this revision, cold index construction, setup selection, observation
collection and frame decoding require separate passes. The subsequent
`2026-09-09-replay-setup-ranges.md` removes the separate setup-selection scan
by selecting ranges during index construction. A cached correction
product would need explicit versioning and invalidation before it could be an
oracle input.

Reproduce with a fresh report path outside git for each reader and compare the
complete output files with `cmp`:

```sh
cargo build -p rondata --release --example replay_memory
zsh tools/memcap.sh 20 /usr/bin/time -l \
  target/release/examples/replay_memory \
  "$RON_INSTALL" "$CAPTURE" 3000 /tmp/new-indexed-report.txt indexed
```

Use `memory` for the paired baseline and `verify-setup` for complete setup
comparison with only the unused audit series removed.

## Evidence

The complete reports are byte-identical at 30 and 3,000 records, including all
notes, applied orders and correction draw counts. Complete setup values also
match on six existing captures: run6, run13, run20 (full dump), run69, run82 and
run906 (coverage). Synthetic tests exercise late lookup children, trailing GAME
fields, late non-GAME roots, repeated frame labels, blank lines and no frames.
A mutation test truncates an indexed source and verifies that setup and final
validation refuse it. A release test compares complete reports at zero and 12
records with a sibling supplying correction data.

The canonical 30-record viewer export is also regenerated with its four setup
siblings and compared at the full frame-record and note level. Artifacts remain
outside git. No primary test fixture has been migrated to indexed replay yet;
the full suite checks the shared state-machine refactor and the added indexed
tests, not an all-indexed test run.

Validation: full release gate passed 267 rondata tests (one ignored), 821 sim,
13 fixed and three doctests. Rondata took 242.08 s; sampled process-tree peak
was 8,618 MiB under the 20 GiB ceiling. Clippy with warnings denied, formatting,
seven paperwork guards and the real-install survey passed. The sampled suite
peak is distinct from the isolated process high-water marks above.
