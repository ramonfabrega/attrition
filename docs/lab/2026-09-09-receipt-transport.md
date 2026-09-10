# Acquisition receipts reject unhealthy transport

## Counterexamples and correction

The Python acquisition receipt validated lifecycle and complete endpoint frame
coverage, but ignored positive INFO dropped-record counts and conflicts between
the two FRAME fields. Thus a successful process with all expected frames could
still receive a successful receipt for a damaged trace.

Five authored negative cases failed against the old validator: mismatched
FRAME fields, and dropped counts 1 and 0xffffffff inserted before or after the
lifecycle. Evidence: `/tmp/receipt-transport-negative.log`. The shared record
loop now rejects both conditions, matching the finalized Rust reader's checks.
Zero dropped records remain valid. Existing lifecycle, endpoint, map, and
fidelity result fields keep their meanings.

The regressions exercise in-memory parsing and faults beyond the streamed
64 KiB chunk boundary. A comparison regression gives both acquisitions the
same malformed trace and requires rejection: equal data is not enough.
Existing authored FRAME fixtures were corrected to populate both frame slots,
including the seed-only divergence case, so it still tests seed divergence.
All 19 focused receipt, comparison, and unattended cleanup tests pass.

## Stored evidence and limits

Offline revalidation passed four existing captures: maps 14 and 18 in both
`/tmp/attrition-dll-confirmation-20260909` and
`/tmp/attrition-unattended-1400`. Each retained all 1401 frame records through
endpoint 1400. Results: `/tmp/receipt-transport-stored.json`. No original
process was launched and no new capture was acquired.

This is transport/lifecycle validation, not complete simulation state parity.
Map identity and fidelity remain separate checks. No format, runtime hook,
simulation behavior, completion score, or acquisition setting changes.
The runtime experiment remains paused; its drafts are untouched.

## Remaining regression wiring

The Rust release gate and `tools/guard.sh` do not invoke these Python tests.
They were run explicitly for this change. A next infrastructure step is a
reviewed offline Python test lane, invoked by the gate, so receipt regressions
cannot disappear behind a clean Rust run. Avoid indiscriminate discovery of
experimental runtime tests; select and document the offline suites.

## Pre-commit gate

The explicit-install gate passed 275 rondata tests (one ignored), 824 sim,
13 fixed and three doctests, plus survey, clippy, formatting and guards.
The fixture audit observed 550 requests with none missing. Rondata test time
was 245.76 seconds and sampled process-tree peak was 9190 MiB under 20 GiB.
These are validation measurements, not a performance improvement claim.
Log: `/tmp/receipt-transport-final-gate.log`; fixture report:
`/tmp/attrition-gate-receipt-transport-20260909/fixture-coverage.json`.
The separate 19-test Python result is `/tmp/receipt-transport-positive.log`.
