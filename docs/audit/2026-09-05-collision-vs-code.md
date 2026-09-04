# COLLISION.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: about 115 stated rules, formulas, predicates, constants and step
orders across §1–§6 — the rules sections. (§7 is the crate's own inventory
of what it models, §8 is coverage and §9 is open questions: context, not
rules, though both were read for what they declare unmodelled, because a
declared seam is not a drift.) **10 rows** below: the places where the code
and the document disagree. Everything else is in
"What was checked and found faithful" at the foot.

One of the ten (R7) turns out to be a place where the *document* is what is
wrong and the code faithfully implements the wrong rule; half of R8 is the
same. They are written up the same way, because the lane adjudicates rows
against the code either way and a row costs less than a miss.

## Rows

### R1 — `find_upath`'s `anti` flag: the code drops half the predicate

| | |
|---|---|
| document | COLLISION.md §6 step 6, "`find_upath(anti = my action is ATTACK and its action is ATTACK)`" |
| code | `crates/sim/src/collide.rs:1122-1125` (`fn collide_repath`) |
| document says | `anti` is set only when **both** my action and the collider's action are `ATTACK` |
| code does | sets `anti` from my action alone; the collider's action is never asked |
| difference shows | `gamelog` `UNITDATA`/`MOVEORDER` path stack of an attacking unit that collides with a non-attacking unit of the same or another player: the `find_upath` plan differs, so the whole waypoint list on the frame after the collision differs. No capture on disk reaches it — every collision in run10/run33/run53/run66/run70 is between citizens, and §8 lists no attacking collider. |
| reached | reached (`Unit::resolve_unit_collision@005f9d30` is **not** in the blind list; nor is `PathFinder::find_upath`. The `005fccc0` on that list is `Unit::resolve_block`, a different function COLLISION.md does not cite.) |

`collide.rs:1122-1125`:

```rust
let anti = self
    .action_of(u)
    .is_some_and(|a| self.units[u].orders[a].index() == index::ATTACK);
let r = self.find_upath(u, anti);
```

The original is a conjunction. `resolve_unit_collision@005f9d30:473-480`
(`~/ghidra-projects/decomp/funcs/Unit/resolve_unit_collision@005f9d30.c`):

```c
iVar5 = 0;
if (((local_3c != (UnitOrder *)0x0) &&
    (iVar8 = (**(code **)(local_3c->_padding_ + 0x10))(), iVar8 == 10)) &&
   (OVar10 = UnitData::action_type(
        *(UnitData **)(*(int *)(&units.field_0x10 + local_1c * 4) + (int)local_2c)),
    OVar10 == ATTACK)) {
  iVar5 = 1;
}
iVar5 = PathFinder::find_upath(this_02, &path, who, o, iVar5);
```

`local_3c` is `update_action(this)` (`:96-97`) — **my** action order, whose
`+0x10` vfunc is `order_type`, tested against `10` = `ATTACK`. `local_1c` /
`local_2c` are `collide_who * 7` and `collide_o * 4`, so the second
conjunct is `action_type(the collider) == ATTACK`. The code implements the
first conjunct and not the second, so it will plan an anti-unit path in
every case where the document (and the original) plan a plain one.

### R2 — §6 step 4's sidestep has no `domain == 0` gate in the code

| | |
|---|---|
| document | COLLISION.md §6 step 4, "Only when *the other unit's* current order is one of `MOVE_TO, …`, **my domain is 0**, and the path top's `flags & 2` is clear" |
| code | `crates/sim/src/collide.rs:981-991` (`fn resolve_unit_collision`) |
| document says | a sea- or air-domain unit never sidesteps; it falls through to steps 5 and 6 |
| code does | tests `its_move`, `collide_o >= 0`, the path top's `SIDESTEP` flag and the presence of a move order — and nothing about the asking unit's own domain |
| difference shows | a Transport Barge or a Fishing Boat blocked by another moving boat: the original spends no sidestep and goes to the wait/repath, the crate pushes a `{cell centre, tol 0, flags 2}` waypoint. Visible as an extra path-stack entry in a `UNITS=3` dump and as the missing `Unit::resolve_unit_collision+0xb52` / extra `SITE_BLOCKED` draws. No capture on disk has two boats colliding (`docs/COLLISION.md` §7, "`detect_boat_collision` (no ships)"). |
| reached | reached (`005f9d30` is not in the blind list; the sidestep arm itself is exercised by run66-era captures for land units) |

