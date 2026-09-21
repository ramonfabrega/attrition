# Complete current-unit record for the next replay boundary

The L59 replay stops on unit `+9`; the same A* prefix soon reads `+0xa`, `+0x6c`
and `+0x18`. The collector now records the whole 344-byte current-unit body at
its delegation event. The version-2 context passes offline tests and one authorized live capture.
Replay now reaches instruction 91 and refuses the uncaptured world pointer.
The earlier version-1 live result remains pinned.

## Evidence and packet contract

The matched PDB export gives both `Unit` and `UnitData` size `0x158`:
`types.txt`, `/rise.pdb/Unit` and `/rise.pdb/UnitData`. Its inherited
`SubObjectData` fields identify owner at `+9`, object index at `+0xa`, encoded
coordinates at `+0x10/+0x14`, and type pointer at `+0x18`. The owned direct listing
at `a_star@0x683770` establishes the immediate reads. These are local input-format
claims, not a new mechanic specification. Original-derived files stay outside git.

Context version 2 inserts 344 bytes after FXSAVE and before the table, increasing
its fixed extent from 792 to 1,136 bytes. Version 1 is still accepted and is never
silently upgraded. Capture uses one checked exact-length unit read. Before
publishing success, it checks the unit pointer against the prefix's object
pointer, its owner/index against the arguments, and both encoded coordinates
against the earlier prefix observation. Failures use INFO 163 reason 24.
The ordinary sidecar receipt's total size must match its version.

Offline validation also compares all 72 overlapping bytes at unit
`+0x104..+0x14c` with the earlier graph snapshot. Any discrepancy rejects the
packet instead of combining observations from different states. The replay then
maps the complete unit as one writable region, removing only the two redundant
coordinate and graph-fragment regions. No other pointer target is implied by
capturing a pointer; table and thread overlaps remain forbidden.

## Tests and limits

The exact C collector emits a packet consumed by Python. Its fixture exercises
36 failed/short memory reads, unit-identity rejection and existing file/extent
controls. Python rejects wrong owner/index/coordinates, inconsistent graph bytes,
truncated versions and a stale version-1-sized receipt. An entirely authored
unit/graph/table input drives the original prefix through the expanded region
assembly and refuses its next absent input. This is a wiring test, not a new
live result. The original September-21 capture still matches its pinned
50-instruction A* entry and instruction-73 refusal.

The general frontier reporter can describe a later missing-memory boundary for
version 2, but retains the exact refusal assertion for version 1. It does not
interpret arbitrary emulator exceptions as successful research results: budget,
uninitialized-scratch and other errors still fail. FXSAVE remains opaque; later
extended-state use still needs a proper import. Path-stack payloads, the type
record, world globals and other units are not captured merely because this unit
contains their pointers.

## Authorized live result

The authorized map-14/seed-12345 run completed in 28.35 seconds (7.72 build,
20.51 launch to exit), exit zero, all 1,401 frames verified. All five settings
files were restored and the user was immediately told the lane was free.

At frame 224, owner 0 / id 16, all 344 unit bytes passed identity/coordinate
checks and every overlapping graph byte agreed. The 19,200 table entries still
match original-initializer regeneration. The 50-instruction prefix matches all
four native A* arguments across 64 resets and a fresh engine. Continuing into A*
first needed a write-only working word at `0xe85eb4`, assigned from captured unit
`+0x6c` at PC `0x6837fa`. Declaring that four-byte word as write-before-read
scratch advances the replay to attempted instruction **91**, PC `0x683808`,
which refuses the absent four-byte global at `0xc06188`. The owned listing names
this as the world-pointer load. No value was synthesized and no search return
was produced. The new capture has its own executable/tracer hash binding.

The congestion comparison against the preceding version-1 capture agrees on all
recorded scenario/search projections: 32 captains, ten orders, 2,314 A* returns,
1,787 suspensions and the capped 64-event census. This is not full-state
noninterference or a general startup-reliability result.

Artifacts are retained at
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-unit-context`, with settings
backup, reports and a hash manifest. Copied bytes were checked and replay rerun
from the retained directory. No main-loop files changed.

## Research direction after this capture

Three acquisition stages have closed concrete gaps, but another isolated global
would require another lane handoff. Before widening one more field, evaluate a
bounded address-space snapshot: capture a broader explicit input once, discover
the actual read set offline, then reduce the input to the records it needs. This
is a proposal, not an implemented or validated snapshotter. It must preserve
capture provenance, refuse unreadable/incomplete ranges, respect a fixed size
and time budget, and distinguish observer-modified state from original inputs.
Concurrent-thread coherence and original-code/import boundaries remain questions.
No further live capture is authorized by this completed slot.

Separately, a scratch FXRSTOR/FXSAVE round-trip preserved eight XMM registers and
MXCSR but did not reproduce the saved x87 metadata fields. Explicit metadata
register writes also did not make the re-saved image identical. Reserved bytes
must be separated from actual lost state before this becomes an import path.
The experiment remains outside the replay implementation, at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-fx-roundtrip`.


## Reproduction

```sh
RON_CONTEXT_IMAGE=/path/to/game/riseofnations.exe \
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_restore_context.py
RON_RESUME_INSTALL=/path/to/game RON_RESUME_CAPTURE=/path/to/version-1-capture \
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_resume_frontier.py
```

Seven context tests, four frontier tests on each live packet version, five
prefix tests, 256 emitted-adapter cases and Windows-target compilation pass.
The full two-worker repository gate passed before capture: 332 rondata, 855 sim,
13 fixed and three doctests; 782 fixture requests with none missing, peak tree
RSS 7,913 MiB. The final post-capture replay adjustment passed the same complete gate and
fixture audit. Reports: `/tmp/unit-context-gate`, `/tmp/unit-context-final-gate`
and corresponding `.log` files. The next experiment is specified in
[the bounded snapshot proposal](2026-09-21-snapshot-direction.md).
