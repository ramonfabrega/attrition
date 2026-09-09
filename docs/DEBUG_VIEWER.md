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
record and Rust record. These are debug representations, not a field-aligned diff.

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

The bounded window does not make setup or frame parsing lazy: the current
harness still constructs whole-capture observations, and replay starts at setup
before reaching `--from`. The HTML retains full selected unit debug records,
which dominate its size. Terrain, borders, per-figure overlays and building
markers are not exported yet. No new original-format claim is made here.

The next useful extension is automatic export around a test's first failing
frame, using that test's exact inputs. After that, compact shared unit metadata
and field-aligned changes can reduce artifact size while making non-position
failures as immediate to inspect as displacement.