`collide.rs:981-991`:

```rust
if its_move
    && self.units[u].collide_o >= 0
    && self.units[u]
        .path
        .last()
        .is_none_or(|t| t.flags & path_flag::SIDESTEP == 0)
    && let Some(mo) = self.current_move(u)
    && self.sidestep(u, mo)
{
    return;
}
```

The original's guard is `resolve_unit_collision@005f9d30:265`:

```c
if (((((local_28 == MOVE_TO) || … || (local_28 == GROUP_ATTACK_TO)))) &&
   (*(int *)(*(int *)&this->field_0x18 + 0x218) == 0)) {
```

`this->field_0x18` is the `ObjectType *` and `+0x218` is `domain`; the
document names it correctly. Nothing in `resolve_unit_collision` or in
`Sim::sidestep` (`collide.rs:1041`) reads a domain, and the crate does stand
sea units up now (`docs/TRANSPORT.md`), so the gate is not vacuous.

### R3 — §6 step 4/5's order set is missing `CHANGE_FORM`

| | |
|---|---|
| document | COLLISION.md §6 step 4, "one of `MOVE_TO, ATTACK_TO, EXPLORE_TO, FLEE_TO, CHANGE_FORM, GROUP_MOVE, GROUP_ATTACK_TO`" (and step 5, "one of the move kinds above") |
| code | `crates/sim/src/collide.rs:974-978` (`let its_move = matches!(…)`) |
| document says | seven order kinds select the sidestep, and six of them (all but `FLEE_TO`) select the wait |
| code does | four: `MOVE_TO`, `ATTACK_TO`, `EXPLORE_TO`, `FLEE_TO`. `GROUP_MOVE`/`GROUP_ATTACK_TO` are covered — the crate carries them as `MOVE_TO`/`ATTACK_TO` with a `GroupMove` (`orders.rs:2216`) — but **`CHANGE_FORM` (18) is not an order kind this crate has at all** (`orders.rs:597-598`, `orders.rs:2033`) |
| difference shows | a unit blocked by a formation member that is mid-`CHANGE_FORM`: the original sidesteps or waits, the crate repaths and snaps to its cell centre. `UNITS=3` position of the blocked unit, and a `SITE_PAUSE`/`SITE_BLOCKED` draw-count difference. No capture reaches it — `CHANGE_FORM` is never constructed here. |
| reached | reached |

`collide.rs:974-978`:

```rust
let its_order = other.map_or(index::NONE, |o| self.order_type(o));
let its_move = matches!(
    its_order,
    index::MOVE_TO | index::ATTACK_TO | index::EXPLORE_TO | index::FLEE_TO
);
```

`docs/ORDERS.md:216` gives `18 | CHANGE_FORM | FormOrder`, and
`crates/sim/src/orders.rs` has no `index::CHANGE_FORM` constant. The
consequence is a *narrowing* of both step 4 and step 5; the document names
the kind and the code cannot express it. COLLISION.md §7 and §9 do not list
this as a seam.

### R4 — §6 step 5's `CAST_SPELL 0x28a` arm is not implemented

| | |
|---|---|
| document | COLLISION.md §6 step 5, "if the other unit's order is one of the move kinds above **(or a `CAST_SPELL` of `0x28a`)** and neither side has given up: set `unit_masks & 0x40`" |
| code | `crates/sim/src/collide.rs:998` (`if its_move && its_order != index::FLEE_TO`) |
| document says | a collider whose current order is a `CAST_SPELL` of spell `0x28a` — the transport cast, `orders::spell::TRANSPORT` — also earns the wait |
| code does | only the three plain move kinds reach the wait; a `CAST_SPELL` collider falls straight through to the repath of step 6 |
| difference shows | a land unit blocked by another land unit that has just queued the transport spell at a shoreline (`Sim::shore_step`, `collide.rs:461`, queues `spell::TRANSPORT` `QUEUE_FIRST` and returns). The original sets `unit_masks & 0x40` and stands; the crate increments `repaths[who]`, unwinds the stack and snaps to the cell centre — a *position* difference on the next frame, plus the `collide`/`collide_frame` pair. Reachable on any shore-crossing capture; none of the collision captures in §8 is one. |
| reached | reached |

