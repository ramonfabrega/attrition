# L86: an intervening A* call does not own the whole shared-state gap

The new native boundary capture falsifies the simple next-step hypothesis from
L85: replaying the one intervening A* call alone would not reconstruct the four
shared-state words. Three change during that call, but changes also occur before
its entry and after its return. Its exit does not match the next selected entry.
The observer narrows the missing interval; it does not name individual writer
instructions or establish a complete caller/world transition.

## Bounded observer and evidence

`RON_SHARED_SEARCH` is an opt-in addition to the existing minimal A* proxy lane.
It observes frames 224–225, at most eight calls, and reads at entry and return.
It records the full 136-byte PathFinderData, full 344-byte current UnitData,
all five active tree headers (28/24/24/28/24 bytes), and both 16-byte recycler
headers implicated by L85. Null active containers are explicit empty records.
There are no node walks, allocations, game-state writes or additional hooks.
The ordinary tracer build is unchanged when the define is absent.

Layout evidence is the owned PDB type stream: PathFinderData's active pointers
and pathing_unit; UnitData.path at +0xb8; head_node at +0xc in both named tree
types; Stack's four-word header. L85 gives the original function/type locators
for the two recycler families. INFO 200–203 bind each snapshot to call sequence,
phase, self, path argument, native return, frame, record identity and complete
byte count. INFO 204/205 denote failure/cap and make the reader refuse. Partial,
duplicate, orphaned, reordered and mismatched records also refuse.

The native run used the existing limit-95 scenario, map 14, seed 12345, ending
at frame 1401. It ran for 18.80 seconds after launch, 26.29 seconds total, exited
zero, restored all five settings files and released the shared lock. No lane
lock was bypassed. The observer's source, prior base, patch and capture command
were archived before the attempt. Preflight included 75 focused tests, actual
C-producer/Python-reader agreement, failed/short reads at every acquisition,
cap/reentrancy/frame controls, sanitizers, three Windows compilation variants
and the full release gate.

Capture:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-shared-search-boundary/map-14`.
Four A* calls yield eight complete snapshots. The first three are owner 0/id 16
at frame 224 (internal return -1), owner 0/id 22 at frame 224 (return 0), and
owner 0/id 16 at frame 225 (return 1). The fourth call is retained too. The
paired outer post-state witnesses validate: first return -1, path length/capacity
1/10; second return 8, path length/capacity 8/40.

## What the native boundaries say

The labels V1/V2/V3 and B1/B2 below denote distinct nonzero addresses within this
run, not portable pointer values. The active header addresses themselves remain
the same across these four snapshots: changes are not merely selecting different
header objects.

| Boundary | Active validlist root | Active blocklist root | Open-node pool length | Open-reference pool length |
| --- | --- | --- | ---: | ---: |
| Selected first A* exits | 0 | 0 | 0 | 1 |
| Intervening A* enters | V1 | B1 | 0 | 1 |
| Intervening A* exits | V2 | B1 | 1 | 0 |
| Selected next A* enters | V3 | B2 | 1 | 2 |

Thus the observed intervals are:

- **Before the intervening call:** both roots become nonzero; the two pool
  lengths stay 0/1.
- **Inside that call:** validlist root changes and pool lengths become 1/0;
  the blocklist root stays B1, but other bytes of its complete header change.
- **After that call:** both roots and the open-reference pool length change;
  the next entry has lengths 1/2, the same pattern isolated by L85.

`shared_search.py` retains every recorded byte and reports all nine records'
changes across all three intervals. Different unit objects are identified as
different extents, not presented as a same-object byte diff. The root/pool
trajectory, stable active-header identities and owner/id sequence are executable
assertions. A counterexample making the intervening exit equal the next entry
is rejected; an altered header identity is also rejected.

The callbacks bracket A* execution on the observed thread. They are not a
process-wide atomic snapshot or instruction-level write watch. State outside
the recorded records, other threads and specific writers are not established.
No code is proposed to clear roots or truncate pools: L85's interventions
remain diagnostics, not a substitute for the missing setup/teardown.

## Perturbation checks and validation limits

Both full path-capacity byte arrays and both outer returns match the preceding
L81 run. All 1,401 frame records agree exactly. The 31,853 game-RNG events agree
on kind, direct caller, generator address, input seed, argument and frame.
This is explicitly a projection: 265 events differ in one ancestor-stack field,
and the full integer-RNG trace (including other generators) has a different
record count. Neither full-trace identity nor complete game-state equivalence
is claimed. Both paired post-state envelopes and all new snapshots validate;
the broad payloads are retained but are not the evidence used to attribute
these boundary changes.

The final focused suite passes 77 tests, including executable counterexamples
to the new native finding. The final release gate also passes 1,203 tests,
782 fixture requests with none missing, clippy, formatting, install survey and
paperwork checks. The capture lane is released; main scores and PR #6
remain unchanged. The new tracer wiring is a separate opt-in adoption concern.

Reproduce the reader with:

```sh
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python \
  tools/explore/shared_search.py CAPTURE/rontrace.log
```

Reports, exact source, capture command, controls and validation are archived at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-shared-search-boundary`.
The next useful boundary is the setup/teardown surrounding A*. The captured
intervening call is real evidence, but treating it as the entire missing work
would discard changes the new observer directly measures.

Follow-up [L87](2026-09-22-shared-prefix-window.md) compares the retained
delegation payloads: both selected callee prefixes preserve all unit/header
bytes through A* entry. This narrows the second gap to before delegation; it
does not settle fresh-search setup or identify the writers.
