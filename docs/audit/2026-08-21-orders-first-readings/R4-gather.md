# R4 — `GatherOrder`, gathering, and `Unit::work`

First reading, from the export under `~/ghidra-projects/decomp`. Every address
is `Class::method@addr`; field names are `types.txt`'s unless marked as a raw
offset. Nothing here is transcribed; where a claim turns on an expression it is
quoted short.

**Headline.** A gather order is a *registration*, not a carry cycle. Income is
paid by the economy from the building's gatherer chain (`BuildData::gather_down`
→ `UnitData::gather_down` → …), which `Unit::add_gather_order` pushes the unit
onto the moment the order is issued and `Unit::kill_current_order` pops it from
when the order dies. `BuildData::calc_gather` counts that chain with
`num_gatherers(1, 1)` — only members whose order has `been_there` set. The
per-frame `Unit::do_gather` is the choreography that gets the unit next to the
building, sets `been_there`, and then keeps it busy (farm tiles, wood/ore
trips, the oil-well stance); none of that motion feeds the rate. Three of its
branches draw from the game RNG, so the motion is still lockstep-relevant.
The `GatherPoint` list on a building is its **rally point**, nothing to do
with gathering slots.

---

## 1. What it is

### 1.1 `GatherOrder` (size 0x34) — `GatherOrder::GatherOrder@00487390`

`GatherOrder : TargetOrder : virtual UnitOrder`. The layout is settled by the
constructor's writes and by reading `log_data`/`clear` with `this` at the
virtual-base subobject (see 1.2):

| offset | field | set by ctor / `clear@00487340` | meaning |
| --- | --- | --- | --- |
| `+0x0` | vptr | `TargetOrder::vftable_for_TargetOrder_` | |
| `+0x4` | vbptr | (shared vbtable, COMDAT-named `TradeOrder::_vbtable_`) | virtual base at `+0x2c` |
| `+0x8` | `TargetOrder::ox` | −1 | the building's object index |
| `+0xc` | `TargetOrder::whom` | −1 | the building's owner |
| `+0x10` | `TargetOrder::uid` (ushort) | 0xffff | the building's `ObjectData::uid`, for staleness |
| `+0x14` | `tx` (TCoord) | −1 | **tiles**: the resource tile a woodcutter/miner is working |
| `+0x18` | `ty` | −1 | |
| `+0x1c` | `build_type` | −1 | the building's `TypeIndex` (FARM 0x1a1, WOODCUTTER 0x1a2, MINE 0x1a3, UNIVERSITY 0x1a4, OILWELL 0x1a5, OILPLATFORM 0x1a6, …) |
| `+0x20` | `wait` | 0 | frames; the non-flat state machine's timer (see §3.3) |
| `+0x24` | `goto_build` (uchar) | 1 | non-flat: 1 = heading to / at the camp, 0 = out at the tile |
| `+0x25` | `non_flat_gather` | 0 | **write-only**: set by `add_gather_order`, logged, never read |
| `+0x26` | `dist_mod` | 0 | non-flat tile-choice weight: 10 for a MINE, 4 otherwise; decremented on each failed tile |
| `+0x27` | `been_there` | 0 | the unit has arrived; **the economy counts only these** |
| `+0x28` | vtordisp (int) | 0 | MSVC plumbing |
| `+0x2c` | `UnitOrder` vptr | `UnitOrder::vftable` then the derived one | |
| `+0x30` | `UnitOrder::flags` (char) | 0 | bit 4 = the issuing call's last argument (`add_gather_order`'s `param_3`); the THINK order always has it |

