# Frame-at-a-time capture reading

The next memory experiment attacks two different retained allocations: the
entire capture String and the Vec of every decoded Frame. Safe file I/O can
avoid both without changing the gamelog parser's borrowed data model.

`capture::indexed::IndexedCapture` scans once for direct GAME/FRAME ranges,
then reads a requested frame into an owned String with a synthetic GAME
wrapper. Existing `Log` accessors parse that String unchanged. Its
`frame_states` iterator yields one owned Frame at a time; dropping the frame
and scratch parse before advancing bounds active storage by the frame size.

The index cache shares only offsets, not file contents or decoded objects.
It retains at most 32 entries and 8 MiB of frame-range/path payload per test
process (plus small container overhead). Length and modification time
invalidate ordinary changes; reads use the same open file handle and check
metadata before and after. This is safe I/O, not an atomic snapshot guarantee.
Use finalized captures. There is no unsafe code or new dependency.

## Separate-process measurements

RSS is `/usr/bin/time -l`'s maximum resident set size on this Mac, in bytes.
Compilation was outside the timings. MB below is decimal, not MiB.

| workload | existing reader peak | indexed reader peak | existing time | indexed time |
| --- | ---: | ---: | ---: | ---: |
| run13, 642 MB source, 10 frames | 713.5 MB | 106.3 MB | 1.107 s | 1.076 s |
| run44, 352 MB source, 701 frames | 636.0 MB | 14.4 MB | 1.501 s | 2.010 s cold |
| run44 with index already cached | same baseline | 14.4 MB | 1.501 s | 1.497 s warm |
| actual frozen-clock test, five captures | 1,313.4 MB | 90.7 MB | 3.84 s test body | 4.07 s test body |

The warm run excludes its one-time index preparation from the reported scan
interval; total process wall time includes that preparation. Cold indexing
is not free. This is primarily a memory win, not a claim that every first
scan gets faster. The real test lost roughly 93% of peak RSS with unchanged
assertions, 26 frozen figure-frames and 89,522 advancing figure-frames.

The verification example compares complete parsed Frame values, including
all nested unit/order/figure/path records, buildings, leaders and cities.
It passed across runs 13, 20, 25, 27 and 44: 726 frames and 89,811 unit records.
This is equivalence to the existing reader, not a new fidelity proof against
the original engine. Only one production test was migrated in this tranche.
Whole-suite memory savings must not be inferred from its isolated savings.

## Boundary and tests

The API covers conventional direct FRAME children under root GAME, with
one-space frame indentation. It excludes setup, sibling FULL DUMPs and the
special closing-state reader. It preserves duplicate frame numbers by indexing
by position, not by number. Unusual nested-frame/trailing-field shapes fail
explicitly; consumers that need those shapes keep the original reader.

Synthetic tests cover complete-record equality, CRLF input, duplicate labels,
shutdown siblings, EOF, file truncation, invalid UTF-8, missing GAME, ambiguous
trailing fields, cache reuse and invalidation. The equality test's fixture
first failed because its synthetic units omitted the parser-required flags;
adding valid flags made both readers produce the records being compared.

`tools/memcap.sh` also now checks process-RSS access before launching the
command. The earlier exploration showed its denied `ps` could be swallowed
into a fictitious zero-MiB peak. A fake failing `ps` confirms exit 125 and
that the child never starts. A second injected failure after successful
preflight confirms the live command is stopped and the guard exits 125.
`python3 tools/explore/test_memcap.py` retains three failure checks. The guard
is still sampled, not an OS-enforced hard allocation limit.

## Reproduce

```sh
cargo build -p rondata --release --example frame_memory
/usr/bin/time -l target/release/examples/frame_memory whole CAPTURE
/usr/bin/time -l target/release/examples/frame_memory indexed CAPTURE
/usr/bin/time -l target/release/examples/frame_memory indexed-warm CAPTURE
target/release/examples/frame_memory verify CAPTURE
```

Raw measurements are `/tmp/frame-*.txt`, `/tmp/frame44-*.txt`, and
`/tmp/clock-{before,after}{,-time}.txt`. No capture content is committed.


## Next migration boundary

`diff::harness::run_traced` currently obtains the complete `frame_states` Vec
before applying its frame limit. Even a short-prefix comparison therefore
materializes every frame. The next structural change is an iterator of owned
frames consumed alongside the simulation, with an independently loaded initial
state. That would both stop parsing at the requested prefix and release each
oracle frame after comparison. The current `Initial` borrows log text, so its
setup lifetime needs an explicit design; changing `capture::read` underneath
all existing callers cannot provide this guarantee.

Validation so far: full workspace release suite passed (rondata 253 passed,
one ignored; sim 821 passed), structural install survey exited zero, repository
guards passed. The ordinary full gate's sampled tree peak was 10,783 MiB
(largest process 10,751 MiB), with rondata test body 285.50 seconds. This is the
post-migration baseline for the separate-process experiment, not a before/after
suite comparison. Raw full-gate output: `/tmp/attrition-streaming-gate.log`.

## Process isolation experiment

A scratch stdlib Python runner enumerates the compiled rondata libtest binary
with `--list --format terse`, subtracts the separately listed ignored tests,
and launches each remaining name with `--exact NAME --nocapture` in a fresh
process. A two-worker executor preserves the concurrency cap; `RON_INSTALL`
points at the real install and `RUST_TEST_THREADS=2` preserves the floor guard.
The entire tree runs under the same 20 GiB memcap. Compilation is excluded.
Per-test output and a name/exit/time manifest live under
`/tmp/attrition-isolated-gate`; the runner is `/tmp/attrition-isolated-tests.py`.
This probes the execution model; it does not replace the workspace gate or
run doctests. Check that each output reports exactly one passed test, rather
than trusting a zero exit from a mistyped filter.

For a maintained runner, [nextest's process-per-test model](https://nexte.st/docs/design/how-it-works/)
implements this separation, and its [heavy-test weights](https://nexte.st/docs/configuration/threads-required/)
can limit concurrent expensive tests. Nextest itself was not installed or
benchmarked here; the experiment cannot establish its end-to-end timing.


The isolated run passed all 253 non-ignored tests; each log independently
reports exactly one pass. It took **287.31 s**, versus **285.50 s** for the
ordinary rondata run. Its sampled tree peak was **9,591 MiB**, versus
**10,783 MiB** for the ordinary workspace gate; largest process was 5,997 MiB.
That is roughly **11% less peak tree RSS at similar elapsed time**, not the
large unlock hoped for. Runs were sequential, not a repeated randomized
benchmark; a brief structural survey/guard build overlapped part of the
isolated run. Sampling can miss short peaks, and the comparison's ordinary
peak includes Cargo/workspace processes. Do not infer fine-grained speed gains.

Isolation lowered the largest individual process substantially, but concurrent
active working sets still dominate the tree. The remaining corpus-wide shutdown
readers are concrete targets: `the_census_is_the_corpus` took 80.71 s and
`a_closing_dump_and_its_own_block_are_one_state` 47.81 s in the isolated run.
Both read entire capture Strings and collect frame states. Prioritize streaming
these and the replay harness over adopting process isolation as the main fix.
Formatting, clippy with warnings denied, structural survey, paperwork guards,
and the three injected sampling-failure tests also passed.
