# L88: the A* census omits a pathing-owner transition

The retained L86 payload supplies a better next target than replaying the
intervening A* alone. That call belongs to owner 0/id 22, but before the next
selected delegation, PathFinderData.pathing_unit points to owner 0/id 59.
The full retained unit record and its occupied registry slot agree on that
identity. No intervening A* in the observed sequence accounts for this owner.
This is evidence of an owner transition outside the A* boundaries, not evidence
that unit 59 performed the shared-header writes.

## Native observations

The input is the same validated map-14 capture used by L86 and L87; no new
capture was taken. At each selected delegation, the new reader extracts the
full 344-byte record named by PathFinderData.pathing_unit, the owner's complete
16-byte registry header and that unit's registry slot. It rejects missing or
short reads, invalid owner/id/capacity bounds and a slot pointing elsewhere.
The records and payload hashes remain in the local JSON artifact, outside Git.

| Boundary | Current pathing owner | Selected request owner |
| --- | --- | --- |
| Before first selected delegation, frame 224 | 0/51 | 0/16 |
| Intervening A* entry and exit, frame 224 | 0/22 | 0/22 |
| Before second selected delegation, frame 225 | 0/59 | 0/16 |
| Second selected A* entry and exit, frame 225 | 0/16 | 0/16 |

The second delegation's previous owner differs from all three units in the
selected A* triple (16, 22, 16). L86's complete bounded census and L87's packet
projection establish the ordering. The first delegation's previous owner 51
is also retained, but no preceding A* boundary brackets its selection here.
It is context, not a second measured transition interval.

The existing PDB evidence names PathFinderData.pathing_unit at +0x14; the
full unit identity uses the same owner/id fields already checked by the
native boundary reader. Registry traversal follows the existing validated
restore collector and prefix runner: header at units' owner registry, then
the id-indexed slot. This extra check distinguishes an occupied unit from
merely decoding plausible bytes at a stale pointer.

## Consequence for the next experiment

An A* entry census cannot be treated as a census of all path requests. The
owned export gives concrete candidate boundaries: find_upath at 0x682f30,
find_wpath at 0x688fc0 and find_tpath at 0x6897d0 each assign the global
pathing owner. These are navigation candidates from a reading, not an exhaustive
writer proof or a blind-audited attribution of the observed transition.

The next bounded observer should bracket these outer requests, retaining
requests that return without reaching A*. It should associate request owner,
return value, nested A* calls and complete shared headers. The falsifier is
straightforward: if those brackets leave the observed owner/header changes
between requests, the proposed boundary is still too narrow. No implementation
should assume that replaying id 59 alone reconstructs the gap.

Neither broad payload is process-wide atomic. Registry membership does not
prove which thread wrote the owner, whether intermediate owners existed, or
which function changed the tree/recycler state. Transient writes and header
ownership remain open. The new assertion proves only the retained owner gap.
Main scores and frozen PR #6 are unchanged; no capture lane was acquired.

## Reproduction and controls

```sh
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python \
  tools/explore/shared_owner_gap.py INSTALL CAPTURE
```

Capture:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-shared-search-boundary/map-14`.
Trace SHA-256:
`d45827de07d77ef061001e7698ecb4f7aace3a14200ec405d3ab27f9d8a47c0a`.
The reader validates both complete payloads and checks the trace hash before
and after. Authored controls reject a vacated registry slot, missing/truncated
records, invalid identities/extents, and a counterexample where the previous
owner is already represented in the selected A* triple. All 85 focused tests
pass, including the existing replay and prefix controls.

Evidence, exact source and validation logs are archived at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-shared-owner-gap`.

The full release gate passes: 1,203 tests, 782 fixture requests with none
missing, clippy, formatting, install survey and paperwork checks. A final fast
guard follows this validation note. The archived observer design is explicitly
unimplemented: it records the entry/return argument split and the stale-owner
check that must change before broadening the native boundaries.