`TargetOrder` alone is 0x20: `ox/whom/uid` at `+0x8..+0x11`, vtordisp `+0x14`,
vbase at `+0x18`. `GatherOrder::get_type@00487310` returns `GATHER`, whose
numeric value is **7** (`UnitData::is_gathering@006089b0` compares
`get_type() == 7`; with `do_job`'s case order this puts a gap at 6). The
`OrderIndex` enum is not in the export's `enums/`; the values used below are
inferred from `do_job` and the discriminators in `kill_current_order`/`work`
(MOVE_TO 1, FLEE_TO 2, ATTACK_TO 3, EXPLORE_TO 4, BUILD_AT 5, GATHER 7,
ATTACK 10, GUARD 12, CAST_SPELL 14, TRADE_ROUTE 15, STRAFE 16, CHANGE_FORM 18,
GROUP_MOVE 19, GROUP_ATTACK_TO 21, SPECIAL_ANIM 25) — consistent everywhere I
looked, but an enum dump would make it certain.

`GatherOrder::operator=@0072f490` copies the eight own fields, then
`ox/whom/uid` and the `UnitOrder::flags` through the virtual-base accessors.

### 1.2 A trap worth recording: the `this` of a vtordisp method

`GatherOrder::log_data@00486eb0`, `clear@00487340` and the `TargetOrder` pair
are compiled with `this` pointing at the **virtual-base subobject** (`A +
0x2c` for a GatherOrder, `A + 0x18` for a TargetOrder), and Ghidra types that
pointer as `GatherOrder *`, so every field prints as `this[-1].<wrong name>`.
Calibrate on `TargetOrder::log_data@0047f3a0`: `this[-1].whom + 4` is the
vbptr deref, so `this = A + 0x18` and `this[-1].uid` is really `ox`. Apply the
same shift to `GatherOrder::log_data` and the "`build_type`, `wait`,
`goto_build…`" it appears to log are really `tx`, `ty`, `build_type`, and the
`field_0x28…0x2f` it logs are `wait`, `goto_build`, `non_flat_gather`,
`dist_mod`, `been_there` — which is also what the key lengths say (10, 15, 8,
10). The vtordisp thunks (`…vtordisp{4294967292,0}'`) subtract the
displacement at `A+0x28` before the call. If another reader's order log looks
"shifted by two fields", this is why.

### 1.3 The building side: the gatherer chain

`BuildData` (CITIES.md §1.2 has the rest): `+0x70 gather_down` (short, the
head of the chain, −1 empty), `+0x80 gather_max` (char, the slot count),
`+0x98 gather_from` (a `MiningList` — an `Array<TCoordData>` of resource tiles
around a woodcutter/mine, `+0x9c` its count, `+0xb4 mtn`, `+0xb5 cliff`),
`+0xb8 gather` (the `GatherPointList` — the **rally point** list, §1.5).
`UnitData +0x92 gather_down` (short) is the *next* link.

- `Build::add_gatherer@0062f640(o, who)`: same owner; `num_gatherers(0,0) <
  gather_max`; the unit exists (vslot 8) and is on the map (vslot 0xbc); its
  `TypeIndex ∈ {PEASANTS 0x32, PEASANTSKOREAN 0x33, SCHOLARS 0x34,
  SCHOLARSKOREAN 0x35}`; not already in the chain → `check_gatherers()` then
  **push-front**: `unit.gather_down = build.gather_down; build.gather_down = o`.
- `Build::remove_gatherer@0062f8d0(o, who)`: unlink `o` from the chain
  (walking stops at the first dead unit).
- `Build::check_gatherers@0062f710`: prune every member that is dead, not
  `is_gathering_at(this)` or not on the map; returns the new head.
- `BuildData::is_gathered_by@0062f520(o)`: chain membership.
- `BuildData::num_gatherers@00630450(arrived, skip_decoys)`: `count_inside(…,
  PEASANTS)` for an OILPLATFORM and `count_inside(…, SCHOLARS)` for a
  UNIVERSITY (garrisoned gatherers), **plus** every chain member for which
  `is_gathering_at(this, arrived)` holds, skipping `unit_masks & 1` (a decoy,
  COMBAT.md) when `skip_decoys`. **`BuildData::calc_gather@0062d360` calls
  `num_gatherers(this, 1, 1)`** — so the economy counts arrived, non-decoy
  chain members plus the garrisoned ones. Every UI/AI caller uses `(0, 0)`.
- `UnitData::is_gathering_at@00608880(o, who, strict)`: a scholar off the map →
  `get_inside() == (o, who)` (then `strict` → 0 if not); on the map, a
  citizen/scholar whose **first** order is GATHER → `get_action()`'s gather
  order has `(ox, whom) == (o, who)` → `strict ? been_there : 1`.
- `Build::all_gathering@0062f570`: `check_gatherers`, then every member's first
  order must be a gather order with `goto_build == 0` and `wait ≥ 0` (all out
  at their tiles); empty chain → 1.
- `BuildData::max_gatherers@00630590` returns `gather_max`; `gather_max` is
  written by `Build::update_max_gatherers@00623310` / `find_gather_tiles@
  00623350` from `BuildTypeData::max_gatherers@0063c430` — **1 for a flat type**
  (`build_flags & 0x10000000`), else `calc_gather`'s slot output (ECONOMY.md's
  open item). `max_flat_gatherers` is a constant 1, `max_knowledge_gatherers`
  a constant 7 (so a university's slot count is not what its name suggests —
  `max_gatherers` goes through `calc_gather`).

`Build::find_gather_tiles@00623350` (non-flat, non-university): fills
`gather_from` from `BuildTypeData::find_gather_tcoords`, sets the world tile bit
`0x1000` ("gathered at", `World::set_gathered_at@006b46b0`) on each, then **if
the list grew, appends `count × 4` entries by picking `Random::get(game_random,
0, 0xffff) % count` and moving that entry to the back** — a shuffled,
fourfold-replicated tile list, drawn from the sync stream at building
creation. `do_non_flat_gather` ranks tiles by `i >> 2`, i.e. by that order.
(This is R-cities' territory — noted because the RNG draw is part of the same
stream the gather choreography draws from.)

### 1.4 Who gathers

`UnitData::can_gather@00608850`: the four citizen/scholar types above. The
citizen AI (`think_peasant`) and the chain gates both use the same four.

### 1.5 `GatherPoint` / `GatherPointList` are the rally point

`GatherPoint` (0x10): vptr, `x`, `y` (Coord), `action` (uchar). `Build::
add_gather_point@00622e70(x, y, action, pos)` appends (`QUEUE_NEW` clears
first); `clear_gather@00623180`; `replace_gather@00622d10`; `get_first_gather`,
`num_gather`, `gather_inside@0046f180` (`action != 3 && (x < 0 || y < 0)`
= "rally inside"). `action == 3` means "rally onto an object": `x` is then
the object index and `y` its owner. Writers: `Group::action_gather_point`,
`action_city_gather`, `ScenarioFuncSet::set_rally`, `Build::train`. Reader:
`Unit::come_out@00617c10` — which, for a rally point aimed at a same-owner
**gather building**, issues `add_gather_order(o, QUEUE_LAST, 1)` to the unit
that just came out (peasants for a non-university, scholars for a university;
an unfinished site gets `add_build_order` instead). So "gather point" is
PRODUCTION.md's rally point and the only link to gathering is that exit rule.
Logged as `GATHERPOINT { X, Y, ACTION }` inside `BUILDDATA`.

### 1.6 Lifecycle

Created by `Unit::add_gather_order@0061a5c0` (§2). Stepped by `Unit::work` →
`do_job` → `Unit::do_gather@005ef2a0` every frame the unit's first order is a
GATHER (§3). Ends through `Unit::kill_current_order@005e2cb0` (§3.5), which is
where the chain link is removed.

---

## 2. Creation — `Unit::add_gather_order@0061a5c0(o, pos, flag)`

1. `pos == QUEUE_NEW`: `unit_masks &= ~0x4000000`; `path.length = 0`
   (`Stack<PathData> +0x8`); `close_orders(0)`; `clear_partial_path()`;
   `update_action()`.
