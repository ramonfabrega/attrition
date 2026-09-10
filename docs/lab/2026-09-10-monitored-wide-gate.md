# Opt-in monitored test width

## Interface and policy

The explicit-install release runner accepts `--test-threads 2`, `3`, or `4`;
its default remains two. For the measured wider lane:

```sh
python3 tools/release_gate.py "$install" --test-threads 4 \
  --report-dir "$fresh_report_directory"
```

Only the release-test child gets the selected CLI width, always beneath
`tools/memcap.sh 20`. The offline checks, survey, lint, formatting and guards
keep an explicit two-thread environment. No tests are filtered. The report
includes `gate-policy.json` with the selected width and 20 GiB cap, alongside
the existing observed fixture-coverage report. Policy metadata describes the
requested configuration; it is not a success receipt.

The runner clears inherited fixture-audit and cap-marker variables. After
successful sampling preflight, memcap supplies `RON_TEST_MEMCAP_GIB` to its
child and descendants, using its actual cap rather than an inherited value.
The Rust width guard allows one/two threads directly, and three/four only
with a positive marker no greater than 20 GiB. Unpinned or larger widths
still fail on an install-backed machine. The renamed guard is
`diff::floors::tests::the_gate_has_a_bounded_test_width`.

This environment marker is a cooperative wrapper contract, not a security
boundary or proof against a manually forged environment. The actual runtime
protection is memcap's process-tree monitor, including sampling-failure
handling. Its ceiling is sampled, not a hard allocator bound between samples.
The normal `.cargo/config.toml` value remains two; its comment now explains
the explicit monitored exception. DECISIONS and QUEUE are unchanged.

## Negative and behavioral checks

Before implementation, the wrapper-marker regression failed because it passed
through an inherited value of 999 instead of its actual cap, and the gate
option was absent. `/tmp/monitored-width-negative.log` retains those results.

The actual Rust guard rejects direct four-thread execution with the marker
absent (`/tmp/unmonitored-four-thread-negative.log`). The pure policy regression
covers one/two direct, three/four monitored, absent/empty/invalid/zero/over-20
markers, and invalid or larger widths. Gate tests check all three supported
widths, an unchanged cap, no skip flags, environment cleanup, policy reporting,
and refusal before any child for unsupported API values.

The reviewed offline lane now includes `test_memcap`: fake RSS samples and
small disposable child processes test denied/invalid preflight, sampling loss,
over-cap termination, and the child-only actual-cap marker. It launches no
memory-heavy workload. All 34 offline tests passed before the full gate;
`/tmp/monitored-width-offline.log`. The Rust policy regression and clippy also
passed. No original game process or paused runtime experiment was launched.

## Adoption scope

This implements the opt-in lane supported by the preceding concurrency
measurements; it does not raise the direct-Cargo default. Reproduce on the
adopted branch/corpus, keep the monitor, and schedule heavy runs sequentially.
A wider width or a higher memory cap is not established by these results.

## Full monitored four-thread gate

The new gate passed all selected tests without filters: 276 rondata tests
(one existing ignored test), 824 sim, 13 fixed and three doctests, plus 34
Python tests, survey, clippy, formatting and guards. The width guard passed
inside the monitored release run. Rondata test time was 117.51 seconds;
sampled tree peak was 13,016 MiB under 20 GiB. All 550 fixture requests were
present. This preserves the roughly twofold rondata throughput demonstrated
against the preceding 240.43-second two-thread control; surrounding build,
survey and lint stages are not claimed to scale by that ratio.

A direct two-thread invocation of the actual width guard also passed without
a cap marker: `/tmp/unmonitored-two-thread-positive.log`. The direct four-thread
negative case remains the contrasting control. Policy tests and the real gate
exercise the default environment cleanup and explicit release-width override.

Full log: `/tmp/monitored-four-final-gate.log`; reports:
`/tmp/attrition-gate-monitored-four-20260910/gate-policy.json` and
`/tmp/attrition-gate-monitored-four-20260910/fixture-coverage.json`.
