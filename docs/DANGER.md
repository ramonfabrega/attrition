# The danger map

*Established 2026-09-02 from the decompile (`tools/ghidra/`), the PE listing
(`llvm-objdump`) and its data (the two spread tables read straight out of the
image), the PDB's own records for `WorldData`, `LeaderData`, `BuildData` and
`SubObjectData`, and one oracle that turns out to be complete: **the `WORLD`
dump prints the whole map**, `danger[who][scan]`, eight rows of `reg_size`
`int`s, and every `DUMP_ALL` capture in the corpus has been carrying it
since the first one. Confidence: **high** for the object, the schedule, the
building pass and the three arms of `do_danger` — all 7,200 values of
run64's frame-6167 map are asserted against the dump's own. **The unit pass
was medium-low until 2026-09-06 and is now high too**: run64 does not
exercise it, but run26's frame 12024 does, and its arithmetic closes to the
unit (§8.1). What is left resting on a reading alone is the peace arm, the
garrisoned case and the `LEADER_VALID`/`LEADER_ACTIVE` split.*

The mechanic is `docs/QUEUE.md`'s item 136, and it was booked as "the danger
grid has no writer, and four readers index it wrongly". Both halves were
true. What the entry did not say is what the missing writer *costs*, and
that is the whole reason this document exists: the map is read by
`PathFinder::calc_cost` on the world grid, it is **negative** around a
leader's own buildings, and leaving it at zero makes every step near your
own city 8 to 16 too dear. That is what sent East Indies' caravan
south-west on frame 6167 (`docs/CARAVAN.md` §4.1) and it is why the map,
not the caravan, was item 177's first half.

## 1. The object — `WorldData::danger[8]`, `world+0x13c`

Eight `int *`, one per leader slot; Gaia is not one of them. Each points at
`reg_size` ints, allocated and zeroed by `World::init@006b76f0:131`–`142`
for all eight slots whatever the lobby holds.

`World::init` also fixes the grid:

```
tile_xs = xs · 4            reg_xs = tile_xs >> 3        reg_size = reg_xs · reg_ys
```

so a **half-resolution cell grid**: one entry per two cells each way, 30 × 30
on East Indies' 60 × 60 map. Note it is not `xs / 2`: an odd cell width
rounds *down* here, because the halving is of the tile count.

Every consumer in the executable indexes it the same way, from a **position**:

```
danger[who][reg_xs · div3(y >> 9) + div3(x >> 9)]
```

which is `(pos / 1536)` on each axis — the cell, halved.
`crate::world::World::danger_half` takes a cell and `danger_at` a position.

## 2. The schedule

`GameDaemon::process_all@00732700:64`:

```
if (game->frame % 200 == 0) calc_danger()
```

Third in that function, after the repath decay and `process_victory` and
before `update_all_seen`. **Nothing decays the map between rebuilds** — it
is thrown away and reassembled from the live objects — so every read is of
a map up to 199 frames stale. A building finished on frame 201 weighs
nothing until 400. That staleness is diff-backed and visibly so: run25's
frame 12129 prints a map whose two unit contributions sit at half-cells the
units had already **left**, and they are the same two values run26 prints at
12024 (§8.1).

**Where it sits in the frame, which is what makes a capture of it exact.**
`Game::do_frame@00591ef0` runs

```
GameLog::begin_frame@00932a70          # a DUMP_ALL block's first FULL DUMP
  ... Leaders::process_all, Leaders::strategy_all@006ed430 ...
GameDaemon::process_all@00732700       # calc_danger, when frame % 200 == 0
  ... Armies::process_all ...
Objects::process_all@0065dce0          # every object moves, here and not before
  ...
this->frame += 1
GameLog::end_frame@009329d0            # the block's second FULL DUMP
```

so a `DUMP_ALL` block labelled `FRAME N` holds the map **before** and
**after** everything sim-frame N did, and on a rebuild frame the objects in
its *first* dump are exactly the ones `calc_danger` read — nothing has moved
between the two. `tools/gamelog/danger.py` is the reader for both halves.

## 3. `GameDaemon::calc_danger@00732d10` — the rebuild

Three passes.

**The clear.** Every leader with `leader_flags & 2` (`LEADER_ACTIVE`) gets
`memset(danger[who], 0, reg_size · 4)`. A defeated leader's row is left
holding whatever it last held; nothing reads it.

**The units.** For every leader with `leader_flags & 1` (`LEADER_VALID`),
every object in its list from 0 to the unit count, that

- `is_unit()` (vslot `+0x8`) and `is_on_map()` (vslot `+0xbc`) — so a
  garrisoned or a dead one weighs nothing; and
