# Unit collision

**Status: first reading done, 2026-08-27.** Read from the full decompile
export (`~/ghidra-projects/decomp/`) by the main thread on Opus:
`Unit::detect_unit_collision@00617060` (470 lines),
`Unit::resolve_unit_collision@005f9d30` (510),
`CollCheck::collide_here@00682540`, `CollCheck::fill_slots@006820e0`,
`CollCheck::move_unit@00682ad0`, `Objects::find_collision@0065b1b0`,
`Object::add_to_world@0064d8c0`, `Object::remove_from_world@00647970`,
`Unit::set_new_location@005f8d20`, `Guy::set_new_location@005d86f0`,
`UnitData::is_here@0060a0c0`, `UnitData::will_be_corner@00609fa0`,
`UnitData::is_corner@0060a040`, `GuyData::is_corner@005de270`,
`WorldData::get_down@004613d0`, and `Unit::move_step@005faf30`'s collision
block. The `move_x`/`move_y` spiral was dumped from the PE (§2.1).

Confidence: **high** on the two indices, the probe, the corner rule and the
recovery's shape — two whole runs of the mechanic are confirmed field for
field against run10's own dump, and since item 64 the dumped collision
block agrees on **every** compared unit-frame of the capture (§8). The
exemption ladder (§4.3) is reading-only: no run has entered five of its
six arms.

This document is the mechanic `docs/MOVEMENT.md`'s open questions called
"Collision and pushing. … Unread." That entry is now struck and points here.

---

## 1. Shape of the mechanic

A unit proposing a step asks whether anything is standing where it wants to
be. The answer comes from **two indices the world keeps**, and nothing else
walks the unit list:

- **the occupancy bitmask** — one bit per **48-unit cell**, set for every
  cell a unit's block covers. It answers *is something there*, in constant
  time, and it is what `CollCheck::collide_here` reads.
- **the per-world-cell object chain** — `WData::down`/`down_who` is the head
  of a doubly linked list threaded through `ObjectData::up`/`down`. It
  answers *which* object, over the 3×3 world cells around the proposal.

`Unit::detect_unit_collision` uses the first to find a blocked cell and the
second to name its owner; `Unit::resolve_unit_collision` decides what to do
about it. Between them they own five dumped fields — `collide`,
`collide_frame`, `collide_o`, `collide_who`, `collide_guy` — and one path
flag (`0x2`).

## 2. The occupancy bitmask

`CollBlock` is a `BitMask<768>` (96 bytes) hanging off `WData +0x18`, one
per **world cell** (`0x300` units), lazily allocated by
`World::new_coll_block@0046d250`. A world cell is 16 × 16 unit cells, so
256 of the 768 bits are used; the index is `(ux & 15) * 16 + (uy & 15)`.

A unit marks the cells of the **Chebyshev disc of radius `coll_size`**
around each of its figures' unit cells, where

    coll_size = ObjectType +0x248 = the type's raw BLOCK_RADIUS

— `UnitType::init@0061ab50:750` writes the XML number to `+0x248` and
`BLOCK_RADIUS × UNIT_BLOCK_RADIUS` to `+0x240`, so `coll_size` is this
crate's `Profile::block_radius / 48`. In `unitrules.xml` it is 0 for ten
types, 1 for 220, and 2–7 for the rest.

Marking is gated three ways, all in `Object::add_to_world` and
`CollCheck::move_unit`:

- `coll_size != 0` — a type with `BLOCK_RADIUS 0` is never in the index;
- `ObjectType::domain != 2` — aircraft do not occupy ground. The
  simulation carried this as a stated seam (`is_air` returned false, since
  nothing flew); it is the loaded domain since 2026-08-28, when gaia's
  bird became the first air unit to stand up (`docs/SYNC.md` §3.9);
- **the region gate**: a cell is marked only when the world cell's
  `WData::region` equals `get_tregion` of the marking figure's **own tile**,
  or the figure's tile has no region. So a block does not spill across a
  coastline into another region's cells.

  **`get_tregion` is not `region_of`, and the difference is the whole gate**
  (2026-09-01). `WorldData::get_tregion@006b52e0` answers a coastal cell's
  `region2` when the *tile* is ocean (`flags & 0x100` and `mask & 0x30 ==
  0x20`) and its `region` otherwise, so for a **boat lying on the water
  half of a coastal cell** the figure's `get_tregion` is the sea region and
  the cell's own `region` is the land one: the two differ, and the boat
  marks **nothing at all** in that cell. `crates/sim` asked the plain
  `World::tregion` here until this was found, which made the gate vacuous
  for exactly the case it exists for — a `BLOCK_RADIUS 3` barge filled its
  own world cell and the passenger it put ashore was pushed four hundred
  units inland (`docs/TRANSPORT.md` §6.4, `docs/SYNC.md` §3.24). The crate's
  own name for `get_tregion` is `World::tregion_alt`.

`Object::add_to_world` sets the bits for figures `0 .. guy_mark`;
`Guy::set_new_location` calls `CollCheck::move_unit(from, to, coll_size)`
whenever a figure changes unit cell, and that **clears** the cells around
`from` that are more than `coll_size` from `to` and **sets** those around
`to` that are more than `coll_size` from `from`.

> The bits are **not refcounted**. Two overlapping blocks share a bit, and
> when one unit leaves it clears bits the other still stands on. The
> original has this and we keep it: an index rebuilt from the unit list
> each frame would answer differently the moment two blocks overlap, which
> for `coll_size 1` is most of the time. **The hole it leaves is not
> permanent** — §2.2 is the repaint that closes it, and this crate did not
> have it until 2026-09-03.

### 2.1 `move_x` / `move_y` / `radius`

The disc enumeration is the global spiral at `move_x@00adcaf0` /
`move_y@00adc400`, with `radius[r] = (2r + 1)²` giving the prefix that
covers Chebyshev radius `r` (verified against the PE for `r ≤ 7`, which is
every value the shipped rules use). The first nine entries are the compass
`docs/PATHFINDER.md` §4.2 names.

The order inside a ring matters, because `collide_here` reports the **first**
cell it finds occupied. Ring `r` is walked clockwise from `(−r, −r)`: the
top row left to right, the right column top to bottom, the bottom row right
to left, the left column bottom to top. **Ring 2 is the exception** — the
same ring with its four corners moved to the end, in the order NW, NE, SE,
SW. Rings 1 and 3–7 are the plain clockwise walk. (Verified entry for entry
against the PE bytes; the quirk is in the table, not in a generator.)

### 2.2 The sixty-fourth frame, which puts the holes back (2026-09-03)

The bits are not refcounted, and the paragraph above says what that costs:
a unit leaving a cell clears the cells another unit is still standing on.
Nothing in `move_unit` or `add_to_world` ever restores them, so a standing
unit's block would rot away one corner at a time as its neighbours walked
past — and the pathfinder, which reads this index through `valid_ucoord`,
would walk through the holes.

**`Guy::process@005e0230` is what puts them back.** After `Guy::move`, on
the frames where

    (game->frame + o) % 64 == 0

— `o` is the guy's own object number (`GuyData +0x8c`), so the map's units
repaint on sixty-four different frames rather than all at once — a guy
re-marks its **whole disc**: `move_x[0 .. radius[coll_size])` around its
unit cell, set-only, allocating the `CollBlock` if the cell has none, with
§2's region gate (`get_tregion` of the guy's own tile against each cell's
`WData::region`). It is `add_to_world`'s walk exactly, minus the chain half.

Four gates, and they are the original's in its order:

- `type::domain != 2` — aircraft still occupy nothing;
- `guy_num < UnitType::squad_size` (`+0x304`) — the **squad** figures, so
  the crew a track offset carries never marks. (`add_to_world` uses
  `UnitData::guy_mark` at `+0xb5` for the same loop; the two are not the
  same field.)
- `coll_size != 0`;
- **`GuyData::avg_speed == 0`** (`+0x84`) — the guy is standing still.
  `Guy::move`'s tail folds `last_speed` into it a quarter at a time
  (`avg = (avg * 3 + last) / 4`), so it reaches zero a few frames after a
  unit stops and is non-zero for every frame of a walk. ~~A moving guy needs
  no repaint: `move_unit`'s set pass has just written its whole new disc.~~
  **That is wrong, and §2.3 is the correction**: `move_unit`'s set pass
  writes only the cells the *old* disc did not already cover, so a hole a
  neighbour punched inside the overlap survives the move. A marching unit
  is healed by the world-cell crossing instead, and by nothing else.

So a hole lives for at most sixty-four frames, and the index is
self-healing rather than exact.

**How it was found, and what it cost while it was missing.** run69's AI
woodcutter `1/9` walked west through unit cell `(849, 366)` on frame 1885
and left it for `(848, 365)` on 1889; the diagonal step's clear pass took
`(848, 367)` with it, which is a corner of the *stationary* gatherer
`1/10`'s block. Eighty-one frames later `1/9` planned its walk to a tree,
and the cell that should have been refused was free: two middle waypoints
one 48-grid step from the original's, an arrival a frame early, and Great
Lakes' word at **2419** (`docs/PATHFINDER.md` §17). In the original,
`1/10`'s own repaint on frame **1910** — `(1910 + 10) % 64 == 0` — had put
the corner back. run70's `callwin` is what proved the search itself agreed:
of every cell the two searches probed on frame 1970, `(849, 366)` is the
**only** one they answered differently.

`crates/sim/src/collide.rs`'s `Sim::coll_repaint`, called from
`process_movement` as `Guy::process` calls it. SEAM: this crate marks one
figure a unit (§2), so it repaints guy 0's disc and no other's, and the
`squad_size` gate is thereby always satisfied.

### 2.3 The world-cell crossing, which heals a unit on the march (2026-09-07)

§2.2's healer asks `avg_speed == 0`, so it never reaches a unit that is
walking. **There is a second one, and it is the only healer a marching unit
ever meets.**

`Unit::set_new_location@005f8d20` brackets the coordinate write with the
object chain's own pair:

```
local_10 = on_map && world_cell(x_internal, y_internal) != world_cell(new)   // v / 768
...
if (local_10)  Object::remove_from_world(this)
LAB_005f9033:  x_internal = new x;  y_internal = new y;  z = find_tcoord_z(tile)
if (local_10)  Object::add_to_world(this)
```

and both halves of that pair walk the occupancy bitmask, not just the chain:

- `Object::remove_from_world@00647970` **clears** `move_x[0 .. radius[coll_size])`
  around each figure's own unit cell, `0 .. guy_mark` (`UnitData +0xb5`),
  under §2's region gate, and only where the `CollBlock` already exists;
- `Object::add_to_world@0064d8c0` **sets** the same walk under the same
  gate, allocating with `World::new_coll_block` where there is none.

Two things make that pair a heal rather than a no-op.

**The two walks cover the same cells, so clear-then-set is set.** Both read
`GuyData::x`/`y` (`+0xc`, `+0x10`) — the *figure's* point, not the unit's
`x_internal` — and the figures have not moved yet: `move_step` calls
`Unit::set_new_location(x, y, 0, 0)` with `param_3` **zero**, so the guys
only take the new point in `Guy::process` afterwards, and `Guy::move`'s own
`set_new_location` is what calls `move_unit` at all (it is the function's
only caller). So both walks are around the unit's **old** cell, and what
comes out of the pair is the whole old disc, set — every hole a neighbour's
`move_unit` clear had punched in it, filled. `move_unit` then moves that
healed disc onto the new cell.

**And it fires on the march.** The trigger is a *world*-cell change — 768
units, sixteen unit cells — so a unit walking a straight line repaints
itself every sixteen cells, about every twenty-nine frames at `MOVES` 26,
wherever it stands in §2.2's sixty-four-frame phase. The two healers do not
overlap: §2.2's is for a unit that has stopped, this one for a unit that
has not.

Note that the figure loop is bounded by `UnitData::guy_mark` (`+0xb5`) in
both halves, where §2.2's repaint uses the type's `squad_size` (`+0x304`).
The two are not the same field, and this crate marks one figure a unit, so
neither bound is reached here.

**The diff that says so** (item 267, run87). Great Lakes' `1/35`, a
Longbowman marching in the AI's second squad, steps 25 units of y on
sim-frame 7252 where the original steps **12** — a half step, and the
original's own `unit_masks` says why: it carries `0x100000`, the one-shot a
**soft** collision leaves behind (§4.3, `docs/MOVEMENT.md`). The soft
collision is with `1/34`, its file-mate one unit cell east, and the cell
that names it is `(953, 543)` — `1/34`'s own centre. This crate did not
have it: on sim-frame 7249 `1/36`, leaving cell `(954, 542)` for
`(954, 541)`, cleared the row `y = 543` from `x = 953` to `955`, and
`(953, 543)` is inside `1/34`'s block. In the original `1/34` crossed from
world cell `(59, 34)` to `(59, 33)` on sim-frame 7251 and repainted its
whole disc on the way — two frames after the hole was punched, and in the
same frame as `1/35`'s probe but one unit ahead of it in the step order.
(Frames are the dump's blocks where a row is quoted and sim-frames where a
step is: block `n` is the end state of sim-frame `n − 1`.)

The row is a diff rather than a reading: the harness compares
`unit_masks & 0x100000` on every unit-frame whose positions agree (the
widening this item landed, §8.4), and the whole 277-block window carries
**10,458** of them. Exactly one disagreed — `f7252 1/35 half_step 0 v 1` —
and with the repaint in, none do, and `1/35` walks the original's own point
for the entire run-up.

## 3. The object chain

`WData::down`/`down_who` (`+0x8`/`+0xa`) name the head object of a world
cell; `ObjectData::down`/`down_who` (`+0x2c`/`+0x2e`) the next, and
`up`/`up_who` (`+0x2a`/`+0x3f`) the previous. `Object::add_to_world` pushes
the object onto the **head** of its cell's list; `Object::remove_from_world`
unlinks it. `Unit::set_new_location` calls the pair only when the unit
changes **world cell** — so the chain's order is the order units last
entered the cell, newest first.

Both functions report `"UNIT LINKED LIST LOOPS <ADD>"` / `<REMOVE>` if the
list ever points at itself, which is how the original says the invariant is
load-bearing.

## 4. `Unit::detect_unit_collision`

`detect_unit_collision(x, y, quick, boats, _, nocoll, top_only)`. The three
call sites are `move_step` (`quick 0` for the proposed step, `quick 1` for
the path top), `resolve_unit_collision` (`quick 1`, four times) and
`do_move`.

### 4.1 The gates, in order

1. `domain == 2` (air) → no collision, ever.
2. `top_only != 0` → the test runs only when the path top's `flags & 8`
   (`DETOUR`) is **clear**; a detour waypoint suppresses collision outright.
3. Otherwise, with `top_only` and `nocoll` both zero, a unit that is
   sea-domain, a hero, a supply unit, or whose type answers `+0x10c`
   (`ObjectTypeData::is_siege`) takes a second arm: `quick`, or no
   `boats`, returns 0; with `boats`, a non-zero
   `detect_boat_collision` returns 0 and a zero one falls through to the
   scan. **Every one of those returns skips the bookkeeping below**:
   they jump to `61782b`, a bare `xor eax, eax; ret` (§13).
4. `path.length != 0` and the path top has `flags & 8` → return without
   testing (the same `DETOUR` suppression from the other direction).
5. `UnitData::safe != 0` → no test. `safe` is the counter
   `find_upath` adds 30 to when a unit-grid search runs out of budget
   (`docs/PATHFINDER.md` §4.3): a unit that has just failed to path around
   its neighbours stops colliding with them for thirty frames.
6. The proposed **unit cell** equals the unit's current one → no test. A
   step inside your own cell can never newly collide.

Falling out of any of these clears `collide_o = collide_who = −1`, clears
`unit_masks & 0x40`, zeroes `collide` if `collide_frame < frame − 5`, and
returns 0.

### 4.2 The probe

`CollCheck::collide_here(o, who, ucx, ucy, coll_size, &hit_x, &hit_y,
nocoll)` walks `move_x[0 .. radius[coll_size])` around `(ucx, ucy)` and
reports the first cell that is (a) occupied in the bitmask, (b) **outside
the caller's own block** when the caller is on the map, and (c) passes a
parity filter:

    move_x[i] + coll_size and move_y[i] + coll_size are both even

For `coll_size = 1` that is exactly the four diagonals `(±1, ±1)` — which
is sufficient, because two discs of radius 1 overlap iff their centres are
within Chebyshev 2, and that is iff one covers a diagonal of the other.

`collide_here` returns 0 immediately for `coll_size == 0`.

#### The fast path is not an optimisation (2026-09-02, item 183)

Ahead of the disc, and **only when `nocoll` is 0**, `collide_here` has a
second sweep. Let `(dx, dy)` be the proposed cell minus the caller's own:
when one of them is zero and the other is exactly ±1, the probe tests the
**leading edge** — the row or column the block is entering — and nothing
else.

    |dx| == 1, dy == 0:  x = ucx + dx·coll_size,  y = ucy − coll_size + 2k
    dx == 0, |dy| == 1:  y = ucy + dy·coll_size,  x = ucx − coll_size + 2k

for `k = 0 ..= coll_size`, in that order, the first occupied cell winning.
The loop is written as `2·coll_size + 1` iterations testing on the even
ones and advancing the swept axis by one on every other, which is the
parity filter by another route; it asks no own-block exemption and needs
none, because the edge is always one cell beyond the caller's own block.

**It is a strict subset of the disc, so it stops at a different cell** —
and §4.3's corner rule is decided *on the cell*, so the two probes can
disagree about whether there is a collision at all, not merely about which
unit is named. run66's sim-frame 6570 is the case. East Indies' AI
Merchant, `BLOCK_RADIUS 2`, walks north-west past two standing citizens;
its own cell is `(721, 786)` and its proposal `(721, 785)`, so the sweep is
the row `y = 783` from `x = 719`. The first cell is `1/11`'s north-east
corner — and the merchant's own north-west corner meets it, `will_be_corner
1` against `is_corner 5`, a difference of exactly 4, so the two slip past
and the step is taken. The whole disc's first parity cell is `(721, 783)`,
two to the right, inside `1/2`'s block and no corner of the merchant's at
all (`will_be_corner 0`): a hard collision the original does not have. One
frame later the proposal is one cell **west**, the sweep is the column
`x = 718`, `1/11` sits square on it rather than cornered, and the original
collides — `collide_o 11`, `collide_frame 6571`, which is what named the
cell.

`nocoll` is 0 at every `detect_unit_collision` call site and at
`Objects::find_collision`'s, and **1** at `PathFinder::valid_ucoord`'s
(`00687c80:24`) and at `resolve_unit_collision`'s own direct probe
(`005f9d30:453`) — so a path search and the stack unwind take the disc and
a step takes the edge.

#### An empty slot does not advance the sweep (item 539, 2026-09-22)

The paragraph above has the fast path's stride right only while every
cell it tests sits in a live world cell. `collide_here` reads the
bitmask through **four slots**, `CollCheck::fill_slots@006820e0`'s 2×2
world cells from `((ucx − r) >> 4, (ucy − r) >> 4)`, and a slot is
**empty** when it was not needed (the disc does not reach it), lies off
the map, has no `CollBlock`, is refused by the region gate below, or
`BitMask<768>::empty@00479150` says it holds no bit. That last test
reads a cached flag, and the cache is exact: `CollBlock::set@00682070`
writes 0 (known non-empty) on a set and 2 (unknown) on a clear, and
`empty` recomputes on 2.

The loop (`00682540:71`–`116` for the x arm, `:126`–`174` for the y
arm) advances the swept axis on **every odd pass**, and on an **even
pass only after testing a cell and missing**. An even pass whose cell
lies in an empty slot tests nothing and **does not advance**. So the
cells tested are `c₀, c₀ + 2, c₀ + 4, …` only while every slot is live.
After a skipped cell the stride is one:

    coll_size 1, edge from y₀:  both slots live → y₀, y₀ + 2
                                y₀'s slot empty → y₀ + 1 (and nothing else)

A unit walking across a world-cell boundary beside a group-mate is
exactly the case. Great Lakes' sim-frame 11304: `1/31` steps west from
unit cell `(913, 464)` onto `(912, 464)`, so its edge is the column
`x = 911` from `y = 463`. That cell is in world cell `(56, 28)`, which
holds no bit. The original therefore tests `(911, 464)`, which is empty,
and takes the step whole. This crate stepped two cells to `(911, 465)`,
found `1/32`'s block there, and went soft. Sim-frame 11356 is the same
shape with `1/35` and `1/33`, and there both edge cells lie in the empty
world cell `(55, 27)`, so the original tests nothing at all.

**The probe side has a region gate too.** `fill_slots` keeps a slot's
block only when that world cell's `WData::region` (`+4`) equals the
`get_tregion` of the **probe centre's** own tile. The centre's tile is
`(ucx >> 2, ucy >> 2)`, and its region is `region2` on the water half of
a coastal cell. When the centre has no region, every slot is kept. A
refused slot is empty in both senses above. This is the probe's half of
§2's marking gate. Great Lakes is one region, so no measurement here
tests it.

`Sim::collide_here`, `ProbeSlots`, `CollGrid::any_in_cell`. ~~**SEAM**: a
`nocoll` probe reads the pathfinder's `+0x4c` tree of block copies
(`fill_slots:81`–`168`) rather than the live blocks.~~ Modelled since
item 678 (`Sim::coll_copies`, `docs/PATHFINDER.md` §26): a `nocoll` probe
reads a slot's copy when the tree holds one, and copies the gated live
slot when it does not. **SEAM**, what stays: a copy is taken from the
gated slot and outlives the probe that took it, so a later `nocoll` probe
from another region reads it ungated. This crate does the same, and no
capture has two regions to test it.

**Coverage.** *Diff-backed*: `run125_s_word_frame_is_widened_whole`.
Before the change, 193 blocks of army 1's squad march over `[11250,
11599]` carried a soft-flag or unit-cell parting, and the two that
parted with every unit cell agreeing were 11305 and 11357. After it
there are none, no squad position parts on any of the 350 blocks, and
the widening's parted keys fall 772 → 403 (`docs/GROUPS.md` §21).
*Guard*: `collide::tests::the_leading_edge_does_not_step_past_an_empty_world_cell`,
made to fail by restoring the fixed stride. *Listing-backed, and no run
reaches it*: the probe-side region gate, and the y arm's identical skip.

### 4.3 Naming the other unit, and the exemptions

With a hit cell in hand, `detect_unit_collision` returns 1 at once if
`quick`. Otherwise it walks the **3×3 world cells** around the proposal
(`move_x[0..9]`), and each cell's `down` chain, skipping itself,
non-units, and aircraft. For each unit whose `is_here(hit)` covers the hit
cell — `|hit − its cell| ≤ its coll_size` on both axes — it asks whether
this is a *soft* collision, which sets nothing but a flag and keeps
scanning:

| when | soft |
|---|---|
| my action is `TRADE_ROUTE` and its action is `0xf`, and both are moving | yes |
| its action is `0xc` and that order's target is me | yes — **built** (item 696, `Sim::soft_collision`): run190's wagon passes through its walking guard on tick 726, and the original's scan there reaches `is_here` on the guard and never `is_corner` (run191's third take). The first row, the caravan pair, is still not carried |
| its action is `ATTACK`, mine is too, same player, both `coll_size 1`, both moving, and my target is more than `0x300` beyond my range | yes |
| we share a `group` (≠ −1), I am not attacking, it has no suspended search (`+0x104 == 0`), and either it has no order or its order is a spell in `{0x28b, 0x28d, 0x28f, 0x291}` or a passable kind — `0` and `0xc` unconditionally, `1, 2, 3, 4, 0x12, 0x13, 0x15` also needing its action ≠ `ATTACK` | yes |

The passable-kind split is **not** "the last four", which is what this
table said until 2026-09-05 and what `Sim::same_group_soft` was written
from. `detect_unit_collision@00617060:366-369` is

```c
else if (((iVar7 == 0) || (iVar7 == 0xc)) ||
        (((((iVar7 == 1 || ((iVar7 == 2 || (iVar7 == 3)))) || (iVar7 == 4)) ||
          (((iVar7 == 0x12 || (iVar7 == 0x13)) || (iVar7 == 0x15)))) &&
         (local_28 != 10)))) goto LAB_00617870;
