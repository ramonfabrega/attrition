# Recycler reuse: track initialization per acquisition

L78 follows [the absent saved-search state](2026-09-22-absent-search-state.md).
The earlier scratch audit tracked fresh writes across a whole invocation. That
is insufficient when one PathNode is returned to its recycler and acquired again:
a write during its earlier use must not authorize a stale read during its next.
An authored counterexample demonstrates this distinction. The new observer starts
a fresh write history for every acquisition.

## Measured result

Both retained immediate 95→300 and 95→95 continuations make **eight acquisitions
of four distinct PathNode addresses**. Their reuse counts are **4, 2, 1, 1**.
Every acquisition writes all **36 bytes**, and no byte is read before being written
in that acquisition. Read-byte counts by acquisition are
`[16,16,22,16,16,16,12,16]`; all eight objects are returned to the recycler.
There are 139 observed length changes: eight acquisitions and 131 releases,
changing the occupied pool length from 377 to 500. The extra 123 releases agree
with the 123 live PathNodes present in the first suspended graph.

This confirms the earlier no-stale-read result under the stronger boundary;
it does not establish a universal initialization rule. The modeled world and
frame are still frozen, native extended CPU state is not imported, and no native
second-call witness exists. The single-call/continued unit differences from L77
remain visible and are not waived. No main score changed and no capture ran.

## Instrument and falsifier

`tools/explore/recycler_reuse.py` observes guest reads/writes during one invocation.
The PathNode recycler address comes from the established search-graph pool list;
the 36-byte extent is the owned PDB's `PathNode` record (`types.txt`, size 0x24).
A decrease of the recycler length identifies removed slots and begins fresh
acquisition histories. An increase closes tracked histories for returned objects.
Release of an object that was already live before observation is recorded without
inventing an acquisition or initialization claim for it.

Every observed read is checked against writes to the same byte in the current
acquisition. Modeled memcpy and memset also execute guest loads/stores, so their
accesses are included. Host observation uses the existing declared-byte and
lifetime guards. The observer changes neither guest bytes nor runner policies.
Acquisitions outside the available pool, overlapping or duplicate identities, partial length stores,
invalid bounds, unobserved length changes and event/acquisition cap violations
refuse. Failure of the invoked model cannot be labeled successful observation.

`continuation_replay.py --audit-pathnode-reuse` enables the audit. Each observed
chain is repeated without instrumentation and must reproduce the exact model
fingerprint; the unchanged control is then restored. The report records each
acquisition's read, write and read-before-write offsets. An old-byte read is a
reported finding, not suppressed or automatically judged a simulation error:
its field would need inspection and comparison with native evidence. Accesses
before acquisition or after release are outside this epoch audit; it is not a
complete recycler lifetime detector.

Eight authored tests include a decisive negative control: acquire an object,
write/read it, return it, reacquire it and read without rewriting. The first use
has no stale read; the second correctly reports four old bytes. Additional
controls cover old reads on first use, write-before-read, partial length stores,
duplicate/overlapping/null initial identities, duplicate returns, no acquisition
and an unsuccessful invocation. The wider focused suite passes 46 tests.

## Consequence for comparison

The unused contents of a recycled object are not automatically meaningful state,
but this experiment is not permission to ignore them globally. Only eight
observed acquisitions have this write-before-read evidence. Most pooled objects
were untouched. Raw post-state differences must stay in the report, while the
read-before-write audit identifies which stale bytes could affect this observed
execution. No comparator normalization or native collector relaxation is added.

A useful next native witness must include the actual next invocation boundary,
its pre/post state and enough intervening state to reproduce it. L76 already
shows why the immediate frozen-frame continuation cannot substitute for frame
225. Before booking that run, the remaining preparation is a bounded second-call
capture contract; another undirected post-state snapshot would not close it.

Reproduce the offline experiment from this worktree, redirecting stdout outside git:

```
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python tools/explore/continuation_replay.py /Users/rf-studio/code/fun/attrition/game /Users/rf-studio/ron-data/lab-captures/2026-09-22-native-postgraph/map-14 --borrow 0x3475ad48:160 --borrow 0x32a40888:256 --borrow 0x34371ac0:256 --audit-pathnode-reuse
```

Evidence is retained under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-recycler-acquisition-epochs`.
PR #6 remains frozen; this checkpoint belongs to `codex/continuation-lab`.
The capture lane is released.

Validation: the full release gate exits zero with 332 rondata, 855 sim, 13
fixed-point and three doc tests. All 782 fixture requests are present; clippy,
formatting, install survey and paperwork checks pass. Peak process-tree memory
is 8,317 MiB. The final reusable report satisfies fixture-specific assertions for
all acquisitions, transitions, reads/writes, exact uninstrumented repeats and
control restoration. The validation/coverage clarification is followed by the
fast guard and whitespace check.
