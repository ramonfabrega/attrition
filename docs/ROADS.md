# Roads

*Established 2026-08-28 from the decompile (`tools/ghidra/`), the PE listing
(`llvm-objdump`) for every arithmetic step, and five oracles: run14's
draw-site trace, run10's start-of-game `WORLD=6` block, run13's `DUMP_ALL`
window at sim-frame 95, **run32**, which drops two buildings on fresh ground
and catches the road they lay, and — since 2026-09-02 — **run62**, which is
run32 again with the search's gate and its price proxied, so the original's
own price for every node it costed is on the record. Confidence: **high**
throughout, and the last two counts that were not exact are exact: the
search now costs the original's nodes on every capture, **node for node**,
and lays the original's road tile for tile. The two mechanics that stood
between are §7.3's `was_seen` and §7.4's terraform; the "six per cent" §7
chased for a session before them was the harness feeding it another game's
terraformed heights.*

The mechanic is `docs/QUEUE.md`'s item 54, and its name was wrong. The draws
appear under `PathFinder::calc_road_cost+0x46 < astar_caravan_road+0x52b <
find_road+0x3a8`, and the middle name says caravan — but **no caravan is in
the game on the frames it runs**. The caller is a *building*, replanning the
road that joins it to its city centre, and `astar_caravan_road` is simply
the one road search the engine has: its `caravan` argument is `−1` here, and
that sign is what turns the wheel from eight directions to four.

## 1. The schedule — `Build::process@0061edf0+0x1377`

`WallData::build_masks & 0x100` is "my roads want replanning". Each frame,
`Build::process` tests it and fires when

```
(game->frame + object number) % 16 == 0
```

clearing the bit and calling `BuildType::place_roads(x, y, o, who, 1,
REGEN_FORCE)`. The object's own number staggers a city's buildings across
sixteen frames.

**The bit is set by `City::regen_roads@00738aa0`**, which walks a city's
member chain from the centre through `BuildData::city_down` and sets it on
every one — the city included. Its callers are `Build::activate` (a building
finished) and `Build::remove_from_city`. `place_roads`' own `REGEN_TOTAL`
arm sets it too, but nothing in a traced game reaches that arm, and it would
never reach a farm or a city, which is how we know `regen_roads` is the
writer that matters.

**But the flag is not how a road first appears.** `Wall::start@0063e810`
calls `Wall::mask_me(this, 1, REGEN_FORCE)`, and the tail of
`BuildType::mask_me@006312a0` is `place_roads(this, x, y, o, who, 1,
REGEN_FORCE)` — so a building lays its ring and plans its road **the moment
it starts**, and the
flag is what makes it happen *again* sixteen frames later. Every other
`mask_me` caller passes `REGEN_NONE` (`Build::activate` line 146,
`Build::finished` line 113) or `REGEN_SIMPLE` (`Wall::close`), and those
plan nothing.

No traced game had shown this before run32, because none had placed a
non-farm building: the setup's own go up before frame 0 and the AI never
gets past its citizens. run32 places two, and frame 100's **first** 2,913
draws are the two searches — before phase 1, out of the channel's `add`.
`Sim::start_building` models it.

**This is diff-backed, and exactly.** `build_masks` is in every `BUILDS`
dump, and on run14 (`gamelog-run14-trace.txt`) its lifetime is the rule
above, object for object:

| object | type | flag set | fires | road draws |
|---|---|---|---|---|
| `0/2000`, `1/2000` | Small City | setup | frame 0 | none — §3's city arm |
| `0/2006` | Market | setup | **frame 10** | **220** |
| `0/2005`, `1/2005` | Library | setup | **frame 11** | **248** |
| `0/2004`, `1/2004` | Farm | setup | frame 12 | none — a gatherer |
| `0/2003`, `1/2003` | Farm | setup | frame 13 | none |
| `0/2002`, `1/2002` | Farm | setup | frame 14 | none |
| `0/2001`, `1/2001` | Woodcutter | setup | frame 15 | none |
| `1/2006` | Farm | — (placed frame 1, `build_masks 0`) | — | — |
| every `1/*` | | **frame 167** | 2005 at **frame 171** | **189** |

Frame 167 is `1/2006` — the AI's first farm, placed at frame 1 —
**activating**, and `Build::activate` regenerating its city's roads. The
three road frames on run14 are 10, 11 and 171 and nothing else, which is
what the trace says, and every frame in between where the flag fires belongs
to a type §3 turns away.

## 2. The weights

`PathFinder::init@00689ec0` copies two sixteen-byte `.rdata` literals into
`PathFinder+0xa0` and `+0xb0` with a pair of `movaps` (`00b69a30`,
`00b69a60`, read out of the PE), then writes 8 to `+0xc0`:

| offset | value | what it buys |
|---|---|---|
| `+0xa0` | **55** | every node's base |
| `+0xa4` | 100 | an ocean tile while `avoid_sea` |
| `+0xa8` | 100 | stepping from land onto ocean |
| `+0xac` | 540 | a cell owned by someone who is not a friend |
| `+0xb0` | 240 | a cell nobody owns |
| `+0xb4` | 200 | the ground term: friendly territory against not, applied on a road and off one alike (§5.2; *row corrected 2026-09-05* — it used to contradict §5.2) |
| `+0xb8` | 60 | rough going — `× 2` for rock, `× 1` for a river tile |
| `+0xbc` | 600 | the climb's clamp, before `× 3` |
| `+0xc0` | 8 | **not read** by the cost function |

`docs/PATHFINDER.md` §2 called these "out of scope"; they are in scope now.

## 3. `BuildType::place_roads@0063c580`

Three arms, chosen by two predicates on the **type**:

- `BuildTypeData::is_city` = `is(VILLAGE 0x19e, 0)`.
- `BuildTypeData::connects_to_roads@006398c0`: a **gatherer**
  (`build_flags & 0x40`) or a type that needs no city (`& 0x10`) connects
  only if it `is(UNIVERSITY 0x1a4, 0)`; everything else connects.

| the type | the ring | the road to the centre |
|---|---|---|
| a city | yes | **no** — it *is* the destination |
| connects to roads | yes | yes |
| otherwise, and a gatherer | **no** | no |
| otherwise, and in a city | yes, one tile wider | yes |