```

— `iVar7` is the collider's **order** type (`:328`, vfunc `+0x10` off its
current order) and `local_28` its **action**'s (`:180-186`, `0` when it has
none), so `0` and `0xc` short-circuit before the action test ever runs and
**seven** kinds are gated, not four. A `GUARD` group-mate whose action is
`ATTACK` is soft; a plain `MOVE_TO` one is hard. Nothing in the arm asks
whether the order carries a group — `0x13`/`0x15` sit *inside* the gated
seven, beside the `1`/`2` this crate writes them as — so the crate's old
`m.group.is_none() ||` escape was an invention with no counterpart here.

A soft collision at the end of the scan sets `unit_masks & 0x100000`
(~~`00617817`~~ `006177f3`, §12), the one-shot half step `docs/MOVEMENT.md` names, and returns
0. **It is set once for the whole nine-cell sweep**, ~~whatever the scan
found~~ **and only when the sweep ends without a hard hit** — a hard hit
returns 1 before the set, whatever soft candidates it passed on the way
(§12, item 560) — and it is set only on the full call — the `nocoll` and quick forms
return before the loop. The step it pays for is the *next* frame's:
`move_step` decides the halving before it probes, so the flag a frame sets
is spent by the frame after. **All of which §11.1 now reads off the
original's own dump** rather than off this paragraph: the bit, the "never
two blocks running", and the halved step on the frame after each one.

Anything else is **hard**, unless the corner rule lets the two slip past:

    hard  ⟺  will_be_corner(me, hit, proposed) == 0
              or  |will_be_corner(…) − is_corner(other, hit)| ≠ 4

`will_be_corner` and `is_corner` return `1, 3, 5, 7` for NW, NE, SE, SW when
the hit cell is exactly a diagonal corner of the block, and 0 otherwise. A
difference of 4 is the two opposite diagonals: the units touch at one
corner from opposite sides, and pass.

#### The two halves are not centred on the same thing (2026-09-04)

`UnitData::will_be_corner@00609fa0` measures the hit cell against the
**proposed** cell — `param_3`/`param_4` are the proposal, and the block is
the asking unit's — which is what the rule wants and what the code above
does.

`UnitData::is_corner@0060a040` does **not** measure against the other
unit's `x_internal`/`y_internal`. It walks that unit's figures
`0 .. guy_mark` and returns the **first non-zero**
`GuyData::is_corner@005de270`, which measures the hit cell against that
figure's own `GuyData::x/y` (`div_3_table[x >> 4]`, the same `ucell`). The
`coll_size` is still the unit's type's `+0x248`, so only the centre moves.
It returns 0 outright when the unit fails vfunc `+8` or `+0xbc`
(`is_on_map`) — both already filtered by the chain walk that reached it.

So the original mixes two frames of reference within four lines:
`UnitData::is_here`, immediately before, reads the **unit's** position, and
`is_corner` reads its **figures'**. The two answers come apart for a crew
figure standing on a track offset (`docs/ANIM.md` §4.8's packing types —
merchants, caravans), and for guy 0 itself on any frame its body has not
caught up with the unit's point (`docs/ANIM.md` §4 step 1). Since
`is_corner` returns the *first* non-zero and guy 0 is usually on the unit,
the effect is one-directional: a figure can only turn a hard collision
**soft**, never the other way.

`Sim::guy_corner`, and
`collide::tests::the_corner_rule_reads_the_blocker_s_figures_and_not_the_blocker`.
**Reading only, and no capture reaches it**: neither long word moved when
it landed (Great Lakes 6848, East Indies 7448, both unchanged), so what it
rests on is the two decompiled functions and nothing else. SEAM: a unit
this crate has stood up without figures — `Sim::add_unit` does not call
`init_guys` — falls back to its own cell, because a live unit's `guy_mark`
is never 0 in the original and the fallback stands in for a state the
original does not have.

A hard collision writes `collide_o`, `collide_who`, `collide_guy = 0`,
stores the proposed point in the move order's `coll_x`/`coll_y`
(`MoveOrder +0x3c/+0x40` — the dump prints them), and returns 1.

**The store is into the order, and every caller keeps it.** `move_step`
and `do_move` hold a `MoveOrder *` and go on writing their own fields
through it, so the pair the probe just refused survives whatever the
caller does next. That is not free in a port that steps on a *copy* of the
order and writes it back: the first store after the probe puts the stale
pair back, and the arm where it shows is §5's blocked stand while a turn
is still owed, which stores and returns without stepping. run10's `1/1`
is the case — on frame 792 the original carries `(40539, 18258)`, the
point its own step proposed, and this crate carried the point it had
refused fifteen frames earlier (item 115, and it was Great Lakes' whole
order score).

## 5. `Unit::move_step`'s collision block

```
if detect(proposed, quick 0):
    if path_top.flags & 2 and not detect(path_top.to, quick 1)
       and |dx| < 0x61 and |dy| < 0x61:
        proposed = order.dest_x/y          # the final snap through a sidestep
    else:
        set_anim(CHAR_DEFAULT)
        if still owing a turn: return
        if (my big_radius + its big_radius) * 3 <= manh
           or collide < 0x1a
           or ((path_top.tolerance == 0 or path_top.flags & 2) and not path_top.flags & 1):
            resolve_unit_collision(proposed); return
        tolerance = manh * 2               # give up: call it arrived
