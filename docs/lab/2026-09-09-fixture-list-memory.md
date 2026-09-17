# Fixture identity without capture allocation

## Refreshed reader profile

The Rust-only profile explicitly set `RON_INSTALL` to the local install and
used the existing two-test-thread release configuration. It passed 269 rondata
tests with one ignored. Test execution took 240.40 seconds; the runner's 249.83
seconds includes compilation and launch overhead. Sampled process-tree peak
was 9236 MiB (9205 MiB in the largest process) under the 20 GiB cap.

Instrumentation counted 425 whole-text reads over 68 distinct paths, totaling
118,489,248,194 logical bytes and 13.011 seconds of accumulated read spans.
run13 still led: 47 loads, 30,178,724,628 logical bytes. run12 had 46 loads;
run38 had 48. There were 2374 parse spans (92.349 seconds) and 2278 child-scan
spans (242.386 seconds). Nested/concurrent spans are not additive CPU costs;
logical input is not physical disk traffic. The instrumented profile is a
work locator, not an uninstrumented suite benchmark.

Evidence: `/tmp/attrition-profile-gate.log` and
`/tmp/attrition-profile-refreshed-summary.json`. The runner restores its two
instrumented Rust source files; both were restored before implementation.

## Validation gap found during the refresh

The test install helper searches the repository's `game/` and a relative
fallback for the older worktree layout. Neither finds this Codex worktree's
install. Explicit `RON_INSTALL` is required here. Several install-dependent
tests return early without it, while tests needing only dump files can run.
Consequently, the earlier roughly 35-second release passes and 3.8-GiB peaks
must not be presented as full install-backed validation or compared directly
with this 240-second run. Those pass counts alone do not prove execution of
the skipped bodies. The separate install survey and live captures are separate
evidence, not substitutes for that gate. Future lab release gates set the
install explicitly. No install-detection behavior changed in this patch.

## Implemented boundary and isolated measurement

`the_setup_lists_are_the_suite_s` was intended to compare two filename lists.
Instead it read both lists' complete contents, retaining 2,174,003,150 bytes of
capture text on this machine. It also skipped missing files and could equate
different names containing identical bytes.

`testkit::SIBLING_DUMPS` now names the exact list consumed by `sibling_texts`.
The endpoint guard compares its independent list directly with that constant,
including order and duplicates. It no longer needs an install or captures.
Actual sibling loading and all gameplay/replay comparisons are unchanged.
There is no new raw-capture cache or parser change.

Both isolated measurements ran the uninstrumented release test executable
through macOS `/usr/bin/time -l`, excluding compilation, with the same exact
test filter:

| Measurement | Before | After |
| --- | ---: | ---: |
| Maximum resident set size, bytes | 2,177,056,768 | 2,736,128 |
| Elapsed seconds, displayed precision | 0.28 | 0.00 |
| Capture text read for list comparison, bytes | 2,174,003,150 | 0 |

The final displayed time is below the timer's 0.01-second resolution, not
zero computational cost. This is a single isolated before/after sample, not
a claim that full-suite RSS falls by the same amount. Evidence is
`/tmp/fixture-list-before-clean.log` and `/tmp/fixture-list-after.log`.
The earlier instrumented isolated measurement is kept separately.

Changing one fixture name deliberately made the new guard fail with the
expected mismatch; the original name was restored. The negative-test output
is `/tmp/fixture-list-negative.log`. The content-level fidelity assertions
remain in their original tests; this contract guard never needed their data.

## Remaining work

Most of run13's repeated loading remains. Inspect consumers that need setup
or a bounded frame window before proposing retention. The current indexed
reader already exposes setup and selected-frame paths, but sibling callers
can also use body/trace data: replacing every call with setup-only text would
silently remove evidence. Select consumers individually and compare all
returned records. No simulation score moved; no live capture was needed.
