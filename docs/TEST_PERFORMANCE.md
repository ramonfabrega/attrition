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

Static pairs now use pointer-and-length identities in an internal HashSet.
Only immutable static schema strings qualify. Final labels are still canonical
by text, so equal strings at different addresses cannot alter coverage results.
`RON_DISABLE_FAST_COMPARE_NOTES` selects the old pair tree;
`RON_VERIFY_COMPARE_NOTES` checks both complete sets. No sampled registration,
persisted pointer identity, fixture value, or verdict enters this recorder.

Direct widening rows retain a static label base and optional figure index;
they allocate the diagnostic string when values differ. All values and presence
guards still run. `RON_DISABLE_LAZY_ROW_LABELS` provides eager reference labels;
`RON_VERIFY_ROW_LABELS` compares complete per-frame results, row counts,
divergence maps and registered fields against that mode. The ordinary required
fixture gate must also pass without either audit. Stage profiling now separates
decoding, values, groups and leaders/gaia from setup and ticking.
See [coverage journal](journal/2026-10-16-coverage-walk.md) for measurements and
validation status. Warm coverage CPU falls21–25%; four/six-suite samples save
15%/14% CPU and16%/18% elapsed. Memory falls6% in the four-suite sample and
is flat in the six-suite sample; no consistent memory-reduction claim.
Independent review of these optimizations remains owed.

## Decoded containers and visibility ticking

Owned decoding reuses optional heap values, arrays and tuples. Stable ordered
map keys are checked byte for byte before retaining their String buffers and
tree nodes; every value is decoded anew. Changed or reordered schemas use the
original decoder, including duplicate-key rejection. Input, allocation and
retained-capacity bounds are unchanged. `RON_DISABLE_DECODE_REUSE=1` selects
original nested-container decoding. The complete oracle audit remains available.
The visibility predicate checks a sufficient live fog bit before scanning live
wonders and units, with the unchanged function as fallback and test reference.
No simulation state, draw, comparison, verdict or field coverage is skipped.

## Test-process allocator policy

On macOS, `RON_TEST_ALLOCATOR=auto` is the default. Combined-process test
children use `MallocLargeCache=0`; isolated children use it only for parsed-field
coverage and the third-pair test group, where repeated walks showed substantial
retention. Other isolated tests retain the system allocator policy. The isolated
gate dispatcher leaves this choice to each child rather than forcing a setting
onto every test. Native heap/VM measurements found roughly 60 MiB of live
malloc data while freed large allocations accounted for gigabytes of footprint.
This child-local policy releases those allocations sooner; it changes no host
configuration or simulation semantics. Explicit `MallocLargeCache` values are
preserved. `RON_TEST_ALLOCATOR=bounded` forces reclamation for all children;
`RON_TEST_ALLOCATOR=system` opts out. Other operating systems receive no setting. Unknown modes fail rather than silently selecting a policy.
The gate manifest and each isolated result record the effective setting.
Direct cargo/binary invocations must set the variable themselves to reproduce it.

The same original coverage binary drops from 2.25–2.37 GB peak RSS to 0.856 GB
with this policy, without an observed CPU penalty. With both binaries bounded,
container reuse and visibility reordering reduce coverage CPU about 11–13%.
The initial coverage candidate with the system allocator increased RSS; it is
not a coverage-memory improvement without the measured policy. Applying bounded
reclamation to every isolated test added about 4% CPU versus the same candidate
with the system policy. The scoped automatic policy avoids that broad cost.
One complete warm rondata comparison saves about 2% CPU/time; its aggregate peak
memory varies upward 5%, so do not extrapolate the contention memory saving to
one complete suite. New or renamed isolated tests use the system policy until
measurements justify inclusion; ordinary resource caps still apply. See the
[ticking/decoding journal](journal/2026-10-16-ticking-decoding.md) for differential
checks, contention results and landing validation. Independent review remains owed.

