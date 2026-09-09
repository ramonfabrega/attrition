# Lab history before the ledger

Preserved branch-owned journal appendices; future findings go in LEDGER.md.


## 2026-09-09 — methodology exploration: stream capture frames

Under the user's exploration mandate, added safe file-backed frame iteration
without changing the borrowed gamelog parser or advancing a mechanic floor.
Complete Frame equality against the existing reader passed across five captures
(726 frames, 89,811 unit records). Migrating the frozen-clock test reduced its
isolated peak RSS from 1,313.4 MB to 90.7 MB, with test-body time 3.84 s versus
4.07 s; this is an observed single-test result, not a suite-wide reduction.
The full release suite passed. RSS sampling failures now fail closed, including
loss after launch, with injected-failure regression checks. The input boundary,
measurements and process-isolation follow-up are recorded in
`docs/lab/2026-09-09-streaming-captures.md`; the next reader migration is the
replay harness that currently decodes all frames before applying its limit.


## 2026-09-09 — native profile of the census

At the user's request to own the performance graph, sampled the 95-capture
census with release debug symbols. `frame_states` occupied 65.5% of its test
thread's samples, although the census only asks about one frame number.
Selecting matching indexed blocks with the existing record reader kept all
95 census assertions and reduced an unsampled test body from 78.61 s to
28.91 s. Peak RSS rose from 6.28 GB to 6.72 GB in that pair; this is a speed
win, not a memory claim. The after-profile contains no `frame_states` stacks.
Reusable native sampling and graph tools, measurement caveats, and the next
storage/iteration experiments are in `docs/lab/2026-09-09-native-profile.md`.
No simulation rule or acceptance floor changed.

## 2026-09-09 — stream shutdown tails

Extended the safe offset reader to retain the last frame and its shutdown
siblings. Complete closing-record equality passed against the full reader on
102 archived captures (67 closing states, 6,442 units). The census's unchanged
95-capture assertions now run at 260 MB maximum RSS versus 6.52 GB, with test
body 27.40 s versus 27.16 s in a fresh sequential pair. The closing/ordinary
comparison now selects only its matching and preceding populated frames;
36 pairs and 110 unit movements are unchanged, at 174 MB isolated peak RSS.
`docs/lab/2026-09-09-shutdown-streaming.md` records boundaries and validation.
No acceptance floor moved. The next replay change must separate initial setup
from `Initial`'s whole-capture observation products before promising that a
prefix request avoids all suffix decoding.

## 2026-09-09 — A read-only differential debug viewer

Added a standalone HTML export over the existing replay comparator: original
and Rust unit positions, path stacks, integer deltas, frame navigation, complete
comparison records and a reproduction manifest. The borrowed observer preserves
the full `Report` in a real-capture regression. No sim dependency or acceptance
floor changed. Captured setup, RNG and figure corrections are labeled explicitly;
this is a view of the existing assisted harness, not autonomous parity.

Two run6 windows independently verified 450 paired positions each against the
harness counts. A deliberate coordinate corruption failed the export verifier;
a labeled synthetic displacement exercised the browser's error display. The
30-frame canonical-sibling export is 7.7 MB and sampled 1,353 MiB resident memory;
whole-capture setup/parsing remains the next memory boundary. Generated capture
records stay outside git. `docs/DEBUG_VIEWER.md` covers usage and limitations.

Validation: release gate 256 rondata tests passed, one ignored; 821 sim, 13 fixed
and three doctests passed. Rondata took 239.56 s; sampled process-tree peak was
10,874 MiB under the 20 GiB ceiling. The exporter escaping test, seven paperwork
guards, clippy with warnings denied, fmt and install survey passed. Next tooling
step: export the first failure automatically with the exact test inputs, then
compact repeated metadata and align non-position fields for inspection.

## 2026-09-09 — Failure-selected replay artifacts

Moved the viewer serializer into `rondata::debug_view`, shared by the CLI and
assertion diagnostics. `write_failure` receives a test-selected record and
replays the test's own closure, retaining eight records either side. It refuses
to write unless the complete repeated report equals the first, and does no
replay or filesystem work for a passing check. Unique temporary outputs preserve
existing files; the viewer opens at the selected failure and displays its reason.

