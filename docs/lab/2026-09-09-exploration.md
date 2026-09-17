# Capture reuse exploration

Independent exploration of the supplied executor primer, on
`codex/capture-reuse-exploration`, from
`bbd51735aba5384b34b6f9a22b884fe6215e9de9`. No queue item taken and no
simulation, parser, comparator, or floor changed.

## Finding

The test input door does not cache captures. More significantly, sharing a
parsed `Log` would not cache all its query work. A repeated `frame_seeds()`
on the same object still costs approximately the first scan's time. On
run29, an input actually queried by army scene setup, that is about 247 ms;
on run39, the repeated scan takes about 770 ms even though it finds no seeds.

A raw-text cache would save allocations and reads, but these measurements
put warm reads well below indexing and seed extraction. They do **not**
measure a full-suite speedup or prove that seed extraction dominates the gate.
The next experiment I would book is invocation timing/counting of the
whole-log owned queries in a complete gate, followed by scoped reuse of the
most frequently rebuilt result. A global cache of all capture text is not
justified by these timings alone.

## What was read

- `crates/rondata/src/lib.rs`, `testenv::dump`/`dump_path`: checks existence
  and returns a path, not text or a parsed capture.
- `crates/rondata/src/capture.rs`, `read`: a fresh `read_to_string` each call.
- `crates/rondata/src/diff/testkit.rs`, `sibling_texts`: reads its sibling
  inputs afresh on each invocation.
- `crates/rondata/src/gamelog.rs`, `Log::parse`: indexes text into a new
  arena. The arena lives behind `RefCell`; this is not a `Sync` object to
  drop into a shared cross-thread cache unchanged.
- In that same file, `frame_seeds` calls `scan_children`. For indexed
  bodies, the latter creates a scratch log, fills it eagerly, and resets
  it between children. A second scan repeats that work. Laziness avoids
  retaining the whole materialized capture, but does not memoize this query.
- `crates/rondata/src/diff/army.rs`, `scene_at`: asks for all frame seeds
  and then searches for one frame's seed. This is an actual consumer,
  unlike the run39 benchmark's deliberately empty query.

These are source observations, not runtime invocation counts. In particular,
literal filename occurrence counts are not counts of tests or file opens.

## Measurement

The new read-only example `crates/rondata/examples/capture_cost.rs` measures
input read, `Log::parse`, frame-number enumeration, seed extraction, a second
seed extraction on the **same** log, and destruction. It asserts exact
agreement of frame-number and seed vectors across repetitions and between
the two scans. No seed values or proprietary text are saved in the report.

Commands used on this machine (release, serial, no cache flush):

```sh
logs="$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
cargo run -p rondata --release --example capture_cost -- 5 \
  "$logs/gamelog-run38-islands-start.txt" \
  "$logs/gamelog-run39-islands-longtrace.txt" \
  "$logs/gamelog-run54-islands-24k-trace.txt" \
  > /tmp/attrition-capture-cost-final.csv
target/release/examples/capture_cost 5 \
  "$logs/gamelog-run29-islands-engagement-window.txt" \
  > /tmp/attrition-capture-cost-run29.csv
```

Medians of iterations 1–4, in milliseconds; iteration 0 excluded to reduce
first-read effects. These are warm-process observations, not cold disk I/O.
All samples, including iteration 0, are in the adjacent
`2026-09-09-exploration-samples.csv`, with local directory prefixes removed.

| Capture | Bytes | Read | Index | Frame enumeration | Seed scan | Same-log repeat |
|---|---:|---:|---:|---:|---:|---:|
| run38 islands start | 153477471 | 10.494 | 120.468 | 0.002 | 106.755 | 105.309 |
| run39 islands longtrace | 482309960 | 32.650 | 435.857 | 0.128 | 778.947 | 769.975 |
| run54 islands 24k trace | 11097398 | 0.804 | 15.582 | 0.004 | 0.163 | 0.154 |
| run29 engagement window | 251324874 | 17.184 | 159.380 | 0.002 | 254.779 | 247.320 |

The frame/seed counts were respectively 3/1, 1851/0, 2/0, and 5/3.
These are the parser's frame-block counts, not the number of simulated ticks
implied by a capture filename. Warm same-log repeat ranges were
101.607–108.028 ms, 769.460–802.022 ms, 0.148–0.169 ms, and
246.398–256.468 ms respectively. Small timing differences are not evidence
of an improvement; the useful contrast is that the repeated query still
costs essentially a full scan.

## Interpretation and limits

Measured: repeated string reads, indexing, and this query's two scans on
four existing files. The same-log experiment rules out assuming that
retaining the existing `Log` automatically eliminates query reparsing.

Reasoned: caching an owned seed vector within a capture's test scope can
avoid subsequent scans without retaining its scratch trees. Whether that
saves appreciable gate time depends on actual query reuse. A content-keyed
persistent cache could also avoid repeated indexing, but hash validation,
serialization, invalidation, memory retention, and concurrency were not
implemented or timed. A path-only cache would not satisfy the primer's
content-identity requirement.

Not established: gate CPU attribution, actual per-capture open/parse counts,
RSS, allocator retention, two-thread contention, comparator output parity,
any gameplay finding, or a speedup for the real suite. Timing `drop` is not
measuring whether the allocator returns pages to the OS. This example also
does not test every parsed field: its equality checks cover only its query
outputs. No full gate was run because there is no product change; a proposed
cache would still owe the full existing assertions and printed-output parity.

Validation: the release probe completed all repetitions and equality checks;
`cargo clippy -p rondata --example capture_cost -- -D warnings` and
`cargo fmt --all -- --check` passed. An initial constant-assert Clippy failure
was fixed by returning an error for debug execution. The measured release
path is otherwise unchanged.