```

`big_radius` is `ObjectType +0x244`.

**And the step's own tail, which is where the leg ends** (`005fb45f`,
item 289). After a step is accepted — either arm, the partial one or the
snap — `move_step` runs

```
if tolerance < |dest_x − x| + |dest_y − y|: return 1        # not there yet
dest = 0; pop the path stack                                # arrived
if not popped.flags & 1: return 1                           # a middle leg
… set_angle, clear the pathed bit, kill the order           # the goal
```

Three things about it matter and each has cost a frame: the distance is
**Manhattan**, not the octagonal `vector_dist` `do_move`'s own take uses;
the tolerance is **`UnitData::tolerance`**, the unit's field, never the path
entry's; and it runs **only after an accepted step**, so a blocked frame
never reaches it. §8.7 is the mechanic that turns on all three.

The `set_anim(CHAR_DEFAULT)` is the call at `005fb74e`, so its draw site is
`Unit::move_step+0x823` and the trace names it
`sim::anim::SITE_BLOCKED` (`docs/SYNC.md` §3.10). Note where it sits: it is
taken **before** all three give-up tests, so a unit that is still owed a
turn has already re-rolled its idle by the time `move_step` returns.

### 5.1 `do_move`'s waypoint test — the third call site

The other pre-step probe, and the one that is not `move_step`'s. `do_move`
takes a waypoint off the path stack on the frame the move order's `dest`
goes 0 → 1 — **once per leg** — and ends that block with
`detect_unit_collision(top.to, quick 0)`. On a hit:

- if the waypoint is **final** (`flags & 1`) and the unit's *action* is
  `TRADE_ROUTE`, `GATHER`, `ATTACK` or `BUILD_AT` → `kill_current_order`.
  The walk is abandoned where the unit stands, and the action re-decides
  next frame.
- otherwise, if the collider's **current order is not a move**
  (`UnitOrder +0x14`, `is_move` — the same set §6 step 4 lists), the unit's
  tolerance is widened to `other.big_radius × 3` and the path top is
  re-pushed with it: give up short of a parked unit rather than walk into
  it.

Then the arrival test `vector_dist(dest − pos) ≤ tolerance` runs, so a
widened tolerance can end the leg on the same frame it was widened.

The full form is used, so a hit here writes `collide_o`, `collide_who`,
`collide_guy` and `coll_x`/`coll_y` — and, being `quick 0`, it also does
the clearing on the way out (§4.1) for every unit that takes a waypoint.
`docs/ORDERS.md` §4.4 has the block in full.

### 5.2 The two queries `find_nearby_spot` asks

The other consumer of both indices, and the one nothing here reached until
item 66. Every walk an order makes ends at a point
`UnitType::find_nearby_spot` returns (`docs/ORDERS.md` §10), and the last
test each candidate takes is a collision test. Which one depends on the
filter, and every build, repair, gather, garrison, idle-wander and stable
call site passes `FILTER_NOT_ME` with the unit's own `(o, who)` and no
squad — the **pairwise** pair below. `nocoll != 0` skips both.

**`Objects::find_collision(x, y, o, who, 0)@0065b1b0`** — "is anything
standing here". A **land** caller returns `CollCheck::collide_here(o, who,
ucell(x), ucell(y), coll_size, 0, 0, 0)` and nothing else: the same probe
§4.2 describes, so the caller's own block is exempt and the parity filter
applies, and a type with `coll_size 0` never collides. A sea or air caller
(or the flag set, which no `find_nearby_spot` call site sets) walks the 3×3
world cells around the candidate and each cell's `down` chain instead,
comparing **current** positions in unit cells: a hit is `|ucell(cand) −
ucell(other)| <= my coll_size + its coll_size` on both axes.

**`Objects::find_ordered_collision(x, y, o, who)@0065b440`** — "is anything
*walking* here". `0` when my `coll_size` is 0; otherwise the same 3×3 chain
walk for every domain, but against each other unit's `UnitData::orders_x/
orders_y` — where it has been told to stand — rather than where it is. So a
spot another unit is already walking to is taken, which is what keeps two
citizens sent to the same camp on the same frame from being given the same
quarter-tile.

Both walks skip the caller, and both require the other object's owner to be
a **player** (`who < 8`): gaia's animals are invisible to them, though a
land caller's `collide_here` sees the cells a sheep paints, because the
bitmask has no owner.

The chain arm is keyed on where the other unit *stands* while it tests
where that unit is *going*, so a unit parked far from the candidate but
ordered next to it is missed. That is deliberate in the original, and it is
why `find_ordered_collision` has a second pass: if I am in a group whose
`+0x49` byte is clear, every other active member with `inside_up < 0` and
a block is tested against its ordered position regardless of where it
stands. ~~this crate does not model~~ — **landed 2026-09-17** (item 328),
because `docs/COMBAT.md` §17's probe is the first thing in this project to
put units in a group and then ask six of them, in one frame, where to
stand. `Sim::pushed_group` is the pool slot, `Sim::push_group` writes it,
and the pass is the difference between six ring walks that pile onto one
spot and six that spread: 8186's chase goes from 51 draws to the
original's 46 and every destination comes out to the unit.

#### 5.2.1 The pair is not always the pair — `find_unit_with_radius` (2026-09-02)

**The flag that selects the pairwise pair needs a real object behind it.**
`find_nearby_spot@0061de70` sets its `bVar17` at `0061deb0` only when the
filter is `FILTER_NOT_ME` or `FILTER_CAN_COLLIDE` **and** `not_o >= 0`
**and** `not_who >= 0`. Every unit call site passes its own `(o, who)` —
`Unit::find_nearby_spot@00617010` fills those two slots from `+0xa` and
`+0x9` before it forwards — so the pair above is right for all of them.
The two sites that do not are `Unit::do_cast` and
`SpellType::cast_transport`, which ask the *type* for the water a
transport barge is born on with `not_o = not_who = -1`
(`docs/TRANSPORT.md` §6.1). They take the general path instead:

**`ObjectsData::find_unit_with_radius(x, y, ·, -1, r_coll, ·, filter,
not_o, not_who)@00659890`** — a **distance** test, not a cell overlap. For
each live, on-map object of a **player** (`who < 8`, so gaia is invisible
here too), with `r_coll` the asking type's own `+0x240`:

- `vector_dist(cand − it) <= its big_radius` → found, returned at once;
- otherwise `vector_dist − its big_radius <= r_coll` → found, and the
  nearest such is the one returned.

The two clauses are one predicate: **a spot is taken when some player's
unit is within `its big_radius + r_coll` of it**, by the engine's own
octagonal `vector_dist`. Its ordered sibling
(`find_unit_ordered_with_radius@00658ef0`) is guarded by `not_who >= 0` in
the caller, so the `(-1, -1)` form never asks it at all — a spot another
unit is only *walking* to is free for a barge.

The original picks between a disc of 768-unit blocks around the candidate
and a walk of all eight players' object arrays, on whether the disc holds
fewer cells than the game has units. The two answer the same here: the
predicate's reach is a block plus a block, under 400 units, and never
leaves the disc. `Sim::find_unit_with_radius` takes the second.

**What it cost.** For a Transport Barge (`BLOCK_RADIUS 3`, so `r_coll =
144`) cast by a scout (`BLOCK_RADIUS 1`, `big_radius = 48`) the reach is
**192** and the caster is 228 units from the bearing the original takes —
free. The Chebyshev pair reads the same pair as `3 + 1` cells against
offsets of 3 and 4 and refuses it. That one refusal moved the AI's barge
two tiles along its own route at birth, and it stayed there: East Indies'
long word was **5819** with the pairwise pair and **6164** with the radius
one (and its draw *count* holds to 6165) (`run59_s_census_is_where_the_ai_s_timber_goes`,
`run63_s_window_is_where_the_ai_s_colony_site_appears`).

### 5.3 The block's **return value**, which is the formation's end condition (2026-09-06)

`move_step` answers `1` almost everywhere and `0` from exactly three
places. Two of them are this block:

- `005fb689` — blocked, and **still owing a turn**: `set_anim(CHAR_DEFAULT)`
  and out.
- `005fb6df` — blocked, and handed to `resolve_unit_collision`: the
  sidestep, the wait, or the snap-and-repath, and out.

The third is the tile refusal a few lines below (`005fb7c1`–`005fb7fd`,
§5's `invalid_loc` arm). Every other exit — the four world-bounds tests,
`set_new_location` refusing, the arrival, the kill — answers `1`.

**And the zero is read.** `Unit::do_group_move@005e79a0` takes it on both
sides of a formation: the leader's through `do_move`, whose own tail
returns whatever `move_step` gave it, and a **follower's straight off
`move_step`** at `5e856d`. Both then run the same tail — re-read the head
order, take the attack hand-off if the collider is a valid target of an
attack-context move, and otherwise **`ungroup_move_order`**, which
degrades the group move into N independent moves for the whole squad, up
the captain chain and back down (`docs/ORDERS.md` §8.3).

So a squad does not merely *stall* on a unit in its way: the first member
whose step is refused by a collision dissolves the formation, and every
Archer re-plans for itself on the world grid the next time `do_move`
reaches the grid roll. That is what run76's 6861 is, and the record names
it without a reading: all three Archers' order kind goes
`GROUPATTACKTOORDER` → `ATTACKTOORDER` on that frame, `1/28` loses
`PATHED` and its whole path stack, the leader `1/27` keeps its stack and
loses `dest`, and `1/29` — processed after the ungroup, in the same frame
— builds a nine-entry `find_wpath` plan 768 apart with `tolerance 384`
and steps the full 26 along it. The unit whose step was refused is
`1/28`, squeezed onto its own cell centre `(42792, 24648)` by step 6's
snap.

This crate answered `Did::Something` from both arms and its follower
ignored the value entirely, so its squad marched on in formation. Reading
it moved Great Lakes' long word 6862 → **6982** (item 236).

### 5.4 The **snap** arm's own collision block — the other half of the `if` (2026-09-18)

§5 above is one of **two** collision blocks, not the only one.
`Unit::move_step@005faf30` splits on `param_2 < local_28` — the frame's
step against the Manhattan distance still owed to `dest_x`/`dest_y` — and
each side probes for itself:

| | the arm | the probe's return address | what it does on a hit |
|---|---|---|---|
| `param_2 < local_28` | the **partial step**, the sine/cosine one | `005fb753`, `Unit::move_step+0x823` | §5: snap-through, wait, resolve, widen |
| `local_28 <= param_2` | the **snap**, which lands on the waypoint exactly | `005fb412`, `Unit::move_step+0x4e2` | this section |

Both call `detect_unit_collision(dest, 0, 1, 0, 0, 0)` with the same seven
arguments, so both write `coll_x`/`coll_y` and both name the collider. The
similarity ends there. The snap arm's block is nine instructions
(`005fb3bd`–`005fb412`):

```
collide_o = -1                       # field_0x8a = 0xffff
collide_who = -1                     # field_0xb3 = 0xff
order->dest = 0
if path.length < 1: path.length = 1
path.length -= 1                     # pop, clamped
flags = path.list[path.length].flags
set_anim(CHAR_DEFAULT, 0, 1)         # ← the draw, +0x4e2
→ join the accepted step's tail at 005fb4c4
```

Three things it does **not** do, each of which §5 does:

- **It never reaches `resolve_unit_collision`.** No sidestep, no pause
  roll, no cell-centre snap, no stack unwind.
- **It does not wait on an owed turn** and does not widen `tolerance`.
- **It clears the collider it just named.** `collide_o` and `collide_who`
  go back to −1 in the two instructions after the probe returns, so a
  dumped unit blocked on its snap carries `coll_x`/`coll_y` from the
  refused point and `collide_o −1` beside them. `collide` and
  `collide_frame` are not touched at all.

What it does instead is **consume the waypoint where the unit stands**:
`dest = 0` and the pop are the accepted step's own arrival bookkeeping,
taken without the step. The popped entry's `& 1` then decides as it always
does — a middle leg returns 1 and the walk resumes next frame from the
same place; the final one falls into `set_angle` and
`kill_current_order`. So a blocked snap costs the unit exactly **one
frame** and one draw, and nothing else.

**Great Lakes 9134 is the frame** (item 360). It is one draw on each side,
and the original's was a bare `5dac7a` — the trace's table names
`Guy::set_anim+0x97a` by its `ebp` chain and had no entry for this caller,
so eleven named chains and one unnamed address read as a match on count
for as long as nobody looked. `1/32` walks its formation slot in ~24-unit
hops, arrives on each within one step — `dest_x`/`dest_y` equal to
`x_internal`/`y_internal` at every frame boundary from 9132 — and on 9134
the hop it snaps to is blocked. The original stands, keeps `(42801,
22824)`, takes `coll_x 42825 / coll_y 22827` and `collide_o −1`, and does
everything else a frame later. This crate ran §5's give-up chain instead,
snapped the unit back to `(42792, 22824)` and ungrouped it on 9134 rather
than 9135. §8.9.

## 6. `Unit::resolve_unit_collision`

In order, with the first that fires winning:

0. **An animal gives up.** The function's *first* statement is a virtual on
   slot `+0x30`, and when it answers non-zero the body is the `QUEUE_NEW`
   clear and nothing else — `unit_masks &= ~0x4000000`, `path.length = 0`,
   `close_orders(0)`, `clear_partial_path`, `update_action`, return. The
   same five lines `Unit::add_move_facing_order@005e55c0:58` runs when a
   new order replaces the queue. **None of steps 1–6 below runs**: no
   sidestep, no wait, no repath, and above all no cell-centre snap.

   The slot is **`SubObjectData::is_animal`**, vftable offset 48, and the
   name comes from the PDB's `LF_ONEMETHOD` list because the map cannot
   give it: both overrides are trivial and COMDAT-folded, so the export
   prints `Buffer::is_pending_load` (`return 1`) in `Animal::vftable` and
   `Window::get_button` (`return 0`) in `Unit::vftable`. The two stubs are
   the predicate: an `Animal` answers, a `Unit` does not.

   So a herd animal blocked by its herd-mate stops dead where it stood and
   stays there until its next wander roll — which is run39's `8/2`,
   blocked on frame 69 at `(28856, 24197)` and standing there for the rest
   of the capture (`docs/SYNC.md` §3.14). The record says so on both
   captures at once: `rondata::diff::a_blocked_animal_drops_its_walk_where_it_stands`
   takes every animal walk in the two long traces that ends short of its
   goal — sixteen of them, twelve on East Indies and four on Great Lakes,
   against seventeen that arrive — and every one of the sixteen ends with
   the animal's position **unchanged** across the frame the order dies.

1. **Attack it.** If the type has `+0x2b4 & 0x2000` and the other unit is a
   valid target: `set_attack`, `fire_ammo`, done.
2. **I am standing in my own *flat* gather target.** Only when the other
   unit is the same player's. If the current action is a `GATHER` whose
   target is an active build whose type answers vfunc `+0x94` —
   `BuildTypeData::is_flat`, `build_flags & 0x10000000` (`docs/CITIES.md`
   §1.5, and `mov eax,[ecx+0x2c0]; and eax,0x10000000` in the listing) —
   and that building's footprint `covers_tile` my tile →
   `kill_current_order`.

   `FLAT` is not a `BUILD_FLAGS` letter: the loader derives it for the
   Farm, the Oil Well and the Oil Platform lineages and nothing else
   (`crate::build::init_final_flags`), so this step is **the farmer's**.
   A citizen bumped while standing on the field it works abandons the
   walk where it stands and re-decides; one bumped on the footprint of a
   woodcutter's camp it is merely gathering *at* — a footprint it may
   well be standing on, because a camp is placed among its trees — falls
   through to step 6 and repaths. `+0x94` is the same virtual
   `Unit::add_gather_order` asks about the target when it sets
   `goto_build` and `dist_mod` (`docs/ORDERS.md` §6.4), which is where
   this crate had already read it correctly.
3. **The enemy ladder** (other player's unit, my order is a target order on
   *it*, or it is in range, or it is attacking something I can reach):
   `kill_current_order`, or `repath` + `add_attack_order(QUEUE_FIRST)`, or
   `find_new_target`. ~~Never reached by any capture so far.~~ **Reached
   by golden chapter one's `0/8` on tick 624** (item 445). The ladder's
   three arms are `docs/COMBAT.md` §48.3; the third, arm C, is §14
   (item 668, run171's `1/6`).
   Step 2 is reached for a foreign collider too, whenever the action is
   not an attack.
4. **The sidestep.** Only when *the other unit's* current order is one of
   `MOVE_TO, ATTACK_TO, EXPLORE_TO, FLEE_TO, CHANGE_FORM, GROUP_MOVE,
   GROUP_ATTACK_TO`, my domain is 0, and the path top's `flags & 2` is
   clear. With `c = ucell(coll_x, coll_y)`, `m = ucell(me)` and
   `d = c − m`, the two candidates are

   - `|dx| == |dy|` (a diagonal step): `(c.x, c.y − dy)`, then
     `(c.x − dx, c.y)` — the two cells that split the diagonal;
   - otherwise (a cardinal step): `(c.x − dy, c.y − dx)`, then
     `(c.x + dy, c.y + dx)` — the two cells either side of it.

   The first that is neither `invalid_loc` nor colliding is pushed as
   `{cell centre, tol 0, flags 2}` and written into the order's
   `+0x2c/+0x30`. Done.

   **Those are the only three stores `LAB_005fa37a` makes**, and the two it
   does *not* make are the mechanic (§8.7): it never touches
   `UnitData::tolerance`, so the entry's own `tolerance 0` is never the one
   the arrival test reads; and it never touches `MoveOrder::dest`, which is
   already 1 whenever `move_step` runs, so the waypoint is never *taken*
   through `do_move`'s `dest == 0` block either. A sidestep is walked under
   whatever tolerance the interrupted leg had.
   *(`d == (0, 0)` raises "Collided in my space?" — an error box, not a
   branch.)*
5. Otherwise **`collide += 1`, `collide_frame = frame`**, and:
   - if the other unit's order is one of the move kinds above (or a
     `CAST_SPELL` of `0x28a`) and neither side has given up: set
     `unit_masks & 0x40` — *wait for it to move* — and done, provided
     `other.collide` and `my collide` are both under `0x20` (`0x80` once the
     player has repathed four times), we are not enemies, it is not already
     waiting on me, and **it is not waiting on something that is waiting**.

     That last clause is one-sided and it has an escape.
     `resolve_unit_collision@005f9d30:396-400` is

     ```c
     (((*(byte *)(iVar5 + 0x68) & 0x40) == 0 ||
      ((-1 < sVar13 &&
       ((*(byte *)(*(int *)(*(int *)(&units.field_0x10 + *(char *)(iVar5 + 0xb3) * 0x1c) +
                            sVar13 * 4) + 0x68) & 0x40) == 0))))))))
     ```

     with `iVar5` the collider, `+0x68 & 0x40` the wait flag and
     `sVar13 = *(short *)(iVar5 + 0x8a)` its own `collide_o`: pass when the
     collider is not waiting, **or** when its `collide_o` is non-negative
     *and* the unit it names is not waiting. So a collider that is waiting
     while naming nobody (`collide_o < 0`) **refuses** the wait and drops
     the unit through to the repath. Nothing in the guard reads *my* own
     flag — an earlier draft's "neither of us has the flag already" had no
     counterpart in either the decompile or the code, and is struck
     (2026-09-05, the docs-versus-code pass's R8).
   - otherwise fall through to the repath.
6. **The repath.** With a non-empty path stack, and under the throttle
   (`repaths[who] < 0x10`; over 4 only every fourth collision counts, over 8
   only every sixteenth object number). **The throttle is a rate, not a
   lifetime count**: `GameDaemon::process_all` halves every player's
   `repaths` at the top of each frame and snaps it to zero under three
   (`docs/PATHFINDER.md` §8), so all three arms above are reached only
   while a player is colliding repeatedly *now*. Then:
   - `repaths[who] += 1`;
   - **pop the path stack** until an entry is worth keeping: stop on
     `flags & 1` (final), or on an entry with `tolerance ≥ 0x60` and no
     `flags & 2` whose tile is not `0x6000`-masked and whose cell
     `collide_here` says is clear. Push the last popped back.
   - **snap the unit onto its own 48-cell centre** — `set_new_location(ucell
     centre)`. This is the visible signature of the whole mechanic: a unit
     that collides jumps to the middle of its cell. **And it takes its crew
     with it**: the call is `set_new_location(·, ·, 1, 0)`, whose `param_3`
     reaches `Guy::set_new_location(guy 0, pos, 1)` and, in that function's
     crew loop, `set_angle(crew, des_angle, 1)` and `set_new_location(crew,
     des, 1)` — every tracked figure is *put* on its rotated offset with
     the leader's own angle rather than left to walk after it
     (`docs/MOVEMENT.md`, "Who writes it, and when"; run67's block 6572).
   - `find_upath(anti = my action is ATTACK and its action is ATTACK)`
     (`docs/PATHFINDER.md` §3).
   - clear the order's `+0x10`;
   - if the search succeeded **and** the other unit is colliding with *me*
     and is not already waiting, roll
     `pause = Random::get(0, 0xffff) % 9 + 1` into my order — **a draw on
     the shared stream** (`docs/SYNC.md`), and the only one the whole
     mechanic spends. It is the stagger for a head-on pair: `do_move`
     will not step while `pause` is non-zero, so the two do not both set
     off on the frame their searches land and collide again. The guard is
     read off the unit `resolve` was *handed*, not off `collide_o` again:
     `other.collide_o == my o`, `other.collide_who == my who`, and
     `other.unit_masks & 0x40` clear. Site
     `Unit::resolve_unit_collision+0xb52`, `sim::collide::SITE_PAUSE`.

`Unit::do_idle` zeroes `collide`.

## 7. What this crate models

`crates/sim/src/collide.rs`, wired into `Sim::unit_step`
(`crates/sim/src/orders.rs`), `Sim::set_new_location` and
`Sim::find_nearby_spot`.

Modelled: the bitmask with its clear-on-move semantics and the region gate;
the object chain over units; the probe with its parity filter and disc
order, **and its leading-edge fast path with the `nocoll` argument that
selects it** (§4.2, item 183); the `safe`, `DETOUR`, same-cell and
`coll_size 0` gates; the corner rule; the same-player-attack exemption; `move_step`'s **two** blocks, **including
both `set_anim(CHAR_DEFAULT)` calls** (§5 and §5.4, item 360) **and the
probe's write back into the order** (§4.3, item 115); **`do_move`'s waypoint test with both of
its arms** (§5.1, item 63); and `resolve`'s steps **0** — the animal's
whole-queue clear, `Sim::clear_orders` behind `Unit::is_gaia` — 2, with
its `is_flat` fence (§6, item 64), 4, 5 and 6, the last including the
throttle **with its per-frame decay**, the stack unwind, the centre snap,
`find_upath`, **the stagger draw of its tail** (item 80) and **the store
beside it** (item 204).

**Step 6's last store is `dest = 0`, and it is the whole store.**
`005f9d30`'s two closing blocks are the same write through
`update_order()->get_move_order()` — the order's `+0x10` — and the only
thing the successful one adds is the pause roll. The recovery does **not**
take the waypoint: `do_move`'s own `dest == 0` block does, on the next
frame, and that block is where `unit_masks & 8` is cleared and the leg's
arrival test runs. Taking it here is a frame's difference, because the top
of a fresh `find_upath` plan is the unit's own snapped cell: a unit handed
that waypoint stands *on* it with `dest` set, its arrival test never runs,
the straight-line check finds a top equal to its position, and the frame
falls through to `do_move`'s grid roll. The frame after that pops the plan
and walks the unit back into the collider it had just recovered from — a
two-frame livelock. run53's `1/7` did that from 5502 to the end of the
capture; `collide::tests::the_recovery_leaves_the_waypoint_for_do_move_to_take`
is the assertion.

**Step 0's predicate is read off the owner.** The original asks the object
what class it is; this crate has no `Animal` class and asks
`Unit::is_gaia()` — owner ≥ 8 — instead. The two agree on every capture
there is: every `ANIMALDATA` record in run12 (360) and run20 (936) carries
`who 8`, the pasture's carry `who 9`, and nothing gaia owns is anything
but an animal or a bird. A player-owned `Animal`, if one exists, would
part them; none has been seen.

**§5.2's pair came with item 66**, and it is the second consumer of both
indices: `Sim::find_collision` is `collide_here` for a land caller and the
3×3 chain walk otherwise, `Sim::find_ordered_collision` is that walk
against `orders_pos`, and `Sim::find_nearby_spot` runs the two as its last
test. `Sim::find_nearby_spot_coll` takes the `Coll` a call site wants —
every site is `Coll::Pairwise` but `come_out`'s two, which are the
original's `nocoll` and its unmodelled general path.

`Animal::do_idle`'s own `detect_unit_collision` came with the last of those
(item 49): a herd animal's wander destination is tested `quick 1` after
`is_valid` and before the order, so a sheep with a neighbour in the way
stays where it is. It costs no draw — all four of the wander's rolls are
already spent by the time the gate runs — and it is what keeps this
lobby's four `HERDSHEEP` standing where the original stands them.

`Unit::set_new_location` came with it, because the mechanic needs the point
that had been implicit: **`move_guys`**. `move_step` passes 0 and leaves the
body to chase; `resolve_unit_collision` passes 1, which teleports the body
onto the new point, so the follow phase reads `last_speed 0` and the next
frame's turn is instant. Without that the snapped unit spends a frame
turning and is one step behind for the rest of its walk — which is exactly
what run10's frame 124 showed before it was modelled.

**The block's return value landed 2026-09-06** (item 236), and it is
§5.3: both collision arms of `Sim::unit_step` answer `Did::Nothing`, the
value `move_step` answers, and `Sim::group_move_follower` reads it and
ungroups the squad — the same tail `Sim::group_move_leader` already took
off `do_move`. The attack-context hand-off above it is the leader arm's
own seam and stays one (§9).

**The soft half-step flag landed 2026-09-06** (item 219), and it is the
mechanic's first appearance in a *score*. `Unit::half_step` had been
written here since §4.3's group arm and nothing read it; `move_step` now
takes it as an argument, halves the step on the arm that owes less than
45°, and reports back so the caller clears the bit
(`sim::movement::Step::half_step_used`, `docs/MOVEMENT.md`). The three
Archers of run76 crowd each other on the frames after the group order and
pay for it exactly there — Great Lakes 6848 → 6862.

Not modelled, each listed in §9: ~~`detect_boat_collision` (no ships)~~
its sea half is modelled (§13); step 1
(`+0x2b4 & 0x2000`) and step 3 (the enemy ladder); the
`TRADE_ROUTE`, `0xc` and group arms of §4.3; §5.2's own group arm and its
general `find_unit_with_radius` path;
`do_move`'s own collision arm — the every-other-frame re-probe of
`coll_x/coll_y` while a search is pending, and the `repaths[who] += 1` in
it; squads, since only figure 0
marks the index; the `WData::block == −1` sentinel; and `CollBlock`'s lazy
allocation, replaced here by one flat bitset over the whole unit grid.

The object chain holds **units only**. The original threads buildings and
goodies through the same list, but the walk skips everything that is not a
unit, and dropping non-units from a linked list does not reorder the rest —
so the order in which units are found is the same. What it costs is a
diff: the dump's `down`/`down_who` cannot be compared field for field until
buildings join the chain, which is why §8 does not claim it.

## 8. Coverage

**Diff-backed** — `rondata::diff` against run10:

- **A gaia animal is a blocker like any other**, and the step's probe has
  no owner test on either side (§8.2). run85's block 7449 carries the
  original's own `collide_who 8`, so this is the dump's word and not a
  reading's.

- **The whole mechanic, once, field for field.** run10's `1/6` proposes
  `(41880, 17065)` on frame 122. `collide_here` reports the hit cell
  `(871, 354)`, inside `1/3`'s block; the corner rule makes it hard
  (`will_be_corner = 1` against `is_corner = 0`, and `|1 − 0| ≠ 4`); the
  dump's `collide 1`, `collide_frame 122`, `collide_o 3`, `collide_who 1`,
  `collide_guy 0`, `coll_x 41880`, `coll_y 17065` all follow. The snap puts
  the unit on `(41928, 17064)` — the original's frame-123 position — and
  `find_upath` adds the **five `flags 2` waypoints the dump prints, to the
  unit**. The unit then walks the original's frames to 208, where it used
  to part at 123. This is the run that moved the headline from 122 to 170.
- **The collision block over the whole capture, and it agrees entire**:
  `collide`, `collide_frame`, `collide_o`, `collide_who`, `collide_guy`
  and `safe` on every agreeing unit-frame of run10 — **48,790
  field-frames, zero disagreements** since item 64. It used to be 285 of
  40,600, of which 277 were one sticky byte: `collide_guy` is written by
  a hard collision and never cleared, so a single collision this
  simulation had and the original did not left `1/3` reading 0 against −1
  for the rest of the run. The fence in §6 step 2 is what stopped that
  collision happening, and with it the whole record. Pinned as emptiness
  in `run10_s_opening_…`, so one field on one unit-frame fails it.
- **The whole block on the second map too, over three thousand frames.**
  `run56_s_collision_block_agrees_past_the_scored_length` asks the same
  five fields of East Indies' longest full-detail capture — **249,293
  agreeing unit-frames, zero disagreements** — where run10's own number is
  139,514 and its capture is 1,772 frames. It was the first thing checked
  when East Indies' word reached 3021 and it came back empty, which is
  what sent that item to the *building* record instead (`docs/AI.md` §20).
- **`coll_x`/`coll_y`** on every dumped move order, as a scoring order
  mismatch — and since item 115 it is what carries Great Lakes' order
  score. §4.3's write-into-the-order was the last thing between that
  score and its own word: with the pair taken back after the probe, run10
  goes **791 → 1374** while its ticks, both first divergences, all
  fourteen units' partings and run33's word and totals hold exactly.
  `a_blocked_stand_keeps_the_point_the_probe_refused` is the sim's own
  half, written to fail first.
- **§5's `set_anim(CHAR_DEFAULT)`, against run14's draw-site trace.** The
  original spends the blocked stand on frames **122, 184 and 256** and on
  no others; this simulation spends its first two on the same frames, to
  the frame, which is what carries the traced word from 122 to **185**
  (item 49, 2026-08-28). The third is past the divergence. The gate on
  `Animal::do_idle`'s wander is measured by the same run: without it
  gaia's `8/1` walks off on frame 108 where the original's does not move
  for 120 frames, and takes a blocked stand of its own at 112.
- **§5.1's waypoint test, both sides of it.** run10's `1/4` re-picks farm
  cell `(0, 3)` on frame 199, where `1/2` is standing; the original names
  `1/2` on `collide_o`/`collide_who`/`collide_guy`, kills the `MOVE_TO`
  without a step, and `do_farm` picks `(2, 1)` on 201. Both order stacks,
  both positions and all three fields match, and run14's trace has the two
  `SITE_FARM_CELL` draws on 199 and 201 — the whole re-target schedule of
  the capture, to the draw. It took the traced word from 201 to **232**
  (item 63, 2026-08-28).
- **§6 step 2's fence, and step 6 behind it.** run10's `1/6` is a
  woodcutter standing inside its own camp's footprint at `(40680, 17688)`.
  On frame 206 it takes a fresh `MOVE_TO` to `(40680, 18168)`, steps south
  to `(40680, 17713)` and is refused by `1/1`. The camp is not `FLAT`, so
  the original falls through to step 6: `collide 1`, `collide_frame 206`,
  `collide_o 1`, `collide_who 1`, `collide_guy 0`, `coll_x 40680`,
  `coll_y 17713`, the snap a no-op because it already stands on its cell
  centre, and `find_upath` puts the dump's **seven-entry stack** on it,
  waypoint for waypoint. This simulation read `+0x94` as true, killed the
  order, re-made it on the next frame and did that for ever; with the
  fence it builds the same stack and walks the original's frames to 253.
  Headline 207 → 209 (item 64, 2026-08-28).
- **§5.2's pair, on the frame a citizen is trained.** run10's AI trains
  `1/7` on frame 206 and sends it to camp `2001`; `do_non_flat_gather`'s
  sweep starts on the unit's own bearing and its first passable candidate
  is `(40680, 17688)` — the exact quarter-tile `1/6` is standing on. The
  original refuses it and the six bearings behind it and issues
  `(40680, 18024)`, seven quarter-tiles further south, which is the first
  candidate clear of both `1/6`'s block and the `(40680, 18168)` it is
  itself walking to. With the pair the two agree, and `1/6` and `1/7` both
  hold to 253. Headline 209 → **252**, and run14's traced *word* went 232
  → the end of all 284 frames (item 66, 2026-08-28).
- **The stagger of §6's tail, and the throttle's decay** (item 80,
  2026-08-30). run33's frame 571 is the original's eight draws against
  this simulation's seven, and the one it was short is
  `Unit::resolve_unit_collision+0xb52` under
  `Unit::move_step+0x896 < Unit::do_move+0x1157`. The AI's `1/2` walks
  into `1/4` on that frame — the dump's next `UNITDATA` block carries
  `collide 1`, `collide_o 4`, `collide_who 1` — and each names the other,
  so step 5 refuses and the repath runs. It did not run here: `repaths[1]`
  had climbed monotonically to **5** over five hundred frames and stuck,
  because nothing halved it, and `(o + collide) & 3` threw the collision
  away three times in four. With the decay and the roll, run33's word
  parts at **576** instead of 571, its totals go 791/662 → **802/688**,
  and the headline goes ticks 571 → **572**, orders 571 → **576** with
  player 0 at 574 → **687**.
- **§6 step 0, on both captures at once** (2026-08-30). Every animal walk
  in run39 and run33 that **ends short of its goal** — sixteen, twelve on
  East Indies and four on Great Lakes, against seventeen that arrive —
  ends with the animal's position *unchanged* across the frame the order
  dies. That is exactly what step 6's cell-centre snap would break, and
  this crate broke it on all sixteen before the step was read.
  `a_blocked_animal_drops_its_walk_where_it_stands` is the check, and the
  sim's own half — an animal and a player's unit walking into the same
  blocker from the same point, one dropping its order where it stands and
  the other snapping and pathing around — is
  `an_animal_drops_its_walk_where_it_stands_and_takes_no_step`, written to
  fail first. It took East Indies' word 91 → **201**
  (`docs/SYNC.md` §3.14).
- **§4.2's fast path, and a `coll_size 2` unit's collision, both for the
  first time** (item 183, 2026-09-02). run66 is 260 blocks of run54's game
  over `[6340, 6600)` at run39's detail, and it holds East Indies' AI
  Merchant walking two hundred frames to its `CITRUS` past two standing
  citizens, colliding with one of them, snapping to its cell centre and
  pathing around. Every position of it is this crate's since the leading
  edge was modelled — **12,094 fields, zero disagreements**: every unit on
  every block as far as the word, and `1/19` alone for all 261, so its
  walk, its collision, its snap and its recovery are pinned past the frame
  the stream parts on. The dump's `collide 1`, `collide_o 11`,
  `collide_who 1`, `collide_guy 0`, `collide_frame 6571` are what named
  the hit cell, and the whole disc could not have produced them. East
  Indies' word 6570 → **6571**
  (`run66_s_window_is_the_original_s_unit_for_unit`).
- **§2.2's sixty-fourth-frame repaint, by the hole it fills** (item 191,
  2026-09-03). run70 is a `callwin` over `PathFinder::calc_cost` on Great
  Lakes frames 1955–1985, and `calc_cost` runs only for a cell that passed
  `valid_ucoord` — so the proxy's argument list *is* the index's answer,
  one row a cell. Of every cell the original's search and this crate's
  probed on frame 1970, exactly **one** was answered differently:
  `(849, 366)`, whose refusal turns on `(848, 367)` — a corner of the
  standing gatherer `1/10`'s block that the walking `1/9` had cleared
  eighty-one frames earlier and that `1/10`'s own repaint on frame 1910
  had put back. With the repaint the two searches are twenty-one
  expansions in the same order, and the routes they build are the same.
  Great Lakes' long word **2419 → 2808**, its collision record 228,821 →
  **247,543 field-frames with none wrong**, its buildings 650 wrong →
  **none**, and East Indies' run68 window clean to its last block with no
  exception but `stance` (`docs/PATHFINDER.md` §17).
- **§6 step 6's last store, by the frame it does not spend** (item 204,
  2026-09-03). run53's citizen `1/7` is walking back to its farm when
  `1/22` blocks it on frame 5501; both sides play the blocked stand, and
  on 5502 the original spends **nothing** for it while this crate spent
  `Unit::do_move+0xe84`. The cause was the recovery taking the top of its
  own fresh `find_upath` plan as the waypoint instead of clearing `dest`:
  that top is the cell the snap just put the unit on, so the arrival test
  — which runs only on the frame a waypoint is taken — never ran. With the
  store as `005f9d30` writes it, `1/7` takes the waypoint on 5502, arrives
  on it the same frame, walks the detour and reaches its farm on 5508.
  Great Lakes' long word **5502 → 5571**.
- **§5.3's return value, and the ungroup it drives** (item 236,
  2026-09-06). run76's window carries the whole of it: from the frame the
  formation ends, the three Archers `1/27`/`1/28`/`1/29` agree with the
  original on **every order and path field the dump prints** — the
  nine-entry `find_wpath` plan each builds for itself, its `tolerance
  384` rows, the `PATHED` bit the two non-leaders lose, and the waypoint
  and `coll_x/coll_y` that follow — and not one of the three leaves the
  original's point for the rest of the capture. Before the follower read
  `move_step`'s zero, that window carried `PathLength`, `PathTo`,
  `PathField`, `Move` and `Coll` rows from 6861 to its end. Great Lakes'
  long word **6862 → 6982**
  (`run76_s_window_is_the_ai_squad_s_march`, written to fail first).

  What the run does **not** back is the attack-context hand-off above the
  ungroup: every collider in every capture on disk belongs to the
  colliding unit's own player, so `valid_target` is false everywhere and
  the tail always reaches the ungroup. It stays a seam, and it is the
  leader arm's own.
- **Steps 4 and 5 in the *refusing* direction, and the whole track of the
  recovery behind them** (item 239, 2026-09-06). run83 closes the last
  forty frames of Great Lakes' run-up that no archive held, and the one
  event inside them is `1/29`'s block on **6892**. Its blocker `1/17`
  holds a `GATHERORDER`, so both fences that read *the other unit's*
  order kind refuse and what runs is step 6 — and the dump prints all
  three of its parts: the walker is put on **its own** cell centre
  `(42408, 23880)`, which is the step "backwards"; `dest` is cleared; and
  a three-entry `find_upath` plan goes on the stack above the untouched
  nine-entry world-grid one. The top two entries share the cell row the
  snap put it on, so it slides **due west at the full 26** — truncated to
  22 and 18 on the two frames it lands on a waypoint — for six frames
  with `y` pinned, and takes its diagonal again on 6900. `idle` is 0
  throughout, and this crate walks every one of those points. So a block
  is a **deflection, not a stop**: a model that stands still and repaths
  is six frames and about 1.6 tiles wrong here.

  The same window pins three things nothing had. The three collision
  fields have **three different lives** — `collide_o`/`collide_who` name
  the blocker for exactly *one* block and are −1 the next, `collide`
  latches 1 for six, `collide_frame` keeps the stamp for ever — so a
  comparison that reads `collide_o` a frame late sees −1 and calls it
  agreement. The AI **re-groups** the same three Archers into a
  `GROUPATTACKTOORDER` on **6907**, 46 frames after run76 watched them
  leave one, and this crate follows it on the frame it happens, carrying
  only the `id` stand-in. And the negative, which is the capture's own
  point: over 53 blocks **no unit parts from the original's position
  inside the hole and no order kind disagrees anywhere in it** — the only
  rows are the caravan `1/23`'s three, already off on the window's first
  block, and the stand-in. It moved no score
  (`run83_s_window_is_the_last_great_lakes_hole`, written to fail first:
  dropping step 6's cell-centre snap for four frames puts `1/29` off
  position from 6893).
- **§6 step 4's sidestep, retired by §5's post-step Manhattan test against
  the *unit's* tolerance and not the entry's** (§8.7). run90's `1/6` walks
  the original's four-block cycle — position, the five collision fields and
  the path stack alike — over the whole shuffle.
- The path stack's length and every waypoint — the headline's own order
  score, which the recovery's output now feeds.
- §2's clear-on-move, §2.2's repaint, §4's naming and §6's snap-and-replan
  end to end, in `crates/sim/src/collide.rs`'s own tests, each written to
  fail first.

**Reading-only** — no capture has executed these:

- §4.3's `TRADE_ROUTE` and `0xc` arms. ~~The group arm~~ — landed
  2026-09-04, §9. ~~The soft half-step flag~~ — landed 2026-09-06, and it
  is **diff-backed twice**: run76's three Archers march 6652 → 6861 on the
  original's own points and run53's word moved 6848 → 6862, and run79's
  second squad is 256 more frames of the bit's own record — seven flagged
  frames, seven halved steps after them, and no other half step in the
  march (`docs/ORDERS.md` §15.1).
- §4.3's **figure-centred `is_corner`** (2026-09-04). Both long words are
  unmoved by it, so no run on disk has a blocker whose figures answer
  differently from the blocker — which is what one would expect while the
  only colliders a capture reaches are one-figure citizens standing still.
- §5.1's tolerance-widening arm. Every hit a capture has reached there was
  a final waypoint under a gather, so the parked-collider branch rests on
  the decompile; `collide.rs`'s own test is what exercises it, and it was
  written to fail first.
- §6 steps 1 and 3.
- §6 step 6's `anti` conjunction and §6 step 4's `domain == 0` gate, both
  taken from the docs-versus-code wave with item 236 (§8.1). No capture
  has an attacking collider or two colliding boats, so neither is
  diff-backed and neither moved a frame.
- §6 step 5's wait-for-it branch (`unit_masks & 0x40`).
- The throttle's `repaths ≥ 4` and `≥ 8` arms. The decay of §6 step 6
  makes them rarer, not commoner: a player reaches 4 only by repathing
  eight times inside two frames.
- `UnitData::safe`: `find_upath`'s `+= 30` is modelled and the gate reads
  it, but no unit in any capture has ever carried a non-zero one — which
  the pin asserts rather than assumes.
- `coll_size ≠ 1` anywhere: every unit in every capture so far is a
  `BLOCK_RADIUS 1` type, so the ring-2 quirk and the parity filter's
  general form are untested behaviourally.

**The captures that would settle them** are in §9.

## 8.1 The docs-versus-code pass, 2026-09-05

`docs/audit/2026-09-05-collision-vs-code.md` (Opus reader, Opus
adjudication). About 115 stated rules of §1–§6 traced to the code that
implements them; ten disagree, all ten confirmed, none struck.

**§4.3's group arm is wrong here, and the code learned the error from it.**
`Unit::detect_unit_collision@00617060` short-circuits `0` and `0xc` *before*
the action test, so the kinds gated on "its action ≠ `ATTACK`" are the
**seven** `1, 2, 3, 4, 0x12, 0x13, 0x15`, and `0xc` is ungated. §4.3 calls
the gated set "the last four" — `0xc, 0x12, 0x13, 0x15` — and
`Sim::same_group_soft` implemented exactly that, with a further invention of
its own: it gated on whether the move carries a `GroupMove`, which the
original never consults, and exempted a plain move from the action test
altogether. Document, code and original were three different rules. This is
the shape a test written from the same reading cannot catch.

**Both were corrected on 2026-09-05** (item 231), from the export rather
than from the row: §4.3 above now carries the decompiled gate and
`same_group_soft` short-circuits `NONE` and `GUARD` before the action test,
gating the four move kinds — `0x13`/`0x15` are those same kinds here — on
`not_attacking` and consulting no `GroupMove`. It cost neither long word a
frame (East Indies 7448, Great Lakes 6848), which is what §9 predicts: both
arms need a marching squad with a group-mate, and the capture that would
reach one is the one §9 already owes.

**Half of §6 step 5's wait guard was the same.** The document had the guard
reading the unit's own wait flag; the original does not, and neither does
the code — so that half was a document error, and it is struck in step 5
above. The other half was a code error: the crate granted a wait where the
original refuses one when the collider's `collide_o` is negative. Both are
fixed, with the decompiled clause quoted in step 5; likewise no frame.

**Two more were taken with item 236** (2026-09-06), because that item's
work went through step 6 and the two are three lines between them. ~~§6
step 6's `anti` flag~~ is a conjunction now — my action `ATTACK` **and**
the collider's, read off the unit `resolve` was handed, as
`005f9d30:473-480` writes it. ~~§6 step 4's sidestep~~ carries its
`domain == 0` gate (`005f9d30:265`, `ObjectType +0x218`), so a boat
blocked by a boat goes to the wait and the repath. Neither moved a long
word — no capture has an attacking collider or two colliding boats — so
both remain the decompile's word and not a run's, and both are listed in
§8's reading-only half.

**Six are stated, unimplemented and unreached.** §6 step 4/5's order set
is four kinds where the original has
five, because **`CHANGE_FORM` is not an order kind this crate has at all** —
and `Unit::do_form_change@005e8670` being on the blind list is the useful
negative, since the missing kind cannot have cost a frame yet; §6 step 5's
`CAST_SPELL 0x28a` wait arm is absent; §5.1's `kill_current_order` action
set omits `TRADE_ROUTE`; §2's region gate also passes when the *cell* has no
region, where `move_unit`'s only escape is the *figure's* region being
negative; §6 step 4 additionally writes `has_waypoint` and zeroes
`tolerance` where the original writes only `+0x2c`/`+0x30`; and the
occupancy index is keyed on `Unit::pos` where `CollCheck::move_unit`'s only
caller keys it on the **figure's** position, which lags the unit's point.

None of COLLISION.md's cited addresses is on the 68-trace blind list, so
every row is "reached" by that list's rule — but that is a fact about the
*function*, not the arm. Both of §4.3's arms need a marching squad with a
group-mate, which is the capture §9 already owes.

## 8.2 run85 — the word's frame is a position, and the animal is not the question (2026-09-06)

`run85_s_window_is_the_east_indies_word_frame`. run54's game, `[7400,
7480)` plus the quit block 7496 — 81 blocks at `UNITS=3, GUYS=4,
BUILDS=7, CITIES=5, DEATHS=1, LEADERS=1`, driven through
[`run_traced`](../crates/rondata/src/diff/harness.rs), so what is compared
is the whole record and not the walker's. It is the first capture East
Indies has ever had within 3,200 frames of its own word, and it closes the
row §9 opened on 2026-09-04.

**The frame, off the disk.** `1/20` walks south-west at `(-13, -19)` a
frame, is put *back* to `(29256, 24888)` on 7449 with `collide 1,
collide_o 0, collide_who 8` — gaia's animal `8/0`, standing at
`(29304, 24696)` — holds that point for four frames while its angle snaps
to `-1073741824` (due west), and slides west with `y` pinned from 7454.
That is §6 step 6's repath, the shape run83 pinned on Great Lakes.

**And the two draws are one unit's two figures.** The trace's 7448 is 34
draws; the two the word is short are
`Guy::set_anim+0x97a < Unit::set_anim+0x56 < Unit::move_step+0x823` and
the same with `+0xb6`. `Unit::set_anim@00616f40` has **two loops** — the
squad's `0 .. guy_mark` and the crew's `type->squad_size .. guy_num` —
and `1/20` is a Merchant, `UBER_SIZE 1, CREW_SIZE 1`, so `guy_mark` is 1
and the record prints one `GUY` while the frame pays two set_anims. The
crew's is not owed on every stand: run66's 6571 spends `+0x56` alone,
because that crew figure's body was still walking and `Guy::set_anim`'s
early return takes it (`docs/ANIM.md` §4 step 1). `Sim::set_anim` loops
every guy and `Sim::body_at_des` is that gate, so both cases are already
modelled;
`collide::tests::a_gaia_animal_blocks_a_player_s_walker_and_a_crew_pays_the_stand_twice`
asserts the pair on a two-figure walker at rest.

**Gaia is not filtered out of a step, and never was.** The candidate cause
this window was opened to test — that `Sim::chain_hit`'s `who < 8` fence
(§5.2) makes an animal invisible — is **dead**, on both sides. Here, a
step probes `Sim::collide_here`, the occupancy bitmask, which
`Sim::coll_paint` writes for every unit with a block and no owner test at
all — §5.2 says so in as many words, "a land caller's `collide_here` sees
the cells a sheep paints, because the bitmask has no owner". `chain_hit`
is the *other* index, and it backs `find_ordered_collision` and the
sea/air arm of `find_collision`: §5.2's spot search, which a step's probe
never enters.  In the original,
`detect_unit_collision@00617060:150-172` walks the `down` chain asking
`alive`, vfunc `+0x18`, `domain != 2`, `is_on_map` and `is_here`, and
never `who`. The dump settles it from the third side: the original's own
`collide_who` **is** 8.

**What the window actually says.** Over 81 blocks and 2,268 compared unit
fields, **three** units are ever off position and none of them parts
inside the window on account of it:

- `1/20`, the Merchant, off from the **first** block — `(36, 792)`, which
  is about 34 frames of its own 23-a-frame walk. It is *behind on its own
  chain, not beside it*: its waypoint's **column** agrees for 28 more
  blocks (`Move.dest_x` first parts at 7428) while the **row** parts at
  once (`Move.dest_y`, 7400), because this crate is one leg further back
  on the same `find_upath` plan, and both sides carry the same
  `MOVEORDER` destination `(28728, 24120)` and the same stale
  `coll_x/coll_y` `(34999, 37321)`.
- `1/19`, the unpacked Merchant, off by a **constant `(24, 24)` on every
  block**, both sides standing still. The original put it on the tile
  corner `(32256, 36864) = 192 × (168, 192)` somewhere in the 470 frames
  no dump covers; this crate left it where run82's closing block had it.
- `0/5`, the human's, from 7468 — past the word and outside the item.

So East Indies 7448 is not a collision hole. It is `1/20` arriving
thirty-four frames late at a meeting it therefore never has, and the
thirty-four frames are spent between run82's last block (6929, where this
crate is one frame **ahead**) and this window's first (7400).

**Where they go is un-oracled.** In that gap `1/20` casts transport —
the trace dates the barge's birth at **7093** on both sides,
`Guy::init_real`, and this crate's is the same frame — rides it, and is
put ashore around 7321; the eject spends no draw at all (`report.py …
when Unit::come_out` is empty), so the word cannot date it and does not
constrain the ride. The arithmetic bounds it. Straight from each
side's own 6929 point — they differ by one frame's step — the original's
7400 point is **11,458** units away and this crate's **10,794**, both
over the same 471 frames; the ceiling for the
mix — 227 frames of barge at 26, which is this crate's ride, and the
other 244 at the Merchant's 23 — is **11,514**. So the original spends
**99.5 %** of those frames' full speed making straight-line progress and
this crate **93.8 %**. That is a
**route**, and it is the barge's: its birth spot, its `find_upath` plan
and `come_out`'s ring are the three places 792 units can hide, and a
48-unit nudge to the last of them puts nineteen units off position in
this same window. *Capture, owed:* East Indies at run85's detail over
`[7080, 7140)` and `[7290, 7340)` — the cast and the landfall — which is
the only thing that can say which of the three it is. Until then the
long word stays at 7448 (`docs/TRANSPORT.md` §6, and `docs/MERCHANT.md`
for `1/19`).

## 8.3 run87 — Great Lakes 7455 is not a collision question either (2026-09-07)

`run87_s_window_is_great_lakes_word_frame`. run53's game, `[7244, 7520)` at
run79's detail with `GUYS` 2 → 4 — 277 blocks, 121 MB, the first Great Lakes
dump within 200 frames of its own word. The parting is the mirror of §8.2's:
**this crate** spends a `SITE_BLOCKED` on 7455 and the original spends none.

**The stand is invented, and the blocker is where we put it.** The walker is
`1/36`, a Longbowman of run79's second squad; the blocker this crate names is
`1/17`, run83's own standing citizen. In the original `1/36` walks an
unbroken (−25, −23) a frame across 7455 with `collide 0`, `collide_o −1` and
`collide_frame −1` on every block of the window, and `1/17` stands at
**(41784, 23928)** — the point this crate has for it — for all sixty blocks
of `[7420, 7480)`. So §4's gates and §5's block are not what is wrong.

**And they are right wherever the two sides stand in the same place.** The
original's only three `collide_frame` transitions in the window are `1/31`
on 7285, `1/7` on 7287 and `1/32` on 7293, and this crate stands on those
three frames, on the same walker, blamed on the same blocker.

What is wrong is `1/36`'s position — 94 units, about three and a half frames
of its walk, adrift by 7455 — and the field diff dates that to two rows
neither of them this document's: `1/35`'s step size on **7253** (25 units of
y where the original takes 12) and the six-slot assignment at the army's
**7418** tick, where four soldiers come out on a different waypoint and
`1/32` and `1/33` on exactly each other's. `docs/RUNS.md`, "run87".

**One thing this window says about the diff itself.** The harness compares
the collision record only where the two positions agree (`compare`, and
rightly — otherwise a walker that is elsewhere colliding with something else
counts the position gap twice). `1/36`'s position parts at 7420, so a stand
invented at 7455 costs a draw and produces no row at all: on this map the
draw stream is the only oracle for the word's own frame.

## 8.4 The soft flag is a dumped field, and now a compared one (2026-09-07)

`UnitData::unit_masks` is printed on every capture that prints the record,
and `0x100000` — §4.3's soft one-shot — sat in it uncompared from the day
the flag was modelled (item 219) until item 267. The harness took `0x80000`
out of the same word for the packed bit and left the rest.

It is now a row of the collision record's own comparison in
`rondata::diff`'s `compare`, beside `collide`, `collide_o`, `collide_who`,
`collide_guy` and `safe`, and gated the same way: only on unit-frames whose
positions agree, because a unit standing somewhere else soft-collides with
something else as a consequence. The alignment needs no allowance —
`detect_unit_collision` sets the flag at the end of the nine-cell sweep and
the *next* frame's `move_step` spends it, so an end-of-frame dump holds
exactly the flag the following step will read.

**What it caught on its first run.** run87's window carries **10,458** of
these rows and one disagreed: `f7252 1/35 half_step 0 v 1`, which dated
Great Lakes' run-up divergence a frame earlier than the position diff had
(7253 → 7252) and named the mechanism outright — a soft collision this
crate did not make, rather than a step size it computed wrong. §2.3 is the
cause and the fix; with it in, the row is clean and the earliest parting in
the window moves to **7315**, a unit older than the window.

The rest of the word is still an occupancy question and not a predicate
one: nothing in §4's gates, §4.3's ladder or §5's block was wrong, and the
index that feeds them was.

## 8.5 East Indies 7806 is a two-citizen shuffle in the frames no dump covers (2026-09-07)

[`LONG_WORD_EAST_INDIES`] has been 7806 since item 271 put the age snap in,
and the block the map's last capture holds is **7799**. Item 276 asked the
cheap question first: run88's `!quit` left a `GameLog::end_game` block at
**7816**, ten frames above the word, and nothing had ever read it as the
word's residue. **It is the residue, and it is not the mechanism.**

**What the closing block gives.** Two units part on it and no others are
new — `1/6` and `1/7`, the AI's citizens walking to the adjacent build
sites `ox 2015` and `ox 2016` — and the diff prices them:

| unit | ours | theirs | out by |
|---|---|---|---|
| `1/6` | (39624, 38808) | (39578, 38757) | (+46, +51) |
| `1/7` | (39666, 38676) | (39672, 38664) | (−6, +12) |

`1/19` and `1/20` are the two standing Merchant constants already on the
row. The pair is pinned in `run88_s_window_is_east_indies_word_frame`, both
sides' coordinates, so the capture below has a number to move.

**Why the block cannot be the whole answer.** A closing dump is frame *n*
except for the one unit the quit caught mid-update (item 257, open), so two
rows on a single block are worth what a second instrument makes them — and
the block is ten frames *downstream* of the word besides. The second
instrument is the **trace**, which is not a dump and cannot be torn: run54's
draw stream over `[7800, 7820]`, against this crate's, and the two sites
that matter are [`sim::anim::SITE_BLOCKED`] — `Guy::set_anim+0x97a <
Unit::move_step+0x823`, the stand a blocked unit plays — and
[`sim::collide::SITE_PAUSE`], §6 step 6's stagger roll, the only draw the
whole mechanic spends.

| frame | the original | this crate |
|---|---|---|
| 7802 | 2 stands, 1 pause | 2 stands, 1 pause |
| 7806 | **1 stand** | — |
| 7807 | — | **1 stand** |
| 7810 | 2 stands, 1 pause | 1 stand |
| 7812 | — | 1 stand, 1 pause |
| 7817 | — | 1 stand |

7802 is the first collision and both sides make it: `1/6` and `1/7` bump,
each takes §6 step 4's sidestep, and `1/7` rolls `pause 3` off the shared
stream. Everything below 7806 agrees. **The word is the second stand: the
original's comes on 7806 and this crate's on 7807**, and every row after it
is that one frame compounding — the original is done colliding by 7810,
this crate takes two more cycles, and its `PathFinder::calc_road_cost`
search of 137 cells runs on **7814** where the original's 136-cell search
runs on **7833**.

~~**Where the frame is lost.** `1/6` arrives on its own sidestep waypoint
(39720, 38808) on `f7804` and pops it; on `f7805` it turns without stepping
(facing −1341784064 against a heading of −671481856) and takes the step on
`f7806`, which is the frame the original was already blocked on. So the
candidate is the **approach** — `f7804`/`f7805`, one frame of turn-versus-step
after a waypoint pop — and not the collision response, which §8.3 and run83
have both diffed clean.~~

**Dead, and refused by the capture it booked** (§8.6, and the answer is
§8.7). The original never arrives on that waypoint at all: it retires it
after one step from fifteen units away, because `resolve_unit_collision`
does not write `UnitData::tolerance` when it pushes it. The frame goes into
the step *to* the waypoint, one block earlier, and there is no late pop to
find. This paragraph is kept because the shape of the error is the method's:
a draw stream dated the word two frames after the value diff would have, and
a reading written from the stream alone put the mechanism on the wrong side
of the pop.

**The suspicion that is not established.** On 7810 the original spends a
pause roll and this crate spends none; the unit this crate has in collision
on that frame, `1/7`, sets §6 step 5's **wait** flag instead
(`coll 1/1/6 wait true`) and does not reach step 6 until 7812. If the
original's roll is `1/7`'s, the wait-versus-repath predicate is wrong —
which is exactly the shape `docs/audit/README.md` says the errors take. But
7810 is downstream of the parting, so it is as likely to be the frame
offset carried forward. **Only the capture separates them.**

**This crate's own rows, for the capture to refuse.** `collide`,
`collide_o`, `collide_who`, the wait bit inside `unit_masks` and the
order's `pause` are all dumped fields, so every line below is a prediction:

```text
f7802  1/6 (39750,38787) coll 0/1/7  pause 0   1/7 (39720,38664) coll 1/1/6 pause 3
f7803  1/6 (39729,38802) coll 0/-1   pause 0   1/7 (39695,38664) coll 1/-1  pause 3
f7804  1/6 (39720,38808) coll 0/-1   pause 0   1/7 (39695,38664) coll 1/-1  pause 2
f7805  1/6 (39720,38808) coll 0/-1   pause 0   1/7 (39695,38664) coll 1/-1  pause 1
f7806  1/6 (39699,38795) coll 0/-1   pause 0   1/7 (39695,38664) coll 1/-1  pause 0
f7807  1/6 (39699,38795) coll 0/1/7  pause 0   1/7 (39672,38664) coll 1/-1  pause 0
f7810  1/6 (39672,38808) coll 0/-1   pause 0   1/7 (39654,38682) coll 1/1/6 wait
f7812  1/6 (39651,38794) coll 0/1/7  pause 0   1/7 (39672,38664) coll 3/1/6 pause 6
f7815  1/6 (39624,38808) coll 0/-1   pause 0   1/7 (39666,38676) coll 3/-1  building
```

`f7815` is the closing block's own frame, and the two positions there are
the table at the top of this section. The dump prints a block as `FRAME
n+1`, so `f7804` is block 7805.

*Capture:* `[7790, 7900)` at run88's detail **exactly** — 110 blocks, about
63 MB, ten blocks (7790..7799) over run88's tail so the overlap is byte for
byte with no `--exclude` at all. It holds the whole shuffle (which begins
after 7799), the word, and the original's 7833 repath. The wider
`[7740, 7900)` costs 50 more blocks of straight-line walking run88 already
shows agreeing.

## 8.6 run90 — the sidestep waypoint is abandoned, not walked to (2026-09-07)

§8.5 booked a capture to be **refused**, and it was. run90 is run54's game
over `[7790, 7900)` at run88's detail exactly — **111 blocks, 71,645,977
bytes**, ten of overlap over run88's tail (`differ: 0`, no `--exclude`) — and
it is the first dump of any frame in `[7800, 7899]` on either map.

**Two predictions held exactly.** Both sides make the first collision on
block 7803 with `collide 0 / collide_who 1 / collide_o 7` on `1/6` and
`collide 1 / collide_o 6` on `1/7`; and `1/7`'s `MOVEORDER pause` reads
**3, 3, 2, 1, 0** over 7803-7807 on both sides, off the shared stream.

**Two were refused, and that is the finding.** `1/6`'s *position* parts on
block **7805**, two blocks below the draw stream's word:

| block | ours | theirs | delta |
|---|---|---|---|
| 7804 | (39729, 38802) | (39729, 38802) | — |
| **7805** | **(39720, 38808)** | **(39729, 38802)** | **(−9, +6)** |
| 7806 | (39720, 38808) | (39708, 38789) | (+12, +19) |
| 7807 | (39699, 38795) | (39708, 38789) | (−9, +6) |

**The waypoint is the same and the arrival rule is not.** The original
pushes the identical §6 step 4 sidestep — `(39720, 38808)` `flags 2` on
7803, `(39672, 38808)` on 7807, `(39624, 38808)` on 7811 — and its path
drops 5 → 4 on the **very next block**, with the unit at (39729, 38802),
which is not that point. It takes one full step along the bearing and
abandons the waypoint. This crate keeps it and walks the remainder, a short
`(−9, +6)` step. So the original's blocked cycle is **four** blocks (block,
step, turn, step) and this crate's is **five**, and the extra frame per
cycle is the whole of the word.

That refuses §8.5's candidate. The frame is not lost in "turn versus step
*after* a waypoint pop": it is lost in the step *to* the waypoint, one block
earlier, and the pop is not late — it never arrives.

**And §8.5's unestablished suspicion is now established.** The `SITE_PAUSE`
the original spends on sim-frame 7810 is `1/7`'s: its `MOVEORDER` carries
**`pause 8`** on block 7811, where this crate sets the wait flag and rolls
nothing. So the wait-versus-repath predicate is wrong — the shape
`docs/audit/README.md` says the errors take.

**The trace and the dump agree on the same three frames**, which is the
corroboration a torn closing block could not give: the original's
[`sim::anim::SITE_BLOCKED`] falls on sim-frames 7802, 7806, 7810, and the
blocks whose `1/6` names `collide_o 7` are 7803, 7807, 7811 — the same
three, four apart. This crate's stands are 7802, 7807, 7812.

**One rule no reading had stated: the `pause` countdown is frozen while the
unit is still colliding.** `1/7`'s 8 stands on eleven blocks (7811..7821,
where `collide` counts 1..9 and then clears) and only then ticks down one a
block to 0 on 7829.

`run90_s_window_is_east_indies_shuffle` is the assertion, and its six claims
were each made to fail on purpose before being believed: the original's
7804 path length moved to 5, its `pause 8` to a 9, `1/6`'s 7805 row to
(39730, 38802), `1/7`'s first parted block to 7812, the parted set's `1/6`
to 7807, and the stand list's 7810 to 7811. All six failed.

**The word does not move.** Nothing was fixed here; three units part
downstream of it (`0/5` at 7824, `1/2` at 7872, `1/5` at 7895) and are
pinned as the shuffle's wake rather than scored.

## 8.7 The sidestep waypoint's arrival rule — one store that is not made (2026-09-07)

§8.6 left the mechanism open in one sentence: *which of `move_step`'s tests
drops a `flags 2` waypoint one step in, and is the drop unconditional or a
tolerance this crate has too small.* It is neither. **The drop is the
ordinary arrival test, and the tolerance is not the waypoint's.**

**The rule.** `move_step@005faf30` ends an accepted step with

```
if (UnitData::tolerance < |dest_x − x| + |dest_y − y|) return 1;
dest = 0; path.length -= 1;                       // arrived: pop
```

(§5, and the fields are the obfuscated `+0x10`/`+0x14` pair). The tolerance
is the **unit's**, and `resolve_unit_collision@005f9d30`'s `LAB_005fa37a`
never writes it: it pushes `{cell centre, tol 0, flags 2}` and stores the
order's `+0x2c`/`+0x30`, and that is all (§6 step 4). A path entry's own
tolerance reaches `UnitData::tolerance` in exactly one place —
`do_move`'s `dest == 0` take (`docs/ORDERS.md` §4.4) — and the sidestep
never goes through it, because `dest` is already 1 by the time `move_step`
runs and step 4 does not clear it.

So a sidestep waypoint is walked under **the interrupted leg's** tolerance.
For a citizen on a world-cell plan that is 384, and the sidestep is one
48-unit cell away, so the first step that is not blocked ends the leg
wherever it lands.

**What it cost.** This crate zeroed `UnitData::tolerance` with the push, so
the arrival test could only fire on the waypoint itself and the unit had to
spend a second step walking the remainder. One frame per collision, and a
five-block cycle where the original's is four.

**The cycle, both sides, run90 blocks 7802-7807.** The original:

| block | `1/6` | stack | what happened |
|---|---|---|---|
| 7802 | (39750, 38787) | 4 | a step on the leg to (38856, 38232) |
| 7803 | (39750, 38787) | **5** | blocked by `1/7`; step 4 pushes (39720, 38808) `flags 2`, no step, **no arrival test** |
| 7804 | (39729, 38802) | **4** | one step along the bearing; 9 + 6 = 15 ≤ 384, so `dest = 0` and the waypoint is popped **from fifteen units away** |
| 7805 | (39729, 38802) | 4 | `dest` 0 → 1 takes (38856, 38232) again and turns; no step |
| 7806 | (39708, 38789) | 4 | a step |
| 7807 | (39708, 38789) | 5 | blocked again — four blocks later |

This crate had 7805 at (39720, 38808) — the waypoint itself — and its
7807 where the original's 7806 is. The pushed points are identical on both
sides on all three cycles ((39720, 38808), (39672, 38808), (39624, 38808)),
which is what said the *sidestep* was right and the *arrival* was not.

**How this was established.** From the dump first and the decompile second,
which is the order the working agreement asks for. run90 is 111 blocks of
`UNITS=3` over `[7790, 7900)`, so the path stack, the move order and the
five collision fields are all printed on every block of the cycle; the
4 → 5 → 4 with the unit **not on the point** is one record's own three rows
and needs no reading at all. The reading then names the store that is
missing rather than guessing at a predicate.

**Confidence: high, and diff-backed.** The change is the deletion of one
line. `run90_s_window_is_east_indies_shuffle` now pins, on the original's
own blocks: `1/6`'s position agreeing through 7826; its collision fields —
`collide`, `collide_o`, `collide_who`, `collide_guy`, `collide_frame` —
never disagreeing anywhere in the window, so `collide_o 7` falls on 7803,
7807 and 7811 as the dump's rows say; and its **order record, path stack
included**, first disagreeing on 7828. A four-block cycle whose stack
length and whose `collide_o` are both the original's is the same cycle, and
the five-block one could not have produced either. run88's closing block
loses `1/6` from its residue at the same time — the citizen the `!quit`
caught 46 east and 51 south of the original's now stands where it stands.

**What moved.** East Indies' long word **7806 → 7812**, and the new word is
`PathFinder::calc_road_cost+0x46`, 142 draws this crate spends on 7812 and
the original spends on 7833. Great Lakes' word does not move; its
**endpoint** at 24,001 falls 84 → 80 off and 2 → 1 extra, which is the
mechanic being one every colliding unit on either map walks through.
East Indies' own endpoint goes 79 → 78 off and 11 → 13 extra, the C rung
46 → 47 and 22 → 24, the B rung 20 → 19 extra.

**What this has *not* established.**

- **Whether `MoveOrder::dest` can be 0 when `resolve_unit_collision` runs.**
  Every path into `move_step` sets it — `do_move`'s take, and both arms of
  the straight-line check — so this crate still writes `has_waypoint = true`
  with the push, which is a no-op on every frame any capture has reached.
  If a route exists that reaches `move_step` with `dest` clear, the original
  would *take* the sidestep next frame and pick up its `tolerance 0` after
  all, and this crate would not. Nothing on disk shows one.
- **The Manhattan-versus-`vector_dist` split.** `move_step`'s tail is
  Manhattan and `do_move`'s take is the octagonal `vector_dist`; both are
  modelled as read, and no capture separates them, because every arrival a
  run has reached is far inside either.
- **Whether a non-384 leg changes the story.** Every sidestep on disk
  interrupts a `find_wpath` leg at `tolerance 384`. A unit sidestepping off
  a `find_upath` leg carries `tolerance 0` and would then have to walk the
  waypoint exactly — the same code, the opposite behaviour, and no run
  reaches it.

## 8.8 run90 block 7809 — the frame this crate probes and the original does not (2026-09-07)

§8.6's second refusal, and §9's standing row since: `1/7`'s **entire**
collision-field divergence over run90's 111 blocks was block **7809**, where
the original prints `collide 1 / collide_o 6 / collide_who 1` and this crate
had the fields clear. The row read as §6 step 5's wait-versus-pause predicate,
which is the shape `docs/audit/README.md` says the errors take. **It is not
the predicate, and it is not §6 at all.** This crate made the collision and
then threw it away, on a frame the original never asks.

**The rule, and it is two instructions.** `Unit::move_step@005faf30`'s two
turn-in-place arms are

```
Guy::do_turn(guy 0, heading, facing, 0, 1); return 1;
```

and nothing else — the far arm at `005fb2b4`, returning at `005fb2b9`, the
near one at `005fb2e1` returning at `005fb2e6`; on the listing each is
`call 5d97a0` / `mov eax,1` / three pops / `ret 8`. So a frame spent turning
in place reaches **none** of what follows: not the four world-bounds tests,
not `detect_unit_collision`, not `invalid_loc`, not `set_anim(CHAR_WALK)`,
not `set_new_location` and its reveal, and not §5's post-step arrival test.
The `return 1` also means the arms are not among the three
`move_step` answers `0` from (§5.3), so a turning follower does not ungroup
its squad.

**The probe is the one that costs a frame**, because `detect_unit_collision`
does its clearing on **every** path out (§4.1): `collide_o = collide_who =
−1`, the wait bit down, and `collide = 0` when the stamp is more than five
frames old. Run it on a turning unit and it wipes what `do_move`'s own
waypoint probe (§5.1) wrote three statements earlier **in the same frame**.
The original keeps both because it never asks again.

This crate walked on deliberately: `move_step`'s `set_anim` was already
guarded against the turn arms (the walk is not owed on a turning frame), and
the comment there said the store *was* owed, since this crate steps on a copy
of the order. The store is owed; the probe is not, and nothing separated
them.

**The frame, in full.** On block 7808 `1/7` stands on its snapped cell centre
(39672, 38664), cell (826, 805), with `collide 1` left over from the 7802
resolve and no collider named. During the frame:

1. `do_move` finds `MoveOrder::dest` clear and takes the next waypoint,
   (39624, 38712) — cell (825, 806) — setting the heading to 225°, 44° off
   the 270° it was walking.
2. §5.1's waypoint probe runs the **full** form on that cell. `1/6` is at
   (39685, 38801), cell (826, 808), and its block paints (825..827) ×
   (807..809); the proposal's own cell is a diagonal step away, so §4.2's
   fast path does not apply and the parity disc's `(+1, +1)` corner lands on
   (826, 807) — `1/6`'s paint. The corner rule lets nothing past (`will 5`
   against `theirs 0`), so it is hard: `collide_o 6`, `collide_who 1`,
   `coll_x/coll_y = (39624, 38712)`.
3. `find_path` verifies the line and `move_step` runs. The 44° turn is over
   the far arm's limit, so `Guy::do_turn` and `return 1`. **The original
   stops here.** This crate probed the proposed point, which is the unit's
   own cell because the step was not taken, hit §4.1's gate 6, and cleared
   everything step 2 had written.

Block 7809 is the result on both sides: the original with the collider named,
this crate with the fields clear and, because the stamp was 7802 and the
frame 7808, `collide` aged to 0 as well.

**How the dump says it, with no reading at all.** `resolve_unit_collision`
step 5 writes `collide += 1` and `collide_frame = frame` **together**, so the
stamp *moving* is the one dumped witness that the resolver ran — and a value
test on `collide_frame` says nothing, because it is a permanent stamp that is
never cleared. Over run90's window `1/7` names a collider on **twelve**
blocks — 7803, **7809**, and 7811 through 7820 — and the stamp moves on
exactly **two** of them, 7803 and 7811. Block 7809 is a naming with no
resolve and no step: `collide_o 6` with `collide` still 1, the stamp still
7802, and the position identical to 7808's. That is a probe that ran and was
not followed by a second one, and nothing else in §4, §5 or §6 produces it.

**Confidence: high, and diff-backed.** `run90_s_window_is_east_indies_shuffle`
now pins, on the original's own blocks: the twelve naming blocks, the two
stamp *changes*, and block 7809's whole row beside 7808's — each made to fail
on purpose first (7809 moved to 7810 in the naming list, a third stamp change
added at 7809, `collide` read as 2). On the comparison's side the window's
collision divergence is **empty for the whole cast** — 111 blocks, every unit,
all five fields — where it was `1/7`'s three rows before. And `1/7` now rolls
the original's `pause 8` on 7811, off the shared stream, with `collide_frame`
7810 to match: §6 step 5's wait-versus-repath predicate was **right all
along**, and was only ever reached with the wrong inputs.

**Does the return suppress a detection the original makes? It cannot.** Two
things say so and neither is a judgement. First, nothing but an owed turn
reaches those two `ret`s: the near arm is `move_step`'s `if (local_10 != 0)`
and the far one its `if (!bVar16)`, where `local_10` is the turn still owed
and `bVar16` the far threshold's comparison against it, and both sit inside
the same turn block — with `local_10` zero neither is reachable. Second, and
decisively, **the probe the return deletes could never have found anything**.
Both arms leave the unit where it stood, so the point the old code then
handed `detect_unit_collision` was the unit's own position, and §4.1's gate 6
refuses a proposal inside the caller's own cell *before* the probe runs. The
call could only ever take the clearing path. So the change removes a
**clearing** and no detection, and a turning unit takes no step either way —
"turning units now walk through each other" is not a state this change can
produce.

What a turn frame does lose, all of it faithfully, is three things that are
not the probe: `set_new_location` and the half-cell reveal behind it, the
crew reseat a same-point `set_new_location` performs, and the leg's Manhattan
arrival test — which §5 already states runs only after an **accepted** step.

**The census moved, and the rosters say which shape it is.** Both East Indies
ladder rungs' `extra` — units this crate has and the original does not, eight
and nine thousand frames past their own words — rose on this change: the C
rung 7 → 18 and the B rung 8 → 16, the largest rise either has taken, on a
change that moves no word. The number alone is the shape a too-broad
suppression would make, so the rosters were read on both sides of the change:

| rung | before | after |
|---|---|---|
| C, 15401 | `1/10 1/12 1/25 1/29 1/48 1/49 1/54` | **the same seven**, plus `1/55 … 1/65` |
| B, 16489 | `1/12 1/56 … 1/60 1/63 1/64` | **the same eight**, plus `1/61 1/62 1/65 … 1/70` |

Not one existing extra moved, every addition is a **contiguous run at the top
of the roster** — the AI's most recently produced units — and `off` is
unmoved on both rungs (46 and 52). Units that had stopped colliding would
move positions and would scatter through the roster; eleven and eight
consecutive object numbers at the head of production are over-production.
East Indies' own endpoint, which has no extras to give, went 78 → **76** off
with `unlinked` 3 → 5: two positions closer against two of the original's
units this crate no longer matches, the same reshuffle item 265 made in
reverse when it bought a position with four spurious units.

**What is not established: which of the three suppressions pays for it.** The
arrival test is the candidate with a mechanism — a leg that could previously
end on a turning frame now needs an accepted step, so a walk ends up to a
frame later, a builder reaches its site later, and the AI's purchase clock
moves with it — but no unit has been traced from a turn frame to a purchase,
and the reveal is an equally good story. *No capture is needed:* both rungs
are on disk at `BUILDS=1`, so the queue records over the frames `1/55`
onward are produced name the purchases, and the rungs' own `UNITS` records
name what each unit was doing. It is a probe of two dumps already held.

**The endpoint figures above are this branch's.** 289 and 290 each re-pinned
these rows on their own branch and the merged tree agreed with neither,
because two independent improvements compose; the merge is where they are
adjudicated, not here.

**The word does not move, and what it now sits on is a pathfinder question.**
`1/7`'s position still parts on 7812 and [`LONG_WORD_EAST_INDIES`] stays at
7812 — the same `PathFinder::calc_road_cost+0x46`, 142 draws here against the
original's on 7833. The new first divergence is its **order record, on 7811**,
one block below the word, and run90 prints both sides of it. The original's
repath on 7810 pops the stack to the move's own goal (39624, 38760) and its
`find_upath` **suspends**: `UnitData +0x104` stays set, `MoveOrder::dest`
reads 0 and the stack stays one entry long on every block 7811..7820 while
the unit stands still on its cell centre, and `do_move`'s suspended-search
block (`docs/ORDERS.md` §4.4 step 2) counts `collide` 1 → 9 with the stamp
untouched — the increment is **unconditional per frame**, not every fourth
as that section says; only the `repaths[who]` bump is on the four-frame
phase. The search finishes on 7819, `collide` is zeroed on 7820 and the unit
steps on 7821.

This crate's `find_upath` does not get that far. Its pre-A\* block walks the
goal toward the unit in 0x18 steps while `valid_ucoord` refuses it — and
(39624, 38760) sits in cell (825, 807), which is `1/6`'s paint on that frame
— until the walked goal lands in the unit's **own** 48-cell, at which point
it is pushed as a one-entry final leg, (39666, 38676). The unit walks to it
on 7812, the leg is final, the `EXPLORE_TO` dies, and the build search runs
twenty-one frames early. Two things are open in that and neither is this
document's: whether the original's `valid_ucoord` refuses the same cell (it
cannot have walked the goal back, because the dumped stack still holds
(39624, 38760)), and the suspend itself, which `docs/PATHFINDER.md` §11 calls
a dormant seam — "suspend returns −1 without stashing (its restorer has no
caller until collision recovery exists)". Collision recovery exists now, and
run90 is the first capture to reach it.

## 8.9 Great Lakes 9134 — the arm this crate did not have (2026-09-18)

Item 360's brief was a widening with no hypothesis, and the widening took
twenty minutes because the frame is **one draw on each side**.

```
frame 9134: ours 1 theirs 1 — at 0,
  ours Some("Guy::set_anim+0x97a < Unit::move_step+0x823")
  theirs Some("5dac7a")
