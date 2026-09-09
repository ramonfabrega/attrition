# A natural suspended-search graph outside Wine

## Result

The first congestion suspension at frame 223 now has a validated structural
snapshot: **223 physical tree nodes, 67 PathNodes and three CollBlocks**. Its
15,468-byte file contains 10,508 bytes of declared original memory in 308 typed
records. The five logical lengths remain 46/46/21/86/3, but physical lengths
are **46/67/21/86/3**: the reference index retains 21 removed nodes. Counting
logical entries alone would have under-sized the capture.

The original's two `PathFinder::kill_tree` overloads dispose all 67 owned
PathNodes outside Wine using the captured recycler, without modifying its
capacity or installing an allocator adapter. The call projection needs **3,484
semantic bytes, 52 KiB mapped and 1,807 instructions** across two calls. There
are 79 natural recycler slots available, enough for the 67 disposals. Sixty-four
repeated pipelines plus their initial baseline take about 0.35 seconds locally;
this includes validation and is not a cross-machine benchmark. Fresh and reused
engines agree on registers, complete memory and write history. Reversing the
order of the two trees preserves ownership; recycler insertion order is allowed
to differ.

Full `Unit::clear_partial_path` still refuses at **0xc8d81c**, the first container
recycler's growth increment. This is the intended undeclared allocator boundary,
not a failed attempt silently repaired by changing pool capacity. The successful
result is native **payload disposal on a natural input**, not a complete live
before/after cleanup comparison, a resumed A* search or a Rust fidelity claim.
No simulation headline floor moved.

## Collection and structural evidence

Build the existing congestion scenario with `RON_SEARCH_GRAPH` in addition to
`RON_CONGESTION_PROBE` and `RON_SEARCH_CENSUS`. The graph callback runs after the
first complete census event, once per process. It uses a 256 KiB static buffer,
a 2,048-node bound and a 1,024-PathNode bound. Traversal is iterative, detects
repeated child pointers, and reads through checked ReadProcessMemory calls.
Seven recycler headers and their full capacity arrays are bounded to 4,096 slots
each. No original state is written; only the diagnostic output file is created.
A failed read, cycle, bound or file write emits a refusal. Missing output never
counts as a successful snapshot.

PDB `Tree`/`BRTree` layouts establish 28-/24-byte headers, 20-/24-byte nodes,
child/parent/data offsets and the BRTree removed byte at +0x15. `PathNode` is
36 bytes with parent at +0x20. The PDB supplies CollBlock's 108-byte size but no
fields: its payloads are copied as opaque records. The collector follows tree
children and active open/closed payload parents, **not pointers guessed from
opaque bytes**. Tree cursor links are checked against captured nodes offline;
this is not the transitive closure of every engine object referenced by a search.

`PathFinder::kill_tree@00687c10` owns the open-tree payload disposal;
`PathFinder::kill_tree@00687ba0` skips removed closed-tree nodes. Their code
bounds (0x64 and 0x6a bytes) were checked in the local PE listing. The native
pool operands are 0xc8da70/74/78, with growth increment at 7c. The full-cleanup
probe reuses the existing cleanup fixture's code regions and adds the closed
variant; recycler increments remain inaccessible. Executable bytes are loaded
from the hash-bound user install and never committed.

`search_graph.py` rejects truncated or overlapping regions, unknown record
shapes, mismatched lengths, cycles, broken parent links, unreachable nodes,
duplicate payload ownership, active reference entries that do not bijectively
index the open tree, and a PathNode that is simultaneously live and recycled.
It checks the exact snapshot receipt and the first census's owner/frame,
container headers and recycler headers. All 67 owned PathNodes' parent chains
close within those same 67 records. Removed reference entries are captured but
their stale data pointers do not establish ownership.

## Replay contract and negative controls

`replay_search_graph.py INSTALL DIRECTORY` verifies image identities, validates
the capture and executes the native disposal entry points with byte-bounded
memory. It maps only open/closed tree nodes and the existing PathNode recycler;
payload contents, other containers, CollBlocks and the unit are not needed by
this substep. Stack scratch requires initialization before every read. All
semantic bytes are compared: only disposed nodes' data fields, recycler length
and the newly occupied slots may change. Existing and unused slots must remain
byte-identical, and each payload must be returned exactly once. Callee-saved
registers and stack balance are checked on both calls.

All **73 individual dependency omissions** refuse. Four corrupted outputs
(argument root, node data, recycler length and a pre-existing pool slot) fail
the complete-record/ownership checker. A fresh engine matches the baseline;
64 resets match its exact register/memory/write history. The full-cleanup
negative control reaches the named allocation boundary after traversing the
natural open and closed trees.

Five install-independent Python tests cover authored graph validity, every
missing record, malformed framing, overlaps, ownership/parent/cycle failures,
invalid pool length and a removed node with a deliberately stale reference.
The exact C collector has a host fixture: successful collection revalidates in
Python, all 28 reads are forced to fail and return short (56 refusals), and cycle,
buffer, node/payload-capacity and pointer-wrap controls refuse. Failed file
creation, failed writes and short writes also refuse. These fixtures contain no game data.

## Live comparison, artifacts and next work

`/tmp/attrition-search-graph` contains the bound images, trace, graph and settings
backup. `/tmp/search-graph-report.json` and `/tmp/search-graph-replay-final.json`
record validation and native replay. The graph run matches the previous
`/tmp/attrition-congestion` run on all scenario validator projections: 1,401
frame/RNG pairs, roster, orders, targets, A* returns and all 64 metadata shapes.
The game was closed through its menu and all five original profile files were
restored and compared byte for byte. Original-derived artifacts stay outside git.

Next, separate two remaining problems: allocator-dependent container/CollBlock
cleanup, and the environment actually read by a resumed search. The snapshot
settles neither. Capturing the native restore call's explicit read set can show
whether that environment is another small capsule or a world-state boundary;
opaque bytes and a passing disposal test must not be mistaken for that evidence.


Capture-selection note: the first owner appears only once in the first 64
suspension events; a later return of −1 is not a reliable way to capture that
owner's next step. The first observed 300-budget suspension is frame 231,
sequence 3. A restore experiment should instrument the native restore **entry**,
rather than infer successful resumption from another suspension event.

Validation: full release gate passes (269 rondata, one ignored, 822 sim,
13 fixed, three doctests); rondata 224.38 seconds, process-tree peak 9,267 MiB
under 20 GiB. Log: `/tmp/search-graph-release.log`. Clippy with warnings denied,
formatting, install survey and documentation guards pass. The graph, census and
scenario Python suites pass (5/9/5 tests), both C collector fixtures pass,
the graph-without-census build rejects, the live DLL links without warnings,
and default tracer preprocessing remains unchanged.

Follow-up: `docs/lab/2026-09-09-restore-entry.md` captures the native restore
entry at frame 224 and replays its wrapper up to delegation. The next native
A* call succeeds. Headless replay of the larger search remains open, starting
with explicitly missing Windows thread/exception context.
