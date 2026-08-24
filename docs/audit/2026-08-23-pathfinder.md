# Second reading — the pathfinder (2026-08-23)

The working agreement's blind pass, run the same day the mechanic landed.

**Method.** The first reading (`docs/PATHFINDER.md`, the main thread) was
followed by two blind readers on Opus — one on `astar_path@00683770` and its
machinery, one on `calc_cost@00684e50` and the three grid wrappers — each
forbidden the repository and given only the decompile export, with the PE
listing allowed for garbled locals. An Opus adjudicator then took every
point of tension between the three documents back to the decompile or the
listing, re-deriving rather than trusting either side, and produced 32
numbered verdicts under marker discipline. The main thread ratified each
verdict — re-checking the five behavioural ones against decompile passages
it had itself read — and folded the results into the document and the
implementation. The working papers (blind reports, adjudication) live
outside the repository with the rest of the decompiler output, at
`~/ghidra-projects/reports/pathfinder/`.

**The headline: the structure was doubly confirmed.** The blind derivation
independently reached every load-bearing fact of the first reading — the
LIFO tie-break on equal f, `PathFinderData` at `PathFinder+0x40`, the
expansion wheel starting at the goal-ward cardinal with a preference that
never changes mid-search, the budgets (3200 / 32000, `limit` only under
`anti_unit`), `anti_unit` meaning "this is the unit grid", the Manhattan
arrival radius `tol/2 + stride`, the metric advancing ±1/±W regardless of
stride, and the reconstruction roots. As with every previous audit, the
corrections were at the **edges**: branch reachability, guard clauses, and
one table the first reading had refused to trust.

## The verdicts

FIRST = the first reading stood; BLIND = the blind reading corrected or
enriched it; both-right rows record scope differences, not conflicts.

| # | subject | verdict | consequence |
|---|---|---|---|
| V1 | the two unit-grid RNG draws are gated (target-is-unit; `+0x20 < 13`), the `safe += 30` is not | FIRST | doc |
| V2 | `repaths` decay: halved, snapped to zero below 3 | FIRST | doc |
| V3 | the 24 corner-cutting probe offsets, byte-verified | BLIND | **behavioural** |
| V4 | corner-cutting is world-grid only; the tile branch jumps over it | BLIND | doc |
| V5 | fleeing/rush reach world-seen + tile only; diplomacy world-seen + fog | BLIND | doc |
| V6 | the no-rush guard's garbled clause is `rr < 9 \|\|` | BLIND | doc |
| V7 | `find_wpath` treats astar's −1 as success (unreachable there) | both right | none |
| V8 | big-unit diagonal probes memoise under a world-unit key — key-space collision, likely an original bug | BLIND | doc |
| V9 | `needs_transport`: 1 = disembark (free), 2 = embark (charged) | FIRST | doc |
| V10 | arrival tolerance = the entry left on top after both pops | both right | none |
| V11 | `invalid_loc`'s full parameter/return spec | BLIND (enrichment) | doc |
| V12 | early-exit tolerances genuinely differ per wrapper (`find_tpath` forces 0 and has the flyer test; `find_wpath` keeps the entry untouched) | both right | none |
| V13 | PDB names: `UnitData::safe` (+0xb2), `WData.blocked` (+0x11), `WData.solid` (+0x13) | BLIND | doc |
| V14 | `leaders.flags & 4` **is `is_human`**; `army`/`worker` are AI-only | BLIND | **behavioural** |
| V15 | the pull-back guard's disjunction; the `invalid_loc` clause is per sea-domain **unit** | BLIND | doc |
| V16 | the human variant's guard, break set, and fog rejoin gate | BLIND | doc |
| V17 | the pull-back give-up returns **without** running A\* | BLIND | **behavioural** |
| V18 | ±4 owner and +200 land terms are non-ocean only | BLIND | doc |
| V19 | the additive clamp sits after the army terms (conclusion unchanged) | BLIND | doc |
| V20 | river ×3 skipped on any `needs_transport > 0`, transporter or not | BLIND | **behavioural** |
| V21 | the unit-grid ×4 sits outside the embark guard | BLIND | **behavioural** |
| V22 | the corner drop needs `== 0x30` on **both** axes; ref stays put on it | BLIND | doc |
| V23 | reconstruction's `0x10`/`4` flags are per emitted node | BLIND | doc |
| V24 | the gate scan truncates the emitted path at the gate nearest the start | both right | doc |
| V25 | region-crossing `avoid_land = 1` — byte-verified (closes a §12 item) | both right | doc |
| V26 | `vector_dist` operands are node→goal — byte-verified (closes a §12 item) | both right | doc |
| V27 | `anti != 0` never takes the failure path | BLIND | doc |
| V28 | blind-a's `z_val` staleness worry — false alarm, the recycler zeroes | both wrong | doc |
| V29 | `saving`/`limit` are cross-call globals; `find_tpath` obeys `saving` too | BLIND | doc |
| V30 | the `+0x20 < 13` gate confirmed; the field stays unnamed | both right | doc |
| V31 | child metric ±1/±W regardless of stride — confirmed | BLIND | doc |
| V32 | the search-loop constants re-verified | both right | none |

## What changed

**In the implementation** (`crates/sim/src/path.rs`, same-day):

- **V14** — `army`/`worker` are now withheld from human players.
  `LeaderData::is_human` is literally `leader_flags & 4` (the bit was
  already established in `docs/ORDERS.md` §8; neither reading had carried
  it across), the sim already had `Nation::human` on the same bit, and the
  harness now sets it from the dump's `leader_flags`. Dormant while the
  danger/diplomacy terms are seams — but player 0 of `gamelog-run6` is the
  human, so it would have surfaced the day one of them landed.
- **V17** — `find_wpath`'s pull-back walk gained its real give-up exit: a
  remainder smaller than the current step on both axes pushes the goal
  where it stands and returns **without A\***. The interim no-progress
  guard is gone.
- **V21** — the unit-grid `extra ×= 4` moved outside the embark guard.
- **V11** (naming) — the path flag `0x4` gained its second name,
  `path_flag::TRANSPORT`, and the pathfinder uses it where the meaning is
  the transport hop, not the turn-in-place.

**In the document**: V3's probe table adopted whole (§5.1), §5.1 re-headed
world-grid-only (V4), the §5 table rows corrected for V5/V6/V18/V19/V20/V21,
§6 given `invalid_loc`'s full spec (V11), §7's flags made per-node (V23) and
the gate truncation stated (V24), §3's wrapper prose corrected for
V12/V14/V15/V16/V17/V22, §2's `saving` row given the cross-call hazard
(V29), and §12 rewritten — four items closed, the V8 key-space bug and the
gate-bits behavioural check added.

**Not changed**: V2 and V9, where the blind reports' prose was wrong and
the first reading stood; V7/V10/V12, where both were right at different
scopes; V28, where both readings had missed that `Recycler::pop` zeroes the
field in question.

## The pattern, third time

The first reading's arithmetic and structure survived intact again; the
blind pass earned its hour at the edges — reachability of cost terms per
branch, the sense of a flag nobody had interpreted (`is_human`), an exit
nobody had transcribed (the give-up), and a table the decompiler had
mangled that only the listing could settle. And one lesson is new: **check
the project's own established facts before leaving a flag uninterpreted** —
`is_human` was sitting in `docs/ORDERS.md` §8 the whole time, and both
readings re-derived around it.
