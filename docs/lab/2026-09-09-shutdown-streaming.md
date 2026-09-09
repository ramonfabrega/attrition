# Stream shutdown data without changing the record parser

The census now peaks at **260 MB instead of 6.52 GB**, with the same 95-capture
assertions and approximately 27-second runtime. The closing/ordinary comparison
uses about **174 MB** and preserves its 36 pairs and 110 observed unit movements.
These are isolated-test measurements, not a whole-suite memory claim.

## Reader boundary

`IndexedCapture::read_shutdown` reads the last indexed GAME frame through EOF
and wraps that suffix in GAME. Keeping the last frame is necessary: fields
written at its own indentation can belong to GAME, including the leader flag
run used by the closing reader. Cutting at the next BEGIN sibling would lose
those fields. The existing `Log::final_state` interprets the resulting text.

The API's retained source text is the last frame plus the entire remaining
suffix, not a fixed-size buffer. A file with a huge suffix still costs that
suffix; this is not a universal constant-memory claim. The largest tail across
the checked corpus was 61,258,359 bytes, including the synthetic GAME wrapper.
Index construction still scans the entire file with the existing reusable line
buffer; no-frame inputs produce an empty GAME. Metadata checks and the existing
index cache restrictions remain: finalized files, length/mtime invalidation,
no unsafe mapping, and no atomic-snapshot guarantee.

`read_frame_and_siblings` reads from a selected frame to the next indexed frame
(or EOF). It lets the existing `dumps` reader classify FULL DUMP siblings.
`frame_state(index)` decodes one ordinary frame, with the same check that exactly
one matching frame was produced. Indices preserve duplicate labels by position.

## What migrated

The census reads the shutdown suffix and every frame with the final label,
including its siblings, to retain its `any` semantics for duplicates. Closing
counts and FULL DUMP/nested/absent classification still come from existing
record readers. The 95 ledger tuples and assertions are unchanged.

The closing/ordinary test reads the first populated matching frame and searches
backward for the last populated frame with a lower label. Those are the same
selection rules its full Vec used. It compares the same unit positions and flags
and counts the same movements. No simulation or acceptance-floor change.

## Equivalence and measurements

`shutdown_memory verify` compared the complete `Option<Frame>` returned by the
whole-file and suffix readers on 99 run captures plus the three fuzz/scratch
archives: **102 files, 67 closing states, 6,442 closing units**. Equality covers
all parsed nested fields, leaders, buildings and cities, not just unit counts.
This establishes equivalence to the existing parser; it adds no new claim about
original-engine behavior. The reference still uses the full reader.

Synthetic checks cover trailing leader flags with setup flags excluded,
shutdown-only units, duplicate frame labels, FULL DUMP sibling pairing, absent
closing state, no frames, and mutation after indexing. They supplement the
existing UTF-8, truncation, cache and frame-boundary checks.

A saved pre-migration binary and the new binary used the same ordinary release
settings. The measurements were separate sequential processes with the real
install and two-thread setting; compilation was outside the intervals.

| census measurement | before | after |
| --- | ---: | ---: |
| test body | 27.16 s | 27.40 s |
| process wall time | 27.50 s | 27.40 s |
| maximum RSS (`time -l`, bytes) | 6,520,619,008 | 260,227,072 |

That is about **96% less peak RSS**, with essentially unchanged runtime in this
single pair. The pair-comparison test passed in 14.01 seconds at 173,948,928 bytes
maximum RSS. Its prior isolated-run log independently confirms the same
36-pair/110-movement counts; that prior run had different concurrency, so it is
not a controlled timing comparison for this test.

## Reproduce

```sh
cargo build -p rondata --release --example shutdown_memory
target/release/examples/shutdown_memory verify CAPTURE_OR_ARCHIVE_DIRECTORY
/usr/bin/time -l target/release/examples/shutdown_memory indexed CAPTURE
/usr/bin/time -l target/release/examples/shutdown_memory whole CAPTURE
```

The directory mode selects `gamelog-*.txt`, excluding the live `gamelog.txt`.
Use finalized archives. `verify` holds the reference data too, so it is not a
memory benchmark. The `whole` and `indexed` modes measure shutdown reading,
not the full census classification workload. Run heavy corpus checks under
`tools/memcap.sh` with working process-sampling access.

Raw logs: `/tmp/attrition-shutdown-verify.log`,
`/tmp/attrition-tail-extra-verify.log`, `/tmp/attrition-tail-{before,after}.log`,
and `/tmp/attrition-tail-pairs.log`. No capture body is committed.

## Next: replay prefixes have two consumers

Reading `run_traced` and `Log::initial` identified a necessary distinction for
the next change. `run_traced` currently collects all frame states before taking
its frame-count limit. But `initial()` also walks the entire capture to collect
animation lengths, frame seeds, figure clocks and observed figure positions.
Limiting only the expected-state Vec would save one pass, not eliminate all
suffix decoding. The limit counts logged frames, not a frame-number cutoff.

The next experiment should separate setup/animation metadata from per-frame
observations, then feed expected frames and observations incrementally. Preserve
existing report values for bounded runs and verify that required initialization
metadata remains available. Do not silently discard later observations while
continuing to call the resulting initialization equivalent.

## Landing checks

The complete workspace release gate passed: rondata 255 passed/one ignored,
sim 821 passed, plus fixed and doctests. The sampled tree peak was 10,844 MiB,
with 10,812 MiB in the largest process. Other whole-file consumers therefore
still determine the suite's peak; the isolated 96% saving must not be applied
to the whole suite. Structural install survey, clippy with warnings denied,
formatting and all seven paperwork guards passed. Full gate log:
`/tmp/attrition-shutdown-full-gate.log`.
