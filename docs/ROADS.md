# Roads

*Established 2026-08-28 from the decompile (`tools/ghidra/`), the PE listing
(`llvm-objdump`) for every arithmetic step, and four oracles: run14's
draw-site trace, run10's start-of-game `WORLD=6` block, run13's `DUMP_ALL`
window at sim-frame 95, and **run32**, which drops two buildings on fresh
ground and catches the road they lay. Confidence: **high** throughout. The
search costs the original's nodes exactly on every capture that shows it a
frame's own world, and the road it lays on ground the map has never had one
on is the original's tile for tile; the "six per cent" §7 chased for a
session was the harness feeding it another game's terraformed heights.*

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
| `+0xb4` | 200 | the ground term: `× 4` off a road, `>> 3` on one |
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
  the root, so neither endpoint's own tile is laid; a blocked tile stays on
  the chain but takes no road.

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

was_seen(tile >> 1, whoA) == 0    → total × 2
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
|node.y − goal.y|)`.

**Unmodelled, and stated as such:** `place_roads`' `REGEN_TOTAL` arm and its
`set == 0` teardown; `BuildType::mask_me`'s call; the caravan itself — the
resumable search that stashes its containers on the caravan, its eight-way
wheel, its `× 40 / 16` heuristic and the `× 7 / 5` a diagonal pays; the
ocean arm of the cost; the alliance arm of the territory test; and
`was_seen`'s ally-territory shortcut, which on every capture so far agrees
with the fog bit because the search never leaves its own ground.

## 7. What is not established — two counts on a placement frame

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

### 7.1 The two counts that are still short

run32 (`docs/ORACLE.md`) places a Granary at tile (6, 171) and a Smelter at
(33, 161) from the cheat channel at sim-frame 100. Both roads come out
**tile for tile the original's** — 62 tiles with the rings — and both counts
do not:

| the search | ours | the original's |
| --- | --- | --- |
| Granary → centre, on the pre-terraform grid | 1,046 | **1,043** |
| Smelter → centre, likewise | 1,460 | **1,870** |

The split is the trace's, not an inference: `Wall::activate` plays a sound
off a *different* generator, so the one non-sync draw inside frame 100 falls
between the two searches.

**What the same capture settles about the terraform.** The grid the search
reads on a placement frame is the **pre**-terraform one: under run32's
frame-104 heights, which have both new footprints flattened, the Granary's
search costs 967 nodes and lays a *different* Smelter road; under run13's
frame-100 heights it costs 1,046 and both roads are exact. So
`terraform_for_building` runs after `place_roads`, and the 128 corners it
moves — the two footprints' boxes and nothing else — are that capture's own
before-and-after (`docs/QUEUE.md` item 57).

**What has been ruled out for the remaining two**, each by measurement:

- The cost function's terms and the endpoints: six other searches of the
  same code, on the same map, cost the original's nodes exactly (§6).
- The territory arm: forcing every unowned cell friendly changes neither
  count — no cell either search touches is unowned.
- The fog: forcing `was_seen` true moves the Smelter to 1,624 **and a
  different road**, so the fog as the dump carries it is the one being used.
- The route: the north/south choice west of the Smelter's ring is a coin
  flip in this cost model (fifty other words: north 20, south 19), and with
  the pre-terraform grid ours takes the original's.
- A boundary elsewhere in the 2,913: no split `B + n = 2913` is consistent
  with our own second search from the word before draw `B`, scanned one by
  one over 600–2,200.

**Where a next session would look.** The two searches that are short are
both on the frame a building is *placed*, and the one that is short by 410
is the **second** of that frame — which is where `PathFinder`'s own state
between two searches would show up (the `Recycler<PathNode>` pool, the
containers' reuse). The first is short by three, which is the size of a
rounding difference: `rondata::diff` means the heights in exact millionths
and the original does `(f32 + f32) × 0.5f` and truncates (item 58). Neither
is on the stream's critical path any more — the road frames of the traced
games are exact — so this is a residue, not a blocker.

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