So a Library and a Market plan a road; a Farm, a Woodcutter's Camp and a
Mine plan nothing at all, which is why run14's frames 12 to 15 fire their
flag and draw nothing.

**The ring** is the border of the box from `corner − 1` to `corner − 1 +
size` — one tile out on the top and left, and *on* the footprint's last row
and column on the bottom and right. A tile is skipped when it is blocked,
when it is ocean, or when it is occupied and sits on the box's first row or
column. Only the third arm widens the box by one more.

That asymmetry looks like a bug and is not: a city stands on seven tiles
each way and **blocks only the inner five** (`BuildType::mask`, the
`7x7 extra space` grid — `docs/DATALAYER.md`), so the box's bottom row and
right column fall on unblocked footprint tiles and take a road. The
original's own map settles it — p0's city centre at tile `(16, 160)`, from
run10's `WORLD=6` block, `R` a road and `B` blocked:

```
      12  13  14  15  16  17  18  19
156    R   R   R   R   R   R   R   R
157    R   B   B   B   B   B   B   R
158    R   B   B   B   B   B   B   R
...
162    R   B   B   B   B   B   B   R
163    .   R   R   R   R   R   R   R
```

A full ring, at exactly the box this arm walks and not one tile wider.

`World::set_road_at@006b43b0` with `set != 0` is the writer: the tile's
surface field becomes road (`& ~0x20 | 0x10`, which takes forest and ocean
alike), and the **cell** gains `WData.flags & 0x80`.

## 4. `PathFinder::find_road@00688a40`

The endpoints, both in tiles:

- a **city**: its own tile, `div3[pos >> 6]`;
- anything else: `corner + size − 1`, the far corner of its footprint —
  the object's own tile for an even footprint, half a footprint past it for
  an odd one.

`WallData::tile_corner@00643440` is `tile − size / 2`; the `+ 0x60` for an
odd size and the round trip through `div3` cancel exactly.

The stack is pushed goal-then-start, so the search runs **from the building
to the city centre**. It returns −1 without searching when either endpoint
is off the map or the two tiles coincide. Before searching it sets
`avoid_sea` = "both endpoints are in the same region" and, because the
caravan index is −1, `can_transport = 0` — which is why no ocean tile is
ever valid and the cost function's ocean arm is dead here.

## 5. `PathFinder::astar_caravan_road@00685990`

A\* on the **tile** grid, stride `0xc0`, and the same containers, tie-break
and wheel `docs/PATHFINDER.md` §2.1 and §4.2 establish for the unit search.
What is its own:

- **Four directions, not eight.** `caravan >> 31` is 1 for `−1`, and that
  gates every **odd** wheel index out. Odd is diagonal, so a building's road
  is cardinal-only — and the `× 7 / 5` a diagonal pays is therefore dead.
- **The stop is exact**: `|dx| + |dy| < 1`, the goal tile itself, with no
  arrival tolerance.
- **The budget** is `traversed >= 0xc80` — 3,200 nodes costed — tested on
  each pop before expanding. Reaching it lays no road at all.
- **The heuristic is two formulas.** The root's is the shared
  `get_estimate` at the tile stride, `vector_dist × 60 / 0xc0`; every later
  node's is `vector_dist × 60 / 0x180`, half as much. Both were read off the
  listing's two divide-by-magic sequences (`0x2aaaaaab`, `sar 5` against
  `sar 6`). The inconsistency is real and harmless: the root is the only
  node in the open list when it is popped.
- **The neighbour that *is* the goal has its cost halved**, as on the unit
  grid.
- **Reconstruction** starts at the arrived node's *parent* and stops before
  the root, so neither endpoint's own tile is laid; a blocked tile
  (`mask & 0x4000`) is **not pushed** onto the chain at all
  (`astar_caravan_road+0x996`; `roads.rs`'s `reconstruct`). *Corrected
  2026-09-05 from the docs-versus-code pass; this sentence used to say the
  tile stayed on the chain and took no road.*

### 5.1 `PathFinderData::valid_roadcoord@00688740`

On the map, and not ocean (`can_transport` is 0). Then, with `blocked` =
`0x4000` and `occupied` = `(mask & 3) == 3 || mask & 0x80`:

```
blocked || (occupied && surface != road)   → the exemption, below
otherwise                                  → valid
```

The exemption: an unoccupied tile is **invalid** — a blocked mountain stops
a road. An occupied one is valid only when it is inside the footprint of an
endpoint **that is a city**; the listing tests `is_city` separately before
each of the two `WallData::covers_tile` calls, so the road may cross the
city's own footprint and never the building's. The diagonal corner
recursion at the tail is dead, the wheel being cardinal-only.

### 5.2 `PathFinder::calc_road_cost@00686300`

**One `Random::get(0, 0xffff) % 0x14` a call, and it is the first thing the
function does** — so a frame's draw count at this site is exactly the number
of nodes the search costed. Two accumulators: `total`, and `removable`, the
part a road step gives back.

```
jitter    = Random::get(0, 0xffff) % 20      ← the shared stream
removable = jitter
total     = jitter + 55

if surface == ocean:                          (dead: valid_roadcoord refuses it)
    total += avoid_sea ? 100 : (parent not ocean ? 100 : 0)
else:
    owner = WData.who of the tile's cell
    owner < 0                       → total += 240
    owner is me, or a mutual ally,
        and is_ally(owner, whoB)    → friendly
    otherwise                       → total += 540
    (mask & 0x6000) == 0x2000 and surface != road  → total += 200
    cell flags & 8 (rock)           → total += 120,  removable += 120
    mask & 0x800  (river)           → total += 60,   removable += 60
    z      = max(find_tcoord_z(tile), 0)
    climb  = min(|z − |parent.z||, 600)
    total += climb × 3 ;  removable += climb × 3
    on a road:
        parent.z < 0 (the parent was on a road too):
            total −= removable ;  if friendly: total /= 2
        else:
            total += friendly ? 25 : 800
        z = −z                                ← the marker the next node reads
    off a road:
        total += friendly ? 25 : 800

was_seen(tile >> 1, whoA) == 0    → total × 2   (§7.3: the shortcut one)
direction is odd (a diagonal)     → total = total × 7 / 5
```

**`PathNode::z_val` carries two things at once**: the tile's height, clamped
at zero, and — in its *sign* — whether the tile is a road. That is what lets
the next node's cost tell "I am continuing along a road" (nearly free: 55,
halved to 27 in your own territory) from "I am joining one" (a full 25 on
top). The root's `z_val` is neither clamped nor negated, so a search that
begins on a road does not get the bonus on its first step.

