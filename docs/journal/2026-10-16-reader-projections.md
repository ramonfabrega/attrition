# Reader projections after the complete serial profile

## Baseline and scope

User-authorized 10× suite target; sequential work, paused loop unchanged.
Baseline tree b44c948a. All 874 nonignored rondata tests ran individually in
release, one process per test, under a 20 GiB process-tree ceiling with the
explicit ch80/map-14 install. Actual exit 0, 3,276 fixture requests, zero missing.
Sum elapsed 1,502.28 s; actual user+system CPU 1,480.96 s. The original 2,762 s
serial elapsed baseline is historical, not a controlled same-tree comparison:
about 1.84× observed reduction, not 10×. Original reports called elapsed CPU;
this profile retains both separately. Individual peak RSS 2,966 MiB; sampled
process-tree peak 2,927 MiB (sampling can miss a short maximum).

Golden accounts for 548.75 CPU s (37%); harness 245.35, second 118.78,
third-pair 109.72, coverage 76.29. Evidence root:
`/tmp/attrition-replay-batch-20261016-01/`, including `current-profile/` JSONL,
CPU summary, fixture verdict, and `2026-10-16-current-suite-timings.txt` and
`2026-10-16-current-suite-memory.txt`. Reproduction wrapper
`profile-current-suite.py` retains actual time counters and failure logs;
its output directory is create-new and must not be reused.

Existing-reader setup experiments (`setup-stage.rs`, `setup-*-measured.log`)
compare the complete Initial, clearing only baseline frame_bodies. All four
sources agree: run53, run56, run7, run38. Setup-only run56 whole/indexed:
1.98/0.88 s and 1,849,835,520/41,746,432 peak resident bytes. Run7:
0.61/0.33 s and 546,095,104/9,060,352 bytes. Run38 indexed is slightly worse
in memory (283,377,664 vs 256,049,152 bytes): do not extrapolate uniform wins.
The first unprivileged `/usr/bin/time -l` attempt could not read the macOS
clockrate counter; retain its failed log, exclude it from measurements, and
use the authorized elevated run. No input files were changed.

A ten-second CPU stack sample of Golden chapter36 (test exit 0) finds repeated
raw text scans alongside setup parsing: 199 of 831 sampled widening stacks
in raw_near, 104 in ammo::blocks, 277 in setup. This is a partial-run sample,
not a precise whole-test time decomposition. Evidence `golden-ch36.sample.txt`.

## Bounded implementation batch

- Golden raw_near retains four optional integers instead of a BTreeMap of
  every numeric field. Preserve first successfully parsed occurrence, nested
  lexical behavior, incomplete-record omission and last duplicate unit wins.
- Ammo reader skips complete prefix lines preceding the first possible literal
  BEGIN AMMO marker. No marker means no record; false marker substrings enter
  the unchanged state machine. Preserve indentation, every following line,
  nested spline handling, duplicate fields, widths and record ordering.
- Keep independent original readers for differential audit. Synthetic cases
  cover malformed values, duplicates, nested records, CRLF, whitespace and
  false markers. RON_VERIFY_GOLDEN_READERS compares complete near maps and
  complete ammo records with widths wherever these readers execute.

No simulation, score, value, word, comparison coverage or pin change intended.
This batch is pending validation and measurement. Independent review remains
owed; this is executable differential evidence, not an independent review.

For the Loop: paused; new serial profile complete; reader projection experiment
pending validation. No claim of a suite-wide gain yet.

## Paired experiment

Same candidate binary, chapter36 whole widening, separate processes in order
legacy/optimized/legacy/optimized; explicit install, warm filesystem, one test
thread. All four exit 0. RON_DISABLE_GOLDEN_READERS selects the legacy readers.
Logs `raw-readers-ch36-{off,on,off2,on 2}.log`:

| Metric | Legacy 1 / 2 | Optimized 1 / 2 |
| --- | --- | --- |
| Test elapsed seconds | 14.30 / 14.26 | 10.22 / 10.21 |
| User+system seconds | 14.23 / 14.25 | 10.21 / 10.20 |
| Retired instructions | 299.710 / 299.455 billion | 236.299 / 236.249 billion |
| Peak resident bytes | 1,170,063,360 / 1,171,685,376 | 1,179,811,840 / 1,169,342,464 |

About 1.40× elapsed/CPU, 21% fewer instructions; no memory improvement. The first
legacy wrapper had 0.50 s outside the reported test body; use the Rust test timer
for the table rather than claiming that scheduling delay as optimization.
No suite-wide extrapolation from this one test.

The serial profile tables are also committed under `docs/audit/2026-10-16-suite-`
`{timings,memory,cpu}.txt`; they describe b44c948a before this batch.
The first differential audit failed because its fixture-audit output directory
had not been created; the writer correctly refused to discard evidence and then
poisoned its mutex. Preserve `raw-readers-audit.log` (exit 101; 14pass/251fail).
Created the missing output directory and reran into `raw-readers-audit2.log`;
no reader or test expectation was changed in response.

## Second part of the bounded batch: unused setup audit bodies