The final required-fixture gate passes on `3b7e6846`: 427.11 seconds,
1,438.50 CPU seconds, 8,089 MiB sampled tree peak. Compared with the preceding
recorded gate, time/CPU are roughly flat and peak memory is 18% lower. The selected
four/six-suite contention samples save 6%/7% CPU and 15%/20% memory; six-suite
elapsed falls 9%, four-suite elapsed is flat. All fixture request multisets match.
These observations preserve the unmet 10× target and outstanding independent review.

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

## Process isolation experiment

On2d61d180, the ordinary four-thread rondata suite took375.88 s; the full
release process tree peaked at 10562 MiB. Running every nonignored rondata test
in a fresh process with at most eight slots, a 6 GiB estimated-RSS scheduling
budget and an 8 GiB enforced cap took236.87 s and 6211 MiB. All 876 tests passed;
test/ignored manifests and every fixture-request row matched the ordinary run.
Four-slot isolation took392.17 s and 5659 MiB. CPU seconds were essentially equal
between isolated runs; concurrency reduces latency, not underlying CPU work.

The prototype is retained in the journal's evidence directory. It is not the
default gate, and covers rondata library tests only. A production runner needs
failure/manifest/resource-policy tests and must retain all other gate phases.
Unknown-test memory estimates need a conservative policy; measured estimates
are hints, not a substitute for the enforced cap. See the journal and
`docs/audit/2026-10-16-isolated-runner.json` for the complete measured limits.
Never compare historical serial totals with parallel wall time to claim10×.

## Shared isolated runner

`release_gate.py INSTALL --test-threads 4 --require-fixtures`
defaults to process isolation with shared host admission. `--combined` selects
the reference in-process runner under the same host budget. The complete initial isolated gate and subsequent
monitor correction measurements are recorded in the shared-runner journal.
Final same-commit gates on `575bad00` passed all six phases and identical fixture
request multisets: ordinary455.87s/1529.20CPU versus isolated285.65s/1540.76CPU.
Sampled tree peaks10473MiB (ordinary release) and9029.5MiB (whole isolated gate).
This is1.60× full-gate latency, with roughly unchanged CPU; rondata itself
342.02→176.80s uses four versus eight workers. The four-suite contention sample
established no aggregate throughput improvement. See
`docs/audit/2026-10-16-shared-runner-final.json` for scope and limitations.
Cargo discovers every workspace release test executable; all non-rondata
executables and all workspace doctests still run. Each nonignored rondata library
test runs in a fresh process with exact-name selection and one test thread.
A successful zero-test execution cannot pass. Manifests, individual logs,
measurements and failures are retained in a fresh report directory. Assertions,
fixture auditing, survey, offline checks, clippy, formatting and guards remain.
Scoped/lane gates pass their unchanged exact skip list to this runner. New tests
still run; manifests record selected, ignored and filtered names. `--combined`
retains the original combined execution and identical selection.

Every cooperating worktree shares `~/Library/Caches/attrition/resources-v1`:
sixteen CPU slots and 32 GiB reserved RSS, with admission serialized by flock.
Each suite uses at most eight isolated workers. An immutable policy file rejects
worktrees configured with different limits. Workloads run at nice+10, preserving
the existing monitor's preference for interactive/capture work.
Builds, clippy, survey and unknown executables reserve eight CPU slots and all
32 GiB, draining concurrent workloads. Offline checks and formatting reserve two slots/1 GiB; guards reserve eight slots/32 GiB because they can rebuild after source edits.
Fixed and sim binaries reserve their requested thread width and 512 MiB/4 GiB
respectively, allowing smaller checks from different suites to coexist. Known
tests reserve measured RSS rounded to 256 MiB (minimum 256 MiB), with a separate
per-test ceiling of 1.5 times that reservation, capped by the host budget.
Unknown tests reserve all memory. Estimates affect scheduling only, never
inclusion. Each supervisor
samples its tree and the union of all active workload trees/groups every two
seconds, counting each process once. It stops its owned workload on a per-test
or aggregate host ceiling breach, unavailable monitoring, or execution timeout
(30 minutes, excluding admission wait). This is a sampled ceiling: brief overshoot is possible. CPU slots bound known test workers,
not OS CPU consumption or arbitrary descendants. Unmodified gates and unrelated
processes do not participate. Rollout to other worktrees is required to control
the user's simultaneous suites; this branch cannot throttle unmodified scripts.

