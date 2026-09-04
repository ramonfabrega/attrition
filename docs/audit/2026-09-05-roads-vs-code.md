# ROADS.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: 106 rules across §1–§5 and §7–§9 — every stated predicate, constant,
formula and order of steps in those sections, each traced to the code that
implements it (`crates/sim/src/roads.rs`, `mesh.rs`, `terrain.rs`, `city.rs`,
`caravan.rs`, `ai_sites.rs`, `world.rs`). Rows below are the nine where the
code and the document disagree; the other ninety-seven the code implements
faithfully and are not written down.

Blind check: of every function ROADS.md cites by address, only
`World::set_behind@006b4230` appears in
`/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`, and it is named
in §9.4 as *somebody else's* mechanic. Every row below is therefore
**reached**.

## Rows

### R1 — a blocked tile does not stay on the road chain; it is never pushed

| | |
|---|---|
| document | ROADS.md §5, "Reconstruction starts at the arrived node's *parent* and stops before the root, so neither endpoint's own tile is laid; **a blocked tile stays on the chain but takes no road**." |
| code | `crates/sim/src/roads.rs:750` (`fn reconstruct`), lines 759–761 |
| document says | the blocked tile is on the returned chain, and the *laying* step is what refuses it (`place_roads`, `roads.rs:232`) |
| code does | `reconstruct` drops the blocked tile from the answer entirely — it never enters the returned `Vec<Pos>` |
| difference shows | a caravan's waypoint list, not a road tile: `gamelog-run73-greatlakes-caravanstart.txt`'s `[5566, 5573]` search, whose answer `1/23` then walks (`docs/CARAVAN.md` §5.3). No road-tile diff can see it — `place_roads` filters blocked tiles anyway |
| reached | reached (`PathFinder::astar_caravan_road@00685990`) |

The code is right and the document is stale. `astar_caravan_road@00685990`'s
reconstruction loop (decompile lines 321–342) reads the tile mask and
**skips the push** when `& 0x4000`:

```
if ((*(ushort *)(... field_0x138 ... ) & 0x4000) == 0) {
    ... param_1->list[param_1->length] = {x, y, tolerance 0x60, flags 0}
    param_1->length = param_1->length + 1;
}
```

so a blocked tile is absent from the `Stack<PathData>` the caller reads.
`roads.rs:744-749`'s own doc-comment says exactly this and cites
`+0x996`; §5's last bullet was never amended when the caravan arm landed.
The two only agreed while the answer's sole consumer was `place_roads`.

### R2 — §2's gloss on weight `+0xb4` names the wrong predicate

| | |
|---|---|
| document | ROADS.md §2 table, "`+0xb4` \| 200 \| the ground term: `× 4` off a road, `>> 3` on one" |
| code | `crates/sim/src/roads.rs:677-681` (`fn calc_road_cost`) |
| document says | the multiplier is chosen by road-versus-not |
| code does | the multiplier is chosen by **friendly**-versus-not: `let ground = if friendly { GROUND >> 3 } else { GROUND * 4 }`, applied on a road *and* off one |
| difference shows | no observable — the code follows §5.2, which is right; §2's row is a wrong gloss on a right constant. A capture would only show it if the code were changed to match §2, and then every foreign-territory node of run62's 2,913 (`gamelog-run62-roadpath.txt`, frame 100) would price 775 low |
| reached | reached (`PathFinder::calc_road_cost@00686300`) |

The decompile settles it: `calc_road_cost@00686300` lines 106–135 branch on
`local_24.value` — the *friendly* flag set at line 62 — **first**, and the
road test `bVar3 == 0x10` second. The unfriendly arms both add
`pathfinder._180_4_ * 4` (lines 113, 118) and the friendly arms both add
`(_180_4_ + (_180_4_ >> 31 & 7)) >> 3` (lines 128, 134). §5.2's own
pseudocode — `total += friendly ? 25 : 800` — states it correctly, so this is
an internal contradiction inside the document and the code sides with §5.2.

