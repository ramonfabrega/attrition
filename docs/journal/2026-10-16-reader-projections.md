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
in memory (283,377,664 vs256,049,152 bytes): do not extrapolate uniform wins.
The first unprivileged `/usr/bin/time -l` attempt could not read the macOS
clockrate counter; retain its failed log, exclude it from measurements, and
use the authorized elevated run. No input files were changed.

A ten-second CPU stack sample of Golden chapter36 (test exit0) finds repeated
raw text scans alongside setup parsing: 199 of831 sampled widening stacks
in raw_near,104 in ammo::blocks,277 in setup. This is a partial-run sample,
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
thread. All four exit0. RON_DISABLE_GOLDEN_READERS selects the legacy readers.
Logs `raw-readers-ch36-{off,on,off2,on2}.log`:

| Metric | Legacy 1 / 2 | Optimized 1 / 2 |
| --- | --- | --- |
| Test elapsed seconds | 14.30 / 14.26 | 10.22 / 10.21 |
| User+system seconds | 14.23 / 14.25 | 10.21 / 10.20 |
| Retired instructions | 299.710 / 299.455 billion | 236.299 / 236.249 billion |
| Peak resident bytes | 1,170,063,360 / 1,171,685,376 | 1,179,811,840 / 1,169,342,464 |

About1.40× elapsed/CPU,21% fewer instructions; no memory improvement. The first
legacy wrapper had0.50s outside the reported test body; use the Rust test timer
for the table rather than claiming that scheduling delay as optimization.
No suite-wide extrapolation from this one test.

The serial profile tables are also committed under `docs/audit/2026-10-16-suite-`
`{timings,memory,cpu}.txt`; they describe b44c948a before this batch.
The first differential audit failed because its fixture-audit output directory
had not been created; the writer correctly refused to discard evidence and then
poisoned its mutex. Preserve `raw-readers-audit.log` (exit101;14pass/251fail).
Created the missing output directory and reran into `raw-readers-audit2.log`;
no reader or test expectation was changed in response.

## Second part of the bounded batch: unused setup audit bodies

Migrated109 test setup-only calls: world51, build33, city15, order5, army4, leader1.
New testkit helper uses existing Log::replay_initial; full-reader unit audit
and every reference-equality test remain unchanged. No call in these six
modules reads frame_bodies; the actual figure-position audit is in unit.rs.
Source text lifetimes are intentionally unchanged in this step. This is a
smaller migration than introducing persistence or changing capture ownership.

RON_VERIFY_REPLAY_SETUP compares every retained optional Initial field with
Log::initial after clearing only baseline frame_bodies. RON_DISABLE_REPLAY_SETUP
selects the old full setup for same-binary measurements. This batch defers the
full gate until raw-reader and setup changes are both validated and committed.

Raw-reader differential audit:265 passed,0 failed,182.22s at four threads;
1,088 fixture requests,zero missing; sampled tree5,703MiB; actual exit0.
This audit used the reader-only binary before the setup migration. Complete
near maps and ordered ammo records/widths were compared wherever called.

The setup migration's first compile caught one call outside the test module
(order::build_for_check). Restored that production call to Log::initial;
final scope109 test setup calls. Build failure log retained; production API
behavior is unchanged. No assertion was removed or weakened.

## Setup paired measurements

Same final candidate binary, each case legacy/optimized/legacy/optimized in
separate processes, one test thread, warm filesystem, explicit install. All
12 exits0. Logs `setup-migration-{run56,goods,wander}-{off,on,off2,on2}.log`.
Elapsed below is the Rust test timer; CPU is user+system; memory is peak bytes.

| Test | Legacy elapsed / optimized | Legacy CPU / optimized | Legacy RSS / optimized |
| --- | --- | --- | --- |
| run56 new camp | 2.59–2.64 /1.62 s | 2.58–2.64 /1.61 s | 2.148–2.152 /1.115–1.117 GB |
| world far wander | 2.96 /2.35–2.51 s | 2.96 /2.34–2.48 s | 2.301–2.302 /1.669–1.673 GB |
| city goods | 2.58–2.59 /2.58–2.69 s | 2.57–2.58 /2.59–2.67 s | 2.250–2.298 /2.227–2.228 GB |

Run56 instructions65.5–65.7→41.5billion (~37% less), memory~48% lower.
Wander73.4→58.6billion (~20% less), memory~27% lower. City goods has no
meaningful time win and only a small RSS movement; further work must address
its remaining whole-text/parsed-record retention rather than overclaim this
migration. No suite-wide memory conclusion before the full gate.

Setup differential audit:112 passed,0 failed,2 ignored;48.18s at four threads;
sampled tree7,440MiB; actual exit0. Every migrated setup executes the full
Initial equality assertion. Fixture requests 395, zero missing.

Development checks: clippy --all-targets -D warnings, cargo fmt --check,
tools/guard.sh --built and git diff --check all exit0. Clippy first caught a
needless extra borrow at one army helper parameter; removed it without changing
behavior, then reran successfully. Full required-fixture gate pending on the
implementation commit. All scores, words, value diffs and pins remain unchanged.

For the Loop: paused; complete serial baseline preserved, raw projections
and109 setup-only migrations differentially checked; local CPU/RSS improvements
above, complete-suite verdict pending. Independent review remains owed.
