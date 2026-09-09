# Native profiling: avoid decoding records the assertion never uses

The 95-capture shutdown census is slow in our own Rust parser, not in an opaque
engine dependency. A native sampled call graph made a selective lookup worth
implementing before changing allocators or replacing parsers.

## Method and evidence

Built a separate optimized binary with `CARGO_PROFILE_RELEASE_DEBUG=1` and
`CARGO_TARGET_DIR=/tmp/attrition-profile-target`. No release optimization,
overflow-check, dependency, or simulation setting was changed. `/usr/bin/sample`
attached to the census test process, requested 90 seconds at 5 ms intervals,
and finished when the process exited. The test passed all 95 archives in
80.12 seconds under sampling.

The graph contains **14,063 samples from the test thread**, including blocked
states. It excludes the libtest main thread waiting for completion. These are
sampled stack frequencies, not exact CPU-time percentages or a time-ordered
trace. Inclusive percentages overlap and must not be added.

| stack | inclusive share of test-thread samples |
| --- | ---: |
| `Log::frame_states` | 65.5% |
| `Log::parse` | 22.1% |
| `Log::dumps` | 6.8% |
| `std::fs::read_to_string` | 4.2% |
| `gamelog::fill`, across callers | 66.7% |

Leaf samples identify line scanning and field lookup: `memchr_aligned` 25.3%,
`Log::field` 14.2%, `fill` 13.9%, string-pattern scanning 11.9%, and platform
`memcmp` 10.0%. The `read` syscall accounts for 3.8%. This profile does not
justify replacing compression or XML dependencies: neither is the hot path of
this workload. `cargo tree --workspace --edges normal` confirms the simulation
has no third-party dependencies; rondata's external roots are flate2 and
roxmltree. Other workloads still need their own profiles.

## Change and unsampled comparison

The census previously called `frame_states()` for each capture, then used the
result only to ask whether a particular frame number contains units. It now
filters the existing frame-name index before calling the same `records` reader
used by `frame_states`. Duplicate labels retain `any` semantics. The closing
state reader, every recorded census tuple, all 95 input captures and the
assertions are unchanged. Other whole-frame fidelity tests remain in the suite.

Both unsampled measurements used the same debug-symbol release configuration,
exact test name, install and two-thread guard, as separate sequential processes.
Compilation and graph rendering were outside the measured intervals.

| measurement | before | after |
| --- | ---: | ---: |
| test body | 78.61 s | 28.91 s |
| process wall time | 78.66 s | 29.38 s |
| user CPU time | 74.03 s | 24.53 s |
| instructions retired (`time -l`) | 1,747,455,986,717 | 525,113,748,980 |
| maximum RSS | 6,278,725,632 B | 6,721,961,984 B |

This is roughly **2.7× faster and 70% fewer retired instructions**, with
**7% higher maximum RSS** in this pair. It is a speed win, not a memory win;
whole-file text and materialized lazy blocks remain. One sequential pair is
not a noise estimate. The prior frame-streaming change addresses those retained
allocations separately; see `2026-09-09-streaming-captures.md`.

## Reproduce and inspect

`tools/explore/profile_native.py BINARY EXACT_TEST NEW_OUTPUT` runs one test
under macOS sampling and verifies it reports exactly one pass. Set `RON_INSTALL`
and wrap it in `zsh tools/memcap.sh 20`. Build first, outside the measurement:

```sh
CARGO_TARGET_DIR=/tmp/attrition-profile-target CARGO_PROFILE_RELEASE_DEBUG=1 \
  cargo test -p rondata --lib --release --no-run
```

`tools/explore/sample_graph.py SAMPLE TEST NEW_OUTPUT --flamegraph CHECKOUT
--cxxfilt LLVM_CXXFILT` produces a zoomable SVG, folded stacks and hotspot JSON.
It preserves waiting stacks and selects the named test thread before computing
the denominator. It demangles Rust symbols with LLVM. The original raw sample's
thread count was checked against the graph's 14,063-sample total.

The external [FlameGraph tools](https://github.com/brendangregg/FlameGraph) were
read from commit `41fee1f99f9276008b7cd112fca19dc3ea84ac32`, in
`/tmp/attrition-flamegraph-tools`; they are not vendored. The
[macOS sample collapser](https://github.com/brendangregg/FlameGraph/blob/41fee1f99f9276008b7cd112fca19dc3ea84ac32/stackcollapse-sample.awk)
normally hides some waiting leaves; the wrapper explicitly disables that filter.

Local artifacts: `/tmp/attrition-native-profile/{sample.txt,test.log,result.json}`,
`/tmp/attrition-census-graph/{flamegraph.svg,test.folded,hotspots.json}`, and
`/tmp/attrition-census-{before,after}.log`. These contain our program's symbols
and measurement metadata; no original game code or capture body is committed.


## After-profile and next targets

The same test under sampling after the change passed in 27.83 seconds overall.
Its test thread has 4,384 samples: `frame_states` no longer appears, while
`Log::parse` accounts for 64.4%, `Log::dumps` 18.5%, and whole-file reading 14.4%.
`fill` is 82.8% inclusive across callers. Those larger percentages describe a
much shorter run, not a regression in absolute work. The before/after graph
artifacts are `/tmp/attrition-census-graph/flamegraph.svg` and
`/tmp/attrition-census-after-graph/flamegraph.svg`.

The next useful experiments are ordered by this evidence:

1. Extend file-backed indexing to the shutdown tail. The proposed reader would
   retain the last frame label and read only the following GAME children into
   the existing closing-state parser. Validate complete records and all census
   classifications, including nested and FULL DUMP cases, before migrating it.
   This is a design proposal, not a claim that the current index supports it.
2. Give the replay harness a frame iterator and apply its limit before decoding.
   Keep setup ownership explicit. This removes work without changing arithmetic.
3. Measure a content-addressed local typed-frame cache for repeated suite runs.
   Bind it to source bytes and parser/schema identity, compare every decoded
   field on cache construction, and fail on stale metadata. No derived game
   artifact belongs in git. The current offset cache is not this typed cache.
4. Only then benchmark parser kernels or field lookup changes. Line scanning
   and repeated linear field lookup are measured costs, but removing unnecessary
   passes has already beaten a speculative dependency or allocator replacement.

The first two own lifetime and query scope; the third amortizes parsing across
processes. None requires changing deterministic simulation rules or adding a
renderer/runtime dependency to the simulation.

## Landing checks

The full workspace release gate passed: rondata 253 passed/one ignored,
sim 821 passed, plus the fixed crate and doctests. The sampled process-tree
peak was 11,548 MiB (11,516 MiB in the largest process), so this tranche does
not claim a whole-suite memory reduction. Clippy with warnings denied,
formatting and all seven paperwork guards passed. The profiler was exercised
on the optimized census, and the graph's sample total matches the raw thread
count. Negative checks rejected a nonexistent test before launching it and a
nonexistent thread before producing a graph. Raw gate output is
`/tmp/attrition-profile-final-gate.log`.