Migrated 109 test setup-only calls: world 51, build 33, city 15, order 5, army 4, leader 1.
New testkit helper uses existing Log::replay_initial; full-reader unit audit
and every reference-equality test remain unchanged. No call in these six
modules reads frame_bodies; the actual figure-position audit is in unit.rs.
Source text lifetimes are intentionally unchanged in this step. This is a
smaller migration than introducing persistence or changing capture ownership.

RON_VERIFY_REPLAY_SETUP compares every retained optional Initial field with
Log::initial after clearing only baseline frame_bodies. RON_DISABLE_REPLAY_SETUP
selects the old full setup for same-binary measurements. This batch defers the
full gate until raw-reader and setup changes are both validated and committed.

Raw-reader differential audit: 265 passed, 0 failed, 182.22 s at four threads;
1,088 fixture requests,zero missing; sampled tree 5,703 MiB; actual exit 0.
This audit used the reader-only binary before the setup migration. Complete
near maps and ordered ammo records/widths were compared wherever called.

The setup migration's first compile caught one call outside the test module
(order::build_for_check). Restored that production call to Log::initial;
final scope109 test setup calls. Build failure log retained; production API
behavior is unchanged. No assertion was removed or weakened.

## Setup paired measurements

Same final candidate binary, each case legacy/optimized/legacy/optimized in
separate processes, one test thread, warm filesystem, explicit install. All
12 exits0. Logs `setup-migration-{run56,goods,wander}-{off,on,off2,on 2}.log`.
Elapsed below is the Rust test timer; CPU is user+system; memory is peak bytes.

| Test | Legacy elapsed / optimized | Legacy CPU / optimized | Legacy RSS / optimized |
| --- | --- | --- | --- |
| run56 new camp | 2.59–2.64 /1.62 s | 2.58–2.64 /1.61 s | 2.148–2.152 /1.115–1.117 GB |
| world far wander | 2.96 /2.35–2.51 s | 2.96 /2.34–2.48 s | 2.301–2.302 /1.669–1.673 GB |
| city goods | 2.58–2.59 /2.58–2.69 s | 2.57–2.58 /2.59–2.67 s | 2.250–2.298 /2.227–2.228 GB |

Run56 instructions 65.5–65.7→41.5 billion (~37% less), memory~48% lower.
Wander73.4→58.6 billion (~20% less), memory~27% lower. City goods has no
meaningful time win and only a small RSS movement; further work must address
its remaining whole-text/parsed-record retention rather than overclaim this
migration. No suite-wide memory conclusion before the full gate.

Setup differential audit: 112 passed, 0 failed, 2 ignored; 48.18 s at four threads;
sampled tree 7,440 MiB; actual exit 0. Every migrated setup executes the full
Initial equality assertion. Fixture requests 395, zero missing.

Development checks: clippy --all-targets -D warnings, cargo fmt --check,
tools/guard.sh --built and git diff --check all exit 0. Clippy first caught a
needless extra borrow at one army helper parameter; removed it without changing
behavior, then reran successfully. Full required-fixture gate pending on the
implementation commit. All scores, words, value diffs and pins remain unchanged.

For the Loop: paused; complete serial baseline preserved, raw projections
and 109 setup-only migrations differentially checked; local CPU/RSS improvements
above, complete-suite verdict pending. Independent review remains owed.

## Required full gate — implementation 2d61d180

Frozen committed tree; command python3 tools/release_gate.py
/Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures, report-dir
/tmp/attrition-replay-batch-20261016-01/projections-gate-report.
Logs projections-release-gate.log/.time. Actual exit 0; all six steps ran.
Fixed 13 passed; rondata 876 passed, 0 failed, 6 ignored; sim 2585 passed;
rondata doctests 3 passed. Fixture audit3276 requests, 419distinct, 0 missing.
All existing fidelity assertions, words, values and pins unchanged.

Rondata375.88 s vs preceding383.16 s: 1.9% observed improvement, not a replicated
speedup estimate. Sim40.50 s. Sampled process-tree peak 10562 MiB /largest 10530 MiB,
versus previous 10025/9992: no whole-suite memory win. Wrapper 482.19 real,
1518.20 user, 145.39 system; previous 492.28 real, 1568.08 user, 146.46 system.
Total wrapper CPU 1714.54→1663.59 s (3.0% lower), elapsed 2.1% lower.
These wrapper totals include every gate step, unlike the rondata phase time.
Hardware counters from the wrapper are not whole-suite instruction counts.

Conclusion: keep the differentially verified shared-reader and setup changes,
but their local gains do not establish10x or suite-memory improvement. The
complete serial baseline remains1502 s on b44c948a; no new serial total has been
measured for 2d61d180. Original2762 s baseline is historical and elapsed-only.

Next measurement: isolate each of the 876 nonignored rondata tests in its own
process; at most4active, measured RSS estimates capped at 6 GiB and an external
8 GiB enforced cap. Compare to the same committed tree's375.88 s ordinary phase.
This tests process-lifetime retention and scheduling, with all existing test
assertions and fixture audits. It is an experiment, not yet a default gate
replacement; full fixed/sim/doc/offline/lint/guard coverage remains necessary.

## Isolated-process experiment: four slots

