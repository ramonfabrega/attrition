# Shared isolated runner and persistent setup observations

User explicitly requests production isolation followed by persistent setup
observations, targeting 10× with lower CPU/RSS. Four to six simultaneous suites
now compete on this 16-core, 128-GiB host. Single-agent work, commander paused.
Baseline df3ea74b; previous isolated eight-slot experiment on2d61d180:
876 pass, 6 ignored, 3276 fixture requests, 0 missing; 236.87s vs ordinary
375.88s. CPU1553.73s; sampled tree6211MiB vs10562MiB. Not a10× result.

## Runner implementation checkpoint

Added resource_budget.py with cross-worktree kernel-owned CPU/memory tokens,
individual owned-process cleanup, preflight sampling, timeout and RSS enforcement.
Added isolated_tests.py with Cargo artifact discovery, every exact rondata test,
strict one-test verdicts, fresh output manifests/logs/counters and all remaining
workspace targets/doctests. --isolated gate mode coordinates other phases too.
Default legacy gate unchanged; explicit candidate option pending validation.
Scheduling profile is the measured 2d61d180 eight-slot experiment; unknown tests
reserve16GiB and run alone. No test is omitted for being new/slow/large.

Focused tests cover atomic admission, inherited orphan locks, six simultaneous
suite supervisors, memory/timeout/nonzero failures, unavailable sampling,
manifest duplicates, zero-test false success, unknown tests, once-only execution
and continued execution after a test failure. Initial killpg attempts were denied
by macOS even elevated; individual PID signaling, as used by memcap, succeeds.
This is an observed permission failure, not evidence of a product test defect.

This checkpoint is pending complete committed-tree validation. No simulation,
word, score, value, fixture selection or coverage change; pins/scoreboard remain
unchanged. Independent review owed. Persistent observations are next sequential
work after this runner is validated; no implementation or speed claim yet.

## First complete candidate gate (25bf8765)

Actual exit0, all6 phases. fixed13, sim2585, rondata876, rondata doctests3;
6 ignored,0 failed. Exact manifests and all3276 fixture-request rows match the
previous ordinary gate (419 sources,0 missing). Evidence retained under
`/tmp/attrition-shared-runner-20261016-01/isolated-gate*`; explicit command is
`python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14
--test-threads 4 --require-fixtures --isolated --report-dir FRESH_DIRECTORY`.
This is the complete isolated gate, not a claim that the default in-process
command ran again. Guard and10 focused runner tests passed too.

Rondata217.25s versus375.88s ordinary:1.73×. Its test subprocess CPU1582.79s
versus1553.73s experimental isolation. Full wrapper326.74s,1680.38user+325.04sys
=2005.42CPU seconds, versus ordinary482.19s/1663.59CPU seconds. Thus latency
improves but total CPU regresses20.5%; do not call this a CPU improvement.
An attached one-second sampler during rondata measured9586.64MiB; it did not
cover the earlier gate phases, so this is not a whole-gate peak claim.

The candidate's frequent independent process samplers are the next falsifiable
CPU hypothesis. Reduced sampling from0.2s to the established2s cadence, kept
preflight permission checking, and avoid a full-process-table query for normal
cleanup. Pending repeat measurement. The original successful result is retained.

## Setup baseline before persistence

Retained public setup-stage driver, existing parser binary (unchanged in this
runner-only checkpoint), three calls per process, sequential after the gate:
chapter36:2.017/2.054/2.117s,2815 clock frames; run12:.215/.051/.059s,4 seeds
and4 clock frames; run56:.966/.011/.011s,no observations. Chapter36 exceeds
the small in-process cache. Empty run56 still rescans on every fresh process.
Logs `setup-{ch36,run12,run56}-before.log` in the evidence root. Persistent
entries must stream bounded chunks so large clock sequences do not require a
second whole encoded/decoded allocation or a clone of every observation.

## Tuned runner and final hardening