`python3 tools/resource_budget.py --status` reports kernel-held slots and owner
metadata; stale metadata alone never establishes ownership. Token files are
never removed. Inherited kernel locks retain reservations while
an orphaned workload runs; normal cancellation terminates owned processes and
reaps the direct child. No stale-PID deletion protocol or shared capture lock is
used. Admission is opportunistic, with no strict fairness guarantee. A malformed
manifest, absent counters, nonzero child exit, changed executable or incomplete
result set fails. Build revision and executable SHA256 digests are retained;
assertion logs are read in bounded chunks.
No result or comparison verdict is reused. Independent review remains owed.


## Persistent setup observations

Indexed setup can load the complete source-derived seed sequence, animation
lengths and FrameGuys from `.ron-observations-v1` beside the canonical source.
`RON_SETUP_OBSERVATIONS_DIR` redirects entries; `RON_DISABLE_SETUP_OBSERVATIONS=1`
disables disk reuse while preserving the previous small in-process cache.
Raw-key recording bypasses both observation caches. No comparison verdict,
simulation state, lobby merge or borrowed observation stream is persisted.

The key binds canonical path, logical/stored lengths, held-source mtime and a
fingerprint of parser, codec, index, observation logic and physical/archive I/O.
Finalized-source validation runs before and after use. This retains the existing
metadata boundary; it does not authenticate a source or defeat adversarial edits
that preserve all metadata. Corrupt/stale/incompatible entries fall back to the
original frame/sibling walk. Existing entries are never overwritten or pruned.

The stream contains seeds, animation rows and each ordered clock frame as
separate bounded LZ4 chunks. Duplicate frames and every FrameUnit/Guy field
survive; exhaustive record patterns make added fields a compilation error.
Writing borrows each row, with no whole-sequence clone or encoded buffer.
Reading moves owned observations into Initial. Bounds:32 MiB decoded per chunk,
64 MiB decoder allocation per chunk,100,000 clock frames,512 MiB entry bytes
and512 MiB accumulated observation heap (plus one bounded decoding chunk).
The checksum covers identity, lengths, chunk order and payload; it detects
accidental corruption, not hostile forgeries. Publication explicitly flushes a
create-new temporary file then links it without replacing an existing entry.
Only that invocation's temporary file is removed. Optional write failures leave
source parsing authoritative. The prior8 MiB/32-entry in-process bound is unchanged.

The safe owned-record codec is shared with the parsed oracle. Its source now
participates in both fingerprints; existing oracle generations are retained,
and the move creates a new generation rather than trusting old schema keys.
`RON_VERIFY_SETUP_OBSERVATIONS=1` compares the entire Initial with independently
rebuilt fresh observations. `RON_PROFILE_SETUP_OBSERVATIONS=1` reports disk
hits/misses/publication. Whole original-reader comparisons additionally cover
all87 Golden source entries in fresh processes, with confirmed disk hits.
Source observations remain outside git. See the shared-runner journal for cold,
warm, CPU, RSS and final gate evidence; local setup speedups are not a10× suite
claim. Independent review remains owed.


## Third-pair window replay

The group/projectile test visits the same 24 windows in source-pair/ascending
frame order and keeps one movable Built between nonoverlapping windows. It
rebuilds on source identity/horizon changes or overlap. The locally loaded
install and RON tuning remain fixed for the test; no process-global replay is
retained. Source keys include canonical path, length, mtime and optional-trace
presence. Keys are checked again after each window. This is the existing
finalized-fixture metadata boundary, not protection against metadata-preserving
edits. Missing fixtures retain the previous audit/skip behavior.

