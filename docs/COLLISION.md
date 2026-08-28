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
recovery's shape — one whole run of the mechanic is confirmed field for
field against run10's own dump (§8). The exemption ladder (§4.3) is
reading-only: no run has entered five of its six arms.

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

`Object::add_to_world` sets the bits for figures `0 .. guy_mark`;
`Guy::set_new_location` calls `CollCheck::move_unit(from, to, coll_size)`
whenever a figure changes unit cell, and that **clears** the cells around
`from` that are more than `coll_size` from `to` and **sets** those around
`to` that are more than `coll_size` from `from`.

> The bits are **not refcounted**. Two overlapping blocks share a bit, and
> when one unit leaves it clears bits the other still stands on. The
> original has this and we keep it: an index rebuilt from the unit list
> each frame would answer differently the moment two blocks overlap, which
> for `coll_size 1` is most of the time.

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

`collide_here` also has a fast path for a proposal exactly one cell away on
one axis, which sweeps the leading edge instead of the whole disc; and it
returns 0 immediately for `coll_size == 0`.

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
| we share a `group` (≠ −1), I am not attacking, it has no suspended search (`+0x104 == 0`), and either it has no order or its order is a spell in `{0x28b, 0x28d, 0x28f, 0x291}` or a passable kind (`0, 1, 2, 3, 4, 0xc, 0x12, 0x13, 0x15`, the last four also needing its action ≠ `ATTACK`) | yes |

A soft collision at the end of the scan sets `unit_masks & 0x100000`, the
one-shot half step `docs/MOVEMENT.md` names, and returns 0.

Anything else is **hard**, unless the corner rule lets the two slip past:

    hard  ⟺  will_be_corner(me, hit, proposed) == 0
              or  |will_be_corner(…) − is_corner(other, hit)| ≠ 4

`will_be_corner` and `is_corner` return `1, 3, 5, 7` for NW, NE, SE, SW when
the hit cell is exactly a diagonal corner of the block, and 0 otherwise. A
difference of 4 is the two opposite diagonals: the units touch at one
corner from opposite sides, and pass.

A hard collision writes `collide_o`, `collide_who`, `collide_guy = 0`,
stores the proposed point in the move order's `coll_x`/`coll_y`
(`MoveOrder +0x3c/+0x40` — the dump prints them), and returns 1.

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

## 6. `Unit::resolve_unit_collision`

In order, with the first that fires winning:

1. **Attack it.** If the type has `+0x2b4 & 0x2000` and the other unit is a
   valid target: `set_attack`, `fire_ammo`, done.
2. **I am standing in my own gather target.** Only when the other unit is
   the same player's. If the current action is a `GATHER` whose target is an
   active build whose type answers vfunc `+0x94`, and that building's
   footprint `covers_tile` my tile → `kill_current_order`.
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
     waiting on me, and neither of us has the flag already.
   - otherwise fall through to the repath.
6. **The repath.** With a non-empty path stack, and under the throttle
   (`repaths[who] < 0x10`; over 4 only every fourth collision counts, over 8
   only every sixteenth object number):
   - `repaths[who] += 1`;
   - **pop the path stack** until an entry is worth keeping: stop on
     `flags & 1` (final), or on an entry with `tolerance ≥ 0x60` and no
     `flags & 2` whose tile is not `0x6000`-masked and whose cell
     `collide_here` says is clear. Push the last popped back.
   - **snap the unit onto its own 48-cell centre** — `set_new_location(ucell
     centre)`. This is the visible signature of the whole mechanic: a unit
     that collides jumps to the middle of its cell.
   - `find_upath(anti = my action is ATTACK and its action is ATTACK)`
     (`docs/PATHFINDER.md` §3).
   - clear the order's `+0x10`;
   - if the search succeeded **and** the other unit is colliding with *me*
     and is not already waiting, roll
     `pause = Random::get(0, 0xffff) % 9 + 1` into my order — **a draw on
     the shared stream** (`docs/SYNC.md`).

