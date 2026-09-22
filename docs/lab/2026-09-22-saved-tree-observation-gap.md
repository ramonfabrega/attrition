# Saved-tree observation needs explicit byte definedness

PR [#5](https://github.com/ramonfabrega/attrition/pull/5) packages L64–L70 for
Fable's review, stacked on unchanged PR #4. Further lab work starts from
`d745446` on `codex/saved-tree-lab`, leaving the PR's head fixed.

The next offline question was whether the existing packet can supply a complete
post-call saved-tree observation before asking for another native capture.
**It cannot yet supply every byte of that observation:** traversal reaches a
newly allocated tree node whose last two bytes were never initialized by the
modeled execution. The guard refuses. No byte was filled in, masked in a native
comparison, or read through the guard. No capture or TypeSafe call occurred;
the lane remains released and the main score is unchanged.

## Changes outside the confirmed native boundary

The retained limit-95 packet from L70 was replayed with exactly the same bounded
services and borrowed objects. The observer compares the previously captured
graph records with guarded post-call memory. This is a **model observation**, not
another native witness. Newly reachable records are not part of this first
inventory, and retired objects are reported instead of read.

| pre-captured record family | records | changed records at limit 95 | changed bytes |
|---|---:|---:|---:|
| five tree headers | 5 | 5 | 70 |
| physical tree nodes | 223 | 35 | 144 |
| PathNode payloads | 67 | 0 | 0 |
| CollBlock payloads | 3 | 0 | 0 |
| recycler headers | 7 | 7 | 21 |
| recycler backing arrays | 2 | 1 | 4 |

Zero changes to the old PathNodes do not establish that no new PathNodes exist.
The table is deliberately an inventory of old records, not graph closure.
The unchanged limit-300 control changes 158 of those 223 tree nodes and retires
one old recycler backing array; the observer does not inspect its retired bytes.

Limit 95 executes 34,665 write events over 7,690 distinct addresses. Of these,
65 fall inside the returned unit/path observation and 7,625 lie outside it.
Those other writes include stack and model-service activity as well as engine
data: **this ratio is not a fidelity score**, nor a claim that every such byte
needs a native capture. The useful result is identifying changed saved-search
records whose values L70 did not compare.

## Bounded structural traversal and refusal

A scratch prototype follows the existing `search_graph.py` schema: five tree
containers, physical child/parent links, owned PathNodes and their parent
closure, CollBlock payloads, and seven bounded recycler arrays. It keeps the
existing record, node, payload and byte caps. Reads use the replay's declared
regions, initialization and lifetime guards; it adds no regions or services.

The prototype reconstructs every record in an authored graph and in the retained
pre-call graph, with identical validation reports. Removing any authored record
refuses; five deliberately corrupted graphs exercise a tree cycle, parent link,
reference relationship, PathNode-parent cycle and pool capacity.

On the modeled limit-95 post-state it reads 191 records, then refuses a kind-3,
owner-1 node at `0x184e0a94`, extent 24, in `_malloc_arena`. The exact undefined
byte offsets are **22 and 23**. The call itself completed without reading these
uninitialized bytes; the whole-record observer is the new reader.

Layout evidence is the owned PDB-derived
`BRTreeNode<TreeNode<PathNode_*,int>_*,unsigned_long>` record in
`/Users/rf-studio/ghidra-projects/decomp/types.txt` (record at line 40483 in this
export), plus the earlier layout evidence in
[L16](2026-09-09-natural-search-graph.md). Its size is 24 bytes and its last
named field ends at offset 21. This identifies the two bytes as outside the
named members; it does not establish their value in native allocator storage
or authorize treating them as zero. This is not a newly derived behavioral rule.

The diagnostic rerun repeats the intervention and restores the complete
unchanged model fingerprint after the refusal. Structural closure remains
**unestablished**: the traversal stops at its first undefined record, so its
remaining byte cost and later obstacles are unknown.

## Next experiment

Before widening live capture, define an observation that carries the full
record extent **and explicit per-byte definedness**, with no invented values.
Compare every defined field; separately report undefined bytes and fail on
missing required field bytes. The existing full-record observation must keep
refusing when asked to promise all bytes. Any later structural-only result must
be labeled differently from L70's complete unit/path byte agreement.

This is a concrete next lab question, not a reason to delay review of PR #5 or
change its existing claims. A native post-tree capture may ultimately be useful,
but the model observer contract should be made honest first. No slot is booked.

## Evidence and validation

Scratch probes and generated reports remain outside git under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-saved-tree-observation-gap`:
`saved-tree-coverage.py/.json/.log`, `saved-tree-closure.py/.json/.log`, and
`check-saved-tree-closure.py/.log`. The last JSON records the refusal and the
successful repeat/control restoration. These are exploratory probes, not new
production tooling or native-state assertions. The release gate passed with two workers: 332 rondata, 855 sim, 13 fixed-point
and three doc tests; all 782 fixture requests present; lint, formatting, install
survey and paperwork checks passed. Peak process-tree memory was 8,262 MiB.
The final validation note and corrected historical ledger label were followed
by the fast paperwork guard. No production source changed.
