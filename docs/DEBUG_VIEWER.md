# Differential debug viewer

## 1. Purpose and boundary

The viewer turns an existing differential replay into a standalone HTML file:
original unit positions, Rust positions, displacement vectors, path stacks,
record inspection, and frame navigation. It runs without the original renderer,
a display during export, graphics dependencies in `sim`, or a browser server.
Open the resulting file locally; HTTP hosting is optional. No network resource
is requested by the page. This is diagnostic tooling, not phase 4's game client.

`diff::run_traced_observed` calls a borrowed observer immediately after the
existing frame comparator. `run_traced` delegates to it with a no-op observer.
The simulation, frame stepping, recording application and comparator remain the
same. The callback cannot mutate `Built` through its shared reference.

The displayed state is **after the harness's corrections**. Setup borrowing,
RNG reseeding and figure-state corrections remain active when supplied by the
capture and its siblings. The header reports configured correction counts;
provenance includes the harness notes and whether the displayed tick had a seed
record. Agreement here is not proof of autonomous continuation. A trace or
recording omitted from the command is visibly identified as not supplied.

## 2. Export and inspect

Run from the workspace root, using a new output path **outside the repository**:

```sh
cargo run -p rondata --release --example debug_view -- \
  "$RON_INSTALL" "$CAPTURE" /tmp/replay.html \
  --from 400 --count 30 --sibling "$SETUP_CAPTURE"
python3 tools/viewer/verify_export.py /tmp/replay.html
```

`--sibling` is repeatable and its order is retained. `--trace TRACE` supplies the
existing trace reader; `--recording REC` supplies the existing order stream.
Use the exact inputs of the differential test being investigated. The example
does not infer compatible siblings or install paths. Existing outputs are never
overwritten. Release mode is required. The maximum window is 200 logged records;
embedded data over 64 MiB is refused. These are export bounds, **not an RSS cap**:
use `tools/memcap.sh` for large inputs.

Original positions are blue circles; Rust positions are outlined orange diamonds.
Coordinates and deltas remain exact integers in the inspector. The canvas uses
floating point only to project them onto pixels. Out-of-scope owners are gray;
missing links and Rust-only units have explicit states. Select by owner/object,
click a marker, pan, zoom, fit the map or focus a unit. Selected path stacks are
shown from their top toward the goal. Empty lists can mean unlogged data.

The timeline steps captured records without interpolation, including gaps or
repeated frame labels. Selection links encode the exported record index, unit
and inspector mode. Playback is a viewing cadence, not the original clock.
“Next mismatch” and timeline marks include **all** existing comparator issue
categories; the position-only filter affects map markers. Zero position errors
can coexist with order, angle, collision, building or other disagreements.
The full frame comparison remains available alongside each unit's parsed original
record and Rust record. The raw records remain debug representations. Section 5 describes the typed
field-difference table, which is now the default inspection mode.

The reproduction download records source paths, capture byte size, revision,
inputs, harness notes, application counts, frame index and the export command.
It is a diagnostic manifest, not a self-contained replay bundle: source files
must remain available and unchanged, and `git describe --dirty` cannot identify
uncommitted changes precisely. Choose a new output path when repeating it.
Generated files contain local capture records and belong outside git.

## 3. Validation and remaining work

The observer regression runs the same real capture twice, comparing the entire
returned `Report` and every callback result. The export verifier independently
recomputes paired-position and displacement counts from the JSON and checks
identity uniqueness and consecutive source indices. A deliberate coordinate
corruption was rejected. The embedded-string test covers a closing script tag,
newline, quote and backslash; records are displayed using `textContent`.

Two real run6 windows, 350–379 and 400–429, each exported 30 records and 450
paired positions with zero positional disagreements. The latter uses the four
canonical setup siblings in `diff::testkit::sibling_texts` order. Neither supplies
a recording or separate trace. The later artifact is 7,700,662 bytes; a
`tools/memcap.sh 20` run sampled 1,353 MiB peak resident memory across its process
tree. Browser checks covered mismatch navigation, provenance selection, deep
links, focus and filtering; a separately labeled synthetic displacement showed
the exact (+1200, −600) coordinate difference and its connecting line. These windows exercise other reported field
issues; they are not evidence of a new parity floor. No queue score changed.

The harness now decodes compared frames incrementally and omits unused figure-
position audit observations from replay setup. Initialization still scans the
whole capture for correction inputs and animation lengths, and replay starts at
setup before reaching `--from`. See `audit/2026-09-09-streaming-replay.md`. The HTML retains full selected unit debug records,
which dominate its size. Terrain, borders, per-figure overlays and building
markers are not exported yet. No new original-format claim is made here.