Run69 now exports context for its earliest unit-position or building-field
failure, using its already loaded install, log, siblings and trace. Existing
assertions remain intact even if artifact production fails. Other assertion
families still need explicit integration; known comparator residue is not a
universal failure predicate. No queue or acceptance score changed.

The canonical CLI export retained every frame record and harness note across the
refactor. Tests cover passing-path nonexecution, changed-report refusal,
overwrite refusal and an intentional acceptance failure on real replay records.
The 17-record test artifact independently agrees with the comparator on 204
paired positions and 82 disagreements; no setup siblings are supplied in that
test. The browser opens at source index 10, frame 11, and shows unit 0/1's exact
(+110, +168) difference. An invalid focus index fails the export verifier.
`docs/DEBUG_VIEWER.md` section 4 documents scope and costs.

Validation: release suite passed 260 rondata tests (one ignored), 821 sim tests,
13 fixed tests and three doctests. Rondata took 245.63 s; the monitored tree
peaked at 10,023 MiB under its 20 GiB ceiling. Clippy with warnings denied, fmt,
seven paperwork guards and the install survey passed. The documented run69
reproduction command was checked to select exactly that test.

## 2026-09-09 — Typed field disagreements in the viewer

The viewer now aligns Original and Rust values directly from every disagreement
category in `FrameResult`. It includes unit, order/path, building, production,
gather and city fields; unlinked building/city identities remain aggregate
coverage counts because the report has no identities for them. Values stay
strings through JSON, preserving integer extremes. Order-score exclusions,
unmodelled kinds and source header inconsistencies retain their distinct meaning.

The table filters by entity/field/value and can follow the selected unit. On
run6 frame 400, unit 1/0 agrees in position but has `move.facing` 1 versus 0,
correctly labeled as excluded from the order score. The canonical 30-frame
window contains 449 field rows, all original records unchanged, and 450 paired
positions. Removing one row deliberately fails the issue-count verifier.
Browser inspection checked filtering and linked unit selection, and exposed an
initial zero-size canvas fit in the narrow panel; the resize path now recovers
from a zero or non-finite scale.

Run69's diagnostic hook now also covers its existing current-waypoint assertion
before the word. The acceptance predicate is unchanged and unrelated order
residue does not trigger diagnostics. No queue or parity floor moved.
`docs/DEBUG_VIEWER.md` section 5 records semantics and coverage limitations.

Validation: full release suite passed 262 rondata tests (one ignored), 821 sim,
13 fixed and three doctests. Rondata took 249.25 s; sampled tree RSS peaked at
10,653 MiB under the 20 GiB ceiling. Clippy with warnings denied, fmt, seven
paperwork guards and install survey passed. The two new presentation tests cover
lossless values and the comparator's score/source distinctions.

## 2026-09-09 — Stream replay comparisons and omit audit-only retention

The replay harness now consumes one typed frame at a time, stopping decoding at
its record limit. `frame_states` remains the collecting API. The new crate-private
replay setup reader keeps all animation, seed and clock-correction inputs but
omits the figure-position audit series, which `build_sim` never consumes. Public
`initial` remains complete. The harness drops setup after building its owned
simulation inputs. No simulation arithmetic or acceptance floor changed.

On isolated run69 with no siblings/trace/recording, 30 records fell from
1,586,479,104 to 530,497,536 bytes peak RSS and from 3.273 to 1.691 seconds of
replay time. All 3,000 records fell from 1,623,834,624 to 567,803,904 bytes, with
replay time essentially unchanged (3.312 versus 3.272 seconds). Saved binaries
produced identical complete reports at both limits. The canonical viewer export
also retained all 30 full frame records and notes unchanged with its four siblings.
`docs/lab/2026-09-09-streaming-replay.md` records the intermediate measurement,
method, report serialization caveat and remaining resident-source boundary.

Tests cover visitor limits and duplicate labels against direct block traversal,
and exact replay setup equality after clearing only the unused audit series on
both a position-only capture and a capture with seeds and clock corrections.
The standalone measurement probe saves complete reports outside git and buffers
its output so future timing does not pay one syscall per debug-format token.

