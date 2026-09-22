# L87: selected delegation prefixes add no net shared-header changes

The retained L86 payloads narrow its missing interval without another capture.
For both selected calls (owner 0/id 16, frames 224 and 225), all 504 bytes of
the selected UnitData, five active tree headers and two recycler headers match
between delegation to find_upath and native A* entry. Their addresses also
match. The shared-state differences are already present at delegation.
This is a net boundary comparison, not proof that no transient writes occurred.

## Evidence and executable comparison

`tools/explore/shared_prefix_window.py` validates both retained payloads through
the existing image, context, graph, inventory and payload validators. Temporary
symlink views project the two packets for those validators without copying the
broad payloads. The capture receipt supplies the locally derived image binding;
the validators recompute identities. Source files are left unchanged, and the
trace digest is checked before and after the comparison.

The selected unit comes from the validated context. PathFinderData.pathing_unit
still names a previous unit before callee setup; using it would compare the
wrong object. Every byte of the selected 344-byte unit, active headers of
28/24/24/28/24 bytes and two 16-byte recycler headers is compared. The complete
136-byte PathFinderData is compared separately and its changes retained:

| Selected call | Changed PathFinderData byte offsets | Changed unit/header bytes |
| --- | --- | ---: |
| Frame 224, call 1 | 20, 21, 22, 28, 36, 64, 65 | 0 / 504 |
| Frame 225, call 3 | 20, 21, 22, 24, 28, 36 | 0 / 504 |

The first call's offsets 64/65 include the known instrumentation intervention
from limit 300 to 95. These changes must not all be attributed to ordinary
callee setup. No claim of unchanged PathFinderData is made.

The existing restore-wrapper prefix runner also validates both new contexts:
31 instructions each, two reused-engine runs and a matching fresh-engine run,
with dependency-omission and corrupt-output refusals. This runner stops before
the find_upath call. Its declared writable regions exclude the active headers
and recycler headers; that is a bounded model result, not a process-wide claim.

## What this settles, and what it does not

L86 located changes before, inside and after the intervening A* call. For the
second selected call, the post-intervening-call changes now precede delegation
to find_upath: its selected callee prefix adds no net changes in these headers.
The first selected call gives the same boundary result. Fresh-search caller
setup for the intervening unit remains unobserved at this boundary. Specific
writer instructions, tree contents beyond headers, transient writes and other
threads remain unestablished. Neither snapshot is process-wide atomic.

The next useful target is the interval from the intervening return to the next
selected delegation, plus fresh-search setup before the intervening entry.
This does not justify clearing roots or truncating pools to make replay agree.
No simulation score moved, no capture was taken and PR #6 remains frozen.

## Reproduction and validation

Input is the existing capture at
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-shared-search-boundary/map-14`.
The experiment completed in 2.76 seconds. Its trace SHA-256 is
`d45827de07d77ef061001e7698ecb4f7aace3a14200ec405d3ab27f9d8a47c0a`.

```sh
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python \
  tools/explore/shared_prefix_window.py INSTALL CAPTURE
```

The focused suite passes 81 tests. New controls alter every byte of each
unit/header record, change object identity, truncate retained reads and change
the source trace; each refuses. A stale global unit pointer cannot redirect
the selected-unit comparison. PathFinderData setup differences remain visible.
Temporary-view tests verify source preservation and cleanup.

Exact source, results and validation logs are archived outside Git at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-shared-prefix-window`.

The full release gate passes: 1,203 tests, 782 fixture requests with none
missing, clippy, formatting, install survey and paperwork checks. The final
document-only validation note is followed by the fast guard and whitespace check.

Follow-up [L88](2026-09-22-shared-owner-gap.md) resolves the second
delegation's previous global owner as registered unit 0/59, distinct from the
intervening A* caller 0/22. Broader path-request boundaries are now the next
target; the owner change does not attribute the shared-header writes.
