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

## Parsed oracle windows

The generic widening harness can read immutable decoded observations beside each
canonical source, in `.ron-oracle-v1`. The original text remains authoritative.
`RON_ORACLE_CACHE_DIR` redirects entries to an external experiment directory;
`RON_DISABLE_ORACLE_CACHE=1` forces the original parser. Entries contain original
derived data and must stay outside git. Other walkers remain on their existing
readers. This source-keyed implementation does not merge captures by lobby.

Every entry identifies the held source path/size/mtime, frame position and byte
range, and a parser/schema fingerprint. Duplicate frame labels remain distinct.
The payload contains the complete owned Frame, pool groups and normalized
original leader rows; no simulation state or comparison verdict. Exhaustive
schemas make omitted new owned fields a compile error. Entries are bounded,
compressed with the existing safe LZ4 dependency, checksummed and strictly
decoded; invalid entries fall back to source parsing. Publication never replaces
an existing file. Old generations are retained; there is no automatic pruning.
The source must remain finalized, as required by the offset reader.

`RON_VERIFY_ORACLE=1` checks warm records against fresh whole-record parsing and
compares complete reports, checkpoints and compared-field sets against uncached
walks. Raw-key instrumentation always bypasses the store. Ordinary warm runs
must separately pass, with hit/miss/write counts proving actual cache use.
Cold creation and warm operation are separate measurements. Scratch buffers
are bounded and reused within a widening; one record buffer is rewritten on each frame, with a retained-capacity limit.

See `journal/2026-10-16-parsed-oracle.md` for experiment results and validation.
The complete warm record/report audit passes. Warm coverage improves time/CPU
but still peaks above its cache-off memory baseline after allocation fixes.
The required four-thread gate passed: rondata 396.15 → 383.16 s, sampled tree
RSS 9,879 → 10,025 MiB. This is a small observed time gain and no memory win;
no 10× result is established. A fresh full profile is the next decision point.

## Evidence and limitations

See `journal/2026-10-16-suite-replay-batch.md` for measurements, executable audits
and validation state. No claim of 10× is established yet. Scope compatibility is
also reviewed directly in code; independent review remains owed. Future shared
replays must distinguish different corrections, interventions and tuning. The
parsed store is initially limited to generic widening windows.

## Raw auxiliary reader projections

Golden's raw `near` reader retains only its four output keys. It keeps the
previous raw lexical semantics (including first valid duplicate field, nested
lines, incomplete-record omission and last duplicate unit). It does not replace
this reader with a differently scoped tree walk. Ammo parsing starts at the
complete line containing the first possible literal `BEGIN AMMO`; the unchanged
state machine handles that line and every later line. A false substring marker
is harmless. No marker means the old state machine could never open a record.
Record widths, nesting, ordering, values and comparison sites remain unchanged.

`RON_VERIFY_GOLDEN_READERS=1` independently compares every complete near map
and every ordered ammo record/width vector with the old readers.
`RON_DISABLE_GOLDEN_READERS=1` selects the originals for paired measurements.
These readers are stateless; there is no new cache or fixture substitution.

## Setup-only consumers

World, order, build, city, leader and army tests use `testkit::replay_initial`
where setup does not consume frame_bodies. This calls the existing replay
reader, retaining every seeding, animation, object, map and sibling-borrowing
input. Explicit figure-position audits and full-reader reference assertions
continue to call `Log::initial`. Source text lifetimes are unchanged in this
migration: it eliminates unused body decoding/retention, not every whole read.

`RON_VERIFY_REPLAY_SETUP=1` compares the entire optional Initial against the
original reader after clearing only reference frame_bodies. It also compares
absence rather than silently accepting missing setup. `RON_DISABLE_REPLAY_SETUP=1`
selects the original full reader for controlled measurements. Neither mode
changes simulation stepping, comparisons, assertions or fixture selection.