Validation: full release gate passed 264 rondata tests (one ignored), 821 sim,
13 fixed and three doctests. Rondata took 240.61 s; sampled tree peak was
8,757 MiB under the 20 GiB ceiling. Clippy with warnings denied, fmt, seven
paperwork guards and the install survey passed. These suite measurements are
reported separately from the isolated paired replay measurements above.


## 2026-09-09 — Optional indexed differential replay

Continued the exploration branch's memory work without taking a queue item.
The indexed capture reader now supplies replay setup and one compared frame at
a time. The existing and indexed entry points share ticking, recorded-order
application, comparison and report assembly. The viewer exposes the new lane
with `--reader indexed`, keeping the existing default.

Complete run69 reports match at 30 and 3,000 records, including the preceding
commit's saved reports. Peak RSS fell from 530,497,536 to 79,413,248 bytes for the
prefix and from 565,444,608 to 117,751,808 for the full run. Source loading plus
replay rose from 2.006 to 3.705 seconds and 3.539 to 6.078 seconds respectively;
this is a memory tradeoff, not a demonstrated throughput win. The updated probe
times source loading too, so its seconds are not directly comparable to the
previous entry's replay-only timer.

The setup selector preserves late first lookup children, trailing GAME fields
and non-GAME roots; whole-capture correction observations are accumulated from
bounded segments through the existing parser. Six real complete setups agree.
Synthetic cases cover the late/flat/blank/duplicate/no-frame boundaries and
changed-source refusal. A real release test covers zero and 12 records with a
sibling. The canonical viewer's four-sibling export has identical full frame
records and notes across 30 records / 450 positions. Details, measurements and
remaining memory bounds are in `docs/lab/2026-09-09-indexed-replay.md`.

Validation: full release gate passed 267 rondata tests (one ignored), 821 sim,
13 fixed and three doctests. Rondata took 242.08 s; sampled process-tree peak
was 8,618 MiB under the 20 GiB ceiling. Clippy with warnings denied, formatting,
seven paperwork guards and the real-install survey passed. The sampled suite
peak is distinct from the isolated process high-water marks above.


## 2026-09-09 — Select setup ranges during indexing

Profiled the new indexed replay lane: it spent 0.895 s scanning a 467.8 MB
capture to retain 10.8 MB of setup text. Selecting byte ranges during the index's
existing scan removes that pass; reading selected setup now takes about 0.001 s.
The ranges coalesce, retain no source text, and count toward the bounded index
cache. Late fields and roots retain their existing semantics.

Paired full run69 replay fell from 6.044 to 5.141 s; a 30-record prefix fell
from 3.754 to 2.927 s. Complete reports agree at both limits, as do all eight
reports in a four-process comparison of both current readers. That comparison
favored the in-memory reader (5.092 s per batch versus 6.879 indexed), so no
default or concurrency setting changed. The method and limitations are recorded
in `docs/lab/2026-09-09-replay-setup-ranges.md`; the next measured target is
correction-observation parsing, not a speculative increase in test concurrency.
Added a release-only phase probe and a regression test covering exact retained
text, skipped bytes, cached range reuse and cache byte accounting. No queue
score moved.

Validation: the full release gate passed 268 rondata tests (one ignored),
821 sim, 13 fixed and three doctests. Rondata took 248.11 s; sampled tree peak
was 9,195 MiB under the 20 GiB ceiling. Clippy with warnings denied, formatting,
seven paperwork guards and the real-install survey passed. This suite run is a
correctness gate, not an isolated performance comparison; setup verification
and viewer export also ran while it was active.


## 2026-09-09 — Select clock observations before full unit decoding

Continued profiling the replay core loop. Observation collection now visits the
same unit blocks as the full record parser, rejects units without clock fields
before constructing typed records, and omits discarded building/leader products.
Selected units retain all figures and correction fields. Public full setup still
keeps the figure-position audit series; seeds and animation lengths are still
collected independently of clock selection. No queue score moved.