### R3 — the ocean arm's parent test is missing: an ocean-to-ocean step is charged 100 it should not be

| | |
|---|---|
| document | ROADS.md §5.2, "`total += avoid_sea ? 100 : (parent not ocean ? 100 : 0)`" |
| code | `crates/sim/src/roads.rs:633-641` (`fn calc_road_cost`) |
| document says | with `avoid_sea` clear, the 100 is charged **only when the parent tile is not ocean** — it is an entry toll, and a run of ocean tiles pays it once |
| code does | `total += if avoid_sea { SEA_AVOIDED } else { SEA_ENTER }` — 100 on every ocean node whatever the parent was, so the second and later tiles of a sea crossing each cost 100 too much |
| difference shows | `PathFinder::calc_road_cost`'s proxied price (the run62 recipe) on a caravan road that crosses water — East Indies, per §8.3; on the corpus as it stands the nearest reachable capture is `gamelog-run64-islands-caravanroad.txt` frame 6166 onward if any of its nodes are ocean, and `gamelog-run73-greatlakes-caravanstart.txt`'s `[5566, 5573]` if that route touches sea. A land-only building road cannot show it (`valid_roadcoord` refuses ocean with `can_transport` 0) |
| reached | reached (`PathFinder::calc_road_cost@00686300`) |

`calc_road_cost@00686300` lines 33–46:

```
if (local_28.value == 0x20) {            /* the tile is ocean */
  if (pathfinder._140_4_ == 0) {         /* avoid_sea clear */
    if ((<parent tile's mask> & 0x30) != 0x20) {   /* parent not ocean */
      local_14 = local_14 + pathfinder._168_4_;    /* +0xa8 = 100 */
    }
    param_1->z_val = 0;
  } else {
    local_14 = local_14 + pathfinder._164_4_;      /* +0xa4 = 100 */
    param_1->z_val = 0;
  }
}
```

The parent's mask is read through `param_1->parent`, so the original has the
datum; `Node` in `roads.rs:94` keeps only `z_val`, and the ocean arm sets
`z_val = 0` (`roads.rs:631`) for ocean *and* the un-negated land case alike,
so "my parent was ocean" is not recoverable from what the sim stores. Fixing
it needs a flag on `Node`, not a re-read. Note `avoid_sea` is true exactly
when the two endpoints' cells share a region (`find_road@00688a40` lines
113–117), which for a route between two coasts is false — precisely the case
where the parent test bites.

### R4 — `Build::remove_from_city` does not regenerate the city's roads

