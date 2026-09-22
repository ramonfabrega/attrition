# Preparing a paired native post-search graph

The optional **`RON_RESTORE_POSTGRAPH`** observer now captures the saved-search
graph after the paired native return. Its decoder binds the complete unit/path
witness before comparing tree state using L73's explicit correspondence. This
landing is **offline preparation only**: no native post-graph has been acquired,
no capture lane is held, and no main score changes.

Work starts from `09f86d8` on `codex/saved-tree-lab`. PR #5 remains frozen.
This follows [L73](2026-09-22-graph-correspondence.md); the future run repeats
[L70's limit-95 intervention](2026-09-22-native-limit-agreement.md).

## Acquisition boundary and bounds

The new option requires `RON_RESTORE_POSTSTATE`, with its existing prerequisites.
The original call still executes once. In the limit-95 configuration, restoration
of the original limit precedes every post-state operation. Only after the full
unit/path witness is written and INFO 181 is emitted does post-graph collection
start. An earlier failure does not enter it; the existing one-shot return guard
also gates this observer. No game state or return value is modified by this option.

The existing graph traversal is factored into collection and emission, retaining
its schema, fixed storage, tree walk, PathNode parent closure and seven recycler
arrays. The pre-call graph remains `search-graph.bin` with its existing receipts.
Its authored fixture output is byte-identical to the previous committed producer.
The producer now explicitly enforces the decoder's existing 4,096-record cap.
Other bounds remain 256 KiB, 2,048 tree nodes, 1,024 PathNodes and 4,096 entries
per recycler capacity. Full records, including inactive slots and node tails,
are copied from readable native memory; no native padding value is invented.

Post acquisition checks a two-second elapsed-time budget before every memory
read and again after traversal. Unsigned elapsed subtraction handles clock wrap.
This is a cooperative acquisition deadline, not cancellation of a blocked OS
read or file write. The outer capture retains its 180-second process timeout.
The unit's 72-byte saved-search view must equal the prior unit witness both at
the first graph record and in a fresh final read. The frame must still match.
This detects a changed root/view but does not prove atomicity of every tree node
against other threads. No global process suspension is introduced.

## Authored packet and provenance

`restore-postgraph.bin` has eight little-endian header words: magic `0x31504752`,
version 1, frame, unit address, embedded post-state byte count, graph byte count,
acquisition milliseconds, and return boundary `0x688faa`. The header is followed
by the exact `restore-poststate.bin` bytes (including the limit trailer when
present), then a complete existing version-1 graph packet. This is an authored
observer format, not a newly inferred original-game layout.

INFO 186 records success, unit, total bytes, record count and elapsed acquisition
milliseconds. INFO 187 records a post-graph failure. Every file write must return
its full count; failure paths close opened handles and restore observer controls
for the ordinary graph collector. A partial file is never accepted merely because
it exists. The decoder requires a unique matching success receipt, no post-graph
failure, the successful paired unit/path receipt first, and no additional traced
A* entry/return between the two receipts. Existing prefix, intervention, payload
and call-order validations remain mandatory.

The decoder checks bounded framing, exact embedded witness bytes, unit/frame,
structural validity, and the graph's unit view. It adapts all native records to
`native-defined-search-graph-v1`, retaining all their observed byte values. That
internal view is distinct from the modeled graph schema. The comparison checks
the replay's payload and pinned executable identities, repeats/read-only/control
checks, and redoes native unit/path agreement from the actual native packet.
The model's graph unit view must equal its paired replay witness as well.

Rooted correspondence is derived before value comparison. Native-known bytes do
not fill model-unknown positions. The output distinguishes required fields,
all known values, literal bytes and unknown positions, and continues to report
optional differences. `--require-required-fields` asserts only that named scope;
`full_native_state_compared` stays false. Mismatched ownership shape refuses;
it is not repaired by searching for a mapping that makes values agree.

## Offline checks

The 105 focused tests pass. Actual C-produced control and limit-95 packets decode
and match the authored Python expectations. Failure cases cover every read and
write, short writes, failed creation, deadline expiry, clock wrap, frame mismatch,
unit-view changes at either end of acquisition and record-count exhaustion.
The pre-existing graph producer also retains its short-read, cycle, bounds and
file controls. Address/undefined-behavior sanitizers pass for both new packet
variants and the existing graph collector.

Python controls reject malformed extents, mismatched embedded witnesses, changed
headers, missing/duplicate/reordered receipts, intervening calls, collector
failures, inconsistent model unit state, wrong executable/payload and missing
model checks. A PathNode payload mutation fails the required-field assertion.
Native-known optional tails paired with model unknowns stay explicitly unknown.

An additional adapter exercise uses the retained 539-record modeled graph at
the relocated allocator arena, with an explicitly synthetic native-format
packet. Its unknown optional bytes are assigned test poison only in that test
packet. Compared with the original arena's model, the adapter reproduces L73:
zero required differences, one known stale-capacity byte difference and 102
unknown positions. The output is labeled model-derived test evidence. **This is
not a native tree fidelity result.**

Default, payload, post-state, post-graph and post-graph-plus-limit95 DLLs compile
and link in isolated temporary directories. The missing post-state prerequisite
refuses compilation. No DLL has been staged into the live install.

## Next run and limits

After this preparation is gated and committed, request one fresh capture slot
through Ramon. Repeat the existing map-14, seed-12345, end-frame-1400 scenario,
180-second timeout, adding only `--tracer-def RON_RESTORE_POSTGRAPH` to L70's
flags. Retain both old and new sidecars. Verify process exit and settings
restoration, then release the lane before offline replay/comparison. Earlier
capture permissions are not reused.

Run `defined_graph_replay.py` on the new packet, with any required allocator
borrows determined from that packet, then `restore_postgraph.py <install>
<capture> --replay <model-report> --require-required-fields`. Capture failure or
native disagreement is a result to retain, not permission to erase differences.
The unit/path witness remains independently checked even if graph acquisition
or correspondence refuses.

The outcome could still fail on topology, opaque CollBlock values, stale pointers
outside observed ownership, allocator-dependent state or native/model fields.
No claim is made about red-black balancing, general allocator behavior, extended
state restoration, all game state or native limit 96. Generated evidence lives
outside git under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-native-postgraph-preparation`.

Final gate: `release_gate.py <install> --test-threads 2` exits zero with 332
rondata, 855 sim, 13 fixed-point and three doc tests. All 782 requested fixtures
are present; clippy, formatting, install survey and paperwork checks pass.
Peak process-tree memory is 8,219 MiB. The final validation-note edit is followed
by the fast guard and whitespace check. The three-line shared tracer prerequisite
guard is committed separately: it prevents a post-graph flag from silently doing
nothing when post-state observation is absent. It changes no default behavior.

Subsequent result: [L75](2026-09-22-native-postgraph-agreement.md) records
the authorized capture and native required-field agreement. The preparation-only
status above describes the evidence at this earlier landing.