2. `unit_masks &= ~0x400` (a citizen-AI bit, cleared here and in
   `think_peasant`; meaning not established).
3. `OrdersMemManager::get_obj(GATHER)` → `ox = o`, `whom = this.who`; `o < 0`
   → `build_type = −1, uid = 0xffff`; else `uid = objects[who][o].uid`,
   `build_type = objects[who][o].ptype->type_index` (`+0x18 → +0x4`).
   `tx = ty = −1`, `non_flat_gather = dist_mod = 0`.
4. **If the target's type `is_gather_type`** (BuildType vslot `0x90`,
   `build_flags & 0x40`): `Build::add_gatherer(this.o, this.who)` — the unit
   joins the chain *now*, before it has moved. Then if the type is not flat
   (vslot `0x94`) and the building is not a UNIVERSITY: `non_flat_gather = 1`,
   `dist_mod = is(MINE) ? 10 : 4`. (A university is non-flat but is gathered
   from inside; an oil platform is treated like wood/ore here but `do_gather`
   never dispatches on `non_flat_gather`.)
5. `UnitOrder::flags` bit 4 ← `flag`.
6. If the unit's tile and the building's tile are in different `tregion`s
   (`WorldData::get_tregion`) and the unit may be transported
   (`UnitData::transport_type(this) ≤ age`, where age is 3 / 2 / 1 / 0 from
   `leader_flags & 0x100 / 0x200 / bit 10`, and `can_ever_transport`):
   `unit_masks |= 0x800000` — the civilian-transport request that
   `think_civilian_transport` reads (MOVEMENT.md's territory).
7. `orderlist.add(order)` (`LinkListBase<UnitOrder*,…>::add`); `update_action()`.
   Only `QUEUE_NEW` is consulted here; FIRST and LAST both go through `add`
   (R1 owns what `add` does with the cursor).

**Callers and what they pass** (all citizens/scholars unless noted):

| caller | args | when |
| --- | --- | --- |
| `Setup::build_units@005aafc0` | `(site_o, QUEUE_NEW, 0)` | game start, the "pre-placed sites" mode: each starting citizen is `init_unit` at a starting building `o ≥ 0x7d1`, `come_out(0)`, then given a gather order on that building unless `game.info.starting_resources == 8` — **this is the order that moves the human's citizens `o 1…5` toward objects `2001…2005` on the dump's frame 4, the divergence DATALAYER.md §3 reports** |
| `Unit::build_done@00603bf0` | `(site_o, QUEUE_NEW, 0)` | the finished site `isnt(OILPLATFORM, 1)` (Build vslot `0x144` = `ObjectData::isnt`) and `num_gatherers(0,0) < gather_max` (lines 77–87); otherwise `find_build_spot` / `find_repair_spot` / `find_gather_spot(UNIT_GATHER_RESPOND_RANGE × 0xc0)` |
| `Unit::do_build@005eebf0` | `(o, QUEUE_NEW, 0)` | target already active, same owner, gather type, not a UNIVERSITY (CITIES.md §3.3 already says this) |
| `Unit::do_repair@005ee420` | `(o, QUEUE_NEW, 0)` | repair finished on a gather type that is not (the `[0x2e]` call with no visible arg — UNIVERSITY by analogy; not verified) |
| `Unit::come_out@00617c10` | `(rally_o, QUEUE_LAST, 1)` | §1.5 |
| `Unit::find_gather_spot@005f5170` | `(best_o, QUEUE_LAST, 0)` | §2.1 |
| `Group::action_gather@00700b90` | `(o, QUEUE_LAST or QUEUE_NEW, 1)` | the group command (UI and AI): per member, target exists, active, `is_gather_type`; `unit.group = −1`; queued vs new by the shift flag; a castable spell `0x293` may be cast first — arguments only, AI is R7's |

### 2.1 The citizen AI that issues it — `think_peasant`, `find_gather_spot`

`Unit::do_think_order@005e5bf0` (the THINK order that `do_gather` queues when
it gives up) kills itself and, if no typed order follows, calls
`think_peasant(1)`; the idle path is `Unit::think@005f6e40` → `think_peasant(0)`
gated by the idle counter (`UnitData +0xb0 idle`, reset in `work` whenever an
order exists) against a threshold from the player's citizen-automation option
(1…5 → 7, 12, 17, 32, 62 idle ticks; default 2; `unit_masks & 0x40000` → 1),
retrying when `idle == thr` and then at `idle ≡ 2 (mod 5)`.
`Unit::think_peasant@005f5760`: civilian transport first (peasants with
`0x40000`); `find_build_spot` for worker stance; then
`find_gather_spot(range)` with `range = UNIT_GATHER_RESPOND_RANGE × 0xc0`
(−1 = unlimited when `0x40000`); then `find_repair_spot`; scouting for the
`0x40000` case.

`Unit::find_gather_spot@005f5170(range, …)`: walk the owner's buildings from
`obj_base[1]`; keep those that exist, are active, `is_gather_type`, not
neutralized, university-iff-scholar, with `num_gatherers(0,0) < gather_max` or
already gathered by this unit, in the unit's `tregion`; for a non-scholar in a
city, skip a building of another city unless this city has `free + gatherers
< 2` or the other city has at least two more (`CityData +0x5a free`, `+0x5c
gatherers` — AI bookkeeping); within `range` (if `range > 0`); **score =
`(Σ_goods rate_g) × 500 / (dist / 0xc0 + 2)`**, where `rate_g` is the leader's
per-good rate for that building's `best_gather_type` (read from
`LeaderDataEncrypt` with the XOR masks ECONOMY.md lists; `+0x640` for the
`has_tribe_bonus(0x16)` nation); pick the max; if not already gathering there →
`add_gather_order(best, QUEUE_LAST, 0)`. Integer division throughout.