```

`5dac7a` is `Guy::set_anim+0x97a`, the address eleven entries of
`rondata::diff::SITES` already carry under eleven different `ebp` chains.
The twelfth chain — `Unit::set_anim+0x56 < Unit::move_step+0x4e2`, read
straight off `report.py … draws 9134` — had no entry, so the trace printed
the bare address and the comparison could not fail on the *count*. It is
the clearest case yet of the rule the working agreement states: a bare
address in a dumped sequence is a comparison that cannot fail, and naming
it is the whole assertion.

**The unit is `1/32`, and run97 has it whole.** Tabulating its
`x_internal`/`y_internal`, `dest_x`/`dest_y`, `coll_x`/`coll_y`, order
flags and path stack over `[9119, 9137]` — every field of the record, not
the ones the brief named — puts the two runs one frame apart from 9134
and nowhere before it:

| | ours, after tick N | run97 at N+1 | |
|---|---|---|---|
| 9133 | `(42801, 22824)`, path `[(44576, 22755) f1]` | `(42801, 22824)`, path `[(44576, 22755) f1]` | agree |
| **9134** | `(42792, 22824)`, path `[]`, order flags 4, ungrouped | `(42801, 22824)`, path `[(44576, 22755) f1]`, flags 5 | **part** |
| 9135 | `(42792, 22824)`, path `[(44568, 22776) f1]` | `(42792, 22824)`, path `[]`, flags 4, ungrouped | ours is 9134's |
| 9136 | the eight-entry `find_upath` plan | `[(44568, 22776) f1]` | ours is 9135's |

Everything the original does on 9135 this crate did on 9134. The cause is
§5.4: `1/32` is close enough to its next formation hop to land on it in
one step, the hop is blocked, and the original's snap arm stands for the
frame while this crate ran §5's give-up chain into
`resolve_unit_collision`.

Three fields of the same record say the arm is the snap's and not §5's,
without reading anything: on 9135 the original carries `coll_x 42825 /
coll_y 22827` (so the probe *did* return a hard collision) beside
`collide_o −1 / collide_who −1` (so something cleared them) and
`collide_frame 7293` (so `resolve_unit_collision` never ran — it is nine
hundred frames stale, and the original's own resolve on 9135 writes
`collide_frame 9135`).

**What it moved.** Great Lakes `8029..` widened either side, both columns
measured at the new word so the windows match:

| | base | now |
|---|---|---|
| Great Lakes long word (run53) | 9134 | **9182** |
| East Indies long word (run54) | 9711 | 9711 |
| point-and-goal fields wrong below the word | 183 | **6** |
| — per frame of `[8029, 9182)` | 0.159 | **0.0052** |
| — units ever off position | 6 | **4** |
| walk-slot residue | 103 | **0** |
| — per frame | 0.089 | **0** |
| standing trio's own (24, 24) | `6 × frames` | `6 × frames` |

`1/31` and `1/32` leave the off-position list entirely; what is left is
the standing trio `1/24`–`1/26`, excused and older than every window on
this map, and `1/33`'s late drift from 8584.

**The successor is 9182, and it is the market's**, exactly where item 358
left it: the original spends three `Leader::use_market+0x1ed` and two
`Leader::make_stuff+0x221` where this crate spends one, three, and two
`+0x63d` slot expiries. Its tail differs too — one
`Guy::set_anim+0x104b` against three. `docs/AI.md` §41 names the
`LEADERS=9` window that would settle the leader half.

**No frame past run97's 9349 was wanted at any point.** The widening ran
`[9119, 9145]`, the cause is at 9134, and the new word at 9182 leaves
**167 frames** of run97 still ahead of it. The next capture is owed rather
than urgent — but the margin is now thin enough that a successor to 9182
may well want `[9350, …)`.

**And the arm has a second capture now** (item 448, `docs/AI.md` §54):
Great Lakes' word at **10161** is the same block on `1/38`, off run100 —
`coll_x/coll_y (42774, 22584)`, `collide_guy 0` sticky, `collide_o` and
`collide_who` back at −1, `dest` 1 → 0, `length` 2 → 1, no step, and the
idle rolled at `Unit::move_step+0x4e2`. This crate takes the step instead.
Which unit the probe finds there is **not** on the disk — the snap arm
clears the collider it names — so §4.2's two sweeps and §4.3's soft table
are candidates and nothing more; `docs/AI.md` §54.4 names the call-proxy
capture that would separate them.

## 9. run116 — the sweep read from inside, and the word is one clause (2026-09-21)

§8.9's last paragraph left Great Lakes' word at 10161 with its arm
unnamed, and named the capture that would settle it. `docs/AI.md` §54.4
wrote the falsifier before the run. This is that capture, and the
falsifier held on its first pass.

**Why no dump could answer it.** `detect_unit_collision` writes
`collide_o`/`collide_who` and §5.4's snap arm clears them **two
instructions after the probe returns**, so the record on disk says a
refusal happened and nothing whatever about which arm made it. That is
not a detail level that was not raised; it is a field that does not
survive the frame. The instrument had to be the trace's call proxies.

### 9.1 The instrument: `RON_COLLIDE_PROBE`

Five sites, ids 8–12, and each is there for a reason the one before it
could not cover:

| site | what it gives |
| --- | --- |
| `Unit::detect_unit_collision@00617060` | the **bracket** — everything nested in it is one unit's probe of one point — and the refusal itself, as the return |
| `CollCheck::collide_here@00682540` | §4.2's probe. Its first two arguments are the **asker's own `o`/`who`**, which is the only place in the sweep a record says who is asking |
| `UnitData::will_be_corner@00609fa0` | called once, immediately after a successful probe, so **its presence is `collide_here != 0`** — and its first two arguments are the **hit cell as values**, which `collide_here` carries only behind out pointers |
| `UnitData::is_here@0060a0c0` | one call per candidate the 3×3 walk reaches, `this` the candidate: the census of *who was looked at*, and the answer is who covers the hit cell |
| `UnitData::is_corner@0060a040` | the blocker's half of the corner rule, reached **only** when `will_be_corner` was non-zero and every soft arm declined — so its absence is as informative as its presence |

Three of the five are `__thiscall` on a `UnitData *` and their arguments
name no object at all, so the build adds one record of its own: **INFO
15**, `(UnitData *, o, who)`, read off `+0xa` (a short) and `+0x9` (a
byte) — the pair `will_be_corner@00609fa0` itself indexes
`units[who][o]` with, so the offsets are the function's own and not a
guess. It is what lets a log full of heap addresses be put beside a dump
keyed on `(who, o)`; `Trace::unit_of` and `report.py … calls` both read
it, and `report.py` prints `this=1/38` rather than `this=0x320f35c`.

Argument counts are each function's own `ret <imm>` divided by four,
read off the image: `0x1c`, `0x20`, `0x10`, `8`, `0xc`.

**The proxies perturbed nothing**, which is the one thing a probe build
owes (`docs/AI.md` §52.1): `rngcmp.py` against run53 is 0 differing and
10,186 identical, and `samegame.py` against run100 — whose window this
one lies inside, at the same detail, so no `--exclude` — is 15 blocks in
common and 0 differing.

### 9.2 The word, step by step, both sides

`1/38` proposes `(42774, 22584)` on sim-frame 10161, from unit cell
`(890, 470)` onto `(891, 470)`. One bracket, `quick 0`, and it returns
**1**.

| step | the original (run116) | this crate |
| --- | --- | --- |
| proposed cell | `(891, 470)` | `(891, 470)` |
| `collide_here(o 38, who 1)` | 1 | hit |
| the hit cell | **`(892, 471)`** | **`(892, 471)`** |
| `will_be_corner` | **5** (SE) | **5** |
| the 3×3 walk, in order | `1/39`, `1/32`, `1/31` | `1/39`, `1/32`, `1/31`, … |
| `is_here` | 0, 0, **1** | false, false, **true** |
| §4.3's exemption ladder | **declined** | **soft** |
| `is_corner` | **0** — asked | not asked |
| verdict | `|5 − 0| ≠ 4` → **hard**, step refused | soft, `half_step`, step taken |

**Six of the eight rows agree**, including the two the widening could
not see: the probe's own cell and the walk's order. `1/38` steps one
cell east with `dy 0`, so §4.2's **fast path** is what ran — the column
`x = 892`, its two parity rows `y = 469` then `471`, the first occupied
one winning — and both sides walked it and stopped on the same cell.
The corner rule agrees too, on the half that was asked.

**So the whole word is §4.3's soft table, and one clause of it.**

A note on how nearly this was mis-attributed. Read at the *start* of
frame 10161 this crate's probe answers `(892, 469)` and not `(892,
471)`, because `1/33` — which steps earlier in the same frame — vacates
unit cell `(893, 469)` during it and `move_unit`'s clear pass takes the
column `x = 892` with it. A sweep compared at a frame boundary and a
sweep compared where the original's bracket actually sits are different
measurements, and the first one says "the occupancy index". **The probe
must be read inside the frame, at the asking unit's own step.**

### 9.3 The clause, and how the other three were eliminated

§4.3's group arm is four clauses and the collision was hard, so exactly
one of them declined:

- **the group** — both carry `group 64`, and `≠ −1`. The dump prints it.
- **`local_24 == 0`, my action is not `ATTACK`.** `1/38`'s order is a
  `GROUPATTACKTOORDER`, and the dump prints its `flags 5`.
  `UnitData::get_action@00608450` walks the order list and returns the
  first order that is not moving **or** carries `flags & 4` — 5 is
  `1 | 4` — so the action *is* that order, `action_type` is **21** and
  not `ATTACK`'s 10. The clause passes. (This crate reads 21 too.)
- **the order-kind test.** `1/31`'s order is an `ATTACKTOORDER`, type
  **2**, which is one of §4.3's gated seven and needs the collider's
  *action* type ≠ 10. Its order carries `flags 5` as well, so its action
  is that same order and `local_28` is **2**. The clause passes.
- **`+0x104 == 0`.** Everything else having passed, this is the one that
  declined — and `+0x104` is **`openlist`**, `Tree<PathNode *, int> *`
  in the PDB's own `UnitData`: the head of a **suspended 48-grid
  search**.

**And the trace says so directly, on the same frame.**
`PathFinder::astar_path@00683770` is one of the eight base proxies, so
every build records it. On sim-frame 10161 two searches return **−1** —
the suspend — and both resume and answer 1 on 10162. Their `stack`
argument is a `Stack<PathData> *`, which is `UnitData +0xb8`; subtract
the offset and the INFO 15 table names the units: **`1/31` and
`1/32`**. `1/31` is the blocker.

Three independent witnesses, and they are three different instruments:

- the **trace**, `astar_path(1/31's stack) = −1` on 10161;
- the **dump**, `1/31 start_dist 1872` on block 10162 — §18.6's "one
  dumped witness that a search suspended", and this crate writes the
  same 1872;
- **this crate's own state**, `Unit::search.is_some()` on 1/31 at the
  moment `1/38` probes it.

So the field was never missing. `Unit::search` **is** `+0x104..0x148`
and has been since `docs/PATHFINDER.md` §18; the *seam comment* in
`Sim::same_group_soft` outlived the field by a fortnight, and reading
the clause as zero widened the arm by exactly the units that are busy
re-planning — which, in a squad walking into each other, is most of
them.

### 9.4 The fix, and what it moved

One clause: the arm declines when the **collider** holds a suspended
search. It is the collider's alone — a suspended search of the asker's
is not in the predicate.

| | base | now |
| --- | --- | --- |
| Great Lakes long word (run53) | 10,161 | **10,232** |
| East Indies long word (run54) | 9,711 | 9,711 |
| the word's own block, 10162 | 15 rows on `1/38` | **0** |
| run53's endpoint at 24001, `off` | 53 | **48** |
| — `unlinked` | 8 | **13** |

The new word is `Unit::do_move+0xe84` here against
`Guy::set_anim+0x97a < Guy::inc_time+0x1ed` there, 99 draws against 95,
parting at draw 92. **Its value diff is 53 rows on four units** on block
10233: the human's `0/5 orders.len: ours 1 theirs 0` — an order the
original drops and this crate still holds — and a three-unit squad,
`1/27`, `1/28` and `1/29`, whose positions, headings, tolerances and
path stacks part together, ours carrying a 44-entry plan where the
original carries one hop. That is a formation-pathing question and not
this document's.

The endpoint's five extra unlinked units are 13,769 frames past the
word, on a clause that fires wherever a squad walks into itself while
one of its members is re-planning; `docs/DECISIONS.md` 36 asks for the
number rather than a trade, and it is on the row.

### 9.5 What this has *not* established

- **The other three clauses are still readings.** This capture exercised
  them — they all passed — but a clause that passes is not a clause that
  was tested. A capture in which a group-mate's action *is* `ATTACK`, or
  whose order is one of the gated seven with an attacking action, is
  what would put teeth in the other three.
- **`is_here`'s own arguments are not read.** They are pointers to the
  hit cell, so the record names the candidate and its answer and not the
  point; the point comes from `will_be_corner` beside it. A candidate
  examined on a frame where `will_be_corner` was not called would
  therefore have no cell on the record.
- **`0x12` (`CHANGE_FORM`) is still an order this crate does not have**,
  so the gated set is six of the seven here.
- **Whether the arm is now *right*, or only right here.** One clause
  moved one word. Nothing in this capture says the remaining seam list
  in `soft_collision` — the `TRADE_ROUTE`/`0xf` and `0xc` arms, which
  need action indices this crate does not carry — is empty of the same
  kind of defect; no capture has entered either. **It was not**: the arm
  was reading `army_of` where the original reads `UnitData +0x80`, so
  every member of a *pushed* group was hard to every other, and that was
  the next word but one (§11).
- **The window is six frames.** `callwin=10158-10163` is what the
  brief booked, so the sweep is read forwards on six frames of one game
  and nowhere else.

### 9.6 Coverage

Diff-backed. `run116_s_sweep_is_the_original_s` puts this crate's
[`Sim::sweep_verdict`] beside run116's own record for the word's
bracket — the proposed cell, the hit cell, `will_be_corner`, the
candidate walk in order with each `is_here`, and the blocker's
`is_corner` — and asserts the refusal itself. It reads the trace's site
table rather than assuming the ids, so a log some other probe build
wrote cannot pass it.

`Sim::sweep_verdict` is not a second implementation: it calls the same
`Sim::scan_colliders` `detect_unit_collision` calls, with a recording
sink where the simulation passes an empty one. A diagnostic that drifts
from the code it diagnoses is worse than none.

`Candidate::is_corner` is an `Option` for the same reason the site is
proxied at all: **`Some(0)` and `None` are different facts** — asked and
answered nought, against never asked because an earlier gate answered
first — and §9.5's first bullet is about exactly that distinction.

Made to fail on purpose before landing: restoring the seam — dropping
the `search.is_some()` clause — turns the word's row from
`1/31=1,corner 0 hard 1/31` into `1/31=1,soft … soft` with nine more
candidates walked, and the word from 10,232 back to 10,161. Both were
run.

Nothing in §9.2 rests on a reading. §9.3's elimination of the other
three clauses rests on `get_action@00608450` read against the dump's own
`flags 5`, and the clause that did decline is diff-backed three ways.

## 10. What is not established

- **The snap arm's `invalid_loc` refusal** (§5.4). The same `if` has a
  *free* sub-arm, and its tile refusal at `005fb443` does **not** behave
  like §5's: it jumps to `005fb48e`, which is the accepted step's own
  `dest = 0` and pop, so the waypoint is consumed without the step and
  `unit_masks & 8` is left standing. §5's refusal (`005fb7c1`–`005fb7fd`)
  clears the bit and answers 0. This crate has only §5's, for both arms —
  item 360 modelled the snap's *collision* sub-arm and left this one
  alone, because no frame on disk demands it. *What would refuse it:* a
  `UNITS=3` window over a unit whose last hop into a waypoint crosses into
  a tile `invalid_loc` rejects — the record would show the original's
  `dest_x`/`dest_y` and path stack moving on where this crate's stand.
- **`Movement::dest` on a blocked snap.** This crate sets it to `None`
  there, matching what the accepted step's arrival tail does, because the
  waypoint has been consumed. The original has no such field — it is a
  sim-side proxy for "is this unit going somewhere", read by
  `docs/COMBAT.md`'s target tests — so the choice is an inference from the
  control flow rather than from a dump. *What would refuse it:* a capture
  in which a unit blocked on its snap is the target of an attack decision
  on the same frame.
- ~~**East Indies' long word, 7806, is a collision question and the frame
  it turns on is undumped.**~~ ~~*Capture:* `[7790, 7900)` at run88's
  detail exactly.~~ **Taken, as run90, and it refused the candidate**
  (§8.6, 2026-09-07): the position parts on block **7805**, two below the
  word, because the original **abandons** §6 step 4's sidestep waypoint
  after one step where this crate walks to it — the same point on both
  sides, a four-block cycle against this crate's five. The turn-versus-step
  reading of §8.5 is superseded and kept for the method. What remains open
  is the *arrival rule itself*: which of `move_step`'s tests drops a
  `flags 2` waypoint one step in, and whether the drop is unconditional or
  ~~a tolerance this crate has too small. That is a reading of `move_step`'s
  waypoint block, not a run — every frame of it is now dumped.~~
  **Read and landed as §8.7** (item 289, 2026-09-07): it is `move_step`'s
  ordinary post-step arrival test, and the drop is neither unconditional
  nor a tolerance too small — the tolerance is the *interrupted leg's*,
  because `resolve_unit_collision` never writes `UnitData::tolerance` when
  it pushes the waypoint. East Indies' word 7806 → **7812**.

- ~~**§6 step 5's wait flag is taken where the original rolls a pause**~~,
  ~~and since 289 it is the whole of East Indies' word.~~ **Settled by
  §8.8** (item 294, 2026-09-07), and the predicate was never wrong: the
  question the row itself asked second — "why does this crate not collide
  on 7809 at all" — is the whole of it. `move_step`'s two turn-in-place
  arms `return 1` before the collision block (`005fb2b4`, `005fb2e1`), so a
  turning unit never probes; this crate probed, and
  `detect_unit_collision`'s unconditional clearing threw away the collider
  `do_move`'s waypoint probe had named in the same frame. With the return
  in, `1/7` rolls the original's `pause 8` on 7811 off the shared stream and
  the window's collision divergence is **empty for the whole cast**. The
  word does not move; what 7812 now sits on is `find_upath`'s **suspend**,
  which is `docs/PATHFINDER.md`'s row and not this one.

- ~~**Great Lakes' long word is a `SITE_BLOCKED` this crate spends and the
  original does not.**~~ **Settled by run76** (item 236, 2026-09-06), and
  the answer was not in this block at all: on 6861 the original
  **ungroups the squad**, and the stand this crate spent on 6862 is the
  stand of a follower still walking a formation the original had already
  dissolved. §5.3 has the rule and the record that names it. The reading
  the row asked for — the group's `speed`/`new_speed` pair, the slot
  table — was not needed: the *dump* said it, because the order kind, the
  `PATHED` bit, the emptied path stack and the nine-entry world-grid plan
  are all printed and only the position had ever been compared. Great
  Lakes 6862 → **6982**, and the new word is
  `Leader::produce_building+0x1805` against `Leader::make_stuff+0x221` —
  an AI purchase, not a movement question.

- ~~**East Indies 7448 is still open, and it is a *position* and not a
  predicate**~~ **Settled as a collision question by run85** (item 241,
  2026-09-06): it *is* a position, and §8.2 has the record. The two
  `SITE_BLOCKED` are one Merchant's two figures, the blocker is gaia's
  animal `8/0` and nothing filters gaia out of a step; what is missing is
  the walker, 792 units and about thirty-four frames short of the meeting,
  and the frames are spent on an un-captured barge ride between 6929 and
  7400. The row that remains is `docs/TRANSPORT.md`'s, not this
  document's. The reading below is superseded — four of its five units
  were named off the trace and the window says only three units on the
  map are off position at all — and is kept for the method.

  Two `SITE_BLOCKED` the
  original spends and this crate does not. Five units are moving there —
  `0/3`, `1/0`, `1/10`, `1/18`, `1/20`, none of them within a thousand
  units of another — and **four of the five propose a point inside their
  own unit cell**, where §4.1's gate 6 refuses the test before it starts.
  Only `1/0`'s step and `1/10`'s `do_move` waypoint probe cross a
  boundary at all, and both miss. So for the original to spend two
  blocked stands on that frame at least two of its units must be crossing
  a cell this crate's are not: a position, and not a predicate. (`1/18`
  lands on `x = 36480 = 760 × 48`, the boundary exactly, and crosses on
  7449.)

  **And it is not the *response* either** (item 239, 2026-09-06). run83
  puts a blocked unit's whole recovery under a diff — step 6's snap, the
  `find_upath` plan it builds and the six frames of walk that come out of
  it, every point the original's — so "this crate answers a block
  differently" is no longer a candidate cause for this word, on either
  map. What remains unwitnessed is the frame itself: **no East Indies
  capture on disk reaches 7448** (the map's windows are 5150–5400,
  6730–6800, 6860–6930, 10150–10400 and 15700–15900), so the five units
  above are a reading of the trace's draw sites and not a record.
  ~~*Capture:* `[7420, 7480)` at run39's detail~~ — **taken**, as run85's
  `[7400, 7480)`, and it answered: a position (§8.2).

