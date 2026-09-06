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
  unit stops and is non-zero for every frame of a walk. A moving guy needs
  no repaint: `move_unit`'s set pass has just written its whole new disc.

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
3. Otherwise, when `boats` is asked for and the unit is sea-domain, a hero,
   a supply unit, or its type answers vfunc `+0x10c`: the test is
   `detect_boat_collision` instead, and a clear answer returns 0.
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
| its action is `0xc` and that order's target is me | yes |
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

A soft collision at the end of the scan sets `unit_masks & 0x100000`, the
one-shot half step `docs/MOVEMENT.md` names, and returns 0.

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
why `find_ordered_collision` has a second pass this crate does not model:
if I am in a group whose `+0x49` byte is clear, every other active member
with `inside_up < 0` and a block is tested against its ordered position
regardless of where it stands. §9 carries it.

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
   `find_new_target`. Never reached by any capture so far.
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
`coll_size 0` gates; the corner rule; the same-player-attack exemption; `move_step`'s block, **including its
`set_anim(CHAR_DEFAULT)`** (§5) **and the probe's write back into the
order** (§4.3, item 115); **`do_move`'s waypoint test with both of
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

Not modelled, each listed in §9: `detect_boat_collision` (no ships); step 1
(`+0x2b4 & 0x2000`) and step 3 (the enemy ladder); the soft half-step flag
(`Unit::half_step` is written and nothing reads it — the halving lives
inside `move_step`, which this crate does not thread it into); the
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
- The path stack's length and every waypoint — the headline's own order
  score, which the recovery's output now feeds.
- §2's clear-on-move, §2.2's repaint, §4's naming and §6's snap-and-replan
  end to end, in `crates/sim/src/collide.rs`'s own tests, each written to
  fail first.

**Reading-only** — no capture has executed these:

- §4.3's `TRADE_ROUTE` and `0xc` arms, and the soft half-step flag. ~~The
  group arm~~ — landed 2026-09-04, §9.
- §4.3's **figure-centred `is_corner`** (2026-09-04). Both long words are
  unmoved by it, so no run on disk has a blocker whose figures answer
  differently from the blocker — which is what one would expect while the
  only colliders a capture reaches are one-figure citizens standing still.
- §5.1's tolerance-widening arm. Every hit a capture has reached there was
  a final waypoint under a gather, so the parked-collider branch rests on
  the decompile; `collide.rs`'s own test is what exercises it, and it was
  written to fail first.
- §6 steps 1 and 3.
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

**Eight are stated, unimplemented and unreached.** §6 step 6's `anti` flag
drops the "and *its* action is `ATTACK`" conjunct
(`resolve_unit_collision@005f9d30:473-480` is a conjunction; the crate tests
only its own side, so it plans an anti-unit path wherever the original plans
a plain one); §6 step 4's sidestep has no `domain == 0` gate, so a sea unit
sidesteps here; §6 step 4/5's order set is four kinds where the original has
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

## 9. What is not established

- **Where both long words now part, and it is a *position* and not a
  predicate** (2026-09-04, queue item 223). Great Lakes 6848 is one
  `SITE_BLOCKED` this crate spends and the original does not; East Indies
  7448 is two the original spends and this crate does not.

  Great Lakes' is named whole. The blocked unit is the AI's Archer `1/28`,
  a **follower of the marching squad** `1/27`/`1/28`/`1/29`
  (`docs/ORDERS.md` §15); the blocker is the standing citizen `1/13` at
  `(42744, 24504)` — exactly where run18b's dump has it, so the blocker is
  not in doubt. `1/28` stands on unit cell `(891, 513)`, its slot this
  frame is `(42773, 24617)` on cell `(891, 512)`, `dy = −1` takes §4.2's
  leading edge, and the row `y = 511` hits `(890, 511)`, a cell of `1/13`'s
  block. `will_be_corner` NW against `is_corner` 0 → hard. Every step of
  that is forced once the two blocks overlap, and they do overlap: **the
  predicate is not what is wrong**. What the original has to be is
  somewhere else, and the trace says by how much — its own first blocked
  stand in that neighbourhood is at **6860, twelve frames later**, and
  another at 6892.

  East Indies 7448 is the same shape read from the other side. Five units
  are moving there — `0/3`, `1/0`, `1/10`, `1/18`, `1/20`, none of them
  within a thousand units of another — and **four of the five propose a
  point inside their own unit cell**, where §4.1's gate 6 refuses the test
  before it starts. Only `1/0`'s step and `1/10`'s `do_move` waypoint
  probe cross a boundary at all, and both miss. So for the original to
  spend two blocked stands on that frame at least two of its units must be
  crossing a cell this crate's are not: a position again, and not a
  predicate. (`1/18` lands on `x = 36480 = 760 × 48`, the boundary exactly,
  and crosses on 7449.)

  Twelve frames at 26 units a frame is ~330 units, about 1.7 tiles: the
  squad marches slower there, or starts later, or walks a different line.
  §15's own two seams are the candidates — the group's `speed`/`new_speed`
  pair, which has no reader here, and `Group::update_positions`' slot
  table — and neither can be told from the other without positions.

  *Capture, owed and not run:* Great Lakes, `frames 6870`,
  `frame_window 6640 6870`, `end: MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,
  DEATHS=1,LEADERS=1` — run75's recipe with the window moved, opened
  before `Army::do_forming` issues the group order on 6650 so the whole
  march is on disk. `UNITS=3` writes each archer's position, `angle`,
  `orders_x/y`, the `MOVEORDER`'s `dest`/`coll_x`/`coll_y` and the path
  stack, and `collide_o`/`collide_who`/`collide_frame` say on which frame
  the original's squad first refuses a step. It is stanza **run 76** in
  `tools/gamelog/captures.txt`.

  *It has not landed.* Two launches through `tools/gamelog/viadriver.sh`
  both stalled in `waitwin.sh` with the game at 0 % CPU and no window,
  and the reason is neither a permission nor the port: **CrossOver's
  bottle licence has expired**, and the window `wineloader` is showing is
  titled `Expired Bottle: ron`. The lane needs a human to renew or
  re-activate CrossOver; nothing in the harness can. See
  `docs/ORACLE.md`, "The window nobody can see".

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
  run33's frame 571 spends it, and §8 has the frame. What the capture
  named in this row would still add is the *value* — no dump in hand
  prints a non-zero `MOVEORDER pause` — so the `% 9 + 1` itself rests on
  the listing. *Capture, still owed:* `UNITS=3`, two units of the same
  player ordered into each other head-on, for the `MOVEORDER` row.
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
- **§5.2's group arm.** `find_ordered_collision`'s second pass tests every
  other member of my group against its ordered position, wherever it
  stands, and this crate keeps no `UnitData::group` back-pointer to reach
  it with. Every unit in every capture so far dumps `group -1`. *Capture:*
  `UNITS=3` + `GROUPS=1`, a selected group ordered to build or gather at
  one site while its members are scattered — fold into the item-23 run.
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
