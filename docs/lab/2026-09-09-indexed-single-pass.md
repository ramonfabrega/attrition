# Decode indexed observation chunks once

## Profile and cause

A temporary profile of the two leader-window tests recorded 332 lazy parses
(1.701 seconds), 330 child scans (2.197 seconds), eight index opens (1.001
seconds), eight setup reads (0.033 seconds), and 320 frame reads (0.161
seconds). These are instrumented span totals, not additive CPU accounting;
outer callback spans overlap recursive sibling construction and simulation.
The profiling edits were restored before implementation. Evidence:
`/tmp/indexed-reader-profile.log` and
`/tmp/indexed-reader-profile-summary.json`.

`IndexedCapture::with_replay_initial` already confines each observation read
to one frame and its following siblings. It passed that chunk to the lazy
parser, which indexed deferred blocks; `append_observations` immediately
walked those blocks and parsed them again. The existing eager parser can
build that bounded chunk once and let the observation walk use its nodes.

That internal eager entry point is now available outside tests and preserves
the same 4 GiB arena-offset limit as the lazy entry point. Only the indexed
observation loop switches to it. Whole captures, setup parsing, and other
indexed entry points retain their previous behavior. The per-chunk parse tree
is released at the end of each loop iteration; no parsed-state cache is added.
The memory bound remains the largest selected chunk and accumulated owned
observations, not a fixed byte ceiling.

## Fidelity checks

All ten indexed-reader tests passed, including late setup lookups, roots,
duplicate frames, and changed-source rejection. The retained-corpus sibling
test also compared every resulting Initial field against the whole-text
reader, excluding only the already documented audit frame bodies. The two
leader-window tests retain their complete record and named-residue assertions.
Evidence: `/tmp/indexed-eager-focused.log` and
`/tmp/indexed-eager-corpus-equality.log`. No simulation logic or scores change.

## Clean isolated timing

The baseline is the clean bounded-sibling binary saved in the preceding
leader-memory experiment. The candidate was rebuilt without profiling. Both
run the same two leader tests with explicit RON_INSTALL, one test thread,
and macOS `/usr/bin/time -l`, sequentially in before/after/after/before order.
Compilation is excluded.

| Sample | Version | Peak RSS, bytes | Test seconds | Process wall seconds |
| --- | --- | ---: | ---: | ---: |
| 1 | Before | 971,702,272 | 6.42 | 6.43 |
| 2 | After | 836,009,984 | 5.36 | 5.77 |
| 3 | After | 864,747,520 | 5.35 | 5.36 |
| 4 | Before | 938,967,040 | 6.41 | 6.42 |

The pair's test time improves by about 17%, recovering the preceding scoped
memory change's time penalty in this workload. Peak RSS also stays lower in
these samples, but allocator variation and other chunk shapes preclude a
universal memory claim. The full gate is separate compatibility evidence, not
an isolated speed benchmark. Logs:
`/tmp/indexed-eager-{1-before,2-after,3-after,4-before}.log`.

The arena-size assertion was added after these measurements; it preserves the
previous production path's input bound and adds no traversal. No original
runtime was launched. The paused runtime drafts remain untouched.

## Full validation

The explicit-install gate passed 27 offline Python tests, 275 rondata tests
(one ignored), 824 sim, 13 fixed and three doctests, plus survey, clippy,
formatting and guards. All 550 fixture requests were present. Rondata test
time was 240.19 seconds; sampled process-tree peak was 9221 MiB under 20 GiB.
These suite figures establish compatibility, not a causal suite speedup or
memory reduction. Log: `/tmp/indexed-single-pass-final-gate.log`; fixture report:
`/tmp/attrition-gate-indexed-single-pass-20260909/fixture-coverage.json`.
