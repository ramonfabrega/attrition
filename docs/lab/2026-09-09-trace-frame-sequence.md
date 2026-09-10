# Finalized traces require consecutive frame records

## Evidence and contract

An offline scan of all 87 stored `rontrace*.log` files found no duplicate,
missing, or backward FRAME records, no disagreement between their two frame
fields, and no files without a frame. The per-file counts and endpoints are
retained locally in `/tmp/trace-frame-sequence-preflight.json`.

The writer in `tools/trace/tracer.c` emits a FRAME record at `do_frame` entry,
with `Game::frame` in slot 1 and the same current frame in slot 7. Coverage
windows filter coverage hits, not these frame records. These are simulation
frame numbers; gamelog FRAME labels are one ahead, as documented in the trace
README. This change does not compare those two numbering systems.

`Trace::parse_finalized` now checks matching FRAME fields and consecutive
signed frame numbers in the existing validation pass, without another vector
or a sorting step. Duplicate frames, gaps, and resets reject a finalized
single-run trace. Arbitrary starting frames remain valid. The permissive
`Trace::parse` still accepts discontinuous diagnostic streams.

## Regression evidence

The new authored regression first failed against the old reader: a duplicate
frame 71 was accepted (`/tmp/trace-sequence-negative.log`). It covers both
supported versions, duplicates, gaps, resets, mismatched fields, intervening
INFO records, arbitrary starts, and signed arithmetic boundaries. The signed
transition -1 to 0 is consecutive; signed maximum to minimum is not.

## Limits

This is a single-run evidence contract, not a claim that a process can never
start another match. Concatenated matches need separate traces or diagnostic
parsing. Header-only traces remain structurally valid. Removing complete
records from the end still passes these checks: completeness requires an
expected endpoint. These checks do not establish trace/dump identity, complete
state coverage, or fidelity. No gameplay rules or completion scores change.
The runtime experiment remains paused.

## Next offline consistency check

`tools/explore/autostart_receipt.py` already requires frames 0 through its
configured endpoint and an ordered lifecycle. Its record loop currently does
not reject dropped-record INFO notices or disagreement between FRAME slots.
Bring those transport checks into agreement with the finalized Rust reader,
using authored negative receipts before changing the validator. This is a
separate Python acquisition concern; no live capture is needed to test it.

## Full gate

The explicit-install release gate passed: 275 rondata tests (one ignored),
824 sim tests, 13 fixed tests and three doctests, plus install survey, clippy,
formatting and guards. Fixture auditing recorded 550 requests with zero
missing. Rondata test time was 245.95 seconds; sampled process-tree peak was
8833 MiB under the 20 GiB cap. No performance improvement is claimed.

The initial sandboxed attempt refused to start release tests because RSS
sampling was denied. The same monitored gate passed with expanded permissions;
it did not disable monitoring. Final log:
`/tmp/trace-sequence-final-gate-retry.log`; fixture report:
`/tmp/attrition-gate-trace-sequence-retry-20260909/fixture-coverage.json`.