---

## 3. The per-frame step

### 3.1 Where `do_gather` is reached — `Unit::work@0060d180`

`Unit::process@00610bc0` line 538 calls vslot `0x188` = `Unit::work` after
healing, cloak, `process_attrition`, the supply check and the bleed
(MOVEMENT.md "Where movement sits in the frame" already pins this); the
`Guy::process` loop follows. `work`:

1. Rewind the order list (`orderlist.head_node->next` → current) and take
   `type = current.get_type()` or NONE. Every 32 frames phased by `o`:
   `visible = 0` (if `flags ≥ 0`) and `unit_masks &= ~4`. Non-ATTACK: `flags &=
   0x7f`; a melee type (`max_range == 0`, no `unit_flags & 0x400`) still
   recharging returns here unless the order has flag 4.
2. Caravan bookkeeping; for MOVE_TO and the move-like types (ATTACK_TO,
   EXPLORE_TO, FLEE_TO, CHANGE_FORM, GROUP_MOVE, GROUP_ATTACK_TO) the
   move-order maintenance: `unit_masks & 0x4000000` → convert to
   `add_move_facing_order`; every 16 frames phased by `o`, `update_action()`
   and, for a target order, `check_target_path`; GUARD repaths every 64.
3. `update_action()` → the **action** (the first order that is not a pathed
   move without flag 4 and not CHANGE_FORM). If it is a target order with
   `(ox, whom) ≥ 0`: ATTACK → `set_in_danger`; **if the target's `uid`
   (`ObjectData +0x30`) differs from the order's `uid`**: STRAFE clears its
   target; an `is_attack` order is re-validated; **anything else (a gather
   order on a building whose slot was reused) → `repath(); kill_current_order();
   return`**. A TRADE_ROUTE validates its second endpoint the same way.
4. Launch, `safe--`, `idle = 0` if an order exists, boat-collision, then
   **`do_job(type, action)`** (`Unit::do_job@00617a10`): `BUILD_AT → do_build`,
   `GATHER → do_gather`, `REPAIR → do_repair`, `THINK → do_think_order`, the
   moves to `do_move`, and so on — `work` is the generic per-frame worker for
   *every* order type; `Job::{Build, Repair, Garrison}` in `crates/sim` are
   three of its cases and GATHER is the fourth. The unit-spreading for
   `unit_flags & 0x20` types follows; `unit_masks &= ~0x10` last.

So `do_gather(order)` is called with the **current (first) order**, once per
frame, on the frame's `type` as read at the top of `work` — an order it adds
`QUEUE_FIRST` is acted on next frame.

### 3.2 `Unit::do_gather@005ef2a0(order)` — the flat/arrival logic

Shorthand: `b` the target building, `u` this unit, `n = b.num_gatherers(0,0)`,
`max = b.gather_max`, "kill" = `kill_current_order(0)`, "think" =
`add_think_order()`, "move(x, y)" = `add_move_order(x, y, 1, 0, QUEUE_FIRST,
0, …, −1, −1)` (the seam, §4). All positions are the raw XOR-unmasked Coords;
tiles are `div_3_table[coord >> 6]`.

```
go = order.get_gather_order()                            # vslot 0x6c (0xe4 const)
if go.whom != u.who                          → kill; return
b = objects[whom][ox]; if !b.is_live_build || !b.activated → kill; return   (vslot 0xc = the alive bit, 0 on a unit; 0x4c = WallData::is_active)

if !go.been_there and !b.is_gathered_by(u.o):
    if n < max:
        if go.build_type == OILPLATFORM or u.type.domain != 0 → (arrival check)
        b.add_gatherer(u.o, u.who)
        if !b.is_gathered_by(u.o) → FAIL
    else → FAIL
    FAIL: kill; think;
          if b.type.is_flat and !b.covers_tile(u.tile): move(b.x, b.y)   # stand by the full farm, then think
          return
(arrival check)
farm = −1
if build_type == FARM:
    if b.city < 0 → FAIL                                  # a farm must belong to a city
    farm = b[+0x78]                                       # the Farms slot
if build_type ∈ {MINE, WOODCUTTER}:
    every 128 frames phased by u.o*4: if n > max → kill; think; return   # over-subscribed
    do_non_flat_gather(go); return                        # §3.3
if build_type != FARM:                                    # university, oil well, oil platform, …
    if !u.adjacent_to(ox):                                # Object vslot 0x170
        d = min(b.type.x_size, b.type.y_size) * 0x60 + 0x30
        if b.type.domain == u.type.domain or !u.can_transport():
            if find_nearby_spot(b.x, b.y, &sx, &sy, d, −1, 0, angle, FILTER_NOT_ME, u.o, u.who, 0,0,−1,0,−1) != 0
                                                          → kill; u.flags |= 0x10; return   # no room
        else (sx, sy) = (b.x, b.y)                        # a boat: aim at the platform itself
        if !b.is_gathered_by(u.o) and !b.is(OILPLATFORM) → go.been_there = 0; return
        move(sx, sy); return
    → ARRIVED
else (FARM):
    if !b.covers_tile(u.tile):
        if tregion(u) != tregion(b) and !u.can_transport() → kill; flags |= 0x10; return
        if n > max                                        → kill; flags |= 0x10; return
        if u.type.domain == 0 and !b.is_gathered_by(u.o)  → go.been_there = 0; return
        move(b.x, b.y); return
    → ARRIVED

ARRIVED:
if !go.been_there:
    if n > max → kill; flags |= 0x10; return
    go.been_there = 1; leader_flags[who] |= 0x2000000     # the economy's dirty flag (ECONOMY.md)
    scholar:                go_inside(ox, whom, 0); b.check_gatherers(); kill; return
    non-scholar, OILPLATFORM:
        u.can_carry(GROUND) (the civilian transport): inside_down ≥ 0 → same_damage(carried, …);
            go_inside(ox, whom, 0); kill; u.die(); b.check_gatherers(); return
        else go_inside(ox, whom, 0); b.check_gatherers(); kill; return
    non-scholar, other:     b.check_gatherers()           # fall through
# gathering, every frame from here:
if unit_masks & 0x40000 (citizen-AI bit) and every 256 frames phased by (o + frame + who)
   and get_diff(leader) > 1 and b.damage != 0 and !(b.build_masks & 0x20) and b.city ≥ 0
   and !(city.city_flags & 2):  add_repair_order(ox, whom, QUEUE_NEW, 0); return
if build_type != FARM:                                    # in practice the OILWELL
    if !(b.build_masks & 0x800): b.recharging++; b.build_masks |= 0x800   # one worker per frame (CITIES.md §1.3)
    if guy0.cur_anim == '%' → return
    set_anim(CHAR_FARM, 0, 1); set_angle(0x40000000, …); set_new_location(b.x + 0x120, b.y, 1, 1); return
FARM: §3.4
```