Section 4 now implements failure-window export for the first test integration.
Extending that integration to other assertion families comes next. Section 5 adds field-aligned disagreements. Compact shared unit metadata remains
a possible artifact-size improvement.

## 4. Failure artifacts from tests

`debug_view::Window` now owns the serializer shared by the command-line exporter
and tests. `write_failure` takes the original `Report`, an optional failing source
record index and reason, metadata, an output path, and a replay closure. With no
failure it performs no replay, serialization or filesystem work. On failure it
retains up to eight records before and eight after the selected record, then
opens the viewer on that record. Selection uses source indices, so repeated
frame labels do not shift the window.

The closure receives a window to observe and uses the test's **same in-memory
inputs**. It must reconstruct any consumed order stream. Before writing anything,
the entire repeated `Report` must equal the original: a changed stream or setup
is an explicit diagnostic error, not a misleading artifact. This deliberately
reruns the complete test replay on failure; passing runs pay no second replay
cost. The HTML window is bounded, but the replay's original memory costs remain.

The first integration is run69's whole-capture unit-position and building-field
assertions, plus its current-waypoint assertion before the word. It selects their earliest failing frame, repeats the existing loaded
data, log, sibling references and trace, and prints the artifact path. Output
uses a unique file under the OS temporary directory's `attrition-diff-failures/`.
No additional environment configuration is needed. Artifact errors are printed
without replacing or suppressing any existing assertion. Other run69 assertions
and other tests are not automatically covered yet: each needs to identify the
record relevant to its own acceptance rule, rather than export known residue.

For this integration, provenance identifies borrowed sibling values by their
index in `testkit::sibling_texts()` and names the parsed trace; the reproduction
command reruns the test. These labels are not invented file paths or a portable
bundle. The command needs the same install and capture environment as the test.

Tests exercise the zero-work passing path, changed-report refusal, overwrite
refusal and a deliberately selected acceptance failure over real, unmodified
replay records. The latter retains source indices 2–18 around index 10 and writes
a clearly labeled local test artifact; it does not assert that the original game
was wrong. The existing independent JSON verifier still checks the exported
coordinate counts against the comparator.

The shared serializer was checked against the pre-refactor canonical run6 export:
all 30 complete frame records and all harness notes were equal, beyond matching
the 450 paired-position counts. The intentional acceptance-test artifact contains
204 paired positions over 17 records; its 82 position disagreements match the
comparator. That test deliberately supplies no setup siblings, trace or recording.
Browser inspection confirmed initial selection of frame 11 (source index 10),
with the exact disagreement for unit 0/1 displayed as (+110, +168).


## 5. Field differences

New exports include a structured presentation of every disagreement category in
`FrameResult`: position and presence, angle, sight, packed state, collision,
orders and paths, building identity, gather records, production queues and cities.
The table reads typed comparator results; it does not parse debug strings or
perform another simulation comparison. Original and Rust values are JSON strings
so even the extremes of `i64` reach the browser without rounding. Unit, building
and city identifiers have distinct prefixes. Missing building/city links carry
aggregate counts because the report does not provide their identities.

Filter by entity, field, value or classification, or show only the selected unit.
A unit link selects its map marker. Buildings and cities remain inspectable in the
table without pretending they have map markers. Paths name their bottom-first
slot, matching the comparator, while the map draws the stack top first. The table
contains disagreements only: an absent row does not establish that a field was
logged, compared or implemented. Coverage counts and the full report remain
available. Old exports without these optional rows display an explicit fallback.

Order rows retain `OrderMismatch::scores()`: reported-only flags and other
excluded fields are labeled as such. A source header/type inconsistency shows
both original statements and explicitly says it is not a Rust value comparison.
An unmodelled order kind is labeled as a structural gap. The order-variant match
is exhaustive, so adding a variant requires its presentation to be considered.

The export verifier checks that the row weights cover the entire existing issue
count, in addition to its coordinate checks. Tests pin score exclusions, header
semantics, slot numbering, integer extremes and aggregate coverage. A deliberate
removed row is rejected by the verifier. The canonical 30-record run6 window
exports 449 difference rows while retaining its original records and 450 paired
positions unchanged. On frame 400, unit 1/0 has matching coordinates but
`order[0].move.facing` is original 1 / Rust 0, explicitly excluded from the order
score. Browser inspection verified filtering and selection on that record.

Failure artifacts now also cover run69's already-existing assertion that
`dest_x` and `dest_y` agree before `LONG_WORD_GREAT_LAKES`. This is the test's
existing predicate, not a new score requirement; other reported order residue
does not trigger an artifact. No acceptance assertion was weakened or removed.