A pair is built with observations through its largest requested window end.
Correction inputs are consumed by exact current frame in Built::tick; no future
correction is applied early. The seed-qualified sibling merge needs special
care: this claim is differentially established for every current window, not
assumed for arbitrary captures. The disabled path retains original window order,
per-window observation cutoffs and fresh-from-zero replay. The same setup/lobby/
seed identity assertions run whenever a Built is constructed; immutable identical
setup is not rechecked by rebuilding it for every window. All frame/group-dumper,
ammo-field, key, count, presence, difference and unmodelled-field assertions are
unchanged. Every source/trace fixture request is still recorded per window.

`RON_DISABLE_THIRD_PAIR_REUSE=1` selects that reference path. Global
`RON_DISABLE_PREFIX_REUSE` and the existing debug/trace diagnostic environment
switches also disable reuse. `RON_PROFILE_THIRD_PAIR=1` prints setup/tick/window
times and resumed frames. `RON_VERIFY_THIRD_PAIR_REUSE=1` additionally rebuilds
each window with its original cutoff and ticks from zero. At every compared
frame it checks exact seed/clock input prefixes and the complete Built Debug
value representation, including full Sim, maps, notes, RNG history and correction
state. Future input arrays are temporarily moved out and restored; no state hash
or selected-field projection is used. Existing Debug semantics (including
nonsemantic order-head identity normalization) apply. This audit intentionally
costs extra CPU/memory and is not the normal execution mode.

Focused tests reject incompatible identities, horizon changes and overlap;
mutation killers cover RNG state, draw history, object maps, notes, seed order,
clock omission and site history. Performance and final gate verdict are recorded
in `docs/journal/2026-10-16-third-pair-replay.md`. Independent review remains owed;
no simulation behavior, fidelity pin or value floor is intentionally changed.

Measured third-pair result:24 windows/360 compared frames,256890→23016 ticks;
36.65–38.29→4.97–4.99CPU seconds and3.37–4.26→0.74–0.76GiB peak RSS in two
alternating warm repeats. A four-suite/eight-test sample falls481.34→326.81CPU
seconds and18503→11896MiB sampled tree RSS, while elapsed remains52.4→52.3s.
Coverage is its remaining critical path. Full release ran with one helper-ledger
registration omission; the four-line correction and all affected checks passed
under the documented combined-validation rule. The original gate exit remains1.
See `docs/audit/2026-10-16-third-pair-replay.json` and the journal for the exact
validation scope; this is not a10× full-suite claim.

## Repeated live-unit/radius scans: measured and rejected

The scoped compact-body and bounded-candidate experiments preserved full
replay diagnostics and fixture multisets, but complete isolated suite time/CPU
were flat:165.18→165.57 seconds,1,190.57→1,191.85 CPU seconds. Four/six-suite
samples saved only1.1%/1.6% CPU, with elapsed flat. The simpler axis-only control
also failed to improve coverage CPU. Production simulation and tests remain
unchanged. The negative result and a reproducible candidate patch are retained
in the [radius journal](journal/2026-10-16-radius-scans.md). No memory or10×
improvement is claimed. Next: profile repeated setup/replay across Golden and
harness tests, which together consume about560 of1,191 isolated CPU seconds;
module totals alone do not identify their internal bottleneck.

## Validation entry points and different revisions

All release-gate modes now use shared admission for every phase. Complete,
worker and scoped gates default to isolated test execution. `--combined` keeps
an executable reference; its wide-run ceiling is now32 GiB under the shared
budget rather than the historical40 GiB. Gate selection, widening, missing
fixture rejection and lane failure classification are unchanged. Compiler jobs
are capped at8. Standalone `tools/guard.sh` joins the same budget; a gate marks
its already-admitted guard child to prevent double acquisition.

