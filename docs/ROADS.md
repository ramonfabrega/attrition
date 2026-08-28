# Roads

*Established 2026-08-28 from the decompile (`tools/ghidra/`), the PE listing
(`llvm-objdump`) for every arithmetic step, and three oracles: run14's
draw-site trace, run10's start-of-game `WORLD=6` block, and run13's
`DUMP_ALL` window at sim-frame 95. Confidence: **high** on the schedule, the
predicates, the endpoints, the ring and the cost function's terms; the
search's **expansion count** is six per cent short of the original's and the
gap is not found (§7), so the search is off by default.*

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

## 7. What is not established — the twelve nodes a search is short

**The search expands fewer nodes than the original's, and the cause is not
found.** With the world, the stream and the schedule all verified
identical, run14 reads:

| frame | the search | ours | theirs |
|---|---|---|---|
| 10 | `0/2006` Market → centre, tile (15, 155) → (16, 160) | 208 | **220** |
| 11 | `0/2005` Library → centre, (12, 160) → (16, 160) | 52 | — |
| 11 | `1/2005` Library → centre, (228, 89) → (220, 84) | 170 | — |
| 11 | both | 222 | **248** |
| 171 | `1/2005` Library → centre, (228, 89) → (220, 84) | 178 | **189** |

**It is a constant, not a percentage** (measured 2026-08-28). The three
deficits are `+12`, `+26` and `+11`, and frame 11 is *two* searches: at
six per cent its 222 would be short by 13, not 26, while `+13` each is
exactly the other two frames' figure. So a search of 52 nodes and a search
of 178 are each short by about a dozen — **roughly three extra expansions,
once per search**, rather than a bias that grows with the path. Anything
proportional — a cost term slightly too small, a heuristic slightly too
large — is the wrong shape for that.

**Why it is the headline.** `Sim::plan_roads` is off, so those 220, 248 and
189 draws are never spent, and by the end of frame 94 the simulation's
stream is **466 draws behind** the original's (`--diff`'s note at frame 94:
"ours 6 draws, the original's 472"). Everything drawn in the untraced
stretch then reads the wrong word — and what draws there is the farms'
sprout (`docs/SYNC.md` §3.3, one coin a frame per farm). Player 0's farmer
`0/3` walks onto its farm on frame 110 on both sides; the original's cell
is **empty** and takes 101 frames of two adds to ripen, so the citizen
re-picks a tile on frame 212 and moves on 213. Here the same cell was
sprouted at frame 49 by a coin off the wrong word and is already 61 adds
old when the farmer arrives, so it ripens on 180, the citizen re-picks on
**181** and moves on 182. That is the headline's own first divergence —
`ticks 181, orders 180, player 0 @ 182` — and it is downstream of this
count, not of anything in `docs/ORDERS.md` §6.5.

**Ruled out**, each by measurement rather than by reading:

- **The world.** Ninety-five frames in, every cell owner, every cell flag
  but one and all 57,600 tile masks are the original's (§6's guard). At
  frame 10 the sim's masks differ from the dump's in 16 tiles, all of them
  the `PLACED` bit on the AI's farm site 200 tiles from the search.
- **The territory term.** Forcing every cell friendly changes no count at
  all: every cell the three searches touch is already the searcher's own.
  Forcing them unowned takes frame 10 to 405 and forcing them foreign to
  304, so the term is live and the map's answer is the one being used.
- **The fog.** Forcing `was_seen` true changes no count: no costed tile is
  unseen.
- **The heights' orientation and scale.** Every dumped object carries
  `z_internal`, and on flat ground it equals `World::tile_z` at the
  object's tile exactly — sixteen of twenty checked objects on run10's
  frame 0, the rest off-centre on a slope, where the object's own
  interpolation and the tile's two-corner mean part company. The transpose
  is nonsense at every one of them.
- **The stream.** Frames 0 to 9 match the trace draw for draw, so the word
  at the head of frame 10 — and therefore every jitter — is the original's,
  and frame 10's comparison carries no jitter noise at all.
- **The cost function**, term for term against the listing, twice, the
  second time including the friendly/foreign split that surrounds the road
  bonus: the halving of `total − removable` is inside the friendly arm, the
  `>> 3` and the `× 4` are the same weight, and the `z` negation happens on
  both arms.
- **`valid_roadcoord`**, likewise, including which endpoint the `is_city`
  guard belongs to (both, separately) and `WallData::covers_tile`'s
  `corner ≤ t < corner + size`. Dropping the guard — exempting both
  footprints — overshoots: 225, 315 and 253 against 220, 248 and 189.
- **The containers.** `Tree::ordered_insert` sends an equal key left and
  `first_open_node` takes the leftmost, so the open list is LIFO among
  equal `value`s, which is what this crate's `(value, Reverse(seq))` does;
  the refs and closed trees are red-black **maps** keyed by metric that
  overwrite on an equal key, and a `seek` reads a tombstoned entry as
  absent. No duplicate-metric behaviour to model.
- **The wheel and the preference**, against the listing's `setg`/`jle`
  pair, and `move_x`/`move_y`'s 1-based compass; reversing either is worse.
- **The endpoints and the stack order.** Four readings of `road_end` and
  two of the city's were tried; every alternative is further away than
  `corner + size − 1`.
- **The search's direction**: running it from the centre to the building
  gives 94, 194 and 139.
- **The ring**: with the pad the map proves, it lays nothing, so it cannot
  be moving the search.

A constant added to each off-road step lands the count on frame 10 at
`+5` — but it overshoots frames 11 and 171, so it is not a missing
constant of that shape, and the per-search reading above says it would not
be.

**How sensitive the count is**, for calibration: flattening the heights
takes frame 10 from 208 to **103**, so the climb term is what shapes this
search; the same search at frames 11 and 171, on an unchanged map, costs
170 and 178 purely because the jitters differ.

**Where a next session would look.** Something that costs about twelve
extra costed nodes **once per search** — three expansions, not a per-step
bias. The shapes are small enough to hold in the head: frame 10 pops 57
nodes for a five-tile road, frame 11's first pops 16 for three tiles, and
the AI's pops 47–49 for twelve. Candidates that fit that shape and have
not been measured: the root's own `z_val` (unclamped and unnegated, so a
search that starts on a road pays the join on its first step — worth
testing against a root whose `z` is clamped like every other node's); the
`Recycler<PathNode>` pool's state across the two searches of frame 11
(nothing in the reading says a recycled node keeps anything, but nothing
has checked it either); and `Build::process`'s call itself — whether the
original reaches `place_roads` by a path that runs the search twice for
one of the three frames, which `build_masks` cannot see.

Until it is exact, `Sim::plan_roads` is **false**. A count that is close is
worse than no count at all: the draws land in the middle of the frame, so
every later draw in it reads the wrong word, and run14's ledger falls from
198 of 284 to 162. With the flag off — the rings still laid, the schedule
still kept — the ledger is unchanged.
