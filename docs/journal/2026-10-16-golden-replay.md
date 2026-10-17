# Repeated golden and East Indies observations

Sequential continuation from `d4e8d083` on `codex/astra-windowed-reader`.
Commander remains paused; independent review remains owed.
Evidence: `/tmp/attrition-golden-replay-20261016-01/`.
[Audit](../audit/2026-10-16-golden-replay.json).

## Measurement and scope

The preceding complete run attributes298.56CPU to golden and242.02 to harness,
47% of rondata CPU. Native samples (`profile.py`, `*-sample.txt`) locate repeated
capture scans and decoding in chapter two near and East Indies widenings.
Inclusive sample stacks overlap; they are localization evidence, not additive
CPU percentages. Simulation replay still executes every tick from its original
start. No simulation result, comparison result or passing-test verdict is cached.

Chapter two near now reads the first matching indexed frame through its trailing
siblings, then uses the original near parser. It uses the established bounded
setup path. Reading just the frame would lose trailing records: the authored
regression covers that seam, duplicate frames and fields, missing fields, CRLF,
and nonmonotonic frame numbers. Every original frame/map assertion remains.

East Indies setup now uses bounded Initial observations for base and sibling.
Frames, groups and leader observations use the existing persistent oracle store;
region rows are a separate optional component with a distinct cache identity.
Other callers do not compute region rows. Existing source/schema invalidation,
codec bounds, corruption rejection, atomic publication and raw-key recording
bypass remain. This schema change causes one-time cache repopulation; old entries
are retained. Shared source caches hold observations, never simulation outcomes.
Every unit, record, leader, region, gaia, near, army-list and floor check remains.

## Differential and fail-first evidence

`audit.py` selects all19 harness and25 second-pair users plus chapter two near.
All45 pass with `RON_VERIFY_EAST_REUSE`, `RON_VERIFY_ORACLE`,
`RON_VERIFY_GOLDEN_SETUP` and `RON_VERIFY_GOLDEN_NEAR` enabled:86.05s/328.88CPU.
This compares complete widening reports plus record-row counts, every retained
Initial field (excluding intentionally omitted frame bodies), cached observations
against raw parsing, and each complete near map. The original parser paths remain
available through `RON_DISABLE_EAST_REUSE` and `RON_DISABLE_GOLDEN_NEAR_INDEX`.
Audit flags deliberately repeat work and are not performance defaults.

Two new authored guards cover near boundaries and region-mode identity/base
observation preservation. `fault.py` makes each fail101 by removing trailing
records or disabling region mode; exact restoration makes both pass0. Existing
oracle corruption, stale-source, duplicate-frame and codec tests also pass.
All-target Clippy passes. Sample normal-mode diagnostics and exact fixture-request
multisets equal the saved baseline binary for all three measured tests.

## Local performance

`measure.py LABEL BINARY` runs each exact test under one CPU/4096MiB admission,
using `/usr/bin/time -l`. RSS below is the OS high-water mark, not the sampler:
short tests can finish between memory samples. These are single local observations,
not a statistical benchmark or a concurrent-suite capacity claim.

| Test | Baseline wall/CPU s | First-use wall/CPU s | Warm wall/CPU s | Baseline → warm RSS MiB |
|---|---:|---:|---:|---:|
| Chapter two near |10.60/10.33|1.03/0.69|0.59/0.59|1410 →398|
| East Indies run227 |7.63/7.64|5.78/5.58|1.95/1.90|381 →256|
| East Indies run299 |4.48/4.43|3.60/3.52|2.11/2.11|384 →262|

Near is about18× faster locally; the two harness samples are3.9× and2.1× warm.
This does not establish whole-suite10×, sub-minute or sub-GiB operation. Warm
observations depend on unchanged source/schema and are still compared against
fresh simulation on every run. Cold parsing/publication cost is reported above.

## Landing status

Implementation checkpoint: complete required-fixture gate pending. No score,
pin, draw-stream or value-floor movement; simulation behavior is unchanged.
The final committed tree must pass the full release gate, preserve the original
fixture-request multiset and test inventory, and receive a separate warm complete
rondata measurement before this landing is reported complete. Fable review of
cache identity, trailing-record semantics and differential coverage remains owed.

## Complete verdict on `55793bfc`

The complete required-fixture gate exits0; all six phases pass. The tree remained
clean and frozen. Receipts: `gate.log`, `gate.exit`, `gate/`, `warm.log`,
`warm.exit`, `warm/`, `gate-equivalence.json`. The gate executable and saved
`final-rondata` used by the audit and warm measurement are byte-identical.

-502 offline tests pass, with three unchanged skips; Clippy, fmt, survey, guard pass.
-Rondata894 pass, six unchanged ignored; sim2586, fixed13, doctests3 pass.
-Every original rondata name retained; exactly two authored guards added.
-Exact3276 fixture-request multiset preserved in both complete runs;419 unique,
 zero missing. This verifies requested fixture coverage, not an exhaustive claim
 about every file in the corpus.
-3 ledger,22 floor and2 source guards pass. Ledger remains25 records/328 fields,
 with9 uncompared and31 single-capture fields; no pins or fidelity floors changed.

| Complete measurement | Previous landing | This landing |
|---|---:|---:|
| Full gate wall / CPU s |271.28 /1472.54|260.39 /1438.62|
| Full gate sampled tree MiB |8864|7255|
| Rondata in gate wall / CPU s |161.22 /1158.05|165.20 /1185.64|
| Complete warm rondata wall / CPU s |161.22 /1158.05|145.58 /1037.78|

The gate repopulates observations after a schema change; its rondata phase is
slower than the previous landing. Warm rondata uses9.7% less elapsed time and10.4%
less CPU. These are unpaired local runs, not statistical confidence intervals.
The warm run's sampled tree peak is8186MiB (largest sampled process1994MiB),
higher than this gate's7255MiB. Scheduling affects aggregate peaks: local test
RSS reductions are established, but a stable whole-suite memory reduction or
10–20 concurrent-agent capacity is not established. No contention experiment
or sub-minute/sub-GiB complete-suite claim is made.

Warm family CPU: golden298.56→283.12, harness242.02→198.12,
second-pair118.54→67.04. Golden remains the largest family (283.12CPU); chapters
36/37/35 still cost8.65/8.25/8.01CPU and use raw auxiliary observations in the
civilian widening loop. Next investigate a complete reusable golden observation
record covering those auxiliary checks, with whole-report and raw-read coverage
proof before enabling it. Existing oracle fields alone do not cover that loop.
The two coverage guards remain the largest individual tests at36.57/33.24CPU.
Whole-suite10× remains open. Score, draw-stream and value differences: no movement.
Independent review remains owed. This branch is not merged into the paused loop.