`Unit::do_idle` zeroes `collide`.

## 7. What this crate models

`crates/sim/src/collide.rs`, wired into `Sim::unit_step`
(`crates/sim/src/orders.rs`) and `Sim::set_new_location`.

Modelled: the bitmask with its clear-on-move semantics and the region gate;
the object chain over units; the probe with its parity filter and disc
order; the `safe`, `DETOUR`, same-cell and `coll_size 0` gates; the corner
rule; the same-player-attack exemption; `move_step`'s block; and `resolve`'s
steps 2, 4, 5 and 6 including the throttle, the stack unwind, the centre
snap and `find_upath`.

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
`TRADE_ROUTE`, `0xc` and group arms of §4.3; the pause draw of §6's tail;
`move_step`'s `set_anim(CHAR_DEFAULT)` (this crate does not set the walk
animation either, so setting the idle one would be a lone half of a pair);
`do_move`'s own collision arm — the every-other-frame re-probe of
`coll_x/coll_y` while a search is pending; squads, since only figure 0
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
- **The collision block over the whole capture**: `collide`,
  `collide_frame`, `collide_o`, `collide_who`, `collide_guy` and `safe` on
  every agreeing unit-frame of run10 — **40,600 field-frames, 285
  disagreements, none before frame 201**, and 277 of those are one sticky
  byte (`collide_guy` is never cleared, so a single extra collision on
  `1/3` reads 0 against −1 for the rest of the run). Pinned in
  `run10_s_opening_…`.
- **`coll_x`/`coll_y`** on every dumped move order, as a scoring order
  mismatch.
- The path stack's length and every waypoint — the headline's own order
  score, which the recovery's output now feeds.
- §2's clear-on-move, §4's naming and §6's snap-and-replan end to end, in
  `crates/sim/src/collide.rs`'s own tests, each written to fail first.

**Reading-only** — no capture has executed these:

- §4.3's `TRADE_ROUTE`, `0xc` and group arms, and the soft half-step flag.
- §6 steps 1 and 3.
- §6 step 5's wait-for-it branch (`unit_masks & 0x40`).
- The throttle's `repaths ≥ 4` and `≥ 8` arms.
- The pause draw of §6's tail.
- `UnitData::safe`: `find_upath`'s `+= 30` is modelled and the gate reads
  it, but no unit in any capture has ever carried a non-zero one — which
  the pin asserts rather than assumes.
- `coll_size ≠ 1` anywhere: every unit in every capture so far is a
  `BLOCK_RADIUS 1` type, so the ring-2 quirk and the parity filter's
  general form are untested behaviourally.

**The captures that would settle them** are in §9.

## 9. What is not established

- **`ObjectType +0x2b4 & 0x2000`** — the "attack what you bump into" bit.
  Read as a flag, not traced to its XML column.
- **`+0x10c`, `+0x94`, `+0x74`, `+0xf4`** — the virtuals §4.1 step 3 and §6
  steps 2 and 3 call. Named by slot, not by identity.
- **The action indices `0xc` and `0xf`** in §4.3. `0xf` sits next to
  `TRADE_ROUTE` in the same arm, so one of the two is the caravan's.
- **`WData::block == −1`.** Where the sentinel is written is unread; this
  crate marks every cell. *Capture:* a `WORLD ≥ 6` dump does not print it,
  so this needs a reading, not a run.
- **Squads.** `Unit::set_new_location` spreads figures `0 .. guy_mark` over
  a formation and each marks its own disc. *Capture:* `UNITS=3` +
  `GUYS=2` over a four-figure squad walking into another unit.
- **`coll_size ≥ 2`.** *Capture:* the same, with siege or a ship.
- **The `pause` draw.** *Capture:* two units of the same player ordered
  into each other head-on, `UNITS=3`; the pause shows in the `MOVEORDER`
  row and the draw in `rontrace`.
- **Whether `collide_guy` is ever non-zero.** Every hard collision this
  reading found writes 0; the field exists, so something writes it.