Paired run69 replay improved from 3.579 to 2.844 s in memory and 5.100 to 4.396 s
indexed, with essentially unchanged peak memory and byte-identical complete
reports. Independent before/after reports also match on clocked/full-dump run13,
run20 and run82. A synthetic differential test covers incomplete and invalid
clocks, zeros, mixed figures, animal wrappers and both record nestings. Details
and measurement limitations are in
`docs/lab/2026-09-09-selective-clock-observations.md`.

Validation: the full release gate passed 269 rondata tests (one ignored),
821 sim, 13 fixed and three doctests. Rondata took 228.40 s; sampled process-tree
peak was 8,881 MiB under the 20 GiB ceiling. Clippy with warnings denied,
formatting, seven paperwork guards and the real-install survey passed. The suite
run overlapped report verification and viewer export, so its elapsed time is
reported as gate evidence rather than a controlled throughput comparison.


## 2026-09-09 — A strict capsule of a live-derived mutating call

Returned to the original-game lane after the replay memory work. The standalone
CommandPackage clear hook was not reached in the first bounded pilot. The
successful probe instead invokes that original function on a byte-for-byte copy
of the live 536-byte package after the existing frame-20 MoveTo issuer. This is
a probe-created call, not a natural call or synthesized scalar fixture.

The capsule replays twice in fresh Unicorn instances, mapping only 572 captured
bytes across page allocations. All eight GPRs, EFLAGS and captured bytes match
the live exit. Two writes are verified, including the equal-value seed write:
length 37 becomes zero; padding seed remains 12345. Uncaptured byte ranges are
rejected even within mapped pages; there are no imported-function adapters or
full-image data mappings. Executable/tracer identity and trace receipts are
checked. A matched normal control agrees on all 19 logged frame bodies and
37 frame/RNG records. The real command packet is preserved.

`docs/lab/2026-09-09-live-call-capsule.md` records the narrow contract, local
artifacts and the remaining gap to pointer-rich mutators and automatic dependency
capture. The PDB type record also corrected FORMATS' old 514-byte payload label:
there are 512 payload bytes, two alignment bytes, then Random padding.

Eleven synthetic strict-replay guard tests and five staging tests pass. Default
and experimental tracers compile with warnings denied; default preprocessed
tokens match the branch's prior tip. The staging helper now targets Logging
Options keys by section after an actual duplicate-key-in-another-section refusal.
All backed-up shared settings and profile files were restored byte-for-byte;
new game files are retained. No production Rust behavior or parity floor changed.

Final checks also passed workspace clippy with warnings denied, formatting,
seven paperwork guards and the install survey. Both unsupported capsule flag
combinations were deliberately compiled and rejected. The shared settings and
all originally backed-up profile files match the pre-experiment bytes; the
launched game processes are closed.

## 2026-09-09 — Sparse pointer-graph mutation and reset

The next capsule target, Guy's angle calculation, now runs its nested turn-speed
call with explicit field-sized dependencies: 453 declared bytes, 44 KiB of guest
mappings. All 4,096 authored cases match Rust on the output-pointer heading and
returned remainder. All match again in reverse order and in fresh emulators;
removing each of 18 regions fails. Reused execution measured 0.868 s versus
2.384 s fresh for the same cases; this is a local experiment, not a whole-loop
speed claim. CPU/data reset and write-before-read scratch prevent stale state
from supplying dependencies. Eleven authored guard tests cover the new executor.

`docs/lab/2026-09-09-bounded-turn-angles.md` records the evidence and scope.
Natural live-call capture remains outstanding: the leaf exit witness would
clobber this nested call's dead stack, so it cannot be reused unchanged. This
tranche establishes the bounded replay and Rust comparison side first. No parity
floor or production sim behavior changed; original-derived rows remain in /tmp.