- whose type has `role & 0x10000`, `UnitTypeData+0x2c8`'s **military** bit,

contributes `(attack() · 5) / 10` — half its `UnitData::attack@006103c0`,
written the long way — to every **active** viewer that is

- not the owner, **and**
- at war either way: `diplos[owner][viewer] == 0 || diplos[viewer][owner] == 0`.

So the unit pass can only ever *add*: it never reaches `do_danger`'s own-side
arm and never reaches its ally arm.

**The buildings.** For every leader with `LEADER_VALID`, every object from
`obj_base[1]` (2000) to the building count, that

- `is_build()` (vslot `+0xc`) and `WallData::is_active` (vslot `+0x4c`,
  `flags & 4` — **finished**), and
- either belongs to a city (`BuildData::city >= 0`) or is a type that needs
  none (`build_flags & 0x10`, `NO_CITY`),

contributes its own weight (§4) to **every active viewer, the owner
included** — there is no diplomatic gate here at all, which is what makes
the map negative. It is written nine times: `value / 2` at each of the eight
neighbouring half-cells that is on the grid, and `value` at its own.

The eight offsets are the executable's own `move_x + 4` (`0xadcaf4`) and
`move_y + 4` (`0xadc404`), read out of the PE: `(-1,-1) (0,-1) (1,-1) (1,0)
(1,1) (0,1) (-1,1) (-1,0)`, which is exactly `docs/PATHFINDER.md`'s
direction wheel 1..8. So the footprint is a 3 × 3 block, not a disc, and the
bounds test is on the *neighbour* only — a centre is on the grid by
construction.

## 4. What a building is worth

In the order `calc_danger` tests it:

| test | weight |
| --- | --- |
| `BuildTypeData::is_fort` (vslot `+0xfc`, `is(FORTX, 0)`) or `is(TOWER)` | `hits_left() / 2` |
| a founded city (`flags & 0x20`) | 100 |
| `is(AIRBASE)` or `is(DOCK)` | 100 |
| otherwise | 10, or **50** with `build_flags & 0x40000000` (`MILITARY_TRAINER`) |

`hits_left()` is the inlined `clamp(hits() − damage, 0, hits())`, so a
damaged fort is worth less and a fresh Tower of 750 hit points is worth 375.

## 5. `GameDaemon::do_danger@00732390` — the one write

Five arguments: the object's number, its owner, the viewer, the index, the
value. Three arms, and the sign is the whole mechanic:

```
if viewer == owner:                       danger[viewer][i] -= value
elif diplos[owner][viewer] == 2:          danger[viewer][i] -= value / 2      # an ally
else:
    if o >= 0 and not objects[owner][o].is_seen(viewer, 0):  value /= 2
    if diplos[owner][viewer] == 1:                            value /= 2      # at peace
    danger[viewer][i] += value
```

`is_seen` is vslot `+0x48` — `BuildData::is_seen@0062e1a0` for a building,
`UnitData::is_seen@00607a60` for a unit. The building's chain is: the
`ObjectData::visible` bit for the viewer, then infiltration, then
`WallData::is_seen@00642bd0`, whose deciding clause is the building's own
`ever_seen` byte ANDed with the viewer's `ally_mask` — a per-object
accumulation of line of sight, not a fog read.

**The AI reads the map whole and `calc_cost` eighths it.** Five producers —
`produce_unit@006cb9e0:142`, `:236`, `:380`, `produce_tech@006ca980:138`,
`:236`, `produce_building@006e1400:533`, `found_cities@006c7a60:235` and
`check_orphaned_buildings@006c9f20:196`/`:228` — read
`danger[who][half-cell of the building]`, clamp it at zero and divide a
score by `danger + 1`, or test it against zero. The pathfinder's tile grid
also takes it whole (`006850a0`). Only the **world** grid shifts:

```
local_2c = (danger + (danger >> 31 & 7)) >> 3        # signed / 8, toward zero
```

and that value is the *first* term of `calc_cost`'s `extra`, before the
owner adjustment, before `avoid_sea`/`avoid_land`, before `tcost · 20`, and
inside the `max(0)` that clamps the lot (`docs/PATHFINDER.md` §5). The whole
block is skipped when `pathfinder+0x7c` (`no_danger`) is set, which is what
an attacking unit's search sets.

## 6. Why it moved the word

run64's frame 6167 is one `astar_path` on the world grid, the caravan's own,
and the trace has all forty-seven of the original's `calc_cost` answers for
it. Eleven of them were wrong here, by 8, 10 or 16 — and the deltas group by
*half-cell*, not by cell or by edge, which is what named the mechanic before
a line of it was read:

