# Same-packet world grids and explicit logger projections

This is score-neutral follow-up on `codex/typed-leader-bridge`, stacked after
[PR #7](https://github.com/ramonfabrega/attrition/pull/7). No new capture was
needed. Recommendation remains **pilot**: broad scalar agreement is now
measured, but whole-frame parity, allocator liveness and atomicity are not.

## Result

The retained Great Lakes packet at logger frame 11186 reproduces **505,425 of
602,211 printed occurrences**, with zero mismatches in the compared scope.
They represent **174,111 distinct (address, PDB path) storage references**;
331,314 comparisons repeat a reference. The distinction matters: this frame
prints WORLD three times, not three different worlds.

| Addition | Distinct values or records | Printed occurrences agreeing |
| --- | --- | --- |
| Explicit leader resource/diplomacy projections | Four leaders; repeated and split-key output checked separately | 276 |
| Tile masks, three visibility grids, world-coordinate visibility, eight danger grids | 111,600 scalar values | 334,800 |
| World-cell records | 3,600 cells, 15 scalar fields each | 162,000 |

These additions start from the previous 8,349 matches. Every WORLD block is
compared independently. All three copies agree with the retained grids and
cell fields. Tests corrupt a value in just the middle copy and require that
copy's mismatch to survive.

Remaining occurrences are explicit: 81,404 unmapped, 15,181 unclassified text,
and 201 ambiguous logger-ownership rows. Among the larger remaining scopes:
WORLD has 30,582 unchecked occurrences and LEADERDATA has 47,146. Equality in
the separate leader candidate inventory is not added to these results.

## Projection evidence and falsifiers

`WorldData::log_data` at `006b6080` walks the world-coordinate, tile, fog and
region grids using their respective dimensions. The world root supplies those
dimensions and typed pointers. The tool requires each stored size to equal its
width times height, bounds element and byte counts, and obtains element stride,
field offset and signedness from the PDB. The fixed danger-pointer array is
also discovered from decoded fields rather than inferred from the payload.
Missing memory, null pointers, dimension disagreement or a logger cardinality
mismatch cannot become an agreeing empty comparison.

`WData::log_data` at `006af7e0` emits a land label, flags, goods, additional
numeric fields, an optional pointed collision mask, and the terminating
`was_seen`. There is no separate BEGIN marker for each cell. The bridge requires
exact flags-before-goods boundaries, the full ordered numeric-field sequence,
a unique terminator and the root's exact cell count. It leaves land labels,
flag-name text and collision-mask referents unchecked. Unrelated flattened
WORLD flags are not assigned to cells. Layouts come from the pointee type;
this emitted-field projection remains an explicit logger contract.

`LeaderData::log_data` at `006e5110` emits seven ordinary resource arrays,
prints each gather-slot-high twice, splits the last bonus-cap element onto
`bonus_cap[NUM_COMMON]`, and repeats `got_diplo_message` inside the diplomacy
loop. The PDB's econ and chat-status array extents bound those projections.
The reader's pointer-relative resource fields were checked against the PDB
addresses and the executable's string labels. Tests corrupt either duplicate,
the final cap, the repeated scalar, array cardinalities and record identities.
They never deduplicate the logger to make the values agree.

These are authored descriptions and formulas. All exports, snapshot bytes,
labels read from the install and decoded reports remain outside Git.

## What the result does not establish

The acquisition checked selected roots before/during copying and after the
logger returned. It did not anchor these grid referents. Agreement across all
three printed copies provides evidence about the compared fields at their
observation points, not a freeze of every thread or proof against ABA changes.

Neither the 174,111 storage references nor the payload's candidate vtable hits
are live-allocation counts. A PDB field not bridged here may already be printed
under a different name. The coverage report therefore calls it unbridged, not
absent from the logger. Exact terrain heights remain available separately and
are not smuggled into the scalar match count.

The result also does not answer the market decision's cause. Trace tick 11185
has finished at this snapshot boundary; a decision requiring intermediate
state still needs a probe inside that decision or retained-call replay.

## Reproduction and validation

`typed_oracle_compare.py` takes install, capture directory, bound export,
plan JSON and a new external output directory. It revalidates and decodes the
packet, extracts the complete logger frame, runs the scalar/leader/world
bridges, and writes `comparison.json.gz`, `coverage.json` and `manifest.json`.
The manifest records source hashes, input identities, timings, errors and the
explicit partial-comparison status. Existing outputs and destinations inside
Git worktrees are refused. A failed stage leaves a failed manifest; exit zero
means analysis completed, never whole-frame parity.

All 78 acquisition and typed-oracle tests pass, including native producer,
emitted-hook, decoder, projection, accounting and runner failure controls.
Fresh end-to-end reproduction from the packet completes in 20.71 seconds,
returns the same 505,425 matches with zero mismatches or bridge errors, and
retains the complete comparison compressed. Commit `cc0bc7e` also passes the full release gate: 1,278 release tests,
885 fixture requests with none missing, clippy, formatting and paperwork
guards. The [GUY follow-up](2026-09-22-guy-state-bridge.md) extends this result. The preceding background gate for
`f87c774` ended without a completion verdict and is **not** counted as passing.
The earlier `851653a` full gate remains a valid result for that earlier commit.

Evidence is external under the existing typed-state-market experiment's
`followup` directory. The [steering index](STEERING-INDEX.md) links the
checkpoints; adoption stays with Fable.