Validation: 11 new executor tests, 11 existing capsule tests, all 4,096 composed
original/Rust comparisons, and both Rust-checker negative controls pass. Full
`cargo test --release` passes: 269 rondata (one ignored), 821 sim, 13 fixed,
and three doctests; rondata took 230.45 s. The memory watchdog reported a
9,115 MiB process-tree peak under its 20 GiB ceiling. Clippy with warnings denied,
formatting, seven documentation guards and the install survey pass. A final
executor rerun also passes every fresh/reversed/omitted-dependency check.
The whole-export caller search points to boat-collision handling, so reachability
needs a witness before assuming the earlier land-unit capture can exercise it.

## 2026-09-09 — A natural pointer-graph call, and fail-closed lobby input

Existing coverage selected Unit's order-cache refresh: frame-zero HITs in runs
53, 54 and 906. The opt-in witness now captures a natural invocation, and two
fresh bounded emulators plus a reset replay match all captured registers,
flags and memory. The 14-instruction replay uses 98 declared bytes in 16 KiB
of guest mappings; it verifies three equal-value stores and rejects five
missing dependencies. Zero live unit bytes changed in this sample. Rust's
order representation differs; no Rust layout-equivalence claim is made.
The normal control agrees on 19 complete logged frame bodies and 37 frame/RNG
records. `docs/lab/2026-09-09-natural-order-capsule.md` records scope and evidence.

One launch faulted in Wine before the first frame; the same staged binary
succeeded on retry. The failure exposed a control bug: no game window counted
as focus success, and the lobby ignored focusing errors. Both now stop input
on failure. Four mocked tests pass; both no-click tests fail against the old
scripts. Eight capsule framing, 11 bounded executor, 11 prior capsule and five
staging tests pass, along with clippy, formatting, documentation guards and the
install survey. Default tracer tokens are unchanged; experimental/default builds
pass with warnings denied. All five backed-up settings/profile files match the
pre-experiment bytes and the launched games are closed. No parity score moved.

The same live capsule now supplies 4,096 deterministic stale-cache mutations:
zero, all-one, and seeded random values in cached data/node/metric. Every case
reconstructs the full live exit state and matches a fresh emulator. This is a
nonempty-list cache-independence check, not broader queue or gameplay coverage;
`replay_order_capsule.py --mutations 4096` makes the experiment repeatable.

Final release gate passes: 269 rondata (one ignored), 821 sim, 13 fixed and
three doctests. Rondata took 228.98 s, with the watchdog recording a 9,710 MiB
process-tree peak under its 20 GiB ceiling. The whole-run comparator also rejects
an altered logged frame body and a missing capsule receipt.

## 2026-09-09 — Natural path deletion, original → bounded replay → Rust

The natural path-deletion capsule observed 2 → 0 waypoints at frame 36 during
shutdown. The original replay matches live registers, flags and captured gameplay
memory; scratch is initialized-on-write and compared across fresh/reset emulators,
not against the live exit callback's overwritten stack. Nine omitted-region
controls fail. The 35-instruction call uses 167 declared bytes in 16 KiB of guest
mappings, including a nested null-search check without mapping its allocator path.

The existing Rust deletion loop is now `Unit::discard_current_path_segment`,
called by production Sim order teardown and by the lightweight oracle example.
The natural result and 4,096 deterministic mutations agree on every surviving
waypoint field; 682 cases retain waypoints. No world or simulation is constructed
by the checker. The normal control agrees on 19 full logged frame bodies and
37 frame/RNG records. All five original settings/profile files were restored,
and the game is closed. No parity score moved. Scope and reproduction are in
`docs/lab/2026-09-09-natural-path-capsule.md`.

Validation: all 4,097 original/Rust rows pass, including 682 retained prefixes.
Eight new framing tests and the prior capsule, executor, staging and focus
regressions pass. Wrong survivor counts and empty oracle input fail; altered
live registers/waypoints/lengths fail; the control rejects changed frame bodies,
missing receipts and receipts for unobserved frames. Clippy, formatting,
documentation guards and the install survey pass. Default tracer tokens are
unchanged; default/experimental builds deny warnings and incompatible flags fail.
The full release gate passes: 269 rondata (one ignored), 821 sim, 13 fixed,
three doctests; rondata 230.02 s, process-tree peak 9,258 MiB under 20 GiB.

## 2026-09-09 — Mismatch reduction into a permanent synthetic regression

