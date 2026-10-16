# 2026-10-16 — Suite replay batch

User-authorized performance work on `codex/astra-windowed-reader`, starting at
`2524783f`; the commander remains paused. No subagents. This bounded batch covers
coverage replay-prefix reuse, scoped installation reuse and allocation reduction
in the compared-field recorder. Full required-fixture validation is deferred to
the batch boundary; no implementation checkpoint is yet a validated landing.

## Reassessment

The original brief requested persistent offset indexes followed by a parsed
per-lobby oracle store. Prior work reduced individual RSS substantially, but
never implemented the latter. The last serialized sweep was 2,012 seconds
versus a historical merged 2,762 seconds; those are summed elapsed times, not
CPU time (correction to the previous journal's terminology). The latest prior
full rondata run was 483.13 seconds at four threads, with 9,446 MiB process-tree
peak for the release command. It does not establish 10× or a controlled speedup
against the older eight-thread gate.

## Experiment and falsifiers

`widen_walk` rebuilt from frame zero for every coverage window. The driver has
63 windows. Compatible, non-overlapping windows now move a single complete Built
forward; unrelated or overlapping inputs fall back. French windows are ordered
by position; all window calls remain and the first closing-window block check
stays first. Installation tables are shared only inside the immutable-fixture
scope. Standalone widenings remain independent.

Falsifiers: any changed complete Widened result, checkpoint or per-walk compared
field set; a different source/metadata or overlapping window reusing state;
state surviving scope exit/unwind; any changed fixture coverage or pin. Audit
mode executes the fresh walk and checks full equality. Unit tests exercise
key mismatch, overlap, moved ownership, cleanup and the old recorder algorithm.
No simulation behavior, value pins, draw floors or queue scores are changed.

Evidence directory: `/tmp/attrition-replay-batch-20261016-01/` (retained).
Baseline command: `RON_INSTALL=/Users/rf-studio/ron-golden/ch80/map-14
/usr/bin/time -l target/release/deps/rondata-4337e333078eb556 --exact
diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned
--test-threads 1`, stdout `baseline.log`, stderr `baseline.time`.
It passed: 155.02 s elapsed, 141.60 s user, 11.97 s system,
2,479,079,424 B peak RSS, 3,658,911,022,148 retired instructions.
Two development compilations overlapped parts of this baseline; use the
same-binary disabled-reuse measurement for a cleaner prefix comparison.

The first differential run (`prefix-audit.log`, `prefix-audit.time`) uses
`RON_VERIFY_PREFIX_REUSE=1 RON_PROFILE_REPLAY=1` and `--nocapture` in addition
to the same command. It predates scoped table reuse and recorder changes;
its phase data guides those follow-ups. Final audit and gate remain pending.

## Review debt and next work

This is an executable differential experiment, not an independent review.
The 10× suite target remains open. After the full-gate measurement, prioritize
remaining total suite costs; do not revert to one full gate per small RSS fix.
Persistent parsed inputs and sharing compatible work across independent tests
remain candidates, requiring fresh profiles and provenance-safe invalidation.

## First result and measured follow-up

The initial prefix audit passed 62 generic widening walks (the first of the 63
coverage windows uses a separate harness). Complete results, checkpoints and
per-walk compared-field sets matched. It avoided 300,063 of 388,493 generic
widening ticks. Phase sums: independent walks 149.68 s (34.84 setup, 49.83 ticks),
reused walks 96.64 s (18.19 setup, 14.08 ticks). This audit predates table and
recorder reuse and includes concurrent development builds; it establishes
semantic equivalence and work counts, not a stable timing ratio.

With table and recorder reuse, `optimized1` passed in 80.04 s real, 77.08 s user,
2.22 s system, 1,587,085,312 B RSS, 1,769,520,161,640 retired instructions. A
15-second sample and paperwork checks ran during this measurement. The sample
(`optimized.sample.txt`) identifies repeated `Block::get`/`all`, key comparisons,
line splitting and key/value parsing as remaining reader costs. This motivates
one additional bounded experiment in the batch: a field-position index, capped
at eight records/32 KiB payload per Log, checked against the old linear scan.
No field or node arena widths change. Parser tests with lookup audit enabled:
27 passed, one measurement-only test ignored. Final whole-window audit and
unprofiled measurement follow before deciding whether to retain the index.

The combined audit passed all 62 reused/fresh reports and compared-field sets,
with every indexed field lookup compared to the original linear sequence.
However, the subsequent unprofiled `indexed-normal1` run took 91.55 s real,
89.08 s user, 1.88 s system, 1,793,703,936 B RSS and 2,183,289,254,612 retired
instructions: worse than the previous 80.04 s / 1.770T instructions. The field
index is rejected and removed. Its exact patch remains at
`/tmp/attrition-replay-batch-20261016-01/field-index-rejected.patch`.

Inspection of the next large cost found `Trace::frame_draws` scanning every
draw for every requested frame, called repeatedly by `labels`. The next bounded
experiment in this batch measures run53, then replaces this lookup only if
profiling supports it. Unsorted traces must retain the full-scan fallback and
all original order, sync filtering, label and trace assertions must remain.

## Long trace lookup

The run53 baseline passed in 47.13 s real, 46.01 s user, 0.46 s system,
1,460,584,448 B RSS, 1,089,808,402,548 retired instructions (`trace-baseline.*`).
It used `RON_DISABLE_FIELD_INDEX=1` on the experiment binary, so the rejected
field index was inactive. A 15-second sample (`trace-baseline.sample.txt`)
recorded 11,514 top-of-stack samples in the frame_draws vector filter against
11,590 in the waiting main thread; label lookup had 36. This interval is not a
whole-test percentage, but directly supports the quadratic-lookup hypothesis.

The optimized reader checks chronological ordering once after parse, makes the
draw vector private with a read-only slice accessor, and uses two binary bounds
for ordered traces. Unordered diagnostic inputs keep the original linear filter.
No extra index allocation. New differential tests cover synthetic ordering and
sync filtering, and every draw/frame group of three complete fixture traces.
Performance measurements and the final required gate remain pending.

The optimized run53 passed twice: 2.73 / 2.69 s real, 2.50 / 2.55 s user,
0.15 / 0.13 s system, 1,488,191,488 / 1,459,781,632 B RSS,
66,395,608,181 / 66,112,700,457 retired instructions. Against 47.13 s and
1,089,808,402,548 instructions, this is 17.3–17.5× elapsed and 16.4–16.5×
instruction reduction for this test; memory is essentially unchanged.
All existing 24,000-frame assertions passed. The 12 trace tests passed,
including complete partitions of run53 (788,283 draws / 23,987 frame groups),
run54 (883,918 / 24,002), and run600 (1,429,005 / 17,381), all chronological.
The smaller frame-group count on run53 counts frames with draw records, not
missing simulation frames; its existing 24,000-frame coverage checks remain.
Reproduction uses the baseline command without `RON_DISABLE_FIELD_INDEX`;
logs are `trace-optimized1.*`, `trace-optimized2.*`, and `trace-tests.log`.

Final ordinary coverage (`coverage-final.*`, no sampler or audit enabled) passed
in 73.55 s real, 71.75 s user, 1.69 s system, 1,802,633,216 B RSS and
1,768,553,816,123 instructions. Versus the initial baseline: 2.11× elapsed,
2.09× user+system CPU, 2.07× fewer instructions; observed RSS 27.3% lower.
The baseline overlapped development compilation; the instruction reduction and
300,063 avoided ticks support the CPU conclusion independently of elapsed noise.
RSS varied between optimized runs (1.59–1.80 GB); no stable suite-memory claim.
The rejected index is absent from this final candidate. Suite-level validation
and timing are still pending; these are test-level gains, not the 10× target.

Pre-landing validation: `cargo clippy --all-targets -- -D warnings`, formatting
check, both prefix unit tests, recorder differential test, `tools/guard.sh
--built`, and `git diff --check` all exited 0. Guard retains the pre-existing
queue-ledger note about item 7. The implementation is ready for its committed
full required-fixture gate; that verdict is not yet claimed.