## 6. Coverage

**Diff-backed:**

- **The search's expansion count, six times over**, against the original's
  own draw record: run14's frames 10 (220 nodes) and 11 (248), and run32's
  104, 105, 106 and 107 (332, 231, 232, 60). Each is seeded with the word
  the trace records *before* the search's first draw, so the jitters are
  the original's and only the search can differ.
- **The road itself, tile for tile**, on ground the map has never had a road
  on: run32's Granary at (6, 171) and Smelter at (33, 161), 62 tiles
  including both rings (§7).
- **The placement-time plan** (§1), against those same 2,913 draws.
- **The height grid the search reads, whole** — run72's `FRAME 4803`, all
  921,600 tiles of Great Lakes at sim-frame 4802, which is what put the
  terraform in the right frame (§7.6). Every cell owner and every tile mask
  of the same frame with it, with the mesh (§9) running for all 4,802
  frames.
- **The Market `1/2015`'s road on run72's 4803, node for node** — all 277
  of them, tile, direction and cost, which is the ring's sixteen tiles *and*
  the seventeenth `Roads::set_diags` lays (§9.4). It was 266 against 277
  until the mesh, and Great Lakes' word was 4803 for it.
- **Every node those 2,913 draws priced**, since run62: tile, direction and
  cost, for both of run32's placement searches, against
  `calc_road_cost`'s own proxied answers (§7.2). This is the record the
  count was standing in for, and it is what closed §7.1 and found
  `crate::terrain`.
- **The caravan `1/23`'s whole road on run73, node for node** — eight
  frames and 22,145 priced nodes of Great Lakes' `[5566, 5573]`, tile,
  direction and cost, and with them the world of 5565: 921,600 heights,
  every cell owner, and every tile mask but the 45 that differ by
  `Wall::mark_behind_tiles`' `0x4` alone. **No tile's surface differs**,
  which is §9.5's assertion (item 206).

- The schedule, object for object, against `build_masks` on run14's frames
  0 to 172 (§1).
- The three frames the search runs on, and the buildings that run it.
- The ring, against run10's `WORLD=6` map and against run13's `DUMP_ALL` at
  sim-frame 95: **95 frames of road regeneration change not one of 57,600
  tile masks**, because everything a ring or a road reaches is already a
  road or is blocked. `rondata::diff::tests::
  run14_s_road_rings_change_no_tile_the_original_does_not` is that guard,
  and widening the city's ring by one column fails it with 26 tiles.
- The weights, read from the PE at the addresses `PathFinder::init` names.
- **The world the search reads**, ninety-five frames in, against run13's
  `DUMP_ALL` block for the same frame: every one of 3,600 cells' owners,
  every cell's flags but one, and all 57,600 tile masks.
  `rondata::diff::tests::run13_s_world_at_frame_95_is_the_original_s_cell_
  for_cell`. The one exception is cell `(52, 22)`'s `BUILDING` bit, which
  nothing in this simulation sets; the cost function does not read it.

**Reading-only, from the listing rather than the decompiler:** every
arithmetic step of §5.2, the two heuristics, the wheel's parity gate, the
budget, `valid_roadcoord`'s guard and its city-only exemption, and the
endpoint derivation. All of these were re-derived a second time, from the
listing again, on 2026-08-28 (§7), together with the containers: the open
tree's LIFO tie-break among equal `value`s, `first_open_node`'s leftmost
pop, the closed set's tombstone semantics, and the heuristic's two
arguments — which the decompiler prints with the goal's `x` and `y`
crossed and the listing settles at `vector_dist(|node.x − goal.x|,
|node.y − goal.y|)` **for a building's arm**. The caravan's arm crosses
them for real; §8.2.

**Unmodelled, and stated as such:** `place_roads`' `REGEN_TOTAL` arm and its
`set == 0` teardown; ~~`BuildType::mask_me`'s call~~ (§9.5, 2026-09-03),
whose own teardown arm remains; ~~the caravan itself~~
(§8, 2026-09-02); the alliance arm of the territory test;
~~`was_seen`'s ally-territory shortcut, which on every capture so far agrees
with the fog bit because the search never leaves its own ground~~ — **it does
not agree**, §7.3, and **the `reg_forts` half of §7.3's disjunction is still
unmodelled** (2026-09-05: disclosed in `ai_sites.rs`, not here, and no
capture reaches it); and ~~the road mesh, which lays road tiles of its own~~
(§9, 2026-09-03), of which the rendering passes and the removal branch
remain — §9.4 lists them one by one.

**The docs-versus-code pass, 2026-09-05** (`docs/audit/2026-09-05-roads-vs-code.md`;
Opus reader, Opus adjudication). 106 stated rules of §1–§5 and §7–§9 were
traced to the code that implements them; nine disagree and all nine are
confirmed against the source. Three are this document lagging the code and
are the lane's to correct — §5's "a blocked tile stays on the chain",
§2's `+0xb4` row (which contradicts §5.2, and §5.2 is the right half), and
§7.4's three terraform refusals where the code has a fourth, `d.land != 0`,
that run72's 921,600 tiles already back. Six are the code lagging this
document, and each is **stated, unimplemented, and unreached** by any
capture on disk:

- **§5.2's ocean arm**, whose `parent not ocean` test the code does not
  have: `crate::roads` charges the 100 on every ocean node rather than on
  the entry to a run of them. Unreached because both constants are 100 and
  every ocean node on the corpus takes the `avoid_sea` arm, where the
  parent is not consulted.
- **§1's second `regen_roads` caller**, `Build::remove_from_city`: the
  sim's `remove_from_city` flags nothing, so no replan follows a building
  leaving its city. Unreached — no capture removes one.
