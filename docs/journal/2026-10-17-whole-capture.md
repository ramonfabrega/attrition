# Bounded whole-capture consumers

Sequential continuation from `01e766ae` on `codex/astra-windowed-reader`.
Commander remains paused; independent review remains owed.
Evidence: `/tmp/attrition-whole-capture-20261017-01/`.
[Audit](../audit/2026-10-17-whole-capture.json).

## Decision and preserved checks

The previous warm suite ranks city goods/trade, run56 figures and two animal
order tests at approximately2GiB OS RSS each. Their capture text, parsed trees
and decoded tables overlap in lifetime. Replace those retained observations with
bounded setup and indexed frames through the existing parsers. No simulation,
expected value, comparison, tick, floor, fixture or test name is removed.

City builds from bounded initial/sibling records, then reads each requested goods
frame and streams all trade frames. The figure test reads body rows as its clock
advances, preserving the frame-minus-one convention, first nonempty duplicate
match and last nonempty positional frame (not maximum label). Animal tests retain
only adjacent frames from one file at a time: duplicates, gaps, nonmonotonic labels
and empty frames are positions, and files are never joined. Small diagnostic
step vectors and simulation observations remain. No new persistent codec/schema
or cached verdict is introduced. Metadata lookup in the body reader is still
linear per tick; measurements support retaining that simple implementation.

`RON_DISABLE_WHOLE_CAPTURE` retains eager readers. Its scoped setup drops earlier
than the original; therefore the saved original binary, not this fallback, is the
memory baseline. `RON_VERIFY_WHOLE_CAPTURE` compares retained Initial fields,
every complete decoded Frame/body table, and all consumed raw goods fields with
whole-file parsing. Verification intentionally holds whole captures and uses more
memory. Existing sibling setup audits and original comparison assertions remain.

## Differential and fail-first evidence

`measure.py LABEL BINARY` runs the four exact tests sequentially with fixture
auditing. Final and audit-final diagnostic bodies and exact request multisets
match baseline for all four tests. The city retains21312 goods comparisons and
1785 trade comparisons; figure checks retain their million-row floor and exact
gaia/quit counts; animal tallies remain17/16 and264/39/39.

Two new guards compare complete results against whole parsing on authored CRLF,
empty/single-frame, duplicate/gap/nonmonotonic, outside-root and body-clock cases.
`fault.py` deliberately drops duplicate-label pairs and removes the body clock
offset. Both exact tests exit101; exact source restoration makes both exit0.
The first authored body fixture omitted required flags and failed; adding the
flags corrected the fixture without changing assertions. Failed logs are kept.
Focused guards, ledger and all22 floors pass. Ledger stays25 records/328 fields,
9 uncompared and31 single-capture. Score/pins, draw-stream and value diffs are zero.

## Local resource measurements

OS high-water RSS from `/usr/bin/time -l`; each is a local measurement, not a
confidence interval. The sampling tool can miss short allocation peaks.

| Test | Before wall/CPU s | After wall/CPU s | Before/after RSS MiB |
|---|---:|---:|---:|
| City goods/trade |3.87/3.02|1.70/1.61|2131/398|
| Run56 figures |3.06/3.02|2.78/2.76|2053/206|
| Blocked animal |2.52/2.50|2.19/2.19|2031/22|
| Hurrying animal |2.38/2.34|2.15/2.16|2030/22|

`pressure.py pressure-before baseline-rondata` and `pressure-after final-rondata`
run four competing copies of this four-test subset, each with four isolated slots
and normal shared admission. All16 tests pass in each experiment. Wall7.54→5.16s,
CPU50.26→41.64s, sampled aggregate RSS28853→1265MiB (96% lower). Individual OS
peak2133→401MiB corroborates the reduction, but aggregate peaks are sampled and
scheduling dependent. This is sixteen selected tests, not four complete suites
or proof of20-agent capacity. An initial driver named concurrent.py shadowed a
Python standard library module and failed before tests; its logs are excluded.

## Implementation checkpoint

All-target Clippy and focused guards pass. Complete required-fixture gate and
complete warm-suite measurement remain pending on the committed implementation.
Whole-suite10×, sub-minute/sub-GiB and concurrent-agent capacity remain unproven.
Independent review of reader equivalence remains owed.

## Complete verdict on `d68ee904`

All six required-fixture gate phases pass, exit0, on the frozen clean tree.
502 offline tests pass (three unchanged skips),899 rondata pass (six unchanged
ignores),2586 sim,13 fixed and3 doctests pass; Clippy/fmt/survey/guard pass.
`gate.log`, `gate.exit`, `gate/`, `warm-suite.log`, `warm-suite/` and
`gate-equivalence.json` preserve the receipts. Reproduce with:

```
python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures --report-dir <new-output>
python3 /tmp/attrition-whole-capture-20261017-01/equivalence.py
```

The complete warm run also passes899 tests. All original names/ignores remain,
exactly two boundary guards are added, and both complete runs preserve the exact
3276 fixture-request multiset (419 unique,zero missing). The gate, restored and
saved audited executables have identical SHA256. Four pressure replicas also
preserve their exact fixture multisets. Score/pins, draw and value floors: no
movement. No simulation behavior changed.

| Complete measurement | Previous landing | This landing |
|---|---:|---:|
| Full gate wall / CPU s |258.16 /1403.69|232.18 /1197.47|
| Full gate sampled tree MiB |8201|7425|
| Complete warm rondata wall / CPU s |134.05 /934.29|132.21 /924.28|
| Complete warm sampled tree MiB |8541|7184|

Warm aggregate peak falls15.9%; warm wall/CPU fall1.4%/1.1%, respectively.
These are unpaired local measurements with scheduling variability. Previous gate
included schema repopulation, so its larger time/CPU difference is not attributable
solely to this change. The focused competing-subset result isolates the larger
memory improvement, while the full suite still has other large consumers.
Whole-suite10×, sub-minute/sub-GiB and20-agent capacity remain unmet.

## Next measured target and review debt

New warm OS RSS ranking: indexed road setup equivalence1942MiB, run12/run13 world
cell comparison1923MiB, harness run14 trace1896MiB, run33 long trace1889MiB,
run13 fog1807MiB and run69 positions1798MiB. Next investigate bounded world and
harness observations; the setup equivalence test intentionally holds the original
whole parser, so preserve its independent oracle even if lifetimes can shorten.
Harness remains the largest CPU family at199.28s; no new CPU speedup is inferred.
Independent review of observation boundaries and methodological changes remains
owed. The paused commander was not resumed and this branch was not merged.
