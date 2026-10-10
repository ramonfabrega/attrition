# Bound golden setup retention

Item 1646, Codex (GPT-6), base `77450ddf`, a sequential continuation after
[sharing golden replay execution](2026-10-10-shared-golden-replays.md).
One worker; commander remains paused.

## Change and fidelity contract

`stage_script` and `walk_script` used whole golden capture text and four
whole sibling texts to build a simulation. Both now call `build_golden`,
using the established `with_sibling_initials` and
`IndexedCapture::with_replay_initial` callbacks. Borrowed setup text stays
inside those scopes; the returned simulation owns what it needs. The same
primary index then supplies frame comparisons. No new cache, eviction policy,
parser, compression default, source selection, script or simulation change.

`stand_up_initial` extracts the existing construction body. It preserves the
golden capture's own frame seeds and figure clocks across sibling borrowing,
then borrows that run's own pasture trace and builds with `Tuning::RON`.
Other whole-text callers keep `stand_up`. Draw-word walks still require
nonempty own units and leaders before sibling borrowing: the original
assertion is extracted verbatim except its binding name. Their complete
replay body after construction remains byte-identical. Every replay loop, report,
assertion and window is unchanged.

The bounded reader retains all Initial fields except `frame_bodies`, the
pre-existing audit-only body oracle: `build_sim` does not consume it. Original
captures and independent body-comparison tests remain. All seeds, figure
clocks, animation observations, setup records and terrain data are retained.
A cold indexed read still scans frames to collect owned observations; this
is bounded retention, not setup-only I/O. The existing finalized-file and
metadata-validation contract applies. Present malformed sibling input fails
through the established indexed reader rather than being filtered out.

## Verification

`RON_VERIFY_GOLDEN_SETUP=1` compares each bounded primary Initial against the
whole-text reader, clearing only the reference's audit bodies. Exact equality
covers every retained field, not only fields affecting these scenarios. A
success marker includes both run and test name. The audit driver requires
all affected tests to emit a marker, plus the existing complete sibling
Initial regression. The two bounded steps cover 74 staged-replay and 56
draw-word tests, 130 distinct tests. Missing captures cannot count as successful audits.

The sibling regression explicitly revisits bounded inputs before comparing
all fields. Ordinary repeated setup calls also exercise the existing cache
when eligible; large entries can exceed its unchanged 8 MiB budget. No claim
is made that every repeated primary is cached. Normal gates leave the audit
flag unset so full-text audit reads cannot fill optimized-path coverage gaps.

The old executable is retained. Alternating fresh-process measurements use
chapters 35–37, raw fixtures, release debug=1, one test thread, a 20 GiB cap,
and no overlapping build or replay. Each process starts with cold in-process
caches; later tests may reuse eligible sibling observations. OS file-cache
state is not reset. All 343 diagnostic lines must match with multiplicity.

## Measurements

Before **48.28 / 48.75 s**, bounded **46.60 / 46.15 s**: median **48.515 →
46.375 s**, **4.4% less time** in this small group. Peak tree RSS before
**4528 / 4352 MiB**, bounded **1322 / 1283 MiB**: median **4440 → 1302.5
MiB**, **70.7% lower**. All **343 diagnostic lines match**. These are local
samples, not a promised suite-wide gain.

The draw-word counterpart for chapters 35–37 was measured separately after
extending the same constructor. Before **18.06 / 18.08 s**, bounded **15.31 /
15.27 s**: median **18.07 → 15.29 s**, **15.4% less time**. Peak before
**5017 / 5029 MiB**, bounded **1306 / 1322 MiB**: median **5023 → 1314 MiB**,
**73.8% lower**. Its six staging/sequence/value diagnostics match exactly.
Same fresh-process protocol, no overlapping builds or replays.