| half-cell | its cells | `danger[1]` | `/ 8` |
| --- | --- | --- | --- |
| (24, 26) | (49, 52) | −65 | −8 |
| (25, 26) | (50, 52), (50, 53) | −135 | −16 |
| (25, 25) | (50, 51), (51, 51) | −80 | −10 |
| (25, 27) | (50, 54) | −65 | −8 |

The affected cells are exactly the expensive ones — the ground around leader
1's own city, priced ~200 by `tcost · 20` — so the discount decides whether
the search goes toward the city or around it. With the term missing, the
seventh node popped was not `(50, 52)`; with it, `(50, 52)` pops at value
260, its Manhattan distance to the goal is 768 which is the arrival
tolerance, and the search stops there. Its parent is the start, so the
reconstruction (which drops both the arrival node and the start) pushes
**nothing**, `find_wpath` answers a stack of one, and `do_move` takes the
move's own destination as its waypoint and walks straight at the city.

That is the whole of `docs/QUEUE.md` item 177's first half: no waypoint bug,
no `add_move_order` bug. East Indies' word went 6189 → **6198**.

## 7. What `at_war` was, and is

`do_danger`'s enemy arm halves twice — once for unseen, once for peace — and
on run64 the original halves exactly once. Two readings fit that: *seen and
at peace*, or *unseen and at war*. The dump settles it. Leader 0's buildings
carry `ever_seen 1` and leader 1's `ever_seen 2` — neither has ever seen the
other's — and both leaders' `diplos` are `0` toward each other from the
start block onwards. **Unseen, at war.**

The harness had it the other way round, because **nothing installed
`diplos`**: `LeaderDump` carried `leader_flags` and not the diplomacy table,
so `build_sim` left `Sim::at_war` all false and every capture ran with two
leaders at peace who have been at war since frame 0. Forty-six readers of
`is_enemy`/`at_war_with` were answering the wrong way, `calc_cost`'s own
enemy-territory `+4` among them. It is installed now, from
`diplos[scan]` on each `LEADERDATA` block, and the full diff suite is
unchanged by it — which says only that no *pinned* claim depended on it,
not that nothing did.

## 8. Coverage

**Diff-backed** (`rondata::diff::tests`):

- **The whole map, all 7,200 values**
  (`run64_s_danger_map_is_the_original_s`): eight leaders' rows against
  run64's own `danger[who][scan]` at frame 6167, from a simulation built at
  frame 0 and driven forward 6,167 frames — so thirty-one rebuilds, of
  which only the last is compared but every one of which had to leave the
  right objects standing. 58 of leader 0's cells and 58 of leader 1's are
  non-zero, positive on the enemy's side and negative on its own.
- **The forty-seven prices of the search it broke**
  (`run64_s_frame_6167_prices_are_the_originals`), by argument list, against
  the original's proxied `calc_cost`.
- **The caravan's whole walk**
  (`run64_s_window_clocks_are_the_original_s`): 2,061 fields of run64's
  eight-frame window, of which thirty-four used to part and none does.
- East Indies' word, indirectly: 6198.
- **Great Lakes' word, directly: 8030 → 8031** (2026-09-17, item 319).
  The map has a **third** consumer, and it had never been wired to one:
  `Unit::think_scout`'s score adds
  `danger[who][(y >> 1) * reg_xs + (x >> 1)]` to every candidate cell —
  `005f66d2` in the city loop and `005f6b45` in the region scan, the same
  five instructions twice — and `Sim::scout_danger` answered a flat zero
  behind a comment saying this grid was keyed differently. It is not:
  [`World::danger_half`] is that expression. Routed, the AI's scout picks
  the cell the original picks (`docs/SCOUT.md` §8.2), and
  `run94_s_window_is_great_lakes_scout_repath` is what checks it.
- **The unit pass — the formula, the military gate both ways, the
  owner skip and the single-cell write.** §8.1; the archives are run26,
  run29, run27 and run25, and no capture was needed.

**Reading-only, and each names the capture that would falsify it:**
- `UnitData::is_seen`, which is a live-visibility read with the stealth and
  detection machinery behind it. This crate answers it with the same
  ever-seen fog bit a building takes, so a unit standing on ground the
  viewer has explored and left counts double here.
- `BuildData::is_seen`'s first two arms — the `visible` bitmask and
  infiltration — and `WallData::is_seen`'s own five: `reveal_map == 3`,
  `leader_flags & 0x800`, ~~owning a `TRANSPORTGALLEON`~~ owning a
  Fouché (`num_units[0x141]` is `TypeIndex` 371: the array starts at
  `BASE_UNITTYPES`; `docs/AI.md` §141), and the
  city-between-two-humans arm. This crate reads the fog instead, which is
  the same stand-in `docs/GOODY.md` §7.2 makes for `ItemData::is_seen`.