- **`ObjectType +0x2b4 & 0x2000`** — the "attack what you bump into" bit.
  Read as a flag, not traced to its XML column.
- **`+0x10c`, `+0x74`, `+0xf4`** — the virtuals §4.1 step 3 and §6 step 3
  call. Named by slot, not by identity. ~~`+0x94`~~ is settled: it is
  `BuildTypeData::is_flat` (§6 step 2, item 64), and reading it as `true`
  cost the score for a week.
- **The action indices `0xc` and `0xf`** in §4.3. `0xf` sits next to
  `TRADE_ROUTE` in the same arm, so one of the two is the caravan's.
- **`WData::block == −1`.** Where the sentinel is written is unread; this
  crate marks every cell. *Capture:* a `WORLD ≥ 6` dump does not print it,
  so this needs a reading, not a run.
- **Squads.** `Unit::set_new_location` spreads figures `0 .. guy_mark` over
  a formation and each marks its own disc. *Capture:* `UNITS=3` +
  `GUYS=2` over a four-figure squad walking into another unit.
- ~~**`coll_size ≥ 2`.**~~ **Settled by run66** (item 183): the Merchant's
  `BLOCK_RADIUS 2` block is the first a capture has ever collided, and
  §4.2's fast path was found by it. What is still owed is the *paint* of
  a block that spans two `CollBlock`s on both axes, which a `coll_size 3`
  siege unit would give.