Notes on predicates:

- The two gates are Object vslots `+0xc` (on a Build `SubObjectData::
  is_active`, the `flags & 1` alive bit; on a Unit a folded `return 0`, so it
  doubles as "is a building") and `+0x4c` (`WallData::is_active`, the
  activated state); an unfinished or dead building kills the order.
- The chain is joined at issue time (§2) so the "join here" branch is the
  retry for a unit that was refused then (e.g. the building was not active, or
  was full). A unit that joined but is not in the chain by the time it arrives
  (someone pruned it) gets `been_there = 0` and waits without moving.
- `SubObjectData::flags |= 0x10` (`+0x8`) marks "could not reach"; `Unit::think`
  reads it (`think@005f6e40` line 87).
- University and oil platform: the gatherer **garrisons** (`Unit::go_inside`,
  CITIES.md §6.5) and the order ends; `num_gatherers` then counts it through
  `count_inside`. A land citizen reaching a platform by water arrives as its
  transport barge, which puts the carried citizen inside and dies.
- `go.wait`, `tx/ty`, `goto_build`, `dist_mod` are **not read on this path**;
  they are the non-flat machine's.

### 3.3 `Unit::do_non_flat_gather@005f0170(go)` — wood and ore

Reached only for WOODCUTTER/MINE. Two sides, selected by `goto_build`. `W` =
`is(WOODCUTTER)`. `rnd` = `Random::get(game_random, 0, 0xffff)`.

```
if go.been_there: if !(b.build_masks & 0x800): b.recharging++; b.build_masks |= 0x800
u.group = −1
if goto_build == 0:                                       # out at / heading to the tile
    unit_masks &= ~0x78000000
    if !been_there: been_there = 1; leader dirty
    if wait < 0:                                          # "return to the camp"
        d = min(x_size, y_size)*0x60 + 0x30
        if find_nearby_spot(b.x, b.y, &s, d, −1, 0, angle, NOT_ME, …) != 0:
            set_anim(CHAR_CHOP_WOOD); wait = 20; return    # no spot by the camp: chop in place
        set_anim(CHAR_DEFAULT); move(s); goto_build = 1; wait = 32
        unit_masks |= W ? 0x8000000 : 0x20000000; return
    a = guy0.cur_anim
    if a == 0x1d → return
    if a == 0x19: wait--; if wait != 0 → return
                 if b.all_gathering(): wait = −1 else wait = 300 + rnd % 100; return
    T = (tx*0xc0 + 0x60, ty*0xc0 + 0x60)                  # the tile centre
    if vector_dist(u, T) < 0x140:                         # working the tile
        wait--; if wait == 0: if all_gathering(): wait = −1; return
                             wait = 100 + rnd % 50
        face T (set_angle if the angle differs)
        if u.hold_doober (+0x86) < 0: add_hold_doober(tree/ore effect at T, or at the midpoint for W)
        set_anim(W ? CHAR_CHOP_WOOD : CHAR_MINE_ORE); return
    set_anim(CHAR_DEFAULT)
    if find_nearby_spot(T, &s, 0xc0, 0x100, 2, angle, NOT_ME, …) != 0
       or s == u.pos or s == (u.avoid_x, u.avoid_y):
        tx = ty = −1; goto_build = 1; wait = −1; if dist_mod: dist_mod--   # give the tile up
        worker with a hold doober → remove it; return
    if !b.is_gathered_by(u.o): been_there = 0; return
    move(s); unit_masks |= W ? 0x10000000 : 0x40000000; return
else (goto_build == 1):
    if wait ≥ 0:
        if u.adjacent_to(ox):                             # at the camp: "dump"
            wait--; been_there = 1 (+dirty); if wait < 0 → return
            face the camp; set_anim(W ? CHAR_DUMP_WOOD : CHAR_DUMP_ORE); return
        if !b.is_gathered_by(u.o) and u.domain == 0: been_there = 0; return
        if tregion differs and can_transport: move(b.x, b.y); return
        d as above; if find_nearby_spot(b, &s, d, −1, 0, …) == 0: move(s); return
        if !been_there and vector_dist(u, b) > 0x600:
            if find_nearby_spot(b, &s, 0x600, −1, 0, …) != 0: kill; return
            move(s); return
        wait = −1; return
    # wait < 0: choose a tile
    if !been_there: been_there = 1; dirty
    set_anim(CHAR_DEFAULT)
    if tx < 0 or ty < 0 or !has_gather_access(tx, ty, who, 1, 0):
        (a citizen/scholar's hold doober is removed)
        dm = dist_mod; if b.gather_from.mtn ≥ 0 and count < MTN_TINY_SIZE and dm > 3: dm = 3
        best = 9999999
        for i in 0..count: (x, y) = gather_from[i]
            if world.tdata[y][x].mask & 0x4000 and has_gather_access(x, y, who, 1, 0):
                score = max(3, vector_dist(|x − b.tx|, |y − b.ty|)) * dm + (i >> 2)
                if score < best: best, pick = score, i
        if best unchanged → return                        # nothing reachable
        move `pick` to the back of gather_from; go.tx, go.ty = pick
    wait = 400 + rnd % 200; goto_build = 0
    if !W: unit_masks |= 0x40000000; wait = 1000000; return # a miner stays out essentially forever
    unit_masks |= 0x10000000; return
```

