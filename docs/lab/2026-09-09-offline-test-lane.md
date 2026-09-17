# Offline Python regressions are part of the gate

## Gap and implementation

Receipt, acquisition comparison, and cleanup regressions previously ran only
when explicitly selected by a worker. A clean Rust gate did not run them.
`python3 tools/offline_tests.py` now runs four reviewed modules explicitly:
`test_autostart_receipt`, `test_compare_unattended`,
`test_unattended_capture`, and `test_release_gate`.

The release gate invokes this lane with its own Python interpreter before
surveying the install or starting the expensive release suite. As before, the
gate validates its install path before any child. Every child still receives
explicit RON_INSTALL, while fixture auditing is enabled only for release tests.
The offline lane can also run directly without an install.

The tests use authored temporary files. Live-session operations in cleanup
tests are mocked; the launch-argument test runs a temporary zsh helper and
`/usr/bin/true`, not Wine or the game. Gate tests substitute the child runner,
so including them cannot recursively execute the release gate. New test files
are not implicitly discovered. Adding another module requires reviewing its
side effects and listing it explicitly. This is a reviewed selection, not an
OS sandbox or automatic proof that future edits stay offline.

## Failure evidence

A new gate regression first failed against the old orchestration: a simulated
offline-lane failure did not stop it because it never called that lane.
`/tmp/offline-gate-negative.log` retains the failure. It now passes and verifies
that no survey or release child follows a failed offline child. The existing
survey-failure check still verifies stopping before release tests.

Separately, a temporary failure was inserted into the receipt success test.
The real offline runner exited 1 and reported that failure; the test file was
restored in a finally block. Evidence:
`/tmp/offline-lane-injected-failure.log`. All 27 tests then passed in 0.028
seconds on the first clean run (`/tmp/offline-gate-positive.log` retains the
latest clean result). No performance improvement is attributed to this wiring.

The reviewed suite covers transport/lifecycle and test orchestration, not
simulation fidelity or the paused runtime experiment. No native process was
launched, and no simulation score changes.

## End-to-end gate

The updated explicit-install gate passed the 27 offline tests in 0.029 seconds,
then 275 rondata tests (one ignored), 824 sim, 13 fixed and three doctests,
plus survey, lint, formatting and guards. Fixture auditing observed 550
requests with none missing. Rondata test time was 249.94 seconds and sampled
process-tree peak was 9359 MiB under 20 GiB. The Python timing excludes
interpreter startup; no full-suite performance improvement is claimed.

Log: `/tmp/offline-lane-final-gate.log`; fixture report:
`/tmp/attrition-gate-offline-lane-20260909/fixture-coverage.json`.