The original, `resolve_unit_collision@005f9d30:395-412`:

```c
if (local_28 != FLEE_TO) {
  if ((((local_28 == MOVE_TO) || (local_28 == ATTACK_TO)) || (local_28 == EXPLORE_TO)) ||
     (((local_28 == CHANGE_FORM || (local_28 == GROUP_MOVE)) || (local_28 == GROUP_ATTACK_TO)))) {
LAB_005fa559:
    …the wait…
  }
  if (local_28 == CAST_SPELL) {
    pUVar6 = UnitData::get_order(…the collider…);
    iVar8 = (**(code **)(pUVar6->_padding_ + 0xf4))();
    if (*(int *)(iVar8 + 0x20) == 0x28a) goto LAB_005fa559;
  }
}
```

`spell::TRANSPORT` is `0x28a` in `crates/sim/src/orders.rs:60`, so the
constant is already in the crate; only the arm is missing. Note the code's
`its_order != index::FLEE_TO` is *correct* and the document's "one of the
move kinds above" is the looser statement — the decompile brackets the whole
block in `if (local_28 != FLEE_TO)` and omits `FLEE_TO` from the inner list,
which is what `collide.rs:998` implements.

### R5 — §5.1's kill set omits `TRADE_ROUTE`

| | |
|---|---|
| document | COLLISION.md §5.1, "if the waypoint is **final** (`flags & 1`) and the unit's *action* is `TRADE_ROUTE`, `GATHER`, `ATTACK` or `BUILD_AT` → `kill_current_order`" |
| code | `crates/sim/src/orders.rs:1716-1723` (`do_move`'s waypoint test) |
| document says | four actions kill the walk where the unit stands |
| code does | three: `GATHER`, `ATTACK`, `BUILD_AT`. `index::TRADE_ROUTE` exists (`orders.rs:43`) and is not in the `matches!` |
| difference shows | a Caravan or Merchant on a `TRADE_ROUTE` action taking a final waypoint into an occupied cell: the original kills the order and re-decides next frame, the crate walks on (or widens tolerance via the second arm). `MOVEORDER` presence/absence in the next `UNITDATA` block and the merchant's position. run66 holds East Indies' AI Merchant at `[6340, 6600)` but that merchant's action there is a gather, not a trade route — no capture on disk reaches this arm. |
| reached | reached (the `do_move` waypoint test itself is pinned by run10 frame 199, §8) |

```rust
if top.flags & path_flag::FINAL != 0
    && matches!(
        action,
        Some(index::GATHER | index::ATTACK | index::BUILD_AT)
    )
```

`orders.rs:1712-1714` calls this out in its own comment ("the original also
spells `TRADE_ROUTE` in the kill's action set … neither is modelled"), so
the omission is deliberate — but COLLISION.md §5.1 states the rule flatly
and neither §7 nor §9 lists it as unmodelled, which is the disagreement.

### R6 — §2's region gate passes a cell that has no region; the original refuses it

| | |
|---|---|
| document | COLLISION.md §2, "a cell is marked only when the world cell's `WData::region` equals `get_tregion` of the marking figure's **own tile**, or the figure's tile has no region" |
| code | `crates/sim/src/collide.rs:167-175` (`fn coll_region_ok`) |
| document says | two ways to pass: the regions are equal, or the *figure's* region is absent |
| code does | three: it also passes when the **cell's** region is absent (`region_of(...).is_none_or(...)`) |
| difference shows | the `WORLD`/`TERRAIN` occupancy of a unit standing on or beside a cell with no region — visible only indirectly, as a `PathFinder::valid_ucoord` refusal that the original does not make (a `callwin` over `PathFinder::calc_cost` in run70's shape) and hence a different route. No capture on disk is known to have an in-bounds region-less cell near a unit. |
| reached | reached (`CollCheck::move_unit@00682ad0`, `Object::add_to_world@0064d8c0`) |

`CollCheck::move_unit@00682ad0:104` and `:157` both write the gate as

```c
if ((local_8.value < 0) || (*(short *)(iVar5 + 4) == local_8.value)) {
  …take the CollBlock…
} else {
  pCVar4 = (CollBlock *)0xffffffff;   /* refused */
}
```

`local_8` is `WorldData::get_tregion` of the pass's own end (`:59`, `:84`,
`:139`) and `*(short *)(iVar5 + 4)` is the cell's `WData::region`. The escape
hatch is `local_8 < 0` — the *figure's* region absent — and there is no
second one for the cell. `World::region_of` (`crates/sim/src/world.rs:1390`)
returns `Option<u16>` and `None` is a real state for an in-bounds cell, so
`is_none_or` is a widening of the gate, not a bounds guard (the bounds guard
is `CollGrid::set`'s own `index`, `collide.rs:120`).

### R7 — §4.3's group arm: the document's "the last four" is the wrong four, and the code follows the document

| | |
|---|---|
| document | COLLISION.md §4.3, "…or a passable kind (`0, 1, 2, 3, 4, 0xc, 0x12, 0x13, 0x15`, **the last four** also needing its action ≠ `ATTACK`)" |
| code | `crates/sim/src/collide.rs:776-796` (`fn same_group_soft`) |
| document says | `0, 1, 2, 3, 4` pass unconditionally; `0xc, 0x12, 0x13, 0x15` pass only when the collider's action is not `ATTACK` |
| code does | exactly what the document says (modulo `0x12`, R3): `MOVE_TO/ATTACK_TO/EXPLORE_TO/FLEE_TO` pass unconditionally unless they carry a `GroupMove`; `GUARD` needs `not_attacking` |
| **the original does** | the split is `(0 \|\| 0xc)` unconditional, and `(1 \|\| 2 \|\| 3 \|\| 4 \|\| 0x12 \|\| 0x13 \|\| 0x15) && its action ≠ ATTACK` — **seven** kinds gated, and `0xc` among the *un*gated ones |
| difference shows | two arms, both needing **group-mates**, so both need a marching squad. (a) a member whose `GROUP_MOVE` has been replaced in place by a plain `MoveOrder` (`docs/ORDERS.md`, "`GROUP_MOVE → MOVE_TO`, `GROUP_ATTACK_TO → ATTACK_TO`") while its **action** is still `ATTACK`, blocking a group-mate: the original calls that **hard** and spends `SITE_BLOCKED` plus a repath, this crate calls it soft and only sets `half_step`. (b) a `GUARD` group-mate whose action is `ATTACK`: the original soft, this crate hard. Either is a `SITE_BLOCKED` draw-count difference in a squad trace and a `UNITS=3` position difference on the frames after. **Not** Great Lakes 6848 — §9 names the blocker there as the standing citizen `1/13`, which is in no group with the archer — so the capture that would settle it is the one §9 already owes: run 76, `frames 6870`, `frame_window 6640 6870`, opened before `Army::do_forming` issues the group order on 6650, with `UNITS=3` printing each archer's order and action. |
| reached | reached |

`detect_unit_collision@00617060:366-370`:

```c
else if (((iVar7 == 0) || (iVar7 == 0xc)) ||
        (((((iVar7 == 1 || ((iVar7 == 2 || (iVar7 == 3)))) || (iVar7 == 4)) ||
          (((iVar7 == 0x12 || (iVar7 == 0x13)) || (iVar7 == 0x15)))) &&
         (local_28 != 10)))) goto LAB_00617870;
```

`iVar7` is `order_type` of the collider's current order (`:328`), `local_28`
is `get_action(collider)`'s type (`:180-186`), and `LAB_00617870` sets
`local_8 = 1`, the soft flag. The parenthesisation is unambiguous: `0` and
`0xc` short-circuit before the `&& (local_28 != 10)` ever runs.

So the code has it **backwards on both ends**:

* `collide.rs:793` — `passable && (m.group.is_none() || not_attacking)`
  lets a plain `MOVE_TO`/`ATTACK_TO`/`EXPLORE_TO`/`FLEE_TO` group-mate be
  soft while its action is `ATTACK`; the original makes that hard.
* `collide.rs:795` — `front.index() == index::GUARD && not_attacking`
  makes a `GUARD` group-mate hard while its action is `ATTACK`; the
  original makes that soft unconditionally.

`UNSURE:` only in that this row overturns the document rather than the
code. What settles it is the listing at `00617500`–`0061760e`
(`llvm-objdump` over the `cmp`/`je` chain), which should show the
`local_28 != 10` test dominating only the second group of comparisons.

### R8 — §6 step 5's wait guard: the code checks neither "my flag" nor the collider's `collide_o < 0` escape

| | |
|---|---|
| document | COLLISION.md §6 step 5, "…provided `other.collide` and `my collide` are both under `0x20` …, we are not enemies, it is not already waiting on me, and **neither of us has the flag already**" |
| code | `crates/sim/src/collide.rs:1010-1014` |
| document says | four conditions, the last of them symmetric ("neither of us") |
| code does | `!(other.waiting_on && collider_of(other).waiting_on)`; it never reads **my own** `waiting_on`, and it treats "the collider names nobody" as passing |
| difference shows | `UNITDATA`'s `collide`/`collide_frame` and the unit's position on the frame after a three-unit pile-up: the crate grants a wait (unit stands, `collide` climbs) where the original repaths (unit snaps to its cell centre and re-plans). One frame of position, then a whole different route. run33's frames 565–580 are the closest shape on disk; no capture is known to reach the `collide_o < 0` arm. |
| reached | reached |

The original, `resolve_unit_collision@005f9d30:396-400`:

```c
(((*(byte *)(iVar5 + 0x68) & 0x40) == 0 ||
 ((-1 < sVar13 &&
  ((*(byte *)(*(int *)(*(int *)(&units.field_0x10 + *(char *)(iVar5 + 0xb3) * 0x1c) +
                       sVar13 * 4) + 0x68) & 0x40) == 0))))))))
```

`iVar5` is the collider, `+0x68 & 0x40` its wait flag and `sVar13` its
`collide_o` (`+0x8a`, loaded two lines above). Written out: pass when the
collider is **not** waiting, or when (its `collide_o >= 0` **and** the unit
*it* is waiting on is not itself waiting). The code's
`!(A && B')`, with `B'` false whenever `collider_of(other)` is `None`,
inverts the `sVar13 < 0` case: the original **refuses** the wait for a
collider that is waiting but names nobody, and the crate grants it.

The document's "neither of us has the flag already" has no counterpart in
either the code or the decompile — nothing reads `this->+0x68 & 0x40` in
this guard. That half of the sentence should be struck; the code is right
not to implement it.

### R9 — `UNSURE:` §6 step 4 writes two fields the document's step 4 does not name

| | |
|---|---|
| document | COLLISION.md §6 step 4, "The first that is neither `invalid_loc` nor colliding is pushed as `{cell centre, tol 0, flags 2}` and **written into the order's `+0x2c/+0x30`**. Done." |
| code | `crates/sim/src/collide.rs:1058-1069` (`fn sidestep`) |
| document says | the push, and a write of the waypoint pair `+0x2c`/`+0x30`, and nothing else |
| code does | also sets `m.has_waypoint = true` (the crate's name for the order's `+0x10`, the `dest` flag §6 step 6 clears) and `self.units[u].tolerance = 0` |
| difference shows | the leg's arrival test on the frame after a sidestep: `do_move` compares `vector_dist(waypoint − pos)` against `UnitData::tolerance`, so zeroing it can turn an arrival into another step. `MOVEORDER`'s `dest` column and the unit's position one frame after a sidestep; run66's block window would carry it if a sidestep were reached there. |
| reached | reached |

`resolve_unit_collision@005f9d30:335-346` writes exactly two things after
the `Stack<PathData>::push`:

```c
Stack<PathData>::push(local_38,&local_4c);
pUVar6 = update_order((Unit *)local_14);
iVar5 = (**(code **)(pUVar6->_padding_ + 0x40))();
*(int *)(iVar5 + 0x2c) = iVar18 + 0x18;
pUVar6 = update_order((Unit *)local_14);
iVar5 = (**(code **)(pUVar6->_padding_ + 0x40))();
*(int *)(iVar5 + 0x30) = iVar16 + 0x18;
```

No `+0x10`, no `UnitData::tolerance`. Marked `UNSURE:` because both extra
writes may be compensating for a genuine structural difference — the crate
takes a waypoint through `do_move`'s `!mo.has_waypoint` branch
(`orders.rs:1681`) and the original through `+0x10`, and `+0x10` is already
1 at this point in the original's flow, so `has_waypoint = true` is likely a
no-op. The `tolerance = 0` is the half worth re-deriving: it is a live write
the original does not make. What settles it is a two-unit head-on test that
reaches step 4 with a non-zero leg tolerance and asserts the arrival frame.

### R10 — `UNSURE:` the index is keyed on the unit's point here, on the **figure's** in the original

| | |
|---|---|
| document | COLLISION.md §2, "`Guy::set_new_location` calls `CollCheck::move_unit(from, to, coll_size)` **whenever a figure changes unit cell**"; and §2.2, "`move_x[0 .. radius[coll_size])` around **its** unit cell" |
| code | `crates/sim/src/collide.rs:381` (`Sim::set_new_location` → `coll_move(u, from, to)`) and `collide.rs:259` |
| document says | the occupancy bits follow the **figure's** `GuyData::x/y` — the body — and move on the frame the *body* crosses a unit cell |
| code does | follows `Unit::pos` (`x_internal`/`y_internal`) and moves the bits on the frame the *unit's point* crosses a unit cell |
| difference shows | one frame of occupancy for a walking unit: the cell a unit is leaving stays lit one frame longer in the original than here, and the cell it is entering lights one frame later. Observable as a `PathFinder::valid_ucoord` answer — the run70-shaped `callwin` over `PathFinder::calc_cost` is the instrument, on any frame a searching unit's neighbour crosses a cell boundary. The `collide`/`collide_o` block does **not** show it, which is consistent with §8's zero-disagreement record. |
| reached | reached (`Guy::set_new_location@005d86f0`, `CollCheck::move_unit@00682ad0`) |

`Guy::set_new_location@005d86f0:22-32` is the **only** caller of
`CollCheck::move_unit` in the whole export (grep over
`~/ghidra-projects/decomp/funcs/`), and it keys the move on the guy's own
fields:

```c
if ((*(int *)(local_c + 0x218) != 2) &&
    ((int)(char)this->field_0xa2 < *(int *)(local_c + 0x304))) {
  iVar5 = *(int *)&this->field_0xc >> 4;                 /* GuyData::x  */
  this_00 = (CollCheck *)div_3_table[*(int *)&this->field_0x10 >> 4];  /* GuyData::y */
  if ((div_3_table[iVar5] != div_3_table[param_1.value >> 4]) || …) {
    CollCheck::move_unit(this_00, …, *(int *)(local_c + 0x248));
  }
}
```

`Unit::set_new_location@005f8d20` does not call it at all. Guy 0's body
lags the unit's point on every frame `move_step` passes `move_guys = 0`
(`docs/ANIM.md` §4 step 1, and COLLISION.md §4.3's own note, "guy 0 itself
on any frame its body has not caught up with the unit's point"), so the two
keys are not the same key.

`UNSURE:` because the lag may be under one unit cell on every frame any
capture reaches — a 25-speed citizen moves half a cell a frame — and the
§8 record is 249,293 agreeing unit-frames on the collision block alone. But
COLLISION.md §7's "Not modelled" line covers only "squads, since only figure
0 marks the index"; it does not say that *figure 0's own position* is
replaced by the unit's. What settles it is a `GUYS=2` + `UNITS=3` capture of
a single walking unit across a unit-cell boundary, compared against
`Sim::coll_move`'s frame.

Note the same gates appear here as §2.2 names for the repaint —
`type->domain != 2` (`+0x218`) and `guy_num (+0xa2) < squad_size (+0x304)` —
which confirms §2.2's reading of them.

## What was checked and found faithful

So a later pass need not re-derive them. Every item below was read in the
document and located in the code; where a claim turned on what the original
does, the decompile settled it.

**§2 — the occupancy bitmask.** `coll_size = block_radius / 48`
(`collide.rs:149-151`, `UNIT_BLOCK_RADIUS = 48` at `:29`). The `coll_size != 0`
and `domain != 2` gates (`collide.rs:233-235` in `coll_paint`, `:261-263` in `coll_move`; `is_air` at `:160-162`). `tregion_alt` as
the figure's key rather than `region_of` — `region2` when the cell is
`HALFLAND` and the tile is `SURFACE_OCEAN` (`world.rs:1103-1112`), which is
`WorldData::get_tregion@006b52e0`'s shape. `CollCheck::move_unit`'s two
passes, each with its own end's `get_tregion` and the Chebyshev-`> size`
test (`collide.rs:259-290` against `move_unit@00682ad0:88-191`). The bits
are not refcounted, deliberately, and `the_index_follows_the_unit_and_is_not_refcounted`
asserts it. The `a == b` early return in `coll_move` matches the original's
`div_3_table` cell comparison in `Guy::set_new_location@005d86f0:25-26`.

**§2.1 — `move_x` / `move_y` / `radius`.** `spiral()` (`collide.rs:67-88`):
the clockwise ring walk from `(−r, −r)`, top row left to right, right column
top to bottom, bottom row right to left, left column bottom to top; ring 2's
four corners moved to the end in NW, NE, SE, SW; `radius[r] = (2r+1)²` as the
length of the prefix (`1 + Σ8k = (2r+1)²`).

**§2.2 — the sixty-fourth frame.** `(frame + o) % 64 == 0` on the unit's own
object number (`collide.rs:219`), the `avg_speed == 0` gate (`:216-218`),
set-only through `coll_paint(…, true)` (`:222`), the region gate carried, and
the call site at the tail of `Guy::process`'s equivalent
(`lib.rs:3123-3127`). The two SEAMs — one figure a unit, so the `squad_size`
gate is vacuous — are stated in both places. `Guy::set_new_location@005d86f0:20`
independently confirms the `domain != 2` and `guy_num < squad_size (+0x304)`
pair.

**§3 — the object chain.** Push onto the head (`collide.rs:302-314`), unlink
(`:316-334`), and both only on a **world-cell** change (`:373`, `:377-385`), so the
chain's order is the order units last entered the cell.

**§4.1 — the gates.** Air, `safe != 0`, the path top's `DETOUR` (`flags & 8`)
and the same-cell test (`collide.rs:605-618` `detect_gates`, and the same-cell test in `detect_quick` `:620-626` and `detect_unit_collision` `:636-638`). The
exit bookkeeping — `collide_o = collide_who = −1`, `collide = 0` when
`collide_frame < frame − 5`, `unit_masks & 0x40` cleared (`:653-658`) — and,
importantly, **`detect_quick` correctly does *not* do it**:
`detect_unit_collision@00617060:456-458` is `if (!bVar14) return 0;` with
`bVar14 = param_3 == 0`, so only the full form falls through to the clearing
block at `LAB_006177fa`. `boats` / `detect_boat_collision` and the `top_only`
argument are absent here and declared absent in §7.

**§4.2 — the probe.** `coll_size == 0 → None` (`collide.rs:485-488`); the
parity filter `(dx + size) % 2 == 0 && (dy + size) % 2 == 0` (`:511-514`);
the own-block exemption gated on `on_map` (`:516-519`); the leading-edge fast
path selected only when `nocoll` is clear and `(dx, dy)` is one cell on one
axis, with `x = at.x + dx·size, y = at.y − size + 2k` and its mirror, `k` in
`0..=size`, first occupied winning, and no own-block exemption
(`:492-508` and `leading_edge` at `:531-535`). The `nocoll` argument is 0 at both
`detect_unit_collision` forms and at `find_collision`, and **1** at
`valid_ucoord` (`path.rs:333`) and at the step-6 unwind's own probe
(`collide.rs:1109`) — which is what the decompile passes at
`005f9d30:453`.

**§4.3 — naming the other unit.** Quick returns at once. The 3×3 world-cell
walk over `move_x[0..9]` and each cell's `down` chain, skipping self, air,
dead and off-map (`collide.rs:670-686`); `is_here` as Chebyshev `≤ its
coll_size` about the *unit's* position (`:538-546`); the same-player-attack
soft arm with `extra > 0x300` and `attack_dist − range·0xc0` clamped at 0
(`:703-725`, matching `00617060:100-120`); the soft flag as `half_step`
(`:694-696`); `will_be_corner` measured against the **proposed** cell and
`is_corner` against the blocker's **figures**, first non-zero
(`:561-584`); the corner values `1, 3, 5, 7` for NW, NE, SE, SW; the hard
predicate `will == 0 || |will − theirs| != 4`; the immediate return on the
first hard hit with `collide_o`, `collide_who`, `collide_guy = 0` and
`coll_x`/`coll_y` written into the order (`:639-650`, matching
`00617060:373-427`). The pack-spell arm of the group rule
(`{0x28b, 0x28d, 0x28f, 0x291}`) is right (`:777-783`).

**§5 — `move_step`'s block.** All five predicates and their order
(`orders.rs:2820-2872`): the sidestep snap-through (`flags & 2`, a clear
`quick 1` probe of the top, `|dx| < 0x61`, `|dy| < 0x61`), the
`set_anim(CHAR_DEFAULT)` **before** the three give-up tests, the owed-turn
return, `(my big_radius + its big_radius) * 3 <= manh`, `collide < 0x1a`,
`(top.tolerance == 0 || top.flags & 2) && !(top.flags & 1)`, and
`tolerance = manh * 2`. `big_radius` is `Profile::big_radius`, the type's
`+0x244`. The write-back of the probe's `coll` pair over the stepped copy
(`orders.rs:2828-2830`), which is item 115's fix.

**§5.1 — `do_move`'s waypoint test.** Once per leg, on the `dest` 0 → 1
frame (`orders.rs:1681`); the full form, so it clears on the way out; the
parked-collider widening to `other.big_radius × 3` with the `tolerance < t`
guard the original also has (`do_move@005f7b30:522-524`) and the re-push of
the top; the arrival test after it.

**§5.2 — the two `find_nearby_spot` queries.** `find_collision`'s land arm
as `collide_here` and nothing else, sea/air as the 3×3 chain walk on current
positions (`collide.rs:812-817`); `find_ordered_collision` as the same walk
on `orders_pos` (`:868-870`); both skipping the caller and requiring
`who < 8` (`:900-902`); `coll_size 0 → false` (`:889-891` via `chain_hit_size`, and `collide_here`'s own `:485-488`); the
call site running the pair last, after the passability tests
(`orders.rs:3248-3258`).

**§5.2.1 — `find_unit_with_radius`.** The single predicate
`vector_dist(spot − it) <= its big_radius + r_coll` over live, on-map units
of players (`collide.rs:846-856`), `r_coll` from the asking type's
`block_radius`, no caller exemption, no ordered sibling, and the `Seeker::Type`
call site that selects it (`orders.rs:3258`).

**§6 — `resolve_unit_collision`.** Step 0's whole-queue clear and its
`is_gaia` stand-in (`collide.rs:943-946`, `orders.rs:801-806`). Step 2's
five conjuncts — same player, a `GATHER` action, an **active** build,
`is_flat`, `covers_tile` of my tile — then `kill_current_order`
(`:960-972`). Step 4's two candidate pairs, diagonal
`(c.x, c.y − dy)` then `(c.x − dx, c.y)`, cardinal `(c.x − dy, c.y − dx)`
then `(c.x + dy, c.y + dx)`, each filtered by `invalid_loc` then a
`quick 1, nocoll 0` probe, the winner pushed as `{cell centre, tol 0,
flags 2}` (`:1041-1073`, matching `005f9d30:296-334`). Step 5's `collide += 1`
and `collide_frame = frame` before the branch; the `0x20`/`0x80` cap on
`repaths[who] > 3`; both `collide` tests; `is_enemy`; the "it is not colliding
with me" test (`:1005-1007`, matching `005f9d30:393-396`); and the
`FLEE_TO` exclusion, which the code has and the document states loosely.
Step 6's whole throttle — `repaths < 4 ? collide : ((o + collide) & 3 != 0 →
return; collide/4)`, then `repaths >= 0x10 → return` and `repaths >= 8 &&
(o + n) & 0xf != 0 → return`, then `repaths += 1` (`:1080-1096` against
`005f9d30:417-431`); the unwind's stop rule (`FINAL`, or `tolerance >= 0x60`
and no `flags & 2` and neither `0x6000`-masked nor `collide_here`-blocked)
with the last popped pushed back (`:1098-1117`); the cell-centre snap through
`set_new_location(…, move_guys = true)` and its crew seating; `dest = 0` on
**both** arms and the waypoint left for `do_move` (`:1118-1141`, item 204);
and the pause roll `Random::get(0, 0xffff) % 9 + 1` guarded on the collider
naming me back and not already waiting (`:1153-1175` against
`005f9d30:492-501`). `Unit::do_idle`'s `collide = 0` (`orders.rs:1318`).
The throttle's per-frame decay — halve, snap to zero under three — at the
top of the frame (`lib.rs:2726-2731`).

**Blind-list check.** None of the functions COLLISION.md cites by address
appears in `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`, so
every row above is **reached**: some run on disk has executed the original
function. (`Unit::do_form_change@005e8670` *is* on that list, which is the
independent confirmation that R3's `CHANGE_FORM` path has never run.)