A persistent Rust worker agrees with the original on all 4,096 path cases. To
validate failure handling, the FINAL bitmask predicate was temporarily replaced
with exact equality in an isolated worker build, then source restored and the
correct worker rebuilt. The injected fault diverged on case 9. Reduction took
24 evaluations and left two zero-coordinate waypoints, flags 0 and 3: the
original retains the earlier segment and the mutant deletes it. Five smaller
neighbors fail to preserve that mismatch, each certified with a fresh original
engine and independent Rust process.

The authored minimal case is a permanent install-independent Rust regression;
it fails under the injected source fault and passes after restoration. Nine
reducer/IPC tests distinguish disagreements from crashed, timed-out or malformed
oracles and require explicit local minimality. Reports carry source/capsule/worker
identities and stay outside git. The local clean search took 0.741 seconds; fault
search/reduction/fresh certification took 1.034 seconds. This is the two-entry,
no-search path domain, not new native coverage or a newly discovered production
bug. No parity score moved. `docs/lab/2026-09-09-path-counterexample-reduction.md`
records the method, limits and artifacts.

Validation: the full release gate passes: 269 rondata (one ignored), 822 sim,
13 fixed and three doctests; rondata 229.49 seconds, process-tree peak
8,663 MiB under a 20 GiB ceiling. Clippy with warnings denied, formatting,
documentation guards, the install survey and nine reducer/IPC tests pass.

## 2026-09-09 — Bounded suspended-search recycler teardown

The original clears authored open-search trees through its real recycler code
in a 44 KiB guest mapping, with 5,208 declared bytes and no mapped PathNode
payloads. All 6,144 combinations of node count (0–31), three tree shapes,
optional empty containers and initial pool occupancy pass ownership and complete-
record checks, reverse-order reset runs and fresh engines. The initial checked
sweep took 11.459 seconds; fresh runs took 15.425 seconds, with different checking
overhead included. This is synthetic cleanup coverage, not live suspension,
resumption, a Rust comparison, host RSS or a parity-score improvement.

All 31 missing-region controls fail, as do eight corrupted outputs and growth
at each of seven full recycler pools. Every failed growth attempt is followed by
an exact clean-baseline replay on the same engine. This earns explicit recycler
state as a bounded test dependency without importing the engine allocator.
`docs/lab/2026-09-09-search-cleanup.md` records evidence and limits. Natural
graph capture remains the next step before expanding into search resumption.

Validation: full release gate passes (269 rondata, one ignored; 822 sim;
13 fixed; three doctests), rondata 231.10 seconds, process-tree peak 9,041 MiB
under 20 GiB. Clippy with warnings denied, formatting, documentation guards,
the install survey and 11 bounded-executor tests pass. The release log is
`/tmp/search-cleanup-release.log`.

## 2026-09-09 — Measure the missing suspension before capturing its graph

The streaming census accepts all 87 retained traces and finds 82 paired A*
returns, none suspended. A minimal live proxy lane then agrees with the retained
36-frame control on 19 full logged bodies and 37 frame/RNG records. Its three
searches all succeed. A scripted run through 8,000 observes 139 successful
searches, including 47 unit-grid searches, and still no suspension. It preserves
the early control window and has all 8,001 frame records; the later gameplay is
not a fidelity comparison. No parity score moved or natural graph was captured.

`RON_SEARCH_CENSUS` now records container/pool sizes only on suspension, with
checked reads, a 64-event cap and explicit completion/error records. Default
tracer tokens are unchanged. Nine parser tests and the exact callback's host
fixture reject missing records, malformed input, all 30 failed/short reads and
missing cap receipts. Four incompatible build combinations fail. Seven staging
tests cover the new bounded endpoint and scripted fast-forward, including
restoration and refusal before settings change.

An interactive normal-speed long attempt paused in chat and was discarded as
evidence; its test process was closed and settings restored before the scripted
restart. The successful captures remain outside git, the game is closed, and
all five backed-up settings/profile files are verified restored. The next
capture should target congestion/obstacles: this passive seed is demonstrated
not to exercise the desired branch through 8,000. Scope and artifacts are in
`docs/lab/2026-09-09-search-census.md`.