On22655a63, all876 rondata tests passed again,6 ignored, exact3276 fixture rows
unchanged.212.26s test scheduling; wrapper212.34s,1498.41user+148.39sys=1646.80CPU,
versus1565.99CPU inside test processes. Thus81CPU seconds of runner/monitor
cost remain; do not report a CPU-work reduction from isolation. Sampled tree
9015MiB, largest3856MiB. Evidence `tuned-rondata*`, `tuned-fixtures/`.

Additional hardening bounds assertion-log parsing, records build revision and
binary SHA256, rejects changed executables, tests cancellation/lease release,
and lets small gate phases and fixed/sim targets share capacity. Compilation and
unknown targets still reserve the whole host budget.13 focused runner tests and
36 gate tests are expected in the final validation; earlier counts are retained
above rather than rewritten. Final full gates remain owed on the committed tree.

## Persistent setup implementation and paired evidence

Shared the existing safe bounded wire primitives and exhaustive Guy/Pos schemas
with production capture code; Oracle keeps its own schema and wrappers. Added
source-keyed, immutable, streaming observations with per-chunk and aggregate
bounds, checksum, explicit flush and atomic no-replace publication. Owned
observations move into Initial; encoding borrows rows and never clones the
entire clock history. Raw-key recording bypasses both caches. Disk-disable
retains the old small memory cache, so paired comparisons do not inflate the
baseline by also disabling previous reuse. No sim, word, score, value or pin
changes. Paused commander unchanged; independent review owed.

Initial synthetic checks:20 indexed tests,1 whole-oracle codec test, clippy/fmt
passed. Corruption at every byte, every truncation, wrong source identity,
empty streams, duplicate/order preservation, invalid lengths, flush failures,
immutable publication and raw-key bypass are exercised. Existing index tests
retain actual changed-source validation. Evidence `observations-focused.log`,
`oracle-codec-focused.log`, `observations-clippy.log`.

Same-binary, separate-process setup measurements (`setup-comparison.json`):
chapter36 warm0.07s/0.06CPU/156.6MiB versus cache-off2.00–2.42s/1.99–2.20CPU/
475.7MiB; run12 warm0.05s/0.04CPU/125.1MiB versus0.21s/0.20CPU/315.8MiB;
run56 warm0.01s/0.01CPU/~37MiB versus0.91–0.92s/0.91CPU/36.8–39.9MiB.
Cold writes cost roughly the original parse; no universal memory improvement
is claimed for empty observations. The3 derived entries occupy13,731,230bytes
outside git. All3 full Initial comparisons passed (clear only original bodies).

Complete chapter36 replay, same final candidate binary and scratch cache:
cache-off10.35–11.29wall/10.34–10.56CPU/1122.7–1125.4MiB; cold10.51wall/
10.49CPU/1117.2MiB; warm7.61–7.62wall/7.60–7.61CPU/758.5–758.8MiB.
Five confirmed setup hits per warm run. Complete setup audit passed; ordinary
comparison run after the audit also passed. About1.36× CPU and32% lower RSS
for this complete replay, distinct from its~30× setup microbenchmark.

All211 Golden-module tests then passed with fresh-observation and original
whole-reader setup audits (192.23s,804 fixture requests,0 missing). Source logs
record87 successful publications,49 hits and10 unsuccessful publication
attempts under concurrent creation. Every corresponding source subsequently
had a valid entry: all87 sources were reopened in separate processes, each
required a hit and complete equality to both fresh indexed and original whole
Initial (only original frame_bodies cleared). All87 passed. The first audit's
publication failures are consistent with competing immutable publishers, not
silently dropped comparisons; their exact I/O error was not logged.
Evidence `golden-setup-audit*`, `golden-source-audit*`, and source manifests.

Generic whole-oracle/replay/field-set audit and final gates are pending.

## Complete generic audit

Cold coverage created9879 oracle records in the new fingerprint generation:
89.73wall/87.96CPU/1629MiB. Warm audit checked all9879 whole records against
fresh parsing, all62 complete replay reports/checkpoints/field sets against
fresh replay, and fresh setup observations:247.08wall/246.01CPU/1917MiB.
586 fixture requests,0 missing. Ordinary coverage afterward passed with9879
hits/0 misses,41.54wall/41.12CPU/2260MiB;338 requests,0 missing. This remains
within the previous40.94–43.36s range: no meaningful coverage speedup or memory
win is established. Raw-key coverage passed at32.82wall/32.55CPU/116MiB,
144 requests,0 missing, no observation/oracle cache hits. Evidence
`coverage-results.json`, `coverage-comparison.json`, `coverage-*.log`.

