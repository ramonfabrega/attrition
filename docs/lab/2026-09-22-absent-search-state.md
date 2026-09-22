# Completed-search observation and retained suspension counters

L77 follows [the immediate continuation experiment](2026-09-22-continuation-boundary.md).
No live capture was requested or run. PR #6 remains frozen at `af4ed73`; this
work stays on `codex/continuation-lab`. No main score changed.

## What the widened comparison establishes

An explicit absent-container observer now accepts all five saved-search roots
being null. It records the same 72-byte unit view, all seven recycler headers,
and their full capacity arrays. Null roots classify absence; only the paired
call result says that a particular invocation completed. Partially absent roots
refuse rather than being treated as an empty graph. The suspended observer's
contract remains unchanged.

On the retained September 22 native-postgraph input, both immediate 95→300 and
95→95 modeled chains produce 15 records covering 2,764 bytes. Of these, 2,620 are
known and 144 unused recycler-capacity bytes remain explicitly unknown. Capacities
are `[1,1,1,1,1,128,512]`; occupied lengths are `[1,1,1,1,1,92,500]`.
Every known recycler header/array byte agrees with the unchanged single-call 300
model. The only differences in this view are the two unit bytes already exposed
by L76. Full unit/path observations remain available alongside the narrower view;
all 640 path-capacity bytes agree with the single-call model.

| unit word | PDB field | single-call model | either chain | observed writes |
|---|---|---:|---:|---|
| +0x134 | `valid_hit` | 83 | 123 | first suspension writes 123; completion writes none |
| +0x148 | `traversed` | 127 | 223 | first suspension writes 223; completion writes none |

Field identities come from the owned PDB export's `UnitData` record
(`types.txt`, fields at offsets 0x134 and 0x148), consistent with
`docs/PATHFINDER.md` §4.3. The explanation of these measured values comes from
instrumented model writes: the single-call control writes neither word, the
first suspended call writes both, and neither second call writes them. Thus
these residual values are retained suspension counters in this experiment.
This does not establish a general formula for either counter or native behavior
on the unobserved second call. L75 already checks the first call's whole native
unit, including its two stored values.

## Reusable tools and boundaries

`absent_search_state.py` preserves existing extent, overlap and defined-byte
guards. `compare_search_graph.py` reconstructs and validates the new explicit
schema before deriving correspondence. It still compares required unit fields
literally: the two residual differences fail required-field agreement; they are
not hidden as acceptable noise. Unused known array bytes remain compared, and
unknown bytes stay visible.

`continued_call.py` graduates the recurring scratch probe into a bounded
re-entry operation. It validates an already returned callee, inactive modeled
services, explicit immutable argument/mode bytes and unsigned 32-bit registers
before changing memory. Heap/global values, ownership, retirement, initializedness,
allocator cursor and cumulative service quotas persist. Only stack definedness
and per-invocation diagnostics reset. Ordinary `.run()` still restores the
captured baseline. The caller controls the number of invocations; this helper
is not a multi-frame scheduler or an imported native CPU-state continuation.

`continuation_replay.py` runs the first native unit/path comparison, two explicit
second-boundary variants, the absent-state comparisons and counter-write audit.
It repeats each chain and restores the original control. The observer is checked
not to mutate runner state. Arguments are explicit install/capture paths and
borrowed extents; generated reports belong outside git.

## Remaining evidence boundary

The observation does **not** follow occupied recycler pointers to target object
contents. The 92 generic-node and 500 PathNode slots, plus five header slots, are
observed pointer values, not 597 newly observed objects. These unobserved targets
are compared literally; they do not acquire a correspondence merely by appearing
at the same array index. A different native allocator could move them. Declaring
such pointers equivalent without inspecting target roles/lifetimes would weaken
the evidence, so native absent-state collector support is not added yet.

The next useful offline question is which recycled object bytes are semantically
live when reused, and which are stale storage that must remain unspecified.
That determines an honest native comparison contract. Natural frame-225 replay
still needs the later call boundary and intervening world changes named by L76;
an immediate frozen-frame model must not stand in for them. No new capture slot
is requested until that missing-input question has a bounded falsifier.

Evidence: `/Users/rf-studio/ron-data/lab-experiments/2026-09-22-absent-search-state`.
The retained input is `lab-captures/2026-09-22-native-postgraph/map-14`, payload
SHA-256 `df96361121222179912a6ede1b72e265c8593a4fc64f0e7e0a01cf7f0c5fe6e2`.

## Validation

The focused suite passes 38 tests. Authored re-entry controls exercise preserved
heap ownership, definedness, retirement, stack-local reset, cumulative allocation
budget, malformed boundaries and refusal before return. Observer controls reject
all 31 nonempty/partially present root masks, missing records, invalid extents,
unknown required bytes, extra records and forged metadata. Required-field and
unused-capacity mutants produce the intended failures. The release gate exits
zero: 332 rondata, 855 sim, 13 fixed-point and three doc tests, all 782 fixture
requests present, clippy, formatting, install survey and paperwork checks clean.
Peak process-tree memory is 7,884 MiB. The reusable report matches the scratch
report exactly apart from wall time; fixture-specific assertions check both
whole-unit and path bytes, counter writes, state differences and repeat/restoration.
The validation-note edit is followed by the fast guard and whitespace check.