- The `LEADER_VALID` / `LEADER_ACTIVE` split. Every capture has two leaders
  and both bits on both, so "owners are the valid ones and viewers the
  active ones" rests on the reading; a capture with a defeated leader would
  separate them.
- `obj_base[1]`, taken as 2000 because every dumped building's `o` is in
  `[2000, 3000)`.

### 8.1 The unit pass, and it was on disk all along

Item 178 was booked for a capture — a `DUMP_ALL` `WORLD` block on a frame
divisible by 200 with an army alive — and closed by a grep instead
(`docs/RUNS.md`, "178 needed no screen"). The whole corpus holds **two**
blocks on a rebuild frame and neither has a military unit in it; what settles
the pass is a different property of the dump entirely.

**The building pass writes to every active viewer, the owner included, over a
3 × 3 of half-cells (§3). So a half-cell with no building anywhere in its 3 × 3
is zero in every row from the building pass, and anything non-zero there is
the unit pass and nothing else.** Every combat window on disk has exactly two
such half-cells, and `tools/gamelog/danger.py` plus a fifteen-line probe finds
them:

| archive | block | rebuild | half-cell | `danger[0]` | `danger[1]` | the units standing there |
| --- | --- | --- | --- | --- | --- | --- |
| run26 | 12024 | 12000 | (27, 20) | **30** | 0 | `1/35` type 324 |
| run26 | 12024 | 12000 | (28, 21) | **212** | 0 | `1/32` **340**, `1/34` 324, `1/38` 334 |
| run29, run27 | 15100 | 15000 | (22, 28) | **30** | 0 | `1/35` type 324 |
| run29, run27 | 15100 | 15000 | (23, 29) | **217** | 0 | `1/32` **341**, `1/34` 324, `1/38` 334 |

The type table in the same dump gives each type's `attack`, and
`(attack · 5) / 10` halved once by `do_danger`'s enemy arm reproduces every
value **to the unit**:

| type | `attack` | `(attack · 5) / 10` | halved |
| --- | --- | --- | --- |
| 324 | 120 | 60 | **30** |
| 334 | 530 | 265 | **132** |
| 340 | 200 | 100 | **50** |
| 341 | 220 | 110 | **55** |

30 + 132 + 50 = **212**, and 30 + 132 + 55 = **217**. The five between them is
`1/32`'s own: its type is **340** at 12024 and **341** at 15100, an upgrade,
and (110 − 100) / 2 = 5 is exactly what the map moves by. Two frames three
thousand apart, four types, and the arithmetic closes on both.

Six claims come out of that, and each was a reading:

- **`(attack · 5) / 10`**, with `attack()` returning the *type's* `attack` for
  all six of these units. What an upgrade does to it is still unexercised —
  340 → 341 is a type change, not a modifier on one type.
- **One halving, and it is `is_seen`, not peace.** Both leaders' `diplos` rows
  are `0` toward each other in the same block, so the `diplos == 1` arm never
  ran; the halving is `UnitData::is_seen@00607a60` answering false. §7 settled
  the same ambiguity for buildings on a different game.
- **The owner's row is untouched.** `danger[1]` is exactly 0 at both cells the
  units of leader 1 stand in — `viewer != owner`, and the pass never reaches
  `do_danger`'s own-side arm.
- **One half-cell, not a footprint.** All eight neighbours of each cell are
  zero in every row. A building there would have left `value / 2` all round.
- **`role & 0x10000` gates it, and the negative side holds too.** run26's
  frame 12024 has **45** half-cells that lie outside every building's 3 × 3
  and hold a non-military unit and no military one; all 45 are zero in all
  eight rows.
- **The map really is 200 frames stale.** run25's 12129 prints the same two
  values at the same two half-cells with the units already gone (§2).

**What a diff should assert**: not all 7,200 values — the block's records are
24 frames younger than the map and only the *military* units are provably
unmoved — but `calc_danger` over the block's own first dump reproducing the
dump's map at every half-cell holding a military unit and at every half-cell
outside every building's 3 × 3. run64's frame 6167 already carries the
whole-map form for the building pass.

**Not established at all:** what the map is *for* beyond `calc_cost` and the
five producers — `Army`'s own reads, if any, are unlooked-at; and whether
anything outside `calc_danger` ever writes it (`World::clear_danger@006b22e0`
exists and no traced game enters it).