## Host throughput policy

The initial8-slot/16-GiB policy above was a single-suite measurement. For the
user's4–6 simultaneous suites, final candidate admission uses16 CPU slots on
this16-core host and32GiB combined reserved RSS, while each suite keeps8 workers.
All workloads regain the established nice+10 priority. Unknown tests reserve
all memory and run alone. Allocation uses256MiB tokens, bounding descriptor
counts even for a full-budget lease. A create-new immutable policy file prevents
differently configured cooperating worktrees from silently sharing one pool.
`resource_budget.py --status` checks live kernel locks and reports supervisor/
child PID metadata; it never infers liveness from a stale file or kills by name.
The intended shared configuration is in
`~/Library/Caches/attrition/resources-v1/policy.json`; no capture locks changed.

Four simultaneous suite samples (eight named tests each, including coverage,
third-pair, Golden, city/world/build and a scalar test) are being compared under
this policy. Warmup runs all eight first; both modes preserve their exact test
and fixture multisets. This is a contention sample, not a full-suite result.
The scratch `contention.py` and all per-suite outputs are retained. Final full
gates and the measured contention verdict remain pending.

The first contention attempt used per-job headroom for both admission and the
hard limit: conventional54.997s vs isolated79.178s, both1832 fixture rows and
all32 selected tests passing. Sampled RSS16086 vs13641MiB; CPU486.34 vs461.37s.
This was an unacceptable throughput regression: an expensive test waited behind
pessimistic reservations while actual memory was low. The initial scratch
sample also submitted listed order rather than production's longest-first;
that detail is retained, not silently treated as identical dispatch.

Final candidate separates measured-RSS admission from1.5× per-job hard ceilings,
and additionally samples the union of kernel-held workload roots/groups against
the32GiB host limit. Metadata publication/status are serialized; unreadable live
ownership fails closed. Group accounting includes orphaned group members and
deduplicates overlapping roots. A breach terminates only the supervisor's own
workload. The revised sample uses production's longest-first submission order.
First corrected repeat:54.002s/483.27CPU/18300MiB, same1832 fixture rows. Thus
throughput is restored, with higher observed RSS than this particular conventional
sample; no universal multi-suite memory win is claimed. Repeat measurements are
pending. The baseline includes the new observation cache too, so it isolates
scheduling rather than crediting the cache's CPU savings a second time.

## Pre-landing validation and contention verdict

Second conventional sample52.629s/483.39CPU/19452MiB; second corrected isolated
sample57.833s/472.49CPU/17906MiB. Across repeats, conventional52.63–55.00s and
16086–19452MiB versus corrected isolation54.00–57.83s and17906–18300MiB.
All32 selected executions passed in every sample; all1832 fixture-request rows
match exactly,0 missing. No aggregate throughput or memory improvement is
established by this sample; the large reservation-induced regression is removed.
The shared ceilings bound competing suites and prevent independent width
choices from oversubscribing the pool. CPU differences are small and noisy.
Committed summary: `docs/audit/2026-10-16-contention.json`.

Final focused tooling:19 runner tests and36 gate tests pass, formatting and
diff checks pass. Kills/timeouts/missing counters retain infrastructure exit
categories instead of becoming an ordinary test assertion exit. Full gates
below remain required before this batch is declared complete.

The97 production observation entries seen by these audits occupy265415701bytes;
the footprint manifest is retained in the evidence root. New oracle generation:
9879 entries; older generations and all cited scratch evidence remain intact.
No original corpus was relocated or deleted. No pins or scoreboard values moved.

Implementation landing: pending committed-tree ordinary and isolated gates.
For the Loop: commander paused; source-keyed setup persistence and shared
isolation implemented, complete final gates pending;10× still unmet.