Use `zsh tools/check.sh test -p sim FILTER` (or `build`, `check`, `clippy`, `run`)
for standalone Cargo work. It preserves Cargo arguments, sets2 default test
threads and8 compiler jobs, and reserves8 CPU slots/32 GiB. Raw `cargo`, native
test binaries and old worktrees still bypass cooperation; this is not a kernel
CPU quota or interception of arbitrary commands. Do not wrap a gate/guard/isolated
dispatcher in another whole-budget reservation. No cache of test verdicts or
shared target directory is introduced. Different revisions run their own binaries.

Every gate phase now retains a log. Supervisors print a JSON timing receipt
separating shared-admission wait from admitted execution (including launch and
cleanup). Isolated per-test rows retain that receipt alongside CPU/RSS. Neither
wait time nor execution wall time is CPU time. The existing fixed policy and
opportunistic admission remain; fairness and short-guard priority are not promised.
Conservative compilation reservations can increase waiting. Measure the queue
before changing budgets or adding priorities. See the
[workflow journal](journal/2026-10-16-validation-workflow.md) for adoption limits,
crate-boundary findings and the required-fixture gate verdict.

### Queue history reads

The ledger guard reads every historical QUEUE/PARKED body through one streaming
`git cat-file --batch` child per file, with a child-local 16 MiB delta cache.
History order and all accounting checks remain unchanged; no verdicts are cached.
On the measured 2,208-body history this reduces wall30.10→2.13s and CPU27.47→2.08s,
with peak RSS50.03→55.38MiB. An exhaustive body comparison against the original
reader and authored deletion failures are recorded in the
[workflow journal](journal/2026-10-16-validation-workflow.md). This is a guard
component improvement, not a measured 10× complete gate.

The default full required-fixture gate passed on `980efbd5`:271.25s wall,
1509.62 CPU seconds,8169MiB sampled tree peak; fixed13/rondata890/sim2586/doctests3
pass,6 unchanged ignored. Exact3276 fixture requests and ordered rondata test
manifest match the reference. The earlier combined gate took427.11s/1438.50CPU/
8089MiB: wall improves with the default executor, while CPU and memory do not.
This is not a new replay speedup or a 10× full-suite result. Warm standalone guard
execution is27.98→4.29s. See the workflow journal for attribution and review limits.

## Incremental release builds

The release profile now uses incremental compilation and256 code-generation
units, retaining optimization level3 and overflow checks. Two small-edit samples
reduce test-only rebuild wall25–26→16–17s/CPU103–108→20s, and sim-edit rebuild
wall44–47→19s/CPU277–289→108–109s. Build memory also falls in those samples.
This adds roughly1.7GiB of local compiler cache in the measured target; fresh
worktrees pay the initial build. No test verdicts or mutable replays are cached,
and no cross-worktree cache hit is assumed. Larger edits may invalidate more work.
The8-test runtime screen retains all458 fixture requests and shows no regression;
the full required-fixture gate passes on `b8494d23`. Whole gate266.23s/1463.74CPU/
9028MiB versus prior271.25s/1509.62CPU/8169MiB: runtime tree peak increases10.5%,
so no full-suite memory reduction is claimed. Exact tests and3276 fixture requests
are retained. See [build journal](journal/2026-10-16-build-invalidation.md)
and [audit](audit/2026-10-16-build-invalidation.json). Reference compiler settings:
`CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`.

## Sweep test partitions

Independent sweep tests now compile in four alphabetical integration targets:
`sweep_a_f`, `sweep_g_o`, `sweep_p_t`, `sweep_u_z`. All736 extracted tests retain
their exact names, original bodies and source paths;1850 other sim tests remain
in the library, preserving2586 total. Shared oracle/install helpers are unchanged.
The sweep generator routes new independent tests; normal guards reject missing,
duplicate or incorrectly mounted registrations. A sweep edit measured21.95→2.21–2.78s,
CPU103.49→2.76–2.83s, sampled build peak1971→212–268MiB. This is8–10× for that
rebuild, not a full-suite10×. Complete required-fixture gate passes on `69e343a3`:
262.15s/1443.50CPU/8360MiB sampled tree. All tests and3276 fixture requests are
preserved; full-suite execution is roughly flat.