Stage audit passed: **75 tests**, comprising all **74 affected staged tests**
and complete sibling equality; **77 exact Initial comparisons across 56
runs**, **175.37 s**, peak **5904 MiB**. All 112 used golden files were
nonempty and unchanged by size/mtime. The final constructor's draw-word audit
also reruns the three measured staged replays and sibling equality. That audit **passed: 60 tests**, 59 exact Initial comparisons, **96.71 s**,
peak **7475 MiB** (audit includes full-text references). Combined differential
evidence covers **130 distinct consumers**, **136 Initial comparisons**,
**59 golden runs / 118 required files**. The final common constructor is
checked by all 56 word tests plus the three staged samples and sibling
regression; the first audit covers the staged constructor before its
mechanical extraction. Every subsequent replay check is preserved. All-target
clippy passed on the final constructor. Full required-fixture gate passed below.

## Evidence and reproduction

`target/golden-setup-1646/` retains `rondata-before`, `golden.before.rs`,
build logs, `measure.py`, all before/after logs, `timings.json`, `callers.json`,
`audit.py`, the audit log/summary and golden fixture metadata manifests.
The first compile exposed an input lifetime relation, made explicit in the
extracted helper; the subsequent same-profile build passed.

Build with `CARGO_PROFILE_RELEASE_DEBUG=1 tools/memcap.sh 20 cargo test -p rondata --release --lib --no-run`.
Set `RON_INSTALL=/Users/rf-studio/code/fun/attrition/game`. The staged audit invokes
the resulting binary with `--exact --nocapture --test-threads 2`, the 74 names
in `callers.json`, and
`diff::testkit::indexed_siblings_preserve_every_initial_field_except_audit_bodies`.
`word-callers.json` lists the 56 additional draw-word tests;
`word-audit.py` adds three staged replays and the sibling regression. Its
logs and measurements use the `word-` prefix, retaining the first audit.
Enable only `RON_VERIFY_GOLDEN_SETUP=1` for these audits. Normal full gate uses
`--require-fixtures --test-threads 4` and no audit flags. Scripts use exclusive
log creation; choose a new artifact directory for a rerun.

The fixture recorder covers archived dump requests, not the separate golden
lookup. Require an audit marker for every affected test and independently
check every used golden dump/trace is nonempty and unchanged by size/mtime
through the gate. The initial inventory is not a completeness claim about
unrequested golden files. Independent review remains owed.

Scores: East Indies coverage 3395, Great Sahara coverage 2323, unchanged.
No value floor, draw pin, original capture or shared install is changed.


## Full release verdict

The complete required-fixture gate on **`5e32b45e` passed, exit 0**, all six
steps. The tracked tree remained frozen. Offline 277, fixed 13, rondata
**778 passed / five existing ignored**, sim **1476 passed**, three doc tests.
All **130 affected consumers**, parsed-field coverage and printed-key coverage
passed with every legacy audit flag disabled.

Rondata **477.04 s**, preceding consolidation gate **520.22 s** (−43.18 s /
8.3%). The two completed items take the observed gate from **542.66 to
477.04 s** (−65.62 s / 12.1%). These single four-thread suite measurements
are observational; the controlled isolated comparisons are above. Sim
2.20 s. Peak tree/largest process **12552 / 12521 MiB**, preceding
**13526 / 13495 MiB**. Other replay work still raises the suite peak well
above the converted golden tests; the **71–74%** reduction belongs to the
isolated golden groups, not the suite. All peaks remain under the 20 GiB cap.

Fixture audit: **2765 requests, 386 unique, zero missing**. Every archived
fixture/test pair and its request count matches the preceding gate exactly.
All **118 used golden dump/trace files** remain nonempty and unchanged by
size/mtime. No unrequested-corpus completeness claim. Evidence:
`release-gate.log`, `gate-report/`, `fixture-set-comparison.json`,
`all-used-fixtures.json`, `golden-fixtures-after.json` under
`target/golden-setup-1646/`. About **71 GiB** remains free; original captures
are intact.

This completes the sequential consolidation/setup batch. Before another
broad optimization, profile the remaining full-suite time and memory peaks;
neighboring test names in a concurrent log are not reliable attribution.
Some remaining whole-text readers are independent oracles or consume bodies,
so they must not be mechanically migrated. Independent review remains owed;
East Indies coverage **3395 → 3395**, Great Sahara **2323 → 2323**, no value
floor or draw-pin change. Commander remains paused.