| | |
|---|---|
| document | ROADS.md §1, "**The bit is set by `City::regen_roads@00738aa0`** … Its callers are `Build::activate` (a building finished) and **`Build::remove_from_city`**." |
| code | `crates/sim/src/city.rs:566` (`fn remove_from_city`); the only `city_regen_roads` call site is `crates/sim/src/city.rs:1443` |
| document says | a building leaving its city flags every remaining member for a replan |
| code does | `remove_from_city` clears the membership and re-syncs territory for a temple, and nothing else — no member is flagged, so no replan follows a building's removal |
| difference shows | `BUILDS`' `build_masks` bit `0x100` on the frame a building leaves a city, and then the `PathFinder::calc_road_cost` draw count sixteen frames later. No capture on disk is known to reach it: run14's only `regen_roads` event is `1/2006` **activating** at frame 167 (§1's table). A capture that destroys or converts a city member — a raid, or `Wall::swap_team` — would show it |
| reached | reached (`City::regen_roads@00738aa0`, `Build::remove_from_city@00622030`) |

`Build::remove_from_city@00622030` calls `City::regen_roads` unconditionally
after the `city_down` chain is spliced and before the temple/lumbermill/
granary/market recounts. The sim's `remove_from_city` is reached from
`city.rs:540` and `city.rs:1678`; neither path flags anything. The
consequence is a *missing* draw and a missing road, and it is a stream
difference the moment it fires.

### R5 — `place_roads` lays the planned road in the reverse of the original's order

| | |
|---|---|
| document | ROADS.md §5, "Reconstruction starts at the arrived node's *parent* and stops before the root"; §9.1, "**each tile is a full pass of the mesh on its own**, not a batch at the end of the frame" |
| code | `crates/sim/src/roads.rs:227-235` (`fn place_roads`), against `crates/sim/src/roads.rs:750` (`fn reconstruct`) and `crates/sim/src/caravan.rs:789-820` (`fn lay_caravan_road`) |
| document says | the original pops its `Stack<PathData>` **from the top down**, and the top is the end nearest the **start** — the building — so tiles are laid from the building outward to the city |
| code does | iterates `reconstruct`'s vector front to back, and `reconstruct` pushes the arrived node's parent first — the **goal** end — so `place_roads` lays from the city inward to the building, exactly reversed |
| difference shows | any tile `Roads::set_diags` lays as a consequence (§9.3), which depends on which neighbours are already road when each tile goes down: the mesh's own tile on `gamelog-run72-greatlakes-marketroad.txt` frame 4803 (the seventeenth tile, `(223, 79)`), and thereafter the `PathFinder::calc_road_cost` price of any later search crossing that ground. It is invisible to a diff only while a road's tiles happen to be order-independent |
| reached | reached (`PathFinder::astar_caravan_road@00685990`, `BuildType::place_roads@0063c580`) |

The stack is written near-goal-first: `astar_caravan_road@00685990` line 317
starts at `pPVar4 = pPVar4->parent` (the arrived node's parent) and walks
`->parent` upward, so the **last** push — the top — is the node one step out
of the root, the building's end. `place_roads@0063c580` then consumes it
top-down (`road.length = road.length + -1; … road.list[road.length]`, lines
144–158 and 240–255). `crate::caravan` reads the same vector the other way
(`for i in (0..nodes.len()).rev()`, `caravan.rs:792`) and its own comment
says so — "which is the near-*start* end first" (`caravan.rs:775`). The two
consumers of one vector disagree, and `roads.rs:227`'s comment ("which is the
near-goal end; the order does not reach the world") is the stale half: it was
true before §9's mesh made each `set_road_at` a full pass.

### R6 — `was_seen`'s territory shortcut is a disjunction; the code has only its left half

| | |
|---|---|
| document | ROADS.md §7.3, "a cell whose owner is an ally … is seen outright when that owner's `reg_cities` **or** `reg_forts` for the cell's *region* is non-zero (`LeaderData +0x125e` / `+0x12de`)" |
| code | `crates/sim/src/ai_sites.rs:183-189` (`fn was_seen_fog`) |
| document says | either count opens the shortcut |
| code does | tests `self.leader_reg_cities(o, r) != 0` and nothing else — a region where the owner holds a fort but no city falls through to the fog read, and `calc_road_cost` doubles the node the original leaves alone |
| difference shows | a doubled `PathFinder::calc_road_cost` price under the run62 recipe, on a node in a fogged cell of a region where the searcher has a fort and no city — the same signature as §7.3's six nodes. No capture on disk reaches it: `ai_sites.rs:146` records that no fort stands in any capture at the sweep, so the term has never been exercised |
| reached | reached (`WorldData::was_seen@006b53f0`) |

`was_seen@006b53f0` tests both arrays in sequence and returns 1 from either:

```
iVar7 = pWVar3[iVar7].region;
if (*(short *)(&pLVar4->list[iVar5].field_0x125e + iVar7 * 2) != 0) return 1;
if (*(short *)(&pLVar4->list[iVar5].field_0x12de + iVar7 * 2) != 0) return 1;
```

The omission is disclosed in `ai_sites.rs:145-147`'s own doc-comment ("Not
modelled: … `reg_forts` (the census does not keep it)") but nowhere in
ROADS.md: §7.3 states the disjunction as the rule and §6's unmodelled list
strikes the `was_seen` entry through as closed. The same comment records two
further gates the code does not have and §7.3 does not mention — the
function's first test is `leader_flags & 0x1000`, `& 0x800` and
`num_units[0x141]`, any of which returns "seen" outright.

### R7 — the terraform's write has a fourth refusal, and §7.4 names three

| | |
|---|---|
| document | ROADS.md §7.4, "**three predicates hold a corner back** — the cell is water by `is_ocean`'s own predicate …, a mountain or cliff tile stands in the corner's own 3×3 …, or the cell carries a good" |
| code | `crates/sim/src/terrain.rs:128-130` (`fn terraform_for_building`) |
| document says | three refusals |
| code does | four: `if d.land != 0 { continue; }` stands between the 3×3 test and the good test, so **any** non-zero land class — not only the shallows and the ocean — is left at its map-generator height |
| difference shows | the height grid, e.g. `gamelog-run72-greatlakes-marketroad.txt`'s `FRAME 4803` block, on a building placed with a forest, rock or mountain **cell** inside its box. `run72_s_road_nodes_are_where_great_lakes_word_parts` pins all 921,600 tiles, so the code's fourth gate is diff-backed; it is the document that is short |
| reached | reached (`TerrainOut::terraform_for_building@00875210`) |

The code is right. `terraform_for_building@00875210` line 135 guards the whole
write with `*(char *)(cell + 2) == '\0'` — the cell's land class — **and**
`((mask & 0x200) == 0 || find_good_at(…, −1) < 0)`. §7.4's third predicate is
the second half of that conjunction; the first half is unstated. It is not a
restatement of the water test either: the water test (lines 111–117, land
`1` or `2` with `flags & 0x100` clear) is a strict subset of `land != 0`.

### R8 — UNSURE: reaching the goal *at* the budget still lays a road here, and does not in the original

| | |
|---|---|
| document | ROADS.md §5, "**The budget** is `traversed >= 0xc80` … tested on each pop before expanding. Reaching it lays no road at all." |
| code | `crates/sim/src/roads.rs:428-443` (`fn step_road`) |
| document says | (ambiguous) the goal test comes first — §5's "The stop is exact" — and the budget is what a pop that is *not* the goal is measured against |
| code does | returns `RoadPlan::Road` on the goal pop unconditionally, and only then tests the budget |
| difference shows | the frame on which a caravan's road finishes: `Caravan::build_road` answering 1 a frame early, and with it the whole `[5566, 5573]` bracket of `gamelog-run73-greatlakes-caravanstart.txt` if that search's final frame ever crosses 3,200 nodes before popping the goal. A building's road cannot reach it — no building search on the corpus exceeds 400 nodes |
| reached | reached (`PathFinder::astar_caravan_road@00685990`) |

The original guards the reconstruction, not the loop: after the goal `break`
at line 190, line 316 is `if (local_2c < 0xc80) { …reconstruct… return 1; }`
and otherwise falls into `LAB_006860bc`, the budget path — which for a
caravan reinserts the goal node and parks the search, and for a building
returns −1. So a search that pops the goal with 3,200 or more nodes already
costed lays nothing and (for a caravan) resumes next frame. What would settle
whether it matters: the per-frame `calc_road_cost` counts of run73's eight
frames — if any is at or above 3,200 on the frame the search finishes, the
two disagree.

### R9 — UNSURE: the friendly test drops the original's second conjunct

| | |
|---|---|
| document | ROADS.md §5.2, "`owner is me, or a mutual ally, // and is_ally(owner, whoB) → friendly`" |
| code | `crates/sim/src/roads.rs:644-656` (`fn calc_road_cost`) |
| document says | friendly needs **both** halves: the owner is the searcher or a mutual ally, **and** `is_ally(owner, whoB)` — the *other* endpoint's owner |
| code does | `Owner::Player(o) if o == who.0 => true` — the first half only. The `is_ally(o, who.1)` conjunct is absent even from the "owner is me" arm |
| difference shows | a `calc_road_cost` price of 540 rather than 0 (and the ×4/>>3 ground term with it) on the searcher's own territory when the two endpoints belong to different, non-allied leaders. Nothing on the corpus reaches it: `place_roads` always searches a building to its own city centre, so `who.0 == who.1` and the conjunct is reflexively true |
| reached | reached (`PathFinder::calc_road_cost@00686300`) |

`calc_road_cost@00686300` lines 58–63 are one `if` with two clauses joined by
`&&`: the ownership/mutual-alliance test, and
`LeaderData::is_ally(leaders.list[owner], param_3)` where `param_3` is `whoB`.
§6 lists "the alliance arm of the territory test" as unmodelled, which covers
the *mutual-ally* half of the first clause; it does not obviously cover the
second conjunct, which bites on the plain "owner is me" path too. What would
settle it: whether any caller can hand `find_road` two endpoints of different
owners — `place_roads` cannot, and `crate::caravan` should be checked.


## Adjudication — 2026-09-05, Opus

Every row above was re-checked against the source. **Nine confirmed, none
struck.** Three are the document lagging the code (R1, R2, R7) and six the
code lagging the document (R3, R4, R5, R6, R8, R9); the six are recorded in
`docs/ROADS.md` §6 as stated, unimplemented and unreached, each with the
reason no capture reaches it.

### R5 is settled, and `crate::roads` is the wrong consumer

The capture could not separate the two consumers —
`run72_s_world_after_the_market_s_road_is_the_original_s` compares the world
on the frame *after* the Market's road is laid and the road tiles agree
exactly, `set_diags`' seventeenth at `(223, 79)` included. So it was settled
by reading, and it settles cleanly.

`astar_caravan_road@00685990`'s reconstruction (export lines 316–342) starts
at the arrived node's parent and walks `->parent` upward to the root, and it
**appends**: `param_1->list + param_1->length`, then `length + 1`. So
`list[0]` is the node nearest the **goal** and `list[length − 1]` the node
one step out of the root, the **start**.

`place_roads@0063c580` consumes it from the top — `road.length = road.length
+ -1;` and then `road.list[road.length]`, at lines 148 and 245, both arms. So
the original lays the **near-start end first**: outward from the building
towards the city.

`reconstruct` (`roads.rs:750`) builds its `Vec` in the original's own push
order, so `out[0]` is the goal end. Therefore:

- `crate::caravan`'s `lay_caravan_road` (`caravan.rs:792`), which walks it
  with `.rev()` and calls that "the near-*start* end first", **matches the
  original**.
- `crate::roads`' `place_roads` (`roads.rs:229`), which walks it front to
  back, is **reversed**, and its comment — "which is the near-goal end; the
  order does not reach the world" — is wrong twice: the end is the goal's,
  and since §9's mesh the order does reach the world.

The fix is `place_roads`' loop and belongs to the main lane; this lane does
not write simulation logic. It is unobserved on the corpus and would stay so
until a capture where two of a road's tiles are diagonal neighbours of
*different* standing roads.

### The fourteenth row

Not in the reader's list, and found by the widening: the tile-mask residue
is 32 at frame 4802 and **45** at 4803, and every one of the thirteen the
road frame adds is `World::set_behind@006b4230`'s `0x4` alone, around the
Market's own ring at `[223, 227] × [78, 81]`. Nothing in `Roads` writes that
bit — it is the Market **finishing** and `Wall::mark_behind_tiles@0063d230`
not running for it. Reached, on a frame the corpus dumps, and pinned by
`run72_s_world_after_the_market_s_road_is_the_original_s` as a count that
may only fall and as a *kind*: every differing tile must differ by `0x4` and
nothing else.