Use `zsh tools/check.sh test -p sim --release FILTER` for cross-target selection,
or add `--test sweep_u_z` for a known partition. `--lib` no longer includes these
sweeps and is not a complete sim suite. The release gate still discovers/runs
all targets and applies the same exact filters. See the
[family journal](journal/2026-10-16-test-families.md) and
[audit](audit/2026-10-16-test-families.json) for evidence and limitations.

### Public-API extension

A compiler-checked follow-up moves196 more unchanged sweeps into the same four
partitions via test-only re-exports of the existing public sim API. No production
APIs are widened. Sim totals remain2586: library1654, partitions251/270/190/221.
New generated `crate::` tests still start conservatively in the library; manual
promotion requires compilation. The guard requires the public API bridge where
used, while Rust rejects private access. Regeneration retains ownership.
The newly extracted corner-tile edit measures14.37→2.47s, CPU16.59→13.98–14.00s,
sampled build memory1071→400–404MiB; an existing independent sweep edit remains
2.15s/2.76CPU/229MiB. Full required-fixture gate passes on `f4e683b6`:
255.00s/1396.42CPU/8369MiB; all tests and3276 fixture requests retained. See the
[public-API journal](journal/2026-10-16-public-sweeps.md) and
[audit](audit/2026-10-16-public-sweeps.json). No whole-suite gain claimed.

## Golden replay case modules

The211 golden test bodies now live in ten chapter-based child modules, called
once by wrappers retaining every original `diff::golden::*` test name. Helpers,
private APIs, compared recorder, fixture audit and runner remain shared in the
same rondata crate. All original bodies are byte-identical; no profiles, filters,
assertions or expected values change. Add a new case to its chapter module and
add a same-named `#[test]` wrapper in `diff/golden.rs`; registration checks reject
missing, duplicate, orphan and misdirected cases. Rebuild measurements show
setup-case edits10.27→3.01–3.12s/CPU24.22→3.66–3.69s and a case in the largest
chapter group10.44→4.38s/CPU24.57→6.42s. Shared-helper CPU stays roughly flat.
Combined validation after the complete required-fixture run: three source-scanner
failures corrected and retested, all original checks/3276 fixture requests retained.
One source-guard test added. Final corrected case builds are2.99s/3.50CPU/818MiB
and4.35s/6.35CPU/803MiB. The original gate exited1; no full-gate speedup claimed.
Source guards reconstruct the bodies from the frozen checkout. See the
[replay-boundary journal](journal/2026-10-16-replay-boundary.md) and
[audit](audit/2026-10-16-replay-boundary.json).

## Harness replay case modules

Eighty-five ordinary harness test bodies now use nine source groups beneath
`diff/harness/tests/`, with original wrappers and attributes in `harness.rs`.
An inner module keeps every original body byte, including string indentation.
Shared widening helpers stay in the parent. Names, fixture labels, comparison
coverage, profiles and runner selection remain unchanged. Nested source guards
reconstruct the original logical bodies; registration guards reject missing,
duplicate and misdirected cases. Add tests with a same-named wrapper and body.

The measured run57 diagnostic edit rebuild goes19.52→15.30s, CPU52.64→20.95s,
sampled tree1566→1209MiB. Seventeen and85 groups do not materially beat nine.
This reduces build pressure; no shared-helper, concurrent-worktree or whole-suite
speedup is inferred. The complete required-fixture gate passes on `5491fe87`:
271.28s/1472.54CPU/8864MiB,892 rondata and2586 sim tests, exact3276 fixture
requests retained. Whole-gate cost rises versus the previous successful run;
this is a rebuild improvement, not a suite-runtime gain. See the [harness journal](journal/2026-10-16-harness-boundary.md) and
[audit](audit/2026-10-16-harness-boundary.json). Independent review remains owed.

## Golden near and East Indies observation reuse