- **§5's reconstruction order.** `place_roads` walks `reconstruct`'s
  vector front to back and `crate::caravan` walks the same vector with
  `.rev()`; only one can be `place_roads@0063c580`, which pops its stack
  from the top. Now falsifiable and **not** falsified:
  `run72_s_world_after_the_market_s_road_is_the_original_s` compares the
  world on the frame after the Market's road is laid and the road tiles
  agree exactly, `set_diags`' seventeenth included. A capture where two of
  a road's tiles are diagonal neighbours of *different* standing roads
  would separate them; none on disk is.
- **§7.3's `reg_forts` half.** The shortcut is a disjunction and the code
  has only `reg_cities`. Disclosed in `ai_sites.rs` and not here; §6's
  "unmodelled" list strikes the `was_seen` entry through as closed, which
  it is not. Unreached — no fort stands in any capture at the sweep.
- **§5's budget.** The original tests `traversed < 0xc80` on the
  *reconstruction*, so popping the goal at or past the cap lays nothing;
  the sim returns the road on the goal pop and tests the budget after.
  Unreached — no building search on the corpus exceeds 400 nodes.
- **§5.2's second conjunct**, `is_ally(owner, whoB)`: absent even from the
  "owner is me" arm. Unreached — `place_roads` searches a building to its
  own city, so the two endpoints share an owner and the conjunct is
  reflexively true.

**And what the widening found instead** (2026-09-05): stepping run72 one
frame further, to the world the search *wrote*, the tile-mask residue goes
32 → 45, and every one of the thirteen the road frame adds is
`World::set_behind@006b4230`'s `0x4` alone, around the Market's own ring at
`[223, 227] × [78, 81]`. Nothing in `Roads` writes that bit: they are the
Market **finishing**, and `Wall::mark_behind_tiles@0063d230` not running for
it. `run72_s_world_after_the_market_s_road_is_the_original_s` pins the
count and asserts the kind — every differing tile differs by `0x4` and
nothing else — so a residue that changes in character fails rather than
hiding inside a tolerance.

## 7. The placement frame, and the two mechanics behind its two counts

**The twelve nodes a search was short were never the search's.** For a
session §7 read: 208, 222 and 178 costed nodes against the original's 220,
248 and 189, with the world, the stream and the schedule all verified
identical and three readings of §4–§5 agreeing line for line. The cause was
outside all of it. `rondata::diff::borrow_from_siblings` fills a dump's
missing fields from a sibling capture, and `Initial::heights` was being
taken from **run3** — the same seed, map style and size as run10–14, but
`GAME_RULES 0` rather than 1, so its starting buildings stand elsewhere and
`TerrainOut::terraform_for_building` flattened different ground before its
first frame. The two grids differ on **237 corners, from (7, 83) to
(230, 163)** — over the human's own city as well as the AI's. A search a
third of whose cost is the climb term cannot survive that, and it did not:
with the map's own heights (run12's and run13's, which agree exactly),
run14's frames 10 and 11 cost **220** and **248**, the original's numbers
exactly. `same_start` is the guard — a sibling's heights are borrowed only
when every starting building matches by owner, object number and position.

`Sim::plan_roads` is therefore **on**.

### 7.1 The two counts on a placement frame, and the day they closed

run32 (`docs/ORACLE.md`) places a Granary at tile (6, 171) and a Smelter at
(33, 161) from the cheat channel at sim-frame 100. Both roads came out
**tile for tile the original's** — 62 tiles with the rings — and for a month
neither count did:

| the search | ours, before | the original's |
| --- | --- | --- |
| Granary → centre | 1,046 | **1,043** |
| Smelter → centre | 1,460 | **1,870** |

The split is the trace's, not an inference: `Wall::activate` plays a sound
off a *different* generator, so the one non-sync draw inside frame 100 falls
between the two searches, at 1,043 of 2,913.