- **The crew figure's draw on a blocked stand, and its walk on the same
  frame** — the residue run66 leaves and East Indies' word since. On
  sim-frame 6571 `move_step`'s `set_anim(CHAR_DEFAULT, 0, 1)` rolls once
  in the original (`Unit::set_anim+0x56`, the first loop, guy 0) and
  twice here; and the original's crew figure then *walks* — `1/19`'s
  `g1` goes `(34442, 37613)` → `(34464, 37595)` on the frame the
  resolve snaps its leader — where this crate's stands still and moves
  a frame late. Both halves are one question about `Guy::set_anim`'s
  walking-guy early return and `Follow::des`
  (`docs/ANIM.md` §4 step 1, `docs/MOVEMENT.md`, "Who writes it, and
  when"), and run66 has both guys' positions on all 261 blocks.
- ~~**The `pause` draw.**~~ **Settled by a run** (item 80, 2026-08-30):
  run33's frame 571 spends it, and §8 has the frame. ~~What the capture
  named in this row would still add is the *value* — no dump in hand
  prints a non-zero `MOVEORDER pause`.~~ ~~*Capture, still owed:*
  `UNITS=3`, two units of the same player ordered into each other
  head-on, for the `MOVEORDER` row.~~ **run90 prints it** (§8.6,
  2026-09-07), and it is that capture in all but the wording — two of the
  AI's own citizens walking into each other at `UNITS=3`. `1/7` rolls
  **3** on block 7803 and **8** on 7811, both inside `% 9 + 1`'s range and
  the first non-zero `MOVEORDER pause` on this disk; the countdown is
  **frozen while `collide` is set** and only ticks once it clears. Two
  samples do not pin the modulus, so the `9` itself still rests on the
  listing.
- **Whether `collide_guy` is ever non-zero.** Every hard collision this
  reading found writes 0; the field exists, so something writes it.
- **`unit_masks & 0x4000000`, which §6 step 0 clears.** Its one writer is
  `add_move_facing_order@005e55c0:28` — a `QUEUE_LAST` move whose type
  carries `+0x2c8 & 0x10`, which the same line turns into an
  `EXPLORE_TO` — and the `QUEUE_NEW` clear at `:58` drops it again; step
  0 is that clear, five lines for five lines. Its two readers are
  `Unit::work@0060d180:124` and `Unit::think@005f6e40:233`, neither of
  which an animal runs (`Animal` overrides both slots), so what the bit
  *means* to a unit is unread here and this crate does not carry it.
  Nothing about step 0 turns on it. *What would settle it:* the two
  readers, which is a reading rather than a run.
- ~~**§4.3's group arm.**~~ **Landed 2026-09-04**: an army's members are
  its group (`docs/GROUPS.md` §1), which is the `UnitData::group` this row
  wanted, and `Sim::same_group_soft` is the arm. It is what keeps a
  marching squad from standing blocked on its own leader
  (`docs/ORDERS.md` §8.6). `UnitData +0x104`, the suspended pathfinder
  search the arm also tests, is still not carried and reads as zero, which
  widens it.
- ~~**§5.2's group arm.** `find_ordered_collision`'s second pass tests
  every other member of my group against its ordered position, wherever
  it stands, and this crate keeps no `UnitData::group` back-pointer to
  reach it with. Every unit in every capture so far dumps `group -1`.
  *Capture:* `UNITS=3` + `GROUPS=1`, a selected group ordered to build or
  gather at one site while its members are scattered — fold into the
  item-23 run.~~ **Landed 2026-09-17** (item 328), and no capture was
  needed: `docs/COMBAT.md` §17's probe pushes a real group (run19's block
  8187 dumps `group 65` on the six), and run19's own six destinations are
  the oracle. **The seam's premise expired without anyone editing the
  seam** — "every unit in every capture so far dumps `group -1`" stayed
  true-looking for a month after it stopped being a reason. Worth reading
  as a class: a seam whose justification is "no capture reaches this yet"
  is a claim with a shelf life, and nothing re-checks it.
