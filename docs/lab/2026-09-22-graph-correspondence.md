# Comparing saved-search graphs across allocator relocation

The graph comparison contract now pairs all **539 records** across a controlled
1 MiB allocator move. Its rooted correspondence equals the independently supplied
address-shift map: **99 records move and 246 typed pointer fields relocate**.
All required fields agree. Strict agreement across known bytes fails on one
explicitly reported byte in unused recycler capacity. The same 102 undefined
node-tail bytes remain unknown. This is model-to-model evidence, not a native
post-tree fidelity result.

Work starts from `75e3f7d` on `codex/saved-tree-lab`, following
[L72](2026-09-22-defined-search-graph.md). PR #5 stays frozen for review.
No capture, external API call or main score change occurs.

## Contract

`compare_search_graph.py` rebuilds each complete observation with the L72
observer and requires exact metadata, record identities and counts. It checks
paired replay payload/image identities and explicit native-intervention inputs.
These are consistency checks on retained reports, not authentication of reports.

A supplied correspondence must be a complete one-to-one map preserving record
kind, owner and extent. Alternatively, `--derive-correspondence` anchors the unit
and seven recycler headers, then follows labeled tree roots, left/right children,
owned data and PathNode parent links. Every binding has a recorded witness;
conflicting edges, absent targets and incomplete coverage refuse. It does not
search scalar values or permutations for a mapping that makes the comparison pass.

Only typed pointer fields use this map: the five unit tree pointers; tree
root/current/parent and pointer-typed current data; node links and pointer-typed
data; PathNode parent; recycler backing pointer and occupied entries. The
validity tree's integer data and all ulong metrics remain literal even when
address-shaped. Opaque CollBlock bytes and unused recycler capacity also remain
literal. Unmapped targets remain literal; no interior-pointer identity is inferred.

The field evidence is the owned PDB's `Tree<PathNode*,int>`,
`BRTree<TreeNode<PathNode*,int>*,ulong>`, `BRTree<PathNode*,ulong>`,
`Tree<CollBlock*,int>`, `BRTree<int,ulong>` and corresponding node records,
plus `PathNode`. See FORMATS' suspended-search cleanup fixture and
[L16](2026-09-09-natural-search-graph.md) for the existing layout evidence.
The PDB gives CollBlock's extent but no member layout in this export, so this
comparator grants it no pointer normalization.

Unknown byte positions and each side's definedness are reported separately.
Known optional bytes are compared and retained in the differences. Two assertions
are deliberately distinct: `--require-known-agreement` rejects every known
difference; `--require-required-fields` rejects differences in the L72 required
fields but still reports optional differences. Neither means full-byte agreement
when unknowns exist. Required fields cannot be undefined because graph validation
refuses them first.

## Controlled experiment

The same retained L70 packet is replayed with allocator arena `0x184e0000` and
`0x185e0000`, both inside its recorded free range. The intervention still takes
165,934 instructions. Native unit/path agreement, repeatability, read-only
observation and restoration of each arena's unchanged control pass.

| comparison | result |
|---|---:|
| paired records / moved records | 539 / 99 |
| known bytes compared | 16,426 |
| unchanged unknown positions | 102 |
| raw changed known bytes | 247 |
| pointer fields agreeing under correspondence | 246 relocated |
| remaining required differences | 0 |
| remaining optional differences | 1 byte |

The difference is kind 7, recycler owner 5, backing array `0x31d2f4e8`, byte +2:
`0x4e` versus `0x5e`. Its first stored word is `0x184e0b74` versus `0x185e0b74`,
both corresponding to an observed open-tree node. But this recycler's post-call
length is zero. The word is outside occupied state and is therefore retained as
a literal stale-capacity value, not silently normalized. The strict assertion
exits one; the required-field assertion exits zero.

The automatically derived 539-pair map exactly equals the controlled map made
from the known arena shift. Changing a PathNode payload byte in a report copy
leaves correspondence derivation possible and makes the required-field assertion
fail. A report's native unit/path witness does not exempt its graph from checking.

## Validation and limits

Authored tests cover topology, keys, payloads, flags, opaque fields, integer
values resembling pointers, inactive capacity, unknown required bytes, wrong
correspondences, duplicate/missing records, metadata tampering and mixed input
provenance. Optional-byte differences fail the strict assertion; required-field
differences fail both. Derived correspondence refuses inconsistent ownership
links, while a scalar mutation remains visible after deriving the same map.

This establishes an offline comparison contract on one retained call. It does
not establish native post-tree agreement, allocator equivalence, red-black
balancing, key ordering, all mutated state or general runtime fidelity. Current
CLI inputs are model observation reports; a native post-graph adapter and its
paired provenance still need implementation and testing. Mapping refuses graphs
whose labeled ownership shape differs; it does not explain every such divergence.

Next: prepare that bounded native post-graph collector and adapter, test their
failure paths, and gate them before asking for a fresh capture slot. The lane
remains released. Generated evidence is kept outside git under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-graph-correspondence`.

Final validation: 100 focused Python tests pass. The full release gate exits
zero with two workers: 332 rondata, 855 sim, 13 fixed-point and three doc tests;
all 782 fixture requests are present. Clippy, formatting, install survey and
paperwork checks pass. Peak process-tree memory is 8,802 MiB. The final
validation-note edit is followed by the fast guard and diff whitespace check.