What this means in plain terms: a woodcutter alternates between chopping at a
tile for 400–599 frames (re-checking every 100–149 until every peer is out) and
walking back to the camp for a 32-frame dump animation; a miner walks out once
and stays. `all_gathering` is what makes the return trips stagger. The
"PANCAKE" error reports are bounds asserts and never fire on a sane map.
**None of it touches income**; the resource is paid from the chain while
`been_there` is set, which happens on the first arrival at the camp (or first
frame out at a tile, or — `goto_build == 0` — immediately).

RNG: three draws here (`% 100 + 300`, `% 50 + 100`, `% 200 + 400`), all from
`game_random`, so a sim that wants the combat stream to line up has to take
them in order. Every other quantity is integer: distances in position units
(192 per tile, 0x60 = half a tile, 0x140 = 320, 0x600 = 8 tiles), `vector_dist`
as MOVEMENT/COMBAT define it, `>> 2` on the list index.

`WorldData::has_gather_access@006b4e50(tx, ty, who, 1, 0)`: the tile has the
high mask bit (`0x8000`), its `wdata` owner is −1 / `who` / an ally, and at least
one of the four orthogonal neighbours (the `orthog_x/y` table) is neither of
terrain class `(mask & 0x30) == 0x20` nor itself a resource tile (`& 0x4000`).
That is the whole "is this tree reachable" test.

### 3.4 The farm choreography (`do_gather`, FARM, arrived)

```
ft = FarmsData::get_farm_type(farm)          # farms.farm_data[farm] +0xbd & ANIMAL_FARM
xs, ys = b.type.x_size, y_size; (cx, cy) = b.tile_corner()
if ft == ANIMAL_FARM (1):
    set_anim(CHAR_SOW)
    every 256 frames phased by (o*7 + frame + who):
        x = (cx + 1 + GameAccess::rnd(?)) * 0xc0 + 0x60; y = (cy + 1 + GameAccess::rnd(?)) * 0xc0 + 0x60; move(x, y)
    return
dx = u.tx − cx; dy = u.ty − cy; out of [0,xs)×[0,ys) → error report, dx = dy = 1
state = farms.farm_data[farm].state[dy][dx]             # byte at FarmStruct +0xac + dy*4 + dx
0: if guy0.cur_anim != '$' → as 1;  else → new tile
1: set_anim(CHAR_SOW); Farms::grow(farm, dx, dy); return
2: if guy0.cur_anim != '#': set_anim(CHAR_REAP); Farms::snip(farm, dx, dy); return;  else → new tile
3: set_anim(CHAR_REAP); return
new tile: x = (cx + GameAccess::rnd(?)) * 0xc0 + 0x60; y = (cy + GameAccess::rnd(?)) * 0xc0 + 0x60; move(x, y)
```

`GameAccess::rnd@0043cca0(n)` is **`Random::get(game_random, 0, 0xffff) % n`**
— the sync stream, not a second one (COMBAT.md §10 says it is "a second,
unsynchronised stream"; that is wrong, and should be amended — the only thing
special about it is the `% n` and `n ≤ 1 → 0` without a draw). The modulus `n`
is passed in ECX and the decompiler lost it; from the geometry it is the farm's
tile extent (3), but that is inferred.

`Farms::grow@008d91c0` sets `state = 1` and adds **`0.005f` to a `float[4][4]
percent`** until it reaches `1.0f`, when `state = 2`; `snip` turns 2 into 3
(something else, presumably `Farms::process`, regrows 3 → 0 — not read). The
state byte decides when the farmer stops reaping and re-targets, and the
re-target draws from `game_random` — so **a float accumulator sits upstream of
the sync RNG** (200 adds of 0.005f does not land exactly on 1.0f; whether the
flip happens on the 200th or 201st add is a float question). For the sim this
is a count-to-N, and N needs a behavioural check (§6).

### 3.5 How a gather order ends

- `Unit::kill_current_order@005e2cb0` on a GATHER (type 7): `leader_flags |=
  0x2000000`; `(o, who)` from the order; the unit concerned is `this`, or the
  carried unit (`inside_down`, `inside_down_who`) when `unit_flags & 0x10`
  and `inside_down ≥ 0` (the transport barge carrying a citizen); if the game
  is playing and not loading, `o ≥ 0`, the object is a live building (vslot
  `0x10`, the same folded alive bit) and its owner matches → **`Build::remove_gatherer(unit.o)`**; then
  `unit_masks &= ~0x78000000`, a citizen/scholar's hold doober is removed.
  Then the order node is popped (`remove_current`; a pathed order kills its
  path), `give_obj`, `clear_partial_path`, `update_action`. It is called by
  `do_gather` in every "give up" branch, by `work` on a uid mismatch, by
  `close_orders` (QUEUE_NEW from any `add_*_order`), by `do_think_order`, and
  by `Unit::clear_orders`.
- The chain also self-prunes: `check_gatherers` on every `add_gatherer` and on
  arrival; `num_gatherers` ignores members whose first order is no longer a
  gather at this building — so a citizen re-tasked without `kill_current_order`
  stops counting immediately and is unlinked the next time the chain is walked
  with pruning.