- **§5.2's general path.** `FILTER_ALL` and a squad placement use
  `ObjectsData::find_unit_with_radius` and its ordered sibling instead of
  the pairwise pair: a `vector_dist` circle of `other.big_radius +
  r_coll`, where `r_coll` is the type's `block_radius` — bumped to
  `0x180` when that is 0 and the filter is `FILTER_ALL`. The `search`
  argument the call passes is a **live register the decompiler loses**
  (`0x61e375`, `push ecx` where `ecx` last held a terrain word), so which
  players it searches is not settled and the listing does not settle it
  either. The only call site that reaches it is `come_out`'s
  `block_radius == 0` arm, and no shipped type any capture trains has a
  zero `block_radius` — a Citizen's is 1. *Capture:* a scenario that
  trains one of the ten `BLOCK_RADIUS 0` types beside a crowded trainer.

## 11. The soft arm asks the army where the original asks the group (item 489, 2026-09-22)

`docs/ORDERS.md` §20 left Great Lakes' word at **10277** — seven draws
against six, parting at index 4 on `Guy::set_anim+0x97a <
Unit::move_step+0x823`, the blocked stand — and block 10278 holding
**seventeen** rows, sixteen of them `1/40`'s and one `1/41`'s. `1/40`
collides with `1/41` here (`collide_o 41`, `collide_who 1`) and stands on
an animation one frame old where the original's is ten frames into a walk.

The seventeenth row was written as a row and not a mechanism: the original
sets `unit_masks & 0x100000` — §4.3's soft one-shot, compared as
`half_step` — on **both** units on this block, and this crate on neither.
It turned out to be the *same event read from the other side*. A frame
that raises the soft flag is a frame on which nothing hard-collided, so
the two rows were never independent, and neither was `1/41`'s.

### 11.1 What the block actually says, re-measured

`run100_s_word_block_is_every_record_the_dump_carries` over
`[9340, 10290]`, every record of every unit, both directions: **313 keys
parted**, seventeen of them first parting on 10278 and none of the other
102 units, 28 buildings or 3 cities disagreeing on any field.

**Both `half_step` rows are readable at the parting**, which §8.4's
position gate is the reason to check: `1/41 pos` first parts on **10279**,
so on 10278 `1/41` stands exactly where the original's does and its
`half_step` row is a clean measurement rather than a field read off a unit
already somewhere else. `1/40 pos` first parts *on* 10278 — it agreed on
10277 — so its own row is the frame the position parts, which is the frame
a word is read on.

**And the dump says the one-shot is a half step, which this document had
only from `docs/MOVEMENT.md`.** `unit_masks` over `[10265, 10290]`, from
`tools/gamelog/track.py`:

| unit | blocks with `0x100000` | `collide` / `collide_o` on all of them |
| --- | --- | --- |
| `1/40` | 10278 | `0` / `−1` |
| `1/41` | 10278, 10281, 10283 | `0` / `−1` |

The baseline is `331784` and the flagged blocks are `1380360`, a
difference of exactly `0x100000`; it is never set on two consecutive
blocks. The step **after** each one is halved — `1/41`'s `y_internal`
goes 31254 → 31266 on 10279, 31316 → 31328 on 10282 and 31353 → 31365 on
10284, twelve against the twenty-five it takes on every other frame of the
window, and `1/40`'s 31293 → 31305 the same. So the original never
hard-collides in this window at all: it raises the flag and walks on.

### 11.2 The sweep, read from inside the frame

§9.1's instrument answers what no dump can, because
`detect_unit_collision` writes `collide_o`/`collide_who` and §5.4's snap
arm clears them two instructions later. [`Sim::sweep_watch`] records
[`Sim::sweep_verdict`] **where the original's own bracket sits** — at the
asking unit's own step, not at the frame boundary, which §9.2's note is
about. On sim-frame 10278, before the change:

| | `1/40` | `1/41` |
| --- | --- | --- |
| stands at | `(2448, 31268)`, cell `(51, 651)` | `(2342, 31229)`, cell `(48, 650)` |
| proposes | `(2444, 31293)`, cell `(50, 651)` | `(2344, 31254)`, cell `(48, 651)` |
| `collide_here` | hit `(49, 650)` | **clear** |
| `will_be_corner` | 1 (NW) | — |
| the 3×3 walk | `1/41` = 1 | — |
| §4.3's ladder | **declined** | — |
| `is_corner` | 0 — asked | — |
| verdict | `\|1 − 0\| ≠ 4` → **hard**, step refused | nothing |

**The two rows are one cause and the arrow runs from `1/40` to `1/41`.**
`1/41` finds nothing to be soft about only because `1/40` stood: with
`1/40` refusing its step it stays in cell `(51, 651)`, whose block is
`(50..52, 650..652)`, and `1/41`'s own probe — §4.2's fast path, `dx 0`
`dy 1`, so the leading row `y = 652` at `x = 47` then `49` — sweeps cells
`1/40` no longer covers. With the change, the same instrument puts `1/40`
at `(2444, 31293)` = cell `(50, 651)`, `1/41`'s probe hits `(49, 652)`,
`will_be_corner` 5, and the walk names `1/40`, soft. Both raise the flag,
which is the block.

### 11.3 The predicate is the group slot, and this crate asked the army

`detect_unit_collision@00617060:307` opens §4.3's group arm with

```c
if ((((*(short *)&this->field_0x80 == *(short *)(iVar7 + 0x80)) &&
     (*(short *)&this->field_0x80 != -1)) && (local_24 == 0)) &&
   (*(int *)(iVar7 + 0x104) == 0)) {
```

— one `short` at `+0x80`, compared for equality and against −1. It is the
`Groups::list` slot the unit's back-pointer names (`docs/GROUPS.md` §1),
and it knows nothing whatever about armies.

The dump prints it. `1/27`, `1/28`, `1/29`, `1/40`, `1/41` and `1/42` are
**`group 65` on every one of the 951 blocks** of `[9340, 10290]`,
unchanging. `Sim::same_group_soft` asked `Sim::army_of`, which answers
`None` for all six on 10278 — because a group has **two seats** here, an
army's and a `group::Pushed` pool slot, and `Groups::push_group` takes its
members out of the army when it installs one (§3.2). Item 465 gave this
probe's six raiders a real `GROUP_MOVE` in a pushed slot, and from that
day every member of a pushed group was hard to every other member.

[`Sim::group_of`] — "the original's `unit +0x80` read straight into the
pool" — is the resolver item 465 built and item 470 measured, and
`Objects::find_ordered_collision`'s second pass has used it since. The
soft arm is the site that was never moved over to it. The change is one
predicate: the **seat pair** rather than the army slot, which is what one
`+0x80` means.

### 11.4 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word, **count** | 10277 | **10294** |
| Great Lakes long word, **sequence** | 10277 | **10294** |
| block 10278, keys parted | 17 | **0** |
| run100 widening over `[9340, 10290]`, keys parted | 313 | **251** |
| Great Lakes endpoint `off` / `unlinked` / `build_diverged` | 42 / 7 / 9 | **50** / **8** / **8** |

**The value diff beside the move.** Block 10278 is empty, and pinned so in
`run100_s_word_block_is_every_record_the_dump_carries` beside 10162,
10234, 10235, 10238 and 10245. `1/40` and `1/41` have **no row of any
kind** anywhere in `[10270, 10307]` now — position, guy clock, order or
collision — where before they had twenty-nine between them.

The widening's own window moved with the word, 10290 → 10307, which is
parked 449's lesson applied at the move; over the wider window the count
is 320, and the rise is seventeen blocks nobody had compared, not the
simulation. The like-for-like number is the 313 → 251 above.

**The endpoint is a trade and DECISIONS 36 asks for the number.** Eight
positions out and one more of the roster unlinked at frame 24,001, 13,707
frames past a word that moved 17 — so on the far side of an unaligned
draw stream, where every later random answer is a coin flip. The eight are
not a roster coming apart: `1/24`, `1/25` and `1/26` are 24 out on both
axes and `1/40`, `1/41`, `1/42` some 6,400 south, a squad that walked a
different route.

**The new word, 10294**, is not a collision and not the AI's. Ours spends
**one** draw and the original **two**, parting at index 0: the extra is
`Guy::set_anim+0x97a < Unit::do_idle+0x7d`. Block 10295 names the unit
without being asked — six rows, all of them the human's citizen `0/5`:
`orders_x`/`orders_y` ours `(4056, 28776)` against `(792, 31800)`,
`idle` ours 0 theirs 1, and the facing and the stand that follow from it.

### 11.5 Coverage

**Diff-backed**: every row of §11.4, from
`run100_s_word_block_is_every_record_the_dump_carries` over
`[9340, 10307]` (969 blocks, 2,533,186 record rows) and
`run100_s_word_frame_is_the_original_s`, whose word-frame set is now the
**empty** one. The word itself is
`run53_s_24000_frames_put_the_ceiling_where_run33_did` and the endpoint
`great_lakes_endpoint_is_pinned`; all four were red before the change and
are the numbers above after it.

**Dump-backed**: §11.1's `unit_masks` table and the halved steps beside
it, read straight out of run100. This is the first behavioural
confirmation that `0x100000` is a *half step* rather than merely a flag —
§4.3 had it from `docs/MOVEMENT.md`'s reading alone.

**Listing-backed**: §11.3's predicate, read from the decompile export at
`detect_unit_collision@00617060:307`. The field is a `short` there, which
is what makes it a slot id and not a pointer.

**Unit-tested, made to fail on purpose**:
`two_members_of_a_pushed_group_pass_through_each_other` — with `group_of`
put back to `army_of` the second assertion collides, and the word returns
from 10294 to 10277 with block 10278's seventeen rows back on it. Both
were run.

**What this does not establish.**

- **`1/41`'s 10281 and 10283 flags are not measured as such.** They agree
  because `half_step` agrees on every block of the window after the
  change, but nothing here reads the sweep at those two frames; only
  10278's was read from inside.
- **The seat pair stands in for a slot id.** A group has one seat by
  construction — `push_group` unseats it from every other — so equality of
  `(army, pushed)` is equality of `+0x80` *here*. Nothing compares this
  crate's seat against the dump's `group` number directly, and `group` is
  a `UNITDATA` field no comparator reads.
- **The other three clauses of the arm are still where §9.5 left them.**
  This capture exercised `+0x104 == 0` and the order-kind test and both
  passed; a clause that passes is not a clause that was tested.
- **Why `1/40` steps before `1/41`.** The order the two are walked in is
  what makes `1/41`'s row a consequence rather than a cause, and it is
  read off this crate's own recorder. No trace covers frame 10278, so the
  original's own ordering on this frame is inferred from the outcome
  agreeing, not observed.

## 12. The one-shot is the walk's, not the candidate's — Great Lakes 11806 → 11903 (item 560, 2026-09-22)

Great Lakes' long word stood at 11806 after item 557: ours 7 draws against
the original's 6, parting at index 1, ours
`Guy::set_anim+0x97a < Unit::move_step+0x823` against the original's
`Unit::resolve_unit_collision+0xb52`. No dump reached it; run130 ends on
11799. No mechanism was named.

### 12.1 run135, and the readings

`docs/RUNS.md`, run135: run134's line over `[11760, 11859]`, 221,771,936
bytes, all six checks green, with `cover=1` over 11796–11816. This crate's
side, before the run: on 11806 ours spends `move_step+0x823` from `1/31`
and then from `1/37` (both army 1, marching east through army 2), and
then `1/64`'s pause roll. The original spends one stop, then the roll.
The stanza wrote four readings first (`tools/gamelog/captures.txt`,
run135): `1/31`'s stop is the extra (R1), `1/37`'s is (R2), a positional
lag (R3), or `1/31`'s group move (R4).

**R2 holds, and the rest are dead.** `run135_s_word_frame_is_widened_whole`
walks run123 → run125 → run130 → run135 from 11400: every unit record, both
leader records, and on run135's blocks every player-1 pool list
(`GROUPDATA` members against `Sim::pool_list`). `1/31` stops on block
11807 on both sides (R1 and R4 dead). Nothing of `1/37` parts under 11805
but (558)'s group-order ids (R3 dead for the word). The word's blocks
part on `1/37` alone, and its first row is block 11805's `half_step`,
ours 1 against theirs 0.

### 12.2 What the dump says

`tools/gamelog/track.py UNITDATA … --where who=1,o=37` over run135:

| block | original's `1/37` | this crate's |
| --- | --- | --- |
| 11803–11805 | (40186, 20962), `collide` 4/5/6, `collide_o 64`, stopped | the same, **and `0x100000` on 11805** |
| 11806 | (40186, 20962), `collide 7`, `collide_o 64`, stopped | (40194, 20971), `collide_o −1`, walking |
| 11807 | (40186, 20962), `collide 8`, stopped | stops: the extra `move_step+0x823` |
| 11810 | steps to (40204, 20980) | — |

`unit_masks` on the original's side is `262216` (`0x40048`) throughout
the wait: the bit is never set. The original's `1/37` hard-collides with
`1/64` on every frame from 11803 to 11809 and waits. This crate's
collided with `1/64` hard on frame 11804 too, `collide_o` agreeing, but
its sweep also passed a soft group-mate on the way. It kept the half step,
spent it on 11805 on a step `move_step` takes before it probes, and walked
off.

### 12.3 The mechanism

`detect_unit_collision@00617060`'s nine-cell walk, in the listing:

```text
6177df  incl %eax ; movl %eax,-0x2c(%ebp)   ; the cell counter
6177e3  cmpl $0x9,%eax ; jl 0x6172a0         ; next cell
6177ec  movl -0x4(%ebp),%eax ; testl ; je 0x6177fa
6177f3  orl $0x100000,0x68(%ebx)             ; the one-shot
6177fa  …collide_o = −1, collide_who = −1, age collide, clear 0x40; return 0
```

The set is reached only by falling out of the loop. A hard candidate
writes `collide_o`/`collide_who` and returns 1 from inside the loop
(the decompile's `return 1` arms), so the soft flag the walk had raised
in `local_8` is dropped. The one-shot belongs to a sweep that **ended
soft**, not to a sweep that **saw** a soft candidate. This crate's
`name_collider` set it on the second reading. It now sets it on the
first: `soft && hard.is_none()`.

§4.3's paragraph had the loop-exit right ("set once for the whole
nine-cell sweep") and its gloss ("whatever the scan found") wrong. It is
amended in place.

### 12.4 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word | 11806 | **11903** |
| run135 widening `[11400, 11859]`, keys parted | 635 | **284** |
| run135 widening, blocks 11805..11859 | 351 keys | **empty**, pinned |
| `1/37`'s stop, blocks 11806/11807 | ours alone | **neither side** |

**The value diff beside the move**: block 11807 and every block after it
to run135's last are pinned empty. The 284 that stand are run130's
standing floor under the word: 11400's residue, 11424's `come_out` push,
(558)'s ids, `1/66`'s form on 11778, 11782's make-list row, and
`1/34`/`1/36`'s one-unit slot points on 11799. None of them is on the
word's chain.

**The new word, 11903**: ours 5 draws against the original's 4, parting
at index 1. Ours spends `Guy::set_anim+0x97a < Unit::move_step+0x823`,
from `1/64`, and the original spends `Guy::set_anim+0x97a <
Guy::inc_time+0x271`, `0/0`'s wrap, which ours spends a draw later. That
is the measurement and not a mechanism (`docs/DECISIONS.md` 42). It is
past run135's last block (11859), so its widening is owed a capture.

### 12.5 What this has *not* established

- **Which soft candidate `1/37`'s sweep passed.** The widening says the
  bit was set and the listing says why it should not have been. Neither
  names the group-mate, because no `RON_COLLIDE_PROBE` build ran over
  11804. The fix does not depend on who it was.
- **The quick form.** `move_step`'s second call and `resolve`'s four
  return before the loop and never set the bit (§4.3). This crate's
  `detect_quick` never did either, and nothing here tested it.
- **The rest of run135 past the word** agrees by the widening's own
  count, but the window is 53 blocks past 11806. The draw stream past
  11903 is not compared by any dump.

### 12.6 Coverage

**Diff-backed**: §12.2's table and §12.4's rows, from
`run135_s_word_frame_is_widened_whole` (460 blocks, 1,560,361 record
rows, 968,760 leader rows, 240 pool lists), which pinned the word's 25
rows before the fix and pins the blocks from 11805 empty after it. The
word itself is `run53_s_24000_frames_put_the_ceiling_where_run33_did`.
**Listing-backed**: §12.3, `llvm-objdump` of `riseofnations.exe`
`0x6177b0..0x617830`. **Unit-tested, made to fail on purpose**:
`collide::tests::a_hard_hit_leaves_no_half_step_whatever_soft_it_passed`.
With `name_collider` setting the bit on any soft candidate, it fails on
the chain order that lists the group-mate first. That was run.

## 13. A ship never scans on the waypoint probe, and pushes its way through on the step — East Indies 10398 → 10582 (item 588, 2026-09-23)

East Indies' long word was **10398**. Ours spent 4 draws against the
original's 5, parting at index 0, where the original spends
`Guy::set_anim+0x97a < Unit::do_idle+0x7d`. The Bark `1/34` was trained
on 10323 and walked to the navy a step behind the original's from its
first step on 10325. It arrived on 10398 still moving here and idle
there. No mechanism was named. The walk back is in the item's journal.
The capture is run143 (`docs/RUNS.md`).

### 13.1 The frame

run99 alone places it on the Bark's **first step**, sim-frame 10324. The
original steps `last_speed` **41** there and this crate steps **20**, on
the same facing (908263424) toward the same first leg's end, (44568,
40824). After that both step 40 a frame, and the 21-unit gap stays to
the muster. A scratch probe put `half_step` true on entry to `move_step`,
raised the same frame by `do_move`'s **waypoint probe**. That probe ran
before the path existed, so its point was the order's own point, (45528,
42360), and its scan found Trireme `1/32` soft. `1/32` stands at its birth
point (45192, 41880), in the navy with the Bark, and its 336-unit block
reaches the muster's cells. The original's record on 10325 shows no half
step spent.

