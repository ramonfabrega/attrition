# Test-suite performance

## Target and measurements

The user-authorized target is 10× lower complete-suite elapsed time, with lower
CPU work and peak memory, preserving fidelity checks. Report elapsed, user and
system CPU time, and peak resident bytes separately. `tools/suite_timings.py`
measures elapsed time in separate test processes, not CPU time or the wall time
of the concurrent suite. Its historical merged baseline is not a controlled
same-tree comparison. Keep concurrency, fixture availability, build profile,
and warm/cold input-cache state explicit. A local test gain is not a suite gain.

## Scoped replay reuse

The coverage driver may open a thread-local `harness::prefix::Scope` over
immutable finalized fixtures and an unchanged installation. It retains at most
one `Built` plus one shared loaded installation. A subsequent widening can move
that complete state forward only when its ordered base/trace/sibling paths,
lengths, modification times, and installation root agree, and its first compared
block is strictly after the retained simulation frame. All observation streams,
links and simulation state travel together. Overlapping or incompatible windows
restart; a missing input is included in the key rather than silently equated
with a present input. The comparison capture is never substituted: every requested
block and all existing comparisons still execute. Per-window result accumulators
start empty. No verdict, comparison result, or recorded coverage is cached.

The guard drops all retained state, including on unwind, and cannot move between
threads. Normal standalone tests retain their from-zero execution. Watched-prefix
diagnostic modes disable reuse. The batch assumes installation files remain
unchanged while it runs; this is not a persistent cross-version replay cache.
`RON_DISABLE_PREFIX_REUSE=1` supplies the independent replay path.
`RON_VERIFY_PREFIX_REUSE=1` compares each optimized widening's complete report,
checkpoints and independently recorded field set with a fresh from-zero walk.
`RON_PROFILE_REPLAY=1` reports setup, ticking, total time and reused frame count.
An audit is followed by the ordinary coverage test, so the legacy walk cannot
supply missing coverage to a normal passing verdict.

## Coverage recording

The comparison recorder stores static `(record, field)` schema names, deduplicated
as pairs, and formats `Record.field` only when returning the result. The caller
still notes a field only when both sides exist. It retains no fixture text and
preserves the public sorted label set. A differential unit test compares with
the previous string-set algorithm, including duplicates, disabled recording and
nested audit recording. This is allocation reduction, not sampled coverage.

## Trace frame lookup

A parsed trace records whether draw frame numbers are nondecreasing. For that
case `frame_draws` uses binary bounds and filters sync draws within the matching
span; duplicate frame labels and file order are preserved. Unordered diagnostic
traces retain the original full-vector filter. `Trace::draws()` exposes a read-only
slice instead of a public mutable vector, preventing an ordering fact from going
stale. This requires no extra per-draw or per-frame index allocation. Synthetic
tests cover unordered inputs, duplicates, missing frames and non-sync draws;
fixture tests partition every draw of run53, run54 and run600 in one independent
pass and compare every frame group. All named-site and value assertions remain.

## Rejected field lookup experiment

An eight-record, 32 KiB field-position index was differentially correct but
increased coverage time and retired instructions. It was removed; the original
linear field lookup remains. The journal retains measurements and the experiment
patch location so this hypothesis need not be repeated without new evidence.

## Evidence and limitations

See `journal/2026-10-16-suite-replay-batch.md` for measurements, executable audits
and validation state. No claim of 10× is established yet. Scope compatibility is
also reviewed directly in code; independent review remains owed. Future shared
replays must distinguish different corrections, interventions and tuning. A
persistent parsed oracle store remains a separate, unimplemented opportunity.
