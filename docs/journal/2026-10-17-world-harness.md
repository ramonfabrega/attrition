# Bounded world and harness consumers

Sequential continuation from `60ace32a` on `codex/astra-windowed-reader`.
Commander remains paused; independent review remains owed.
Evidence: `/tmp/attrition-world-harness-20261017-01/`.
[Audit](../audit/2026-10-17-world-harness.json).

## Decision and preserved checks

The preceding complete warm suite ranks the world farm/fog tests and harness
run14/run33/run69 among the remaining~1.9GiB consumers. Save the exact previous
binary and measure these five tests before editing. All five retain whole setup
captures and sibling parse trees during simulation. Use bounded initial inputs
with the same ordered sibling sources and corrections; release setup after
building. The first candidate still retains1.1–1.5GiB in the world tests, so
stream those comparison captures through their existing farm/fog readers too.

The new observation collector parses each positional frame plus its following
GAME siblings. This preserves a FULL DUMP nested under a frame, an unconsumed
sibling dump after an empty frame, duplicate labels, gaps and original order.
Its callback must be frame-local (frames/dumps), not a global/setup reader.
Owned farm/fog observations retain the original sorting, filtering and clocks.
Every world cell comparison, tick, diagnostic and floor remains unchanged.

Run69 uses existing run_indexed_observed with the same trace and sibling inputs.
Its complete Report is retained, including all comparisons, missing/diverged rows,
notes and RNG results. The failure diagnostic loads original capture text only
when selected; it reuses loaded tables, resolved path, trace and sibling inputs.
write_failure requires that diagnostic replay reproduce the complete Report.
Failure artifact metadata now labels bounded sibling Initials accurately.
No simulation, cache codec/schema, verdict cache or test filter changes.

`RON_DISABLE_WHOLE_CAPTURE` keeps eager primary readers; scoped setup still drops
earlier and run69 siblings stay bounded, so the saved baseline binary is the
resource baseline. `RON_VERIFY_WHOLE_CAPTURE` checks every retained primary and
sibling Initial against whole-file parsing (excluding only unused body audit rows),
world observation vectors against original complete parsing, and run69's entire
Report against run_traced. Existing observations and reader guards remain active.

## Differential and fail-first evidence

`measure.py LABEL BINARY` runs five exact tests sequentially under one CPU/4096MiB
admission with fixture audits and OS high-water RSS. Both candidates and both
audit runs preserve exact normal diagnostics and fixture-request multisets against
the original executable. Farm/fog/trace/position assertions remain unchanged.

A new authored CRLF guard compares complete frames/dump observations with whole
parsing: nested-versus-sibling precedence, duplicate labels, empty frames,
nonmonotonic labels and orphan dumps. Dropping the final chunk deliberately fails
that guard (101). Truncating run69 to one indexed frame fails its complete-report
audit (101). Exact source restoration makes both pass0; restored executable SHA256
matches final-rondata. `fault.py` also forces run69's diagnostic branch without
changing the report: artifact generation passes its complete second-report equality
and all original assertions. Its unique retained HTML path is in forced-artifact.log.
No failure-path fixture rediscovery was introduced.

One formatting-wrapper invocation rejected unsupported fmt before formatting;
ordinary cargo fmt then succeeded. All development/build/fault logs are retained.
No failed command is counted as a passing check. Existing boundary guards pass.
Score/pins, draw-stream and value floors: no movement. Independent review owed.

## Local resource measurements

| Test | Before wall/CPU s | After wall/CPU s | Before/after RSS MiB |
|---|---:|---:|---:|
| Farm records |3.29/2.58|1.41/1.06|1924/407|
| Fog grids |2.16/2.17|0.86/0.85|1809/394|
| Run14 trace |2.13/2.14|0.55/0.51|1886/436|
| Run33 trace |2.10/2.06|0.60/0.55|1897/402|
| Run69 positions |3.22/3.20|1.96/1.92|1790/403|

These local measurements are not confidence intervals. Audit mode intentionally
retains original parses and costs more memory/CPU. Source-keyed setup observations
were warm; this does not establish cold-cache cost or cross-worktree reuse.

`pressure.py pressure-before baseline-rondata` and `pressure-after final-rondata`
run four competing copies of the five-test subset, each with four isolated slots
and normal shared resource admission. All20 tests pass each side. Wall7.94→4.46s,
CPU58.88→25.35s, sampled tree26769→1840MiB:44% faster,57% less CPU and93% less
sampled memory. Sampling/scheduling affect peaks. This is20 selected tests, not
four complete suites or proof of20-agent capacity.

## Implementation checkpoint

Complete required-fixture gate and complete warm-suite measurement pending on
the committed implementation. Preserve every original name and exact fixture
multiset. Independent review of observation boundaries remains owed; whole-suite
10×, sub-minute/sub-GiB and concurrent-agent capacity remain unproven.

## Complete verdict on `7deda699`

All six required-fixture gate phases pass, exit0, on the frozen clean tree:
502 offline tests (three unchanged skips),900 rondata (six unchanged ignores),
2586 sim,13 fixed and3 doctests. Clippy/fmt/survey/guard pass. A complete warm
run also passes900 tests. All899 original rondata tests and ignores remain;
exactly one boundary guard is added. Both complete candidate runs preserve the
exact3276 fixture requests (419 unique,zero missing). Pressure replicas each
preserve their exact fixture multisets. Saved audited, restored and gate binaries
have identical SHA256. Score/pins and draw/value floors do not move.

Receipts: gate.log, gate.exit, gate/, warm-suite.log, warm-suite/,
baseline-control.log, baseline-control/ and gate-equivalence.json. Reproduce:

```
python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures --report-dir <new-output>
python3 /tmp/attrition-world-harness-20261017-01/equivalence.py
```

| Complete measurement | Previous landing | This landing |
|---|---:|---:|
| Full gate wall / CPU s |232.18 /1197.47|230.33 /1191.14|
| Full gate sampled tree MiB |7425|8698|
| Complete warm rondata wall / CPU s |132.21 /924.28|132.86 /923.69|
| Complete warm sampled tree MiB |7184|7813|

The full warm suite is effectively flat on time/CPU; sampled peak rises8.8%.
Gate peak rises17.1%. These measurements do not establish a complete-suite
memory improvement, despite the five tests' large local and competing-subset
savings. Changed overlap between remaining large tests is a possible explanation,
not a measured attribution. Scheduling and sampling influence aggregate peaks.

A nearby control reruns the saved original binary after the candidate:899 pass,
exact original test/fixture inventories preserved, 133.53s/930.32CPU,
7384MiB sampled peak. Compare this with candidate132.86s/923.69CPU/7813MiB;
one nearby control is not a confidence interval or proof of capacity. Both the
previous result and this control are retained, rather than selecting the better
baseline. No whole-suite10×, sub-minute/sub-GiB or20-agent claim is made.

## Next measured targets and review debt

Warm OS RSS: whole-parser road setup oracle1930MiB (intentionally independent),
run89 unit window1735MiB, piece-length equivalence1686MiB, run58 harness1655MiB,
run87 unit window1651MiB, run93 army window1643MiB, world frame95 comparison1605MiB
and run94 harness clocks1602MiB. Next investigate the same bounded-setup pattern
across the remaining unit/window consumers, retaining independent reference
readers. Harness remains largest CPU family at195.23s. The runner still uses its
existing conservative profile; updated admission/order is a separate experiment,
not a free performance claim. Independent review of observation boundaries,
failure diagnostics and these methodological changes remains owed. Commander
stays paused; this branch is not merged.