Every hypothesis §7 could reach was ruled out by measurement — the cost
function's terms and the endpoints (six other searches of the same code, on
the same map, cost the original's nodes exactly); the territory arm (forcing
every unowned cell friendly changes neither count); the fog *as the dump
carries it* (forcing `was_seen` true moved the Smelter to 1,624 and a
different road); the route (a coin flip in this cost model, and ours took
the original's); and a boundary elsewhere in the 2,913 (no split `B + n =
2913` is consistent with our own second search from the word before draw
`B`). What no reading could reach was **which node** was priced wrong,
because a count is not a sequence.

**Both counts are exact as of run62, and they were two different mechanics:
§7.3's fog and §7.4's terraform.** 1,043 and 1,870, every one of the 2,913
nodes agreeing in tile, direction and price, and the roads still tile for
tile.

### 7.2 The oracle: proxy the gate and the price together

run62 is run32's own recipe with `RON_CALLWIN=99-101` and three more sites
in `tools/trace/tracer.c`'s `CALLS` table:

| site | what it gives |
| --- | --- |
| `PathFinder::astar_caravan_road@00685990` | the bracket — one plan, entry to return |
| `PathFinderData::valid_roadcoord@00688740` | the candidate's **world coordinate**, and whether it was admitted |
| `PathFinder::calc_road_cost@00686300` | the **price**, which is the answer nothing else records |

The pairing is the point, and it is run61's recipe one map over. A
`calc_road_cost` record cannot name its own tile: the function is handed a
pooled `PathNode *`, so the argument is an address out of
`Recycler<PathNode>::temp_pool`. The tile it prices is the one the
`valid_roadcoord` that returned immediately before it admitted — the two
calls are adjacent in `astar_caravan_road`'s expansion loop with nothing
between them — so proxying the gate *and* the price turns a count into a
per-node record. `rondata::trace::Trace::road_nodes` is the fold and
`sim::roads::RoadCostMark` is the row.

`tools/gamelog/rngcmp.py` says run62's `game_random` word is run32's on all
111 frames, **zero differing**: three more proxies cost the stream nothing,
as run55's two did.

Frame 100 carries 2 `astar_caravan_road` calls, 3,028 `valid_roadcoord` and
**2,913** `calc_road_cost` — the last exactly the draw count, which is what
says the draw is the function's first statement and the count is the count
of nodes costed.

**What it found, in the first run.** The sequences agreed in *coordinate*
for 19 nodes and in *price* for 8 of those 19, and every one of the eleven
differences was **a multiple of three**. Three is the climb's multiplier and
nothing else in §5.2 is a multiple of it, so the disagreement was the
heights and only the heights — a conclusion no count could have reached and
no reading had.

### 7.3 `calc_road_cost` calls `was_seen`, not `was_really_seen`

With the heights corrected (§7.4) the sequence ran to 460 nodes and six
prices differed, each **exactly twice** the original's. Doubling is
`was_seen(tile >> 1, whoA) == 0`, the last line of §5.2, and the six tiles
are all in the searcher's own territory with the `seen2` bit clear.

`WorldData::was_seen@006b53f0` and `WorldData::was_really_seen@006b54f0`
are two functions with one letter between their names, and this called the
bare one. The one `calc_road_cost` calls has a **territory shortcut** ahead
of the fog read: a cell whose owner is an ally — oneself included, `is_ally`
being reflexive — is seen outright when that owner's `reg_cities` or
`reg_forts` for the cell's *region* is non-zero (`LeaderData +0x125e` /
`+0x12de`). `crate::ai_sites`' `was_seen_fog` already had it, from run20's
site census; the road cost now calls that.

**And the shortcut needed a leader-level count.** `reg_cities` is recounted
from the four city types at the census sweep's step 8, and this crate runs
the sweep for AI leaders only (`docs/QUEUE.md` item 158) — so the human's
array is empty and the shortcut could never fire for the player whose road
this is. `Sim::leader_reg_cities` answers from the census where there is one
and from the same recount where there is not, which is the number the
original's array would hold.

### 7.4 A building flattens its ground **before** it plans its road — and a farm never does

`Wall::start@0063e810` is five statements, and the order is the finding:

```
kill_competing_buildings(this)
WallData::tile_corner(&cx, &cy)
Terrain::object_placed(cx, cy, x_size, y_size, 1)   ← the refresh, not the terraform
mask_me(this, 1, REGEN_FORCE)                        ← whose tail is place_roads
… the footprint's own fog cells, check_ever_seen, mark_behind_tiles
```

The flattening runs before `place_roads`, and the road search prices its
climbs off the *flattened* grid. **The call is not the one named above.**
`Terrain::object_placed@00850c40` is the renderer's — `spot_update_land`
and `invalidate_wcoord` over the footprint's cells — and moves no height
at all. `TerrainOut::terraform_for_building@00875210` is called one step
earlier in the building's life, from `Wall::init@0063e9b0:70`, under two
gates:

```
if (param_6 == 0 and type != FARM):
    tile_corner(&cx, &cy)
    terraform_for_building(cx, cy, x_size, y_size)
    Terrain::refresh_good_z()
    Farms::recalc_heights(cx, cy, who)
```

Both land on the same frame for a building placed and started at once —
run32's two enhancers, which is why run62 could not tell them apart —
so §7.1's counts are unaffected either way.

**A farm never terraforms**, and the test is an identity on the type
(`TVar10 != FARM`), not a lineage. run64's frame-6166 block is the
measurement: the AI's three farms had moved **182 tiles of height** that
the original leaves exactly where the map generator put them, and one of
those tiles — a nine-unit climb, priced `× 3` — was where East Indies'
caravan road parted from the original on frame 6168
(`docs/CARAVAN.md` §8). §7.1 had concluded the opposite, from the one experiment available to
it — run32's frame-104 heights make the Granary's search cost 967 — and the
experiment was right about its own grid and wrong about the mechanic: frame
104 carries **both** footprints' terraforms plus the four scheduled replans
behind them, and the Granary's search must see only its own.

`crate::terrain` is the implementation. The box is the footprint grown by
one corner on the near side and two on the far —
`[cx − 1, cx + w + 2) × [cy − 1, cy + h + 2)`, clamped to the corner grid —
and it is walked twice:

- **the mean**, over every corner of the box plus the column and row that
  close it; a corner at or below zero anywhere in that sweep abandons the
  whole terraform, which is how a building on the shore leaves the water
  alone;
- **the write**: the interior takes the mean outright, the box's own border
  takes `(h + mean) / 2`, and **four** predicates hold a corner back — the
  cell is water by `is_ocean`'s own predicate (`flags & 0x100 == 0` and
  land 1 or 2), a mountain or cliff tile stands in the corner's own 3×3
  (`mask & 3` is 1 or 2), the cell's `land` byte is not 0 (`terrain.rs`,
  between the 3×3 test and the good test; *added 2026-09-05*, run72's
  921,600 tiles back it), or the cell carries a good
  (`mask & 0x200` and `find_good_at(…, −1) ≥ 0`).

`World` therefore carries the **corner** grid now, not only the per-tile
table `find_tcoord_z` derives from it: the table was pinned once by the
loader on the reading that nothing ever wrote it, and a building writes it.
`World::retile_z` is the derivation, run over the box and one further out
each way because a tile reads the corner above it and the one to its right.

**The dump's grid is already terraformed for every building the dump
lists** — run12's frame-0 heights have p0's city on its own plateau — so the
harness puts it back after standing the roster up (`start_of_game`'s tail);
flattening flat ground is not the identity when the border blends.

The arithmetic is in **millionths**, the scale the dump prints, not the
original's `f32`. The two part on a truncation on three tiles in 58,081
(`docs/QUEUE.md` item 58), none of them near any search measured so far.

### 7.6 The terraform runs at **placement**, not at start — run72

§7.4 read the call correctly and this module put it in the wrong frame for
a month, because until run72 no capture could tell the two apart.
`TerrainOut::terraform_for_building` is `Wall::init@0063e9b0:70`'s, under
`param_6 == 0` and `TVar10 != FARM`; `Wall::start@0063e810` calls
`Terrain::object_placed@00850c40`, which moves no height. For a building
**placed and started at once** — run32's two cheat-channel enhancers, the
only ones any capture had — both land on the same frame, so the
implementation's `start_building` was indistinguishable from the truth.

A building the AI **builds** is placed and started **226 frames apart**.
Player 1's Market `o 2015` on Great Lakes is placed on frame 4577 and
finishes on 4803, and run72's `FRAME 4803` block — the world the Market's
own road search is about to read — carries the original's height grid
whole. The original's is already flat at **175** across the Market's
4 × 4 and tapered around it; this crate's was still the map generator's on
**62 tiles**, from (223, 78) to (230, 85).

`Sim::init_build` is where the call is now, after `update_hits` and before
`find_city`, gated on `!restore` (the original's `param_6`, which
`Wall::swap_team` passes) and on the type not being a farm. The road
search reads the flattened grid either way; what changed is *when*, and
the diff is `run72_s_road_nodes_are_where_great_lakes_word_parts`:
**921,600 tiles, none off**.

### 7.5 What is still not established here

- The `f32` residue above: the exact-millionths mean and the original's
  `f32` mean differ on three corners of run13's grid.
- `Terrain::object_placed`'s other arms — this models the
  `terraform_for_building` call it makes for a building being placed;
  `prep_terrain_lighting`, `calculate_norms`, `calculate_tangents` and
  `TerrainVis::invalidate_wcoord` are visual and unmodelled by design.
- The terraform's own callers other than `Wall::init`: map generation and
  the scenario editor, neither of which a traced game reaches.
- `find_good_at`'s `who = −1` arm is modelled as "a live good that is not
  oil", which is the fast path with the availability test dropped; no
  capture has put a good inside a terraform box.

## 8. What the road costs the stream

`Sim::plan_roads` spends one `game_random` draw a node costed, in the middle
of the frame the plan runs in, so it is worth stating what turning it on did
to the scores on 2026-08-28:

- **the headline, ticks before divergence: 181 → 202**, and player 0's first
  parting 182 → 213;
- **the first frame whose draws differ from the trace at all: 10 → 18** —
  the number that is not luck;
- `orders` 180 → 168 and the ledger 198 → 173, both of which are past that
  first divergence, where two streams have parted and which of them happens
  to label a frame the same way is chance. `docs/JOURNAL.md`, 2026-08-28.

## 8. The caravan's arm

`docs/CARAVAN.md` is the mechanic — the object, the order and the
schedule. Three things in *this* module change when `astar_caravan_road`
is handed a caravan slot rather than −1, and they are what make a trade
route's road cost thirteen thousand nodes where a building's costs three
hundred. All three are diff-backed against run64, node for node.

### 8.1 Eight directions, and the diagonal's two rules

`local_18 = param_6 >> 31` is 1 for −1 and 0 otherwise, and it gates the
**odd** wheel indices out. A caravan therefore expands all eight, in the
same `pref + 1 … pref + 8` order, and the two rules a diagonal carries
come alive with them:

- `calc_road_cost`'s tail, `total = total × 7 / 5`;
- `valid_roadcoord@00688740`'s own tail, which was dead prose until now.
  A candidate whose `x` **and** `y` both differ from its parent's is
  admitted only if at least one of the two tiles that share a side with
  both — `(cand.x, parent.y)` and `(parent.x, cand.y)` — is itself
  admissible. The recursion is one deep: each corner call shares an axis
  with the parent, so its own diagonal test is false.

### 8.2 The heuristic is wrong, and that is the mechanic

Every non-root node's estimate is one of two arms, on `param_6 < 0`:

```
building:  vector_dist(node.x − goal.x, node.y − goal.y) × 60 / 0x180
caravan:   vector_dist(node.x − goal.x, goal.y)          × 40 / 16
```

The second is the listing's at `00685fd9`: `ecx = node.x − goal.x`, then
`edx = −goal.y`, then the call. **The goal's own `y` coordinate stands
where the `y` difference belongs**, and `vector_dist` takes the absolute
value of both, so even the sign is lost. Read from the disassembly rather
than the decompiler, which prints both arms as `vector_dist(dx, dy)`.

The estimate is therefore all but constant — it varies only as
`dx² / (2·goal.y)` — so two nodes in the same column score identically
whatever their `y`, and the search is a breadth-first flood along that
axis. That is the whole difference between a road that costs 400 nodes
and one that costs 12,965.

**The measurement that says so** is run64's frame 6166. The root is city
2000's tile and its eight neighbours are priced 86 (W), 135 (NW), 96 (N),
133 (NE), 86 (E), 124 (SE), 92 (S), 116 (SW). With the building's
heuristic the second node popped is the **west** one; with an honest
`× 2.5` it is the **north-west** one, 120 units nearer the goal and 49
dearer. The original pops west — which it can only do if west and
north-west carry the same estimate, and they share nothing but their `x`.

### 8.3 The search stops at the budget and carries on

`docs/CARAVAN.md` §5.2. The parked containers, the wheel preference and
the goal go into the `CaravanData`; `traversed` goes with them and is
never read back, so each resumed frame starts its budget again at
`0xc80`. `crate::roads::RoadSearch` is the parked half and
`Sim::step_road` the loop that takes it.

`can_transport` comes alive with the caravan too (`docs/CARAVAN.md`
§5.1): an ocean tile stops being refused, and §5.2's ocean arm — dead for
every building road — prices one at `jitter + 55 + 100`.

## 9. The road **mesh** — `Roads::add_roads`, and the tiles it lays

**Every road tile carries a record, and the record's own upkeep lays road.**
That is the whole of this section, and it is what Great Lakes' word was
short of for a month. `crate::mesh` is the implementation and its `dir`,
`tables` and `Elem` are the names below.

### 9.1 The one door, and what is behind it

`World::set_road_at@006b43b0` with `set != 0` writes two bits and then
hands over:

```
tile.mask = (mask & ~0x20) | 0x10          # road on, ocean off
cell.flags |= 0x80                         # the cell carries a road
if (mask & 0x30) != 0x10 and param_5 == 0: # it was not already a road
    Roads::road_added(roads, x, y, param_4, 1)
```

`param_5` is a recursion stop and only the mesh's own calls set it.
`BuildType::place_roads` passes `(param_4, param_5) = (0, 0)` for every one
of a ring's tiles and for every tile of the road it plans, so **each tile is
a full pass of the mesh on its own**, not a batch at the end of the frame.

`Roads::road_added@008954d0` queues a `RoadModification` — a packed index
`x + width_in_tiles · y`, an `added` byte and a `valid` byte — and, when its
fourth argument is set, runs `Roads::add_roads@0088f4b0` over the queue.

The record is `RoadElementCandidate`, sixteen to a terrain patch
(`RoadsPieces::candidates`, `TerrainOut::get_road_data@008744d0`). Four of
its fields matter here:

| field | what it is |
|---|---|
| `+0x4` `flags` | the top byte is eight **connection** bits; `set_diags` and `mark_and_trim_directions` are its writers |
| `+0xa` `ref_count`, `+0xb` `pending_camel_steps` | two reference counts; the element goes when both reach zero |
| `+0xc` `support_codes` | `0x2000` "I laid the road beside me", `0x80` "eastward" |
| `+0xf` `is_terrain_creation` | this road tile is the mesh's own |

The eight connection bits, settled by which corner each excludes in
`set_diags` and which one `mark_and_trim_directions` writes back on the
neighbour it names:

```
N  0x40000000   E  0x10000000   S  0x04000000   W  0x01000000
NW 0x80000000   NE 0x20000000   SE 0x08000000   SW 0x02000000
```

`RoadsOut::get_orthog_connects@00893560` counts the four cardinals and
nothing else. It answers **zero for a tile with no element**, which is a
gate and not an accident.

### 9.2 `mark_and_trim_directions` — where the cardinals come from

`RoadsOut::fill_cache@00893b30` fills two nine-entry compass caches for the
tile being worked, `[0]` being the tile itself:

```
road_cache[d]     = (mask & 0x30) == 0x10                       # a road
neighbor_cache[d] = (mask & 3) == 3 and not road_cache[d]       # a footprint
```

Off the map both read zero. `RoadsOut::mark_and_trim_directions@008935c0`
then reads them, and two things in it are not guessable from its shape:

- Because `neighbor_cache[d]` **excludes roads**, its gate is open whenever
  the neighbour is one. So `if neighbor_cache[d] == 0: flags |= bit; if
  road_cache[d] == 0: flags &= ~bit` reduces to **`bit = road_cache[d]`** in
  every ordinary case. The gate only ever *withholds a recomputation* beside
  a building, leaving whatever the bit already was.
- The two halves are gated on **each other's** axis. With
  `a = neighbor_cache[N] or neighbor_cache[S]` and
  `b = neighbor_cache[E] or neighbor_cache[W]`: the north–south pair is
  recomputed when `b`, the east–west pair when `a`, and with neither — no
  footprint adjacent at all — both are.

Each bit it sets is answered on the neighbour: setting `N` here sets `S`
there, through `get_road_data(create = 1)`, and a bit that was not already
there puts the neighbour on the redo list.

Its tail is a repair. If the tile ends with **exactly one** connection, it
takes every road neighbour it has, gates and all.

### 9.3 `set_diags` — the pass that lays road

`Roads::set_diags@0088e9d0` runs over the four corners, `corner_x/corner_y`
indices 1 to 4 — NW, NE, SE, SW, whose compass indices are the odd ones 1,
3, 5, 7. Its first line is the mechanic's shape:

> **A tile with two or more orthogonal connections does none of this**, and
> nor does one with no element. What the pass exists for is a road that ends
> or turns beside another one.

Per corner, in order:

1. the corner must be on the map;
2. a cardinal connection **bars the two corners that touch it** — `N` bars
   NW and NE, `E` bars NE and SE, `S` bars SE and SW, `W` bars SW and NW;
3. `road_cache[diag]` — the corner must itself be a road;
4. then `get_orthog_connects(corner)` splits the two arms.

**Under two** — the corner is a road end — the two elements are joined
diagonally: the tile takes its own corner's bit, the corner takes the
opposite one, and the join is refused if either **adjacent** diagonal is
already claimed (NW is refused by NE or SW, and so round). This writes no
world.

**Two or more** — the corner is a junction — and the pass **lays road**, at

```
(x + corner_x[i], y)
```

the corner's *horizontal* neighbour, not the corner. It is refused unless
**both** flanking cardinals — the two compass directions either side of the
diagonal — are free of road **and** of footprint: `road_cache` and
`neighbor_cache` both zero at both. Then:

```
World::set_road_at(x + corner_x[i], y, 1, 0, 1)   # param_5 = 1: no recursion
Roads::road_added(x + corner_x[i], y, 0, 0)       # queue it; do not re-enter
TerrainOut::road_changed(x + corner_x[i], y, 1, 0, 0)
new.is_terrain_creation = 1
this.support_codes |= 0x2000  (| 0x80 if corner_x[i] == 1)
```

The queue entry lands in the very list `add_roads`' second pass is walking,
so the new tile is processed inside the same call.

### 9.4 What run72 says, and what is not established

**The measurement.** run72's frame 4803 is player 1's Market `o 2015`
planning its road to London. `place_roads` lays the ring — sixteen tiles,
the border of `[224, 228] × [79, 83]` — and the original lays a
**seventeenth**, `(223, 79)`. It is not a ring tile and no search put it
there: the ring's brand-new `(224, 79)` has the standing road at `(223, 80)`
diagonally below it, `(223, 79)` and `(224, 80)` are both empty, and
`(223, 80)` is an elbow with two connections. §9.3, exactly.

Without it the search priced that tile as plain ground at **387** where the
original priced it as road at **27**, and the road came out **266** nodes
against 277. With `crate::mesh`:

- **277 nodes, node for node** — tile, direction and cost, against
  `calc_road_cost`'s own proxied answers;
- **Great Lakes' word 4803 → 5502**, by draw *and* by sequence;
- **run71's whole 5,000-frame capture has no unit anywhere off the
  original's point** — the 4827 position parting, which was `1/15` re-picking
  a farm cell off a seed that was nobody's, is gone with its cause;
- the world at 4802 is untouched by it: 921,600 heights, every cell owner,
  every tile mask exactly as they were, with 4,802 frames of the mesh
  running. It lays its **first** tile on 4803, which is the honest limit of
  that evidence — the mesh not over-laying is diff-backed over 4,802 frames,
  the mesh laying is diff-backed at one tile.

`rondata::diff::tests::run72_s_road_nodes_are_where_great_lakes_word_parts`
is all of the above, and it also pins the invariant nothing else would
catch: **every road tile has an element and nothing else has one**, 126 of
them.

**The 33 tile masks are somebody else's mechanic**, and this section had
them wrong for a day. Every one of them is bit `0x4`, whose writers are
`World::set_behind@006b4230`'s low arm — called from
`Wall::mark_behind_tiles@0063d230` (itself from `Wall::start@0063e810`,
`Wall::close`, `Wall::refresh_nearby_tiles` and `SpellType::cast_bribe`) and
from `Mountains::add_mountain@0089c2e0`. **Nothing under `Roads` writes it
at all.** It is the strip of tiles a building stands *in front of*, walked
`buildtype+0x2dc` rows deep, and it is item 203's, not this one's. ~~Three of
the 33 carry a second difference: `(217, 124)`, `(218, 124)` and
`(219, 124)` are road here and are not road there, which no reading in this
section explains.~~ **They are §9.5's** — a farm's footprint, and the road
under it is taken away when the farm starts. The residue is **32** now, bit
`0x4` and nothing else.

**Not modelled, and each of them could write a bit §9.3 reads:**

- **`RoadsOut::mark_splits@00891d40`.** It runs in `add_roads`' second pass
  whenever `set_diags` answered zero, and it **sets cardinal bits** — on a
  tile that already claims a corner pair (`N|W`, `E|W`, …) and has a further
  road neighbour, it claims that one too. Every bit it sets corresponds to a
  road that is really there, so a derivation that simply sets every cardinal
  whose neighbour is a road reaches the same place; that is the argument,
  and it is an argument rather than a measurement.
- The rendering tail of `set_diags` (`element_num`, `rotation`, the `0xc000`
  and `0x1000`/`0x200` support codes) and `mark_white_lines`,
  `mark_yellow_lines`, `mark_intersections`, `fixup_lines`, `find_piece`,
  `leech_codes`, `delete_straglers`. Of these only the tail matters beyond
  texture choice: it calls `get_road_data(create = 1)` on tiles **that are
  not roads**, which is the one way an element comes to stand on a plain
  tile.
- …and that is why `add_roads`' duplicate branch is **unreachable in this
  crate**. It fires when a tile being laid already had an element, re-queues
  it as a removal, and `Roads::clear_roads@0088fef0` +
  `Roads::clear_support@0088e3f0` run — which is where a mesh-laid tile can
  be taken away again. All of it is modelled and none of it is reached; the
  unit test drives it directly and says so.
- `Roads::scan_and_kill_stray_roads@008956a0`,
  `scan_and_kill_bad_tcoord@0088e100` and
  `scan_and_kill_straggled_tcoord@0088e050` — three functions whose names
  say they remove roads, none of them read.
- `Roads::generate@00894350` does not derive anything: it clears the render
  helpers and calls `add_roads` if the queue is not empty. So the elements a
  **map's own** roads carry are built by `set_road_at` at load time, one
  tile at a time, and this crate cannot replay that for a world the harness
  borrowed mid-game. `RoadMesh::seed` reconstructs them instead — an element
  per road tile, flags cleared, `mark_and_trim_directions` over the cache,
  which is `redo_changed_roads`' own second pass. On Great Lakes at 4802
  that reconstruction and the original agree everywhere the diff can see,
  but it is a reconstruction.
- The ambience loop in `add_roads`' first pass and
  `TerrainOut::road_changed`'s patch allocation: visual, and unmodelled by
  design.

### 9.5 The other door — a footprint's own road, laid and taken away

`BuildType::mask_me@006312a0` writes roads, and for a month nothing here
knew it. Its footprint loop runs once per tile of the rectangle, and with
`param_4` set — the building is being *marked*, which is `Wall::start`
(`docs/CITIES.md` §3.6) — it has two road arms that are the same predicate
read from opposite sides:

```
mask &= ~0x40; mask &= ~0x80; mask |= 3
if not is_city and (is_gather_type or FLAGS e) and not is(UNIVERSITY):
    set_road_at(x, y, 0, 0, 0)          # the road under me goes
if template[x_size · v + u] == 1:
    set_blocked_at(x, y, param_4)
else:
    set_blocked_at(x, y, 0)
    if is_city or connects_to_roads:
        set_road_at(x, y, 1, 0, 0)      # and here one is laid
```

`BuildTypeData::connects_to_roads` **is** `(is_gather_type or FLAGS e) ?
is(UNIVERSITY) : true` (§3), so the first gate is exactly
`not (is_city or connects_to_roads)` and the second exactly
`is_city or connects_to_roads`: a footprint tile either loses its road or
gains one, and which it is is a property of the *type*.

Both calls pass `param_5 = 0`, so both go through the mesh door (§9.1) —
the laying arm through `Roads::road_added` and the removal arm through
**`Roads::road_cleared@008955d0`**, which queues the tile with `added = 0`
and `valid = 1` and enters `add_roads` with a zero. That is `clear_roads`'
own path (§9.4), and it is what takes the `RoadElementCandidate` down: a
tile that stops being a road and keeps its element would go on answering
`get_orthog_connects` for a road that is not there.

**What it is worth.** run53's frame 5573 is the caravan `1/23`'s
`Caravan::build_road` answering 1, and Great Lakes' word parted on it —
1,754 `calc_road_cost` draws here against the original's 1,528. run73's
`callwin` carries all eight frames of that search node for node, and the
first node to part is 2,170 of **5572**: the diagonal from tile
`(216, 123)` to `(217, 124)`, which this crate admitted and the original
refused. `(217, 124)` is a footprint tile of player 1's Farm `o 2012`, and
it was **road** here and plain ground there — `valid_roadcoord`'s occupied
arm (§5.1) lets a road through a footprint and refuses everything else, so
one stray tile of tarmac opened a door the original keeps shut and every
node after it was somebody else's.

The road under the farm was laid on frame 1104 by another building's plan;
the farm started on 3465, and the original took it away there. With the arm
in: all eight frames of the search are the original's node for node, the
world at 5565 has **no surface difference anywhere** — the residue is bit
`0x4` alone — and Great Lakes' word runs **5573 → 5786**.

`rondata::diff::tests::run73_s_caravan_road_is_the_original_s_node_for_node`
is that, and it pins the surface residue at zero as well as the search.

**What it has not established.** No capture has reached the **laying** arm
with a visible consequence: on Great Lakes at 5565 and at 4802 the world is
tile for tile the original's with it running, which says it lays nothing
the original does not, but no tile on disk is one it laid. The teardown
(`param_4 == 0`) is still unmodelled here — `mask_building(b, false)` does
not run either arm, and `Build::close` is what would reach it.
