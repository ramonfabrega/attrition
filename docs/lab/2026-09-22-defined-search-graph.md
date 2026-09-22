# Structural post-search observation with explicit undefined bytes

The model's limit-95 post-search graph is now structurally observable without
inventing byte values: **539 records, 392 tree nodes, 123 PathNodes and four
CollBlocks**. It contains **102 undefined bytes**, exactly the two trailing
bytes in 51 BR-tree nodes. They remain explicit nulls, so this is not a
full-record byte-equivalence claim or a native post-tree witness.

Work starts from `b8d5c9b` on `codex/saved-tree-lab`. PR #5 remains fixed for
Fable's review, PR #4 is unchanged, and no capture, TypeSafe call or main-score
change occurs. This follows [L71](2026-09-22-saved-tree-observation-gap.md).

## Observation contract

`defined_bytes.py` reports an immutable sequence of byte values or `None`.
It enforces declared data extents, modeled allocation bounds and retirement
before reading. Defined spans pass through the existing guest read guards;
undefined spans are recorded without reading their backing storage. Asking
for a required field or a complete byte string containing `None` fails.
Guest execution's initialization, allocation and lifetime guards are unchanged.
Here "defined" means known to this replay: captured input bytes or initialized
modeled storage. It is not a C++ language-level initialization judgment.

`defined_search_graph.py` follows the existing five-container graph, PathNode
parent closure, CollBlock records and seven recycler arrays. Every full record
extent and known byte is retained. Only BR-node bytes +22/+23 and unoccupied
recycler-capacity bytes may be undefined. Named node bytes, all other records,
and occupied recycler entries must be defined. This is an observer contract,
not a change to the modeled allocator or a promise about native padding values.
L71 cites the owned PDB layout evidence; the original graph schema and bounds
are documented in [L16](2026-09-09-natural-search-graph.md).

The observer keeps the existing caps (4,096 records, 2,048 tree nodes, 1,024
PathNodes and a 256 KiB equivalent full-packet bound), refuses overlapping,
cyclic, shared or malformed graph records, and is single-use even after failure.
The existing structural validator is shared internally. Its native binary
entry point still requires complete byte records and preserves its original
behavior. The partial mode consumes known node fields and occupied recycler
entries; undefined bytes are never replaced with zeros for validation.

Output is a distinct JSON schema, `model-defined-search-graph-v1`, with record
values including explicit nulls, completion/refusal status, undefined-byte
count and `native_compared: false`. It never emits a native `search-graph.bin`.
A complete structural result may still have `full_record_bytes_known: false`.

## Retained-packet result

`defined_graph_replay.py` first repeats the L70 native limit-95 unit/path
comparison and requires agreement. It then runs the modeled intervention,
observes the graph, verifies that observation did not change the model's full
fingerprint, repeats both execution and observation, and restores the unchanged
control. No captured region or runtime service is added.

| observation | measured result |
|---|---:|
| complete structural records | 539 |
| declared record bytes | 16,528 |
| defined / undefined bytes | 16,426 / 102 |
| equivalent full binary packet size | 25,184 bytes |
| physical tree nodes | 392 |
| owned PathNodes / parent closure | 123 / 123 |
| CollBlocks | 4 |
| logical lengths: open, refs, closed, valid, blocks | 90, 90, 33, 142, 4 |
| physical lengths in the same order | 90, 123, 33, 142, 4 |

All 102 undefined bytes are at offsets 22 and 23: eight nodes in the reference
tree and 43 in the validity tree. Recycler capacity happens to be completely
defined in this packet, although the observer can explicitly represent an
undefined unused tail. Known bytes in these optional positions remain recorded;
the observer does not blanket-mask them.

The retained pre-call graph also round-trips through the new observer: every
record and structural count agrees with the original validator, with zero
undefined bytes. The post-call observation repeats exactly, the control
restores, and the unchanged baseline `last` result equals L70's in full.
Native comparison remains confined to L70's unit/path boundary, which still
passes. The modeled graph requires no new capture to inspect.

## Checks and limits

Ninety focused tests pass, including the new observer and existing graph,
allocator, lifetime, replay and native-provenance tests. Authored controls poison
undefined backing bytes and prove they are never read; requesting them as guest
loads or required fields still refuses. Other checks cover allocation gaps,
extents, retired objects, code addresses, every required BR-node byte, missing
records, graph corruption, capacity limits and single-use refusal. Known tail
bytes remain visible. The original native binary graph tests still pass.

This closes structural observation of this modeled call, not native tree
fidelity, all mutated state, red-black balancing/key-order correctness, opaque
CollBlock ownership, pointer-identity normalization or general runtime behavior.
The size above is the graph schema's equivalent byte extent, not a captured
native post-graph or the JSON file size. A native comparison must retain explicit
pointer correspondences and definedness; neither is an arbitrary byte mask.

The next lab step is a graph comparison contract with negative controls for
wrong topology, keys, payloads and undefined required fields. Prepare that
before requesting a bounded post-tree native capture. No slot is booked.

Evidence stays outside git under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-defined-search-graph`.
A second fresh process reproduces the complete baseline and graph observation
exactly, including every undefined-byte position. It also restores the control.
The final-source release gate exits zero with two workers: 332 rondata, 855 sim,
13 fixed-point and three doc tests; all 782 fixture requests present; clippy,
formatting, install survey and paperwork checks pass. Peak process-tree memory
was 8,090 MiB. The final validation-note edit is followed by the fast guard.