Validation: full release gate passes (269 rondata, one ignored; 822 sim;
13 fixed; three doctests), rondata 231.67 seconds, process-tree peak 9,064 MiB
under 20 GiB. Clippy with warnings denied, formatting, documentation guards and
the install survey pass. Default and census tracer builds deny warnings; the
default preprocessed tokens match the prior commit. Gate log:
`/tmp/search-census-release.log`.


## 2026-09-09 — A repeatable natural suspension scenario

Targeted congestion turns the passive census's zero witnesses into 1,787 native
A* suspension returns over 1,400 frames. Two launches match all 1,401 observed
frame/seed pairs, roster/order receipts, ordered search returns and the first
64 metadata shapes. The first witness is frame 223; its five containers total
202 entries. The captured maximum simultaneous total is 1,059, so the synthetic
cleanup fixture's 31-node bound cannot stand in for a live graph. The new opt-in
driver uses the original spawn and MoveTo paths, never writes the search budget,
and rejects missing/stale identities and overflowing packets. Tests and the
repeat validator turn setup failures into errors. Original settings restored.
No headline floor moved; no full-state parity or Rust resumption claim. Evidence
and the next complete-graph capture boundary:
`docs/lab/2026-09-09-congestion-probe.md`.

## 2026-09-09 — A natural graph in a 52 KiB native replay

The congestion witness now yields a validated structural graph: 223 physical
nodes versus 202 logical entries, because the open reference index retains 21
removed nodes; 67 PathNodes close their own parent chains. Native payload
disposal runs on the captured recycler without an allocator adapter: 3,484
semantic bytes, 52 KiB mapped, 1,807 instructions. Fresh/reused executions
agree; 73 missing dependencies and four corrupt outputs refuse. Full cleanup
explicitly stops at its first container allocator boundary. The instrumented
run matches the earlier scenario's RNG/order/search projections and all original
settings are restored. No headline moved; no resumption or full-cleanup claim.
Evidence: `docs/lab/2026-09-09-natural-search-graph.md`.

## 2026-09-09 — Capture the restore before it succeeds

Native restore entry is captured at frame 224 for owner 0/object 16. The
wrapper reaches its live delegation registers, stack and modes exactly in
31 instructions with 36 semantic bytes and 32 KiB mapped. The subsequent
native A* call succeeds; waiting for another suspension would miss it.
The larger pathfinder's first uncaptured input is Windows FS:[0] exception
state, reached after five instructions with the captured graph available.
This narrows the next dependency experiment but is not resumed-search replay.
Reference scenario projections match; settings restored; no headline moved.
Evidence: `docs/lab/2026-09-09-restore-entry.md`.

## 2026-09-09 — Drive the native lobby without clicks

An opt-in native menu/modal driver now acquires both maps through frame 1400
without mouse or keyboard input. Map and seed read-back, contiguous frame
receipts, closing dumps, controlled post-match exit, and all five restored
settings files are verified. The shared prefix agrees with independent short
captures on 18 complete logged bodies and 37 RNG frame/seed pairs per map.
An incorrect modal ABI wrapper failed live and now has a compiled x86
regression; a mutated argument offset is rejected. Two pre-menu startup
failures remain unexplained, and full application teardown still faults, so
this is an opt-in acquisition tool, not reliable device-free execution.
No headline moved. Evidence: `docs/lab/2026-09-09-autostart.md`.

## 2026-09-09 — Skip native intro playback and compare complete acquisitions

The unattended launcher now passes the original's separately parsed
`+skipIntro` flag and records its exact arguments. Three two-map trials reached
frame 1400 with six successful exits and verified restoration. Every trial
matches the preceding successful capture on all 19 logged bodies, including
closing 1401, and all 1401 frame/seed pairs. A streaming comparison tool now
checks that projection with negative fixtures. This improves the evidence from
a shared prefix to the full observed run; it does not prove full-state parity
or diagnose the earlier pre-menu faults. No headline moved.
Evidence: `docs/lab/2026-09-09-autostart.md`, intro-skip follow-up.