Unchanged2d61d180 binary; no builds, benchmarks or tracked edits concurrently.
`zsh tools/memcap.sh 8 python3
/tmp/attrition-replay-batch-20261016-01/isolated-experiment.py`.
Actual exit 0. All 876 nonignored tests listed from the binary executed exactly
once, each with --exact and --test-threads 1; each log asserts 1 passed rather than accepting a zero-test
exit. Six ignored tests match the ordinary gate. The manifest, per-test logs,
JSONL counters and final summary are retained under isolated-experiment/.

Four slots, longest predicted tests first when they fit a 6 GiB estimated-RSS
budget, with a 128 MiB floor for small/new tests. External hard ceiling8 GiB.
The estimates are scheduling hints, not guarantees; the sampled cap is the
backstop. Future unknown tests need a conservative scheduling policy before
this becomes a reusable gate tool. Current new tests are the two small
synthetic reader checks. Actual reported counters are required, not zero-filled
on tool failure. All 3276fixture requests/419 sources match the normal run;
zero missing. This checks requested fixtures, not an unrequested-corpus claim.

Elapsed 392.17 s vs ordinary rondata 375.88 s (+4.3%). Sampled tree 5659 MiB vs
10562 MiB (-46.4%). Individual max reported peak 3527442432 bytes; sampled
largest 3273 MiB. Summed user+system 1543.76 s. Ordinary gate does not isolate
rondata CPU from other gate steps, so do not compare that with whole-gate CPU
or claim CPU savings from isolation. No simulation or assertion changes.

For the 874 common tests, retired instructions from individual processes fell
34.6353→33.4108 trillion (3.54%) between b44c948a serial profile and 2d61d180
four-slot run; none missing. Golden accounts for 0.804 T of 1.2245 T reduction.
This supports reduced instruction work from the reader batch, while concurrency
and allocator/kernel behavior differ. Evidence instruction-comparison.json.
The CPU result remains modest; it is not 10x.

## Isolated-process experiment: eight slots

Same committed binary and 6 GiB scheduling budget, external 8 GiB cap; host reports
16 physical/logical cores and 128 GiB RAM. Experiments ran sequentially.
`zsh tools/memcap.sh 8 python3
/tmp/attrition-replay-batch-20261016-01/isolated-experiment-8.py`.
Actual exit 0: 876 passed, 6 ignored; 3276 requests/419 sources, 0 missing.
Compared the complete test/ignored manifests and the multiset of every fixture
request row against the four-slot experiment and ordinary gate: exactly equal.
All comparisons and pins still execute; no result cache or test selection.

Elapsed 236.87 s, summed user+system 1553.73 s, sampled tree 6211 MiB /largest 3967 MiB.
Individual peak 4159799296 bytes. Versus ordinary rondata 375.88 s, elapsed 37.0%
lower (1.59×); sampled peak versus full release 10562 MiB is 41.2% lower. Versus
four-slot isolation 392.17 s, 39.6% lower elapsed and 0.65% higher CPU seconds.
Instructions 33.2598 T vs four-slot 33.4109 T (0.45% difference). Treat CPU work as
essentially unchanged by concurrency; this is a scheduling/memory-lifetime win.
These are single complete runs, not repeated estimates with confidence bounds.

Do not compare the historical 2762 s serial aggregate to 236.87 s parallel elapsed
and call it 10×. The target remains unmet. The isolated experiment covers the
rondata library suite; it does not replace fixed/sim/doc/offline/lint/survey/
format/guard checks. The ordinary required-fixture gate remains the default.
The JSON summary is committed at docs/audit/2026-10-16-isolated-runner.json.
Per-test logs, counters, manifest, script and actual cap log remain in the
cited evidence root. No corpus files were removed or relocated.

## Handoff and next item

Implementation 2d61d180 has a green complete required-fixture gate; all scores,
word and value diffs unchanged. This follow-up records measured results only.
No merge into the commander branch; paused loop remains paused. Independent
review remains owed.

The next infrastructure candidate is a reusable process runner with tested
manifest completeness, failure/signal propagation, fixture errors, ignored and
filtered test behavior, conservative unknown-test estimates and resource limits.
Its benefit is measured above, but the prototype is not yet installed as the
gate default. Keep every other gate check when integrating it.

The next CPU candidate is persistent exact setup observations: the existing
indexed reader still scans all frames on a process-cache miss (cache 8 MiB,
32 entries; oversized sets never retained). Measure Golden ch35/ch36 and shared
siblings first; preserve complete Initial equality, source identity/versioning,
order/duplicates, raw-key instrumentation, Golden's refusal of borrowed frame
corrections, corrupt-cache fallback and create-new publication. See retained
next-observation-plan.md. Further memory work must bound whole source lifetimes,
including city goods, which this batch did not materially improve.

For the Loop: full gate exit 0 on 2d61d180; rondata 383.16→375.88 s, sampled tree
10025→10562 MiB; eight-slot isolated experiment 236.87 s/6211 MiB, 876 tests and exact
fixture-request multiset preserved. CPU instruction work down 3.5% on 874 common
separate-process tests. 10× not reached; no scores or pins moved; loop paused.
