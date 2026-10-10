# Reuse the parsed frame within each widening block

Item 1643, Codex (GPT-6), starting at `d3e871b8`. User-authorized follow-through
on the [earlier measured prototype](2026-10-09-core-loop-costs.md). One worker;
commander remains paused. This builds on the nine-window consolidation.

## Contract and change

The shared widening helper decoded each indexed frame through `frame_state`,
which reads the slice and eagerly parses its typed state. It then read the
same slice again into a lazy `Log` for group, leader and optional record
comparisons. The new default reads one slice, parses one eager tree, and
reuses it for both typed state and every auxiliary comparison.

`IndexedCapture::frame_from_log` centralizes the existing check that the
slice yields exactly one frame with the indexed frame number. Both the
existing reader and the reused-tree path call that validation. No change
to frame selection, duplicate-label handling, metadata checks on reads,
source offsets, comparisons, simulation, snapshots or assertions is intended.
This still requires finalized source files; it does not provide atomic
snapshots of concurrently edited captures. Whole-capture parsing stays lazy.

The new tree lives for one compared block. It is not cached across frames,
tests or parser versions. Parsed/comparison coverage must be recorded during
normal execution, without depending on a cached result or an audit's reads.

## Evidence plan

- Retain the prior executable and run fresh before/after processes with the
  same release profile (`CARGO_PROFILE_RELEASE_DEBUG=1`), raw fixtures, one
  test thread and a 20 GiB cap. No builds overlap measured replays.
- Compare value/summary diagnostics and all existing pins.
- `RON_VERIFY_FRAME_REUSE=1` reruns each requested widening through the old
  two-read path and compares the complete report and all checkpoint reports.
  The legacy path is only for this explicit migration audit.
- Run broad parsed-field and dumped-key coverage checks with the audit off,
  so legacy reads cannot conceal a regression in the reused path.
- Check empty, wrong-number and multiple-frame trees are rejected; compare
  reused typed records with a full parse on duplicate labels and siblings.
- Run the full required-fixture gate on the committed implementation, with
  the audit off, before recording a landing verdict.

Evidence is retained under `target/frame-reuse-1643/`; original captures and
the shared install remain untouched. No new capture, corpus conversion,
dependency or unsafe allowance is needed. Draw scores remain 3395 / 2323
and value floors are unchanged. Independent review debt remains.

## Results

Fresh processes in alternating order (before, after, before, after), using
identical release profiles and no concurrent builds:

| Path | Test seconds | Peak MiB |
|---|---|---|
| Prior two-read executable | 53.61, 52.45 | 1393, 1392 |
| Reused eager tree | 40.07, 39.24 | 1392, 1394 |

The two-run medians are **53.03 → 39.655 s**: **25.2% less time** for the
consolidated Great Lakes test. Memory is essentially unchanged. This is a
small repeated comparison on one workload, not a universal or whole-suite
speed claim. All nine original assertion groups passed; **276 diagnostic
rows match exactly** across all four processes, including comparison counts.

The initial log-analysis script missed one line attached to libtest's
unterminated test-name prefix in the second baseline. Stripping that prefix
preserves the full diagnostic line and establishes equality; no replay
failed or result row was discarded. Both the original analysis failure and
the corrected analysis are retained in `measure.log` and `analysis.log`.

All **21 capture tests** and all-target clippy passed. Tests cover invalid
frame cardinality/number, duplicate labels and siblings, metadata invalidation,
and the existing gzip/LZ4 archive integrity checks. Repository guards,
formatting and diff checks also passed.

The explicit legacy audit **passed, exit 0**: **40 complete widening reports**
match, including all nine Great Lakes checkpoints and the whole row-count
summaries (typed records, leader rows, pools, missing keys and compared blocks).
These cover 38 widenings from the broad comparison-coverage test, the shared
Great Lakes walk and run382's gaia-enabled/no-group-record window. The three
registered audit tests passed in 279.88 s with a 2378 MiB peak. This untimed
correctness run overlapped lint, guards and archive integration; do not use
its duration as a speed comparison.

run721 additionally passed through **raw, gzip and LZ4** inputs with the legacy
audit enabled in each: complete reports match, as do **5028 diagnostic rows**
across all three modes. Original files remain intact.

Normal-path coverage and the full committed-tree gate **passed** without
`RON_VERIFY_FRAME_REUSE`; the audited coverage test observes both paths and
is not used as proof of optimized-only comparison coverage. The full release
log contains no legacy-audit marker.

Artifacts: `rondata-before`, `harness.before.rs`, `build.log`,
`before-{0,1}.log`, `after-{0,1}.log`, `measure.py`, `analyze.py`, `timings.json`,
`reader-tests.log`, `clippy.log`, under `target/frame-reuse-1643/`.


Audit artifacts: `equivalence.log`, `check-audit.py`, `equivalence-summary.json`,
`archive-check.py`, `archive-check.json`, `archive-{raw,gzip,lz4}.log`,
`guard.log`. The audit-summary analyzer checks only the named shared-helper
summaries; the second-pair's separate closing-window helper is unchanged.

Reproduce the complete audit after building the release test binary:

```sh
RON_INSTALL=/Users/rf-studio/code/fun/attrition/game RON_VERIFY_FRAME_REUSE=1 \
  tools/memcap.sh 20 cargo test --release -p rondata -- --test-threads 1 --exact \
  diff::harness::tests::great_lakes_run163_through_run226_are_widened_whole \
  diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned \
  diff::third::tests::run382_s_word_frame_is_widened_whole --nocapture
```

The normal path has no feature flag: omit `RON_VERIFY_FRAME_REUSE`. The old
prototype patch remains a historical experiment and is not applied wholesale.


## Full release verdict and handoff

Implementation **`46a31162`** was committed before the full required-fixture
gate and the tracked tree stayed frozen. Gate **exit 0**, all six steps ran:
offline (277), clippy, formatting, survey, release and guard. Release fixed
13; rondata **778 passed**, five existing ignored; sim **1476 passed**; three
doc tests. The added test covers malformed frame cardinality/number; no
assertion was removed or re-pinned. Both broad coverage checks passed with
only the reused path executing.

Rondata elapsed **552.05 s**, versus **566.31 s** at the preceding landing:
**14.26 s / 2.5% shorter** in these two four-thread runs. This is an observed
pair, not a controlled repeated whole-suite benchmark. The focused 25.2%
saving does not generalize to the whole suite. Sim elapsed 2.23 s. Peak tree
RSS **12666 MiB**, largest process **12634 MiB**, below 20 GiB; prior gate
12885/12852 MiB. Memory remains similar; no broad memory improvement claimed.

Required-fixture audit: **2893 requests, 386 unique fixtures, zero missing**.
The request count and fixture-name set exactly match the preceding gate.
This is observed-request coverage, not a claim that every corpus file was
exercised. Evidence: `release-gate.log`, `gate-report/`,
`fixture-set-comparison.json` under `target/frame-reuse-1643/`.

No captures, archive defaults or shared configuration changed; about 72 GiB
remains free. Independent review remains owed. The investigation branch is
not merged into the commander's checkout, and the commander stays paused.

A read-only follow-up inventory found similar double reads in the golden
scenario readers (including chapters one and four) and separate East Indies
widenings. They are candidates for measurement, not included in this landing;
some nearby read pairs are different frames or sibling sources and cannot be
collapsed mechanically. Measure the next candidate before extending reuse.