### 13.2 `detect_unit_collision`'s second arm

From the listing, `617070`–`617113`, and every caller's pushes:

- Air (`domain == 2`) is its own arm, `61707e`.
- With `top_only` (the seventh argument) zero, a unit takes the second
  arm when it is sea-domain, or its type answers `+0x10c`, or it answers
  `is_hero` (`+0xc4`, by default `ptype +0x2b8 & 0x20`), or it answers
  `is_supply` (`+0xcc`, by default `& 0x40`). The type record names
  `ObjectTypeData`'s slot `+0x10c` **`is_siege`**, and
  `UnitTypeData::is_siege@00470460` is `unit_flags & 0x20000`.
- In the arm, with `nocoll` (the sixth argument) zero, it returns at
  `61782b`, a bare `xor eax, eax; ret 0x1c`, in three cases: when
  `quick` is set; when `boats` is not; and when `boats` is set and
  `detect_boat_collision(x, y, 1)` answers non-zero. Otherwise it falls
  through to the scan. None of these returns clears `collide_o`, ages
  `collide` or clears the wait bit.

So the callers, for a unit that takes the arm:

| caller | `(quick, boats, _, nocoll, top_only)` | the arm |
| --- | --- | --- |
| `do_move`'s waypoint probe, `5f86f9`–`5f8707` | `(0, 0, 0, 0, 0)` | **returns 0, never scans** |
| `move_step`'s probe (both arms, one call here) | `(0, 1, 0, 0, 0)` | `detect_boat_collision` |
| `move_step`'s second call, `resolve`'s four, `find_merchant_spot`, `Animal::do_idle` | `(1, 1, 0, 0, 0)` | **returns 0** |
| `do_move`'s pending-search probe | `(1, 1, 0, 0, 1)` | skipped (`top_only`) |
| `PathFinder::valid_ucoord` | `(1, 1, ·, 1, 0)` | skipped (`nocoll`) |

### 13.3 `Unit::detect_boat_collision@005fa8b0`

Listing `5fa8b0`–`5faf24`. It returns 1, "handled", except where noted.

- A pusher of player 8 or above, or one with `push_size` 0, returns 1 at
  once.
- **The profile.** `push_size` (`UnitTypeData +0x2f8`) and `push_circles`
  (`+0x2fc`), by the type record. `UnitType::init@0061ab50:664`–`691`
  stores the circles clamped to `[1, 100]`, and the size clamped the same
  way but **replaced by `BLOCK_RADIUS` when there is one circle**, times
  48. All 364 types agree with run3's start dump
  (`typesdump::compare`). The unit is `push_circles` circles of radius
  `push_size / push_circles` along guy 0's facing, centred on the point.
  The spacing is `project(facing, r)`, with the angle in `ecx` and the
  length in `edx` (`5fa992`); the circles sit `2r` apart, starting
  `(n − 1)` spacings back.
- **The candidates.** `Objects::find_units(x, y, 0, −1, push_size, 0x200,
  FILTER_NOT_ME, o, who, …)` (`5fa935`–`5fa953`). A candidate must be of
  the pusher's domain and, for a non-land pusher, of a player below 8. It
  must not be a group-mate (same player, same `group`, not −1) when the
  third argument is set, unless the pusher's action is index 10. It must
  not be the unit the pusher already collided with this frame
  (`collide_o`/`collide_who`/`collide_frame`), and its own `push_size`
  must be non-zero.
- **The overlap.** Take `d` as the least distance over every pair of
  circle centres. The distance is `5fabf3`'s: the longer leg plus the
  shorter's square over twice the longer, unsigned, and past 60,000 on
  the shorter leg `(shorter + 2 × longer) / 2`. The overlap is
  `(r_other − d + r_mine) / 2`. Zero or less is no contact.
- **In contact**, it returns 0 when the pusher is a transport
  (`unit_flags & 0x10`) and the other has an attack, or when the other's
  player is not the pusher's and not a mutual ally (`LeaderData::who`,
  `diplos == 2` both ways) and is below 8. A land pusher also returns 0
  for a packer that is not packed or is unpacking, for `unit_masks &
  0x2000000`, and for a ~~cargo (`+0x110`, `is_cargo`)~~ **tank**: the
  `+0x110` there is on the *type's* vtable, `**(other+0x18) + 0x110`, and
  the PDB's `ObjectTypeData` method list names that slot `is_tank`
  (`UnitTypeData::is_tank@00470450`, `unit_flags & 0x80000`, the `FLAGS`
  letter `t`, "Unit is a tank"); the unit's own slot 272 is `is_cargo`,
  which is where the misreading came from (item 696).
- **The push.** It uses the bearing from the point to the other, clamped
  to at least 45° off the pusher's facing when the other is not moving
  (`UnitData::is_moving@00610af0`). A **moving** other within 45° of the
  facing returns 0. The distance is `min(overlap, 48)`. If
  `UnitData::invalid_loc` allows the new tile, the other is
  `set_new_location`'d there. With the third argument set and one circle
  on the other (or a land pusher), it also records its pusher in
  `collide_o`/`collide_who`, and when it has no orders it is `set_angle`d
  and its guy 0 turned. Its `collide_frame` is stamped whenever it is
  pushed.

**So a navy never blocks itself.** The Bark and `1/32` share the navy's
group, and a group-mate is skipped.

### 13.4 What it moved

This crate now takes the arm for a **sea** unit: `Sim::takes_boat_arm`,
`Sim::detect_boat_collision`, the waypoint probe and the quick form.

- **East Indies' word 10398 → 10582.** The old word's blocks,
  10397..10399, are empty on run99 and run143. The Bark agrees on every
  row but its group number (68 against 66, the navy's numbering) from its
  birth on.
- On run99 the floor under the old word goes from 242 to 219 keys: the
  Bark's walk is gone. On run143, 284 keys stood under the old word before
  the fix and 285 stand under the new one after it. The window's first
  block goes from 269 to 254, and the whole window from 830 keys parted
  to 748.
- **The new word, 10582**: player 1's `make_stuff` buys differently. The
  make list parts on 10581, where the original holds a Citizen (type 50,
  `val 1714`) in slots 3 and 5 and this crate two Scholars at `val 0` in
  slot 3. Under it, the peasant census parts on 10576 (`free_peasants` 2
  against 1, `xport_peasants` 1 against 2, city `1/2000`'s `free`). On
  10583 `1/2018` is placed a tile off and `1/6` walks to it. That is a
  measurement, not a mechanism (`docs/DECISIONS.md` 42).

### 13.5 What this has *not* established

- ~~**The land half of the arm.** A siege engine, a hero or a supply wagon
  takes the same arm in the original. Its quick probes and waypoint probe
  never scan, and `detect_boat_collision` searches land units of every
  player, gaia included, and shoves them aside. This crate gives them the
  land scan as before. It is a stated seam: no diff has reached it, and
  Great Lakes' armies carry all three.~~ **Built** (item 696,
  `Sim::takes_boat_arm`): golden chapter eleven's supply wagon, on its
  first step under a player's move on run190's tick 721, pushes the guard
  standing on its post (21, 1) and names itself in the guard's
  `collide_o`. run191's brackets show it, `set_new_location(0/6)` nested
  in the wagon's `detect_unit_collision(…, boats 1)`. The land pusher's
  candidates include gaia, its refusals are an unpacked or unpacking
  packer, an entrenched unit and a tank, and it records itself on every
  unit it pushes. Chapter eleven 724 → 726.
- **The pushed unit's guy turn** (`Guy::turn_angles`, `Guy::do_turn` at
  `5faec8`/`5faedb`) is not modelled. Its `set_angle` is.
- **`find_units`' list path** indexes its cell grid with tile coordinates
  (`docs/ORDERS.md` §5.10), which is not reproduced, as in `build_crowd`.
- ~~**No capture has shown a push.**~~ run190 shows a land one (above),
  and its second, on tick 733, is the next word (`docs/GOLDEN.md` §19).
  Of the sea half: Run143's ships agree, but the only
  contact in it is between group-mates, which are skipped. The push
  arithmetic, the clamp and the refusals rest on the listing and a unit
  test.

### 13.6 Coverage

- **Diff-backed:** §13.1 and §13.4, by `run99_s_word_frame_is_widened_whole`
  (the old word's blocks empty, the floor 219) and
  `run143_s_word_frame_is_widened_whole` (10399 empty, the new word's make
  list, placement and counts). The word is
  `run54_s_24000_frames_are_where_the_second_map_s_word_now_parts`. The
  push profile is `run3_s_type_dump_is_the_program_s_on_every_check`, made
  to fail on purpose with `CIRCLE_RADIUS` (129 types off).
- **Listing-backed:** §13.2 (`617060`–`617113`, `5f86f9`–`5f8707`) and
  §13.3 (`5fa8b0`–`5faf24`).
- **Unit-tested, made to fail on purpose:**
  `collide::tests::a_ship_s_quick_probe_never_scans` fails with the arm
  off, and `a_ship_pushes_an_idle_stranger_and_passes_its_group_mate`
  fails without the group-mate skip. `a_boat_measures_with_the_listing_s_distance`
  pins the distance. `a_supply_wagon_pushes_a_standing_unit_and_not_a_tank`
  fails with the arm for a ship alone, and without the tank refusal.
- **Diff-backed, the land half:** `chapter_eleven_s_word_frame_is_widened_whole`
  (run190's 722 no longer parts), with run191's brackets as the call
  graph. The other land refusals rest on the listing.
- **Read from the decompile and listing only**, and owed a blind second
  reading: all of §13.3 but the group-mate skip, which run143 exercises,
  and the land push itself, which run190 does.

## 14. The enemy ladder's arm C: a captain bumped by another enemy strikes what it can reach — golden chapter eight 659 → 900, closed (item 668, 2026-09-23)

Chapter eight's word was **659**, 9 draws against 10 at draw 2, the
original's `Unit::fight+0x9b0`. The value diff was a frame earlier.
who=1's hoplite `1/6` chases the General `0/9` and is blocked by who=0's
`0/7` on 658, on both sides (`move_step+0x823`, §5). On block 659 the
original's stands where it was, (1780, 7844), with one order, an
`ATTACK` on `0/7` with flags 0. It has `collide_o 7` and `collide 0`.
This crate's had taken §6 step 6: `collide 1`, the snap to (1800,
7848), and its walk and its attack on `0/9` kept. The item's journal
has the widening read and the three readings each killed or kept.

### 14.1 The arm

§6 step 3's third arm, `Unit::resolve_unit_collision@005f9d30:189-252`,
read off the listing at `005f9ee0`–`005fa052`. It is reached when my
action is an attack, the collider is another player's, and arms A and B
of `docs/COMBAT.md` §48.3 have not returned. Then, when the action's
attack is not `mandatory` (`+0x1c`), the current order is not a group's
(vslot `+0x2c`), and `LeaderData::is_enemy(collide_who)`:

- **A follower** (`is_captain`, `+0x8e >> 15`, clear). Its captain's
  (vslot `+0xe4`) action is tested for `ATTACK` (`0x60a850`, `cmp eax,
  0xa`), then `update_action` runs on the captain. If the target exists
  and `is_in_range@00648d70` from where I stand (margin off, the same
  pushes as arm B), then `repath`, `kill_current_order(0)` and
  `add_attack_order(ox, whom, QUEUE_FIRST, 0, 0)`.
- **A captain, while `leaders[who].retargets < 10`** (`cmp dword
  [+0xe3ad84], 0xa; jge`, that is `leaders + 0x9f4`): `find_new_target
  (this, NULL, 1)` (pushes `1, 0`), then return.
- **A captain at ten or more.** Unless the type's `+0x10c` answers, and
  that is `is_siege` (§13.2), and the collider is a `valid_target`:
  `repath`, `kill_current_order`, and the collider queued `QUEUE_FIRST`.

Anything else falls through to step 4.

**`LeaderData +0x9f4` is `retargets`** in the type record.
`Leader::process@006b88b0` zeroes it at the head of each leader's frame.
`Leaders::process_all` runs before the objects (`Game::do_frame:199`
against `:275`). The only other writer is `Unit::fight@005fd4d0`'s
invalid-target tail, `LAB_005fdb9e`, which adds one after its search.
`Leader::init` and a `HotKeyGroup` constructor write the same offset of
other types. `LEADERS=5` does not print it. `LEADERS=9` prints it on the
start block, 0 for every leader.

**`find_new_target(this, NULL, 1)@005ff6a0`.** It runs `repath` and then
kills the current order (a group order's `kill_group_order` instead). So
the walk and the attack both go. With the third argument set it writes
**2, `STAND_GROUND`**, into the stance byte `+0xb1` across
`find_melee_target(-1, NULL, 0, 1, 0)` and restores it after. Inside,
`find_nearby_target`'s `local_24` then tests every candidate with
`is_in_range` from where the unit stands (`docs/COMBAT.md` §60.2). The
add reads the same stance, so it is `QUEUE_NEW`. The defensive-post arm
(`bVar3`) needs a `DEFENSIVE` stance and is not reached.

### 14.2 What moved

- **Chapter eight 659 → 900 of 901: closed.** On block 659 `1/6` is
  run171's field for field: it stands at (1780, 7844) with a lone attack
  on `0/7`. On 659 its re-search roll and the first blow land on the
  original's draws. On block 660 it stands snapped at (1800, 7848) with
  `recharging 32`, and `0/7`'s `damage_frame` reads 659. Every draw
  agrees to the trace's last frame, 900.
- `Sim::retargets`, its reset at the head of the leader loop, and the
  increment in `fight`'s invalid-target arm. `Sim::find_new_target` in
  the one shape above. Arm C is `Sim::enemy_ladder_arm_c`.
- `a_captain_bumped_by_another_enemy_strikes_what_it_can_reach` covers
  all three branches and the siege exemption. It was made to fail with
  the arm taken out: the walk is kept.

### 14.3 What is not established

- **Two rows stand under the closed word**, both on 660, and neither
  spends a draw in the capture. `0/7` takes 2 hits and `damage_frac` 5
  from `1/6`'s blow in run171 and 3 here. That is the General's rally
  armor, which this crate does not apply (parked 666). Both leaders'
  `treaties[·]` read 3 in run171 from the blow's block on and 0 here. The
  meeting on a first blow has no writer here (`docs/VISION.md` §6.2 has
  the met bit's one writer).
- **`fight`'s own budget arm**, `waiting < 5 && retargets > 10`, which
  returns without searching, is still not modelled
  (`crates/sim/src/orders.rs`). The counter it reads now is.
- The follower branch, the captain-at-ten branch and the siege exemption
  are reading only. run171 reaches the first captain branch alone.
- `find_new_target`'s defensive post, and its `GUARD`-activity and
  group-order add paths, are seams.

### 14.4 Coverage

- **Diff-backed**: the captain branch under ten, and `find_new_target`'s
  stand-ground search and `QUEUE_NEW` add, by run171's `1/6` on blocks
  659 and 660 (`chapter_eight_s_word_frame_is_widened_whole`, 605–900).
- **Reading only**: the follower branch, the captain-at-ten branch, the
  siege exemption, and `retargets`' increment and reset. No capture on
  disk is known to reach ten retargets in a frame.

## 15. The suspended search's `GATHER` park: a gatherer near its point gives the walk up — Great Lakes 14650 → 14982 (item 698, 2026-09-24)

Great Lakes' word was **14650**. The original spent 3 draws there and this
crate spent 2, parting at index 0 on `Unit::do_non_flat_gather+0x54b`. On
block 14651 only the Woodcutter's Camp `1/2009`'s gather-tile list parted,
192 keys, one entry along. Under it, on block 14650, the citizen `1/43`
parted on eight rows. The booking read `collide 7` against `6` as two
collision partners, `1/7` and `1/6`. **`collide` is `UnitData +0x88`, the
counter** (§6 step 5; `docs/ORDERS.md` §4.4 step 2). The partner is
`collide_o`, and it is `18` on both sides from `collide_frame` 14643.

### 15.1 The frame

`1/43` walks to its tile under a `GATHERORDER` (camp `1/2009`, tile
(228, 139)) with a transit `MOVEORDER` to (43704, 26856) in front. On 14643
it steps into the standing `1/18`, a woodcutter at (43512, 26712), and
stops at (43608, 26616) on both sides. Its 48-grid search suspends, and
§6's step-2 block in `do_move` counts `collide` one a frame, 1 to 6 by
block 14649, on both sides. Positions, `collide_o` and the orders all
agree through 14649.

On block 14650 the original's `1/43` holds the gather order alone. The
tile is −1, `wait` −1 and `goto_build` 1, `orders_x/y` is its own position,
the path is empty, and `collide` is still 6. This crate's still held the
walk, with `collide` 7. On 14650 the original's `do_non_flat_gather` draws
a tile afresh (the word's extra draw), and on 14651 the camp's list takes
the new entry. On 14651 the original sends `1/43` towards the camp.

### 15.2 The arm

`Unit::do_move@005f7b30:90-128`, read off the listing at
`005f7c84`–`005f7d8a`. It sits inside the suspended-search block
(`openlist != 0`), ahead of the blocker probe and the `collide` increment.
It fires when all three hold:

- the action under the move (`get_action`, vslot `+0x10`) is `GATHER`, 7;
- `elapsed = frame − collide_frame` is positive and `(elapsed + 2) &
  0x80000007 == 0`, so on elapsed 6, 14, 22 and so on;
- `vector_dist(|mo.x − x|, |mo.y − y|) < 0x120`. The pair is loaded into
  `ecx`/`edx` at `005f7caa`–`005f7cd4` from the **move order's** `+0x4/+0x8`
  and the unit's decoded position. The decompiler prints
  `vector_dist(unaff_EDI, unaff_ESI)`, which is the fastcall trap in
  `tools/ghidra/README.md`.

Then `avoid_x/y` (`UnitData +0x120/+0x124`) takes **the move's `x/y`**
(`005f7ce6`–`005f7cf4`). ORDERS §4.4 had written `pos` and is amended. The
action's `GatherOrder` gets `+0x14 tx = −1`, `+0x18 ty = −1`, `+0x24
goto_build = 1` and `+0x20 wait = −1`, each through
`Unit::update_action@0060a870` and vslot `+0x6c`. A worker holding a
doober at `+0x86` has it removed (`Doober::remove_hold_doobers@00846cd0`,
which fades an icon, so it is presentation). Then `kill_current_order(0)`,
and `return 0`.

For `1/43` on 14649: elapsed 6, and `vector_dist(96, 240) = 240 + 96² /
480 = 259 < 288`. The first frame the arm could fire, it did.

`Sim::do_move`'s suspended block (`crates/sim/src/orders.rs`) carries it
now, replacing the SEAM line that called it dormant. The `ATTACK` retarget
above it is still a seam.

### 15.3 What it moved

- **Great Lakes 14650 → 14982.** On 14982 the original spends three
  `Guy::init_real+0x52` before the frame's `Guy::inc_time` wraps, 15 draws
  against this crate's 11. That is a three-figure birth this crate does not
  make, `1/79`. It is **past run178's last block (14899)**, so run192 was
  captured over it and widened (`run192_s_word_frame_is_widened_whole`,
  `docs/RUNS.md`).
- **The value diff, on block 14651** (`run178_s_word_frame_is_widened_whole`):
  the eight rows of `1/43` on 14650 and the 192 on 14651 are gone, and
  nothing parts on 14649..14653. The rows standing on 14651 went 517 →
  317, and the keys first parting to run178's last block 1,279 → 416:
  the 200 and 663 of the cascade above them, none under 14650.
- `path::tests::a_gatherer_on_a_suspended_search_gives_its_walk_up_near_its_point`
  covers the case and three controls: seven frames, a point 288 short
  (`vector_dist` 304), and a walk with no gather under it. It was made to
  fail with the arm taken out.

### 15.4 What is not established

- **The doober.** This crate models no hold doober at any of the four sites
  that clear `+0x86` (`do_move`, `kill_current_order`,
  `do_non_flat_gather`, `close`). It is taken as presentation on the
  strength of `remove_hold_doobers`' body alone.
- **Only elapsed 6 is diff-backed.** Elapsed 14, 22 and so on would need a
  gatherer whose search stays suspended past the blocker probe.
- A farmer (`FLAT`) is a `GATHER` action too. No capture has put one on a
  suspended search.

### 15.5 Coverage

- **Diff-backed**: the arm on elapsed 6 inside `vector_dist < 0x120`, the
  four gather fields, the kill, and `collide` not counted. All come from
  run178's `1/43` on blocks 14650 and 14651.
- **Reading only**: `avoid_x/y`'s value, since `UNITS=3` does not print it,
  and every later elapsed.
