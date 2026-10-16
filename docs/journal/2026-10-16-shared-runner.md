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