Chapter two near reads indexed frame-plus-sibling spans through the original
parser. East Indies uses bounded setup and persistent frame/group/leader/region
observations, with region mode included in cache identity. Every simulation tick
and comparison still runs. Optional `RON_VERIFY_EAST_REUSE` compares complete
reports and setup; `RON_VERIFY_GOLDEN_NEAR` compares each near map. Existing
`RON_VERIFY_ORACLE` and `RON_VERIFY_GOLDEN_SETUP` audit underlying observations.
`RON_DISABLE_EAST_REUSE` and `RON_DISABLE_GOLDEN_NEAR_INDEX` retain raw fallbacks.
The oracle schema change requires one-time repopulation; old caches are preserved.

All45 affected replay tests pass differential auditing. Two new guards fail under
intentional faults and pass after restoration. Local baseline→warm examples:
near10.60→0.59s,1410→398MiB RSS; run2277.63→1.95s,381→256MiB;
run2994.48→2.11s,384→262MiB. Full required-fixture gate passes on `55793bfc`: 894
rondata and2586 sim tests; original3276 fixture requests retained. Complete warm
rondata161.22→145.58s, CPU1158.05→1037.78 (about10% lower); sampled tree8186MiB.
Schema repopulation makes the gate rondata phase165.20s/1185.64CPU. Whole-suite
10×, sub-minute/sub-GiB and concurrent-agent capacity remain unproven.
See [journal](journal/2026-10-16-golden-replay.md) and
[audit](audit/2026-10-16-golden-replay.json). Independent review remains owed.

## Golden civilian observations

Civilian widening now optionally decodes persistent near, cycle/visibility,
world-plane, goods and ammunition observations alongside the existing frame,
leader and pool records. Every simulation tick and comparison still runs.
Ammunition inclusion is part of cache identity; civilian reader source is hashed.
`RON_DISABLE_GOLDEN_ORACLE` retains eager parsing; existing raw-reader opt-outs
and whole-report audit flags remain effective. Raw-key coverage bypasses caches.

All72 affected cases pass complete civilian/pool differential auditing. Three
new guards cover values, bounded codecs and cache-mode separation; deliberate
faults fail. Warm chapter36 improves8.89→1.72s/CPU8.22→1.67; memory765→763MiB.
First-use cost rises to12.21s/1141MiB; this is not a cold-cache memory saving.
Full required-fixture gate passes on `571c0298`;897 rondata and2586 sim tests,
exact3276 fixture requests preserved. A second72-case audit verifies persisted
decodes against raw readers. Complete warm rondata145.58→134.05s,
CPU1037.78→934.29; golden CPU283.12→164.85. Sampled tree memory rises8186→8541MiB;
no aggregate memory improvement is claimed. Whole-suite10× remains unmet. See
[journal](journal/2026-10-16-golden-civilian.md) and
[audit](audit/2026-10-16-golden-civilian.json). Independent review remains owed.

## Bounded whole-capture consumers

City goods/trade, run56 figures and two animal tests now consume bounded setup
and indexed frames. Every original tick/comparison remains; adjacency follows
positions within each file, including duplicate/gap/empty frames. Figure bodies
retain frame-minus-one, first nonempty duplicate and last nonempty position.
`RON_DISABLE_WHOLE_CAPTURE` selects eager parsing; `RON_VERIFY_WHOLE_CAPTURE`
compares full observations with original parsing. Two fault-tested guards added.
Local OS RSS falls2131→398MiB,2053→206MiB and2030→22MiB; four competing copies
of these four tests use28853→1265MiB sampled tree, wall7.54→5.16s andCPU50.26→41.64s.
This selected pressure experiment is not four complete suites. Full required-fixture
gate passes on `d68ee904`:899 rondata/2586 sim; exact3276 requests retained.
Complete warm suite134.05→132.21s, CPU934.29→924.28s; sampled tree8541→7184MiB
(16% lower). Whole-suite10× remains unmet. See [journal](journal/2026-10-17-whole-capture.md)
and [audit](audit/2026-10-17-whole-capture.json). Independent review remains owed.