- Building destroyed: `do_gather` kills on `!exists || !is_active`; `work`
  kills on a `uid` mismatch once the slot is reused. The building's own
  `Build::close` (CITIES.md §9.1) does not walk the chain.
- Full: `add_gatherer` refuses (`n ≥ max`); `do_gather` then kills and queues
  THINK, which runs `think_peasant` → `find_gather_spot`.

---

## 4. The seams

| call | from | what it expects |
| --- | --- | --- |
| `Unit::add_move_order(x, y, 1, 0, QUEUE_FIRST, 0, ?, −1, −1)` | `do_gather`, `do_non_flat_gather` | MOVEMENT.md's `MoveOrder` placed before the gather order; `do_move` → `move_step` runs it on following frames and pops it on arrival, after which `work` sees GATHER again. The `1, 0` are the tolerance/flag pair MOVEMENT.md documents; the `?` is a dead register in the decompile. |
| `UnitType::find_nearby_spot(x, y, &sx, &sy, min, max, step, angle, FILTER_NOT_ME, o, who, 0, 0, −1, 0, −1)` | both | **returns 0 and writes `(sx, sy)` when a clear quarter-tile spot exists in the ring `[min, max]`** (max −1 = type default), nonzero when none. Confirmed against the university branch (nonzero → `flags |= 0x10`, kill). Owned by movement/cities. |
| `Object::adjacent_to(o)` (vslot `0x170`) | `do_gather`, `do_non_flat_gather` | CITIES.md §3.3's adjacency (footprint-aware). |
| `WallData::covers_tile(tx, ty)`, `tile_corner` | `do_gather` | CITIES.md §2.2. |
| `WorldData::get_tregion`, `UnitData::can_transport`, `transport_type`, `can_ever_transport`, `unit_masks 0x800000` | `add_gather_order`, `do_gather` | MOVEMENT.md's civilian transport. |
| `Unit::go_inside(o, who, 0)` | `do_gather` (university, platform) | CITIES.md §6.5 / `garrison.rs`; the gatherer is then counted by `count_inside`. |
| `Build::add_gatherer / remove_gatherer / check_gatherers / num_gatherers(1,1)` | here ↔ ECONOMY | `BuildData::calc_gather@0062d360` line 43 — this is `Site::gatherers`'s real source: the chain length under `is_gathering_at(strict)`, plus garrisoned gatherers. `economy.rs` takes the count as an input today; the count is **`|{u in chain : u.first_order is GATHER(this) and u.been_there and !(unit_masks & 1)}| + inside(PEASANTS on a platform / SCHOLARS in a university)`**. |
| `leader_flags \|= 0x2000000` | arrival, kill | ECONOMY.md's dirty flag: the rate re-assembles at the next `(who + frame) % 8 == 0`. |
| `Unit::set_new_location(b.x + 0x120, b.y, 1, 1)`, `set_angle`, `set_anim` | oil well | MOVEMENT.md (`set_new_location` writes guy 0's `des_x/des_y`); anim/angle are presentation but `angle` is `UnitData +0x50`, which COMBAT reads as facing. |
| `Random::get(game_random, 0, 0xffff)` ×3, `GameAccess::rnd` ×2 per farm re-target | §3.3, §3.4 | COMBAT.md §10's stream. |
| `Doober::add_hold_doober / remove_hold_doobers`, `UnitData +0x86` | §3.3 | effects; the index is sim state only in that it is logged. |
| `Farms::grow / snip`, `FarmsData::get_farm_type` | §3.4 | the farm tile state machine; not documented anywhere yet. |

What `crates/sim` has and lacks: `Job::{Build, Repair, Garrison}` are the
`do_build/do_repair/do_garrison` cases of `do_job`; a `Job::Gather(building)`
needs (a) the chain on the building (`gather_down` as a `Vec<usize>` or an
intrusive next-index on `Unit`, push-front/unlink semantics do not matter for
the count but do for `check_gatherers`' first-dead stop), (b) on the order:
`been_there`, and for wood/ore `tx, ty, wait, goto_build, dist_mod`, (c) the
per-frame arrival test (`adjacent_to` / `covers_tile`) feeding a move order,
(d) `economy.rs` reading the count instead of taking it. The choreography can
be staged: arrival → `been_there` first (that is the income); farm/wood/ore
motion and its RNG draws second.

---

## 5. Where it sits in the frame

`Unit::process` (per unit, per frame): healing → cloak → attrition → supply →
**`work`** (→ `do_job` → `do_gather`: may `add_move_order` QUEUE_FIRST, may
`kill_current_order`, may `go_inside`) → `Guy::process` ×n. The move a gather
step adds is executed by `do_move` from the **next** frame. The building's
own clock (`Wall::process`) clears `build_masks & 0x800` every frame, so the
`recharging++` a gatherer does on an oil well / wood camp / mine is one per
frame with at least one arrived worker — the same counter construction uses
(CITIES.md §3.3) — and it is *not* read by the economy (readers: `log_data`,
`BuildData::BuildData`, `Wall::do_construct`, `inc_time`, the two gather
steps, `fight`, `do_strafe`). `Leader::gather` (every frame per player) reads
the chain through `calc_gather` only on the dirty/periodic cadence.

`Unit::process`'s ordering means a citizen arriving this frame sets
`been_there` this frame and the dirty flag; the rate picks it up at the next
eighth-frame slot, and `do_gather`(Leader) pays it from then.

---

## 6. What the gamelog writes

- `UnitData::log_data@0060b070` writes `GATHER_DOWN` (key length 11; the
  string's address is `0xad965c`) alongside the other shorts, and
  `OrderList::log_data@00730070` at the end: a length key (6 chars) then per
  order a 4-char key = `get_type()` and a 6-char key = the node's metric, then
  the order's own `log_data`.
- `GatherOrder::log_data@00486eb0` writes a `GATHERORDER` block: `TARGETORDER`
  { `UNITORDER` { `FLAGS` }, `OX`, `WHOM`, `UID` } then `TX`, `TY`,
  `BUILD_TYPE`, `WAIT`, `GOTO_BUILD`, `NON_FLAT_GATHER`, `DIST_MOD`,
  `BEEN_THERE` (names from the key lengths and the field order; the block
  names are literal). So a `UNITS=…` log at the right detail shows a citizen's
  whole gather state per frame, `been_there` included.
- `BuildData::log_data@0062e810` writes `GATHER_DOWN` (key length 11 at
  `0xada128`), then `MiningList::log_data` (`gather_from`) and the
  `GATHERPOINT` list (`X`, `Y`, `ACTION`).
- `LeaderData::log_data@006e5110` writes `gatherers` (`LeaderData +0x9c4`, a
  9-char key) and `reg_gatherers[scan]` / `reg_gather_slots[scan]` (the
  per-region AI arrays `+0x115e`, `+0x11de`); `CityData::log_data` writes
  `CityData::gatherers`. Those are the **AI's** tallies (`City::count_gather_
  slots`, `Leader::plan_strategy/create_units/create_buildings`), not the
  economy's count — the economy has no stored gatherer count; it walks the
  chain. To diff "how many are gathering here" from a log, follow
  `BUILDDATA.GATHER_DOWN` → `UNITDATA.GATHER_DOWN` and read each unit's
  `GATHERORDER.BEEN_THERE`.

DATALAYER.md §3's "citizen `o 1` moves on frame 4 toward object 2001" is the
`Setup::build_units` gather order running §3.2's FARM/other branch and adding
the move; the harness will hold still until `Job::Gather` exists. (Whether the
first position change lands on frame 4 exactly depends on `do_move`'s first
frame — R-movement.)

---

## 7. Confidence, what is not established, checks

**High** (read end to end, cross-checked between callers): the struct layout
and defaults; the chain (`add/remove/check_gatherers`, `num_gatherers`'s
three terms, `is_gathered_by`, `is_gathering_at`); that `BuildData::calc_gather`
counts `num_gatherers(1, 1)`; `add_gather_order`'s steps; the `do_gather`
branch structure and its kill/think/move outcomes; the `do_non_flat_gather`
state machine and its four timers; `kill_current_order`'s GATHER case; the
`GatherPoint` = rally point identification; `GameAccess::rnd` being the sync
stream; `work`'s position and the uid check.

**Medium**: the `OrderIndex` numbering (inferred, consistent); the exact
`QUEUE_FIRST`/`LAST` behaviour of `LinkListBase::add` (R1); `find_nearby_spot`'s
return convention (confirmed from one caller's failure branch, and the body's
`return !found`); which `unit_masks` bits `0x400`, `0x40000`, `0x4000000` mean
(read as citizen-AI bits from usage); the rally branch in `come_out`
(the peasant/scholar split is read from the two `add_gather_order` sites, the
surrounding conditions only skimmed); `do_repair`'s "not a university" test
(argument lost).

**Not established**
- The `UnitAnim` codes behind `'%'`, `'$'`, `'#'`, `0x19`, `0x1d` (no enum in
  the export) — they only gate animation re-entry, but `0x19/0x1d` also gate
  a `wait` countdown path, so a sim that models the timers needs them. The
  obvious reading is `0x19 = CHAR_CHOP_WOOD` (set just before that branch
  becomes reachable).
- The moduli of the two `GameAccess::rnd` draws in the farm re-target (ECX,
  dropped by the decompiler); geometry says the farm's tile extent. The
  listing (`llvm-objdump` at `0x5efd9x`–`0x5eff6x`) will settle it in a minute.
- `Farms::process` and how a `3` tile returns to `0`; how many `grow` calls
  flip a tile (the float question) — and whether farm tile state is logged at
  all (`FarmsData` has a `farm_data` array; I did not look for its `log_data`).
- The `unit_masks` bits `0x8000000/0x10000000/0x20000000/0x40000000` the
  non-flat machine sets (chop-at-camp / walk-to-tree / mine-at-camp /
  walk-to-ore by construction) — readers not grepped; presumably the renderer
  and `is_gathering`-style UI.
- The meaning of `SubObjectData::flags & 0x10` beyond "think reads it".
- `BuildTypeData::max_gatherers` / `calc_gather`'s slot count — ECONOMY.md's
  open item, unchanged.

**Behavioural checks worth running** (each a logged run, `UNITS=3 BUILDS=6`):
1. **The arrival count.** Two citizens sent to one farm: `BUILDDATA.GATHER_DOWN`
   should list both from the issue frame, and the leader's food rate should
   step only on the frame after the first `BEEN_THERE` flips — confirms
   `num_gatherers(1,1)` and the eighth-frame cadence together.
2. **Woodcutter timers.** One citizen on a camp: `GATHERORDER.WAIT` should
   read 400–599 on the walk-out frame, 32 on the walk-back frame, and the
   `GOTO_BUILD` flips should match §3.3 — also pins whether the three
   `Random::get` draws land in the order written (diff the seed-pinned combat
   stream with and without a woodcutter).
3. **The farm float.** One farmer on a wheat farm: count frames between
   `CHAR_SOW` start and the first re-target (a `MOVE` order appearing ahead of
   the `GATHERORDER`) — 200 or 201 grows.
4. **The miner's 1,000,000.** Trivial to confirm: `WAIT` on a miner out at a
   tile reads 1000000 and never changes.

**One correction for another document.** COMBAT.md §10: `GameAccess::rnd` is
`Random::get(game_random, 0, 0xffff) % n` (`GameAccess::rnd@0043cca0`), the
same stream — not a second unsynchronised one. The radar-jam roll it cites is
therefore on the sync stream too (still gated on a display flag, which is the
part that actually decides whether it desyncs).
