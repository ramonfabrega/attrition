# R6 — The combat orders and the group orders (first reading, 2026-08-20)

Reading note for the order system's combat half (`AttackOrder` and its
relatives, the seam into `Unit::fight`) and the group orders (what a
multi-unit right-click produces). Read from the decompile export under
`~/ghidra-projects/decomp/`, `types.txt` and `vtables.txt`; `docs/COMBAT.md`
§8 and §12 already specify `Unit::fight`, `think`, `think_attack`,
`find_melee_target`, `find_new_target`, `target_opportunity` and
`Object::find_nearby_target` — this note does not re-derive those, it names
the seam into them and the order lifecycle around them.

---

## 1. What it is — the structures

### 1.1 The hierarchy, settled from the constructors

Every order class **virtually** inherits `UnitOrder` (size 0x8: `+0x0`
vfptr, `+0x4 char flags`). The MSVC layout puts a `vbptr` in the derived
class, the derived fields, a 4-byte `vtordisp`, then the `UnitOrder`
sub-object last — so every class in `types.txt` has an unnamed tail of
`0xc` bytes (`vtordisp`, `UnitOrder` vfptr, `flags`, padding) that is **not
a field**. The constructors (`AttackOrder::AttackOrder@0047edb0`,
`GuardOrder::GuardOrder@00486980`, `FollowOrder::FollowOrder@00486cb0`,
`GatherOrder::GatherOrder@00487390`, `BuildOrder::BuildOrder@004820c0`,
`RepairOrder::RepairOrder@00482330`, `GarrisonOrder::GarrisonOrder@00484850`,
`MoveOrder::MoveOrder@00488a10`, `AttackToOrder::AttackToOrder@00481e30`,
`AttackGroundOrder::AttackGroundOrder@00487d50`,
`PatrolOrder::PatrolOrder@00483720`, `StrafeOrder::StrafeOrder@004800c0`,
`GroupMoveOrder::GroupMoveOrder@00485b10`,
`GroupAttackOrder::GroupAttackOrder@004853b0`,
`GroupAttackToOrder::GroupAttackToOrder@00482400`,
`GroupPatrolOrder::GroupPatrolOrder@00481fe0`, `FormOrder::FormOrder@00485f20`)
settle it — each writes `TargetOrder::vftable_for_TargetOrder_` /
`vftable_for_UnitOrder_` or calls the base constructor by name:

```
UnitOrder                                  (virtual base of everything)
├─ TargetOrder      {+0x0 vfptr, +0x4 vbptr, +0x8 ox, +0xc whom, +0x10 uid}  size 0x20
│  ├─ AttackOrder   {+0x14 def_x, +0x18 def_y, +0x1c mandatory, +0x1d defensive,
│  │                 +0x1e in_range, +0x1f ever_in_range, +0x20 new_ord}     size 0x30
│  │  ├─ StrafeOrder        : AttackOrder, AirOrder                          size 0x54
│  │  └─ GroupAttackOrder   : AttackOrder, GroupOrder
│  │                        {+0x3c temporary, +0x40 oxxx, +0x44 whosoever}  size 0x54
│  ├─ GuardOrder    {+0x14 dx, +0x18 dy, +0x1c guard_x, +0x20 guard_y,
│  │                 +0x24 idle, +0x28 retry}                                size 0x38
│  ├─ FollowOrder   {+0x14 oxx, +0x18 whose, +0x1c uid2}                    size 0x2c
│  ├─ GatherOrder   {+0x14 tx, +0x18 ty, +0x1c build_type, +0x20 wait, ...} size 0x34
│  ├─ TradeOrder    {+0x14 oxx, +0x18 whose, +0x1c started, +0x20 loaded, +0x24 uid2}
│  ├─ BuildOrder    (no fields of its own)                                   size 0x20
│  ├─ RepairOrder   (no fields of its own)                                   size 0x20
│  ├─ GarrisonOrder {+0x14 search}                                           size 0x24
│  └─ CastOrder     {+0x14 x, +0x18 y, +0x1c paid, +0x20 spell}             size 0x30
├─ MoveOrder        {+0x0 vbptr, +0x4 x, +0x8 y, +0xc angle, +0x10 dest, +0x14 tolerance,
│                    +0x18 pause, +0x1c retry, +0x20 attempts, +0x24 timer, +0x28 facing,
│                    +0x2c dest_x, +0x30 dest_y, +0x34 last_x, +0x38 last_y,
│                    +0x3c coll_x, +0x40 coll_y, +0x44 orig_x, +0x48 orig_y,
│                    +0x4c off_x (short), +0x4e off_y (short)}               size 0x5c
│  ├─ AttackToOrder, FleeToOrder, ExploreToOrder  (no fields of their own)   size 0x5c
│  ├─ FormOrder     {+0x50 newform, +0x54 delay}                            size 0x64
│  └─ GroupMoveOrder : MoveOrder, GroupOrder  {GroupOrder base at +0x50;
│                      +0x54 oxx, +0x58 whose, +0x5c id, +0x60 form_id,
│                      +0x64 group_angle, +0x68 in_group}                    size 0x78
│     └─ GroupAttackToOrder : GroupMoveOrder (NOT AttackToOrder; it calls
│                      GroupMoveOrder::GroupMoveOrder)                       size 0x78
├─ PatrolOrder      {+0x4 x_pos SimpleArray<Coord>, +0x20 y_pos, +0x3c waypoint} size 0x4c
│  └─ GroupPatrolOrder : PatrolOrder, GroupOrder  (GroupOrder base at +0x40) size 0x64
├─ AttackGroundOrder {+0x4 att_x, +0x8 att_y, +0xc accuracy, +0x10 attack_unit} size 0x20
├─ GroupOrder       {+0x4 oxx, +0x8 whose, +0xc id, +0x10 form_id, +0x14 group_angle} size 0x24
└─ AirOrder         {+0x4 oxx, +0x8 whose, +0xc cruising_alt, +0x10 sharp_turn,
                     +0x14 old, +0x18 returning}                             size 0x28
```

So the answer to "what are `TargetOrder +0x14..+0x1f`": nothing semantic —
`+0x14` is the vtordisp, `+0x18` the virtual `UnitOrder`'s vfptr, `+0x1c`
its `flags`. `AttackOrder`'s own fields start at `+0x14` because the
virtual base moves to the end of the most-derived object.

Constructor defaults that matter: `TargetOrder` `ox = whom = -1`, `uid =
0xffff`. `AttackOrder` `mandatory = defensive = in_range = ever_in_range =
0`, `def_x = def_y = -1`, **`new_ord = 1`**. `GuardOrder` all zero.
`FollowOrder` `oxx = whose = -1`, `uid2 = 0xffff`. `MoveOrder` all zero
except `facing = -1`. `GroupMoveOrder`: `form_id = -1`, `in_group = -1`
(from `GroupMoveOrder::clear@00485a90`, which also re-zeroes the `MoveOrder`
half and sets `facing = -1`). `GroupOrder::clear@00485a60`: `form_id = -1`,
rest zero.

### 1.2 `UnitOrder::flags` bits, as written by the `add_*` functions

- `0x01` — `add_move_facing_order@005e55c0` `param_5` (the "pathed/direct"
  bit of a move; `Group::action_move_near` passes `1`; `ungroup_move_order`
  clears it on a follower's degraded move; `kill_group_move` clears it).
- `0x04` — `add_move_facing_order` `param_7`; `add_attack_order@005e5410`
  `param_5`; `add_group_move_order` `param_13`; set unconditionally by
  `add_guard_order`, `add_follow_order`, `add_patrol_order`. **It is the
  "explicit action" bit**: `CommandPackage::process_move_to` passes `1`
  for it on every player move and `Group::action_attack` passes `1` on
  every player attack, while the engine's own inserted moves (`fight`'s
  chase, `do_guard`, `do_follow`, `go_to`) pass `0`. Read by
  `UnitData::get_action@00608450` / `Unit::update_action@0060a870`: the
  "action" is the first order in the list that is not (`is_move()` with
  flag 4 clear) and not `CHANGE_FORM` (type 0x12) — i.e. the player's
  intent under any pathing sub-steps. Read by `Unit::work@0060d180`: a
  `recharging` melee unit (`type.max_range == 0`, not cavalry-archer)
  returns from `work` before doing anything unless the head order has this
  bit — so a recharging melee unit finishes a player's move but not an
  engine-inserted chase step. `work` also clears `unit_masks & 0x100` on a
  non-`EXPLORE_TO` head order carrying it. (`do_attack`'s transport-barge
  move is the one engine-inserted move that sets it — a barge's "attack" is
  its move.)
- `0x08` — set by `fight` on the move it inserts to send a DEFENSIVE unit
  back to its post; `UnitData::find_def_pos@00608560` reads it ("this move
  is my post").
- `0x10` — the "re-target requested" bit on an attack order: `fight`
  toggles it on the *action* when the chase cell it chose is not the unit's
  own cell, and on entry treats a set bit as "bad target" → `find_new_target`.
- `0x20` — `add_move_facing_order` `param_11` / `add_group_move_order`
  `param_17`, passed through from `MoveToCommand.disembark`.
- `0x80` — cleared by `add_attack_order` and `add_attack_ground_order`; set
  by `do_attack_ground` once it fires.

### 1.3 The order list

`UnitData +0xc8 orderlist` (`OrderList`, 0x1c): 4 bytes, then a
`LinkListBase<UnitOrder*, uchar, RecycledOrderNode>` at `+0xcc`:
`current_data +0xcc`, `current_metric +0xd0` (the `OrderIndex` byte of the
current node), `current_node +0xd4`, `length +0xd8`, `head_node +0xdc`,
`ordered +0xe0`. Every accessor (`Unit::update_order@006179d0`,
`UnitData::get_order@0060b030`, `UnitData::order_type@00616e80`) rewinds to
the head: **the head of the list is the order being executed**. Orders are
recycled through `OrdersMemManager::get_obj(kind)@00730ac0` (a per-kind
clean pool, else `get_new_order`) / `give_obj`.

`QueuePos`: `QUEUE_NEW` — `close_orders(0)@005e37f0` (kill every order
from the head until the head's type is `NONE`/the list is empty), clear
the path, then add; `QUEUE_FIRST` — add at the tail, then rotate the list
head to the new node (`head = head->next`) so it executes now, ahead of
what was there (the chase-move and opportunity attacks use this);
`QUEUE_LAST` — append (shift-click queue). Every `add_*` ends in
`update_action`, which also refreshes `orders_x/orders_y/dest_angle`
(`UnitData +0x70/+0x74/+0x58`) from the action's move order.

### 1.4 `OrderIndex` numbering

From the `get_type` bodies (names), the literals `kill_current_order@005e2cb0`
tests (`1,2,3,4,0x12,0x13,0x15` have a `MoveOrder`; `7` is the gatherer;
`0xe` unpays a cast; `0xf` ends a trade route; `0x19` is a
`SpecialAnimOrder`), the `== 10` tests that accompany `ATTACK` everywhere,
`0xc` in `update_guard_order`, `0x16` in `update_patrol_order`, and a real
gamelog (`type 3 → EXPLORETOORDER`, `type 6 → BUILDORDER`, `type 7 →
GATHERORDER`; §5): `NONE 0, MOVE_TO 1, ATTACK_TO 2, EXPLORE_TO 3, FLEE_TO
4, (5 — no do_job case), BUILD_AT 6, GATHER 7, BOARD_SHIP 8, AWAIT_BOARD 9,
ATTACK 0xa, FOLLOW 0xb, GUARD 0xc, REPAIR 0xd, CAST_SPELL 0xe, TRADE_ROUTE
0xf, STRAFE 0x10, AIR_PATROL 0x11, CHANGE_FORM 0x12, GROUP_MOVE 0x13,
GROUP_ATTACK 0x14, GROUP_ATTACK_TO 0x15, GROUP_PATROL 0x16, ATTACK_GROUND
0x17, AIR_ATTACK_GROUND 0x18, SPECIAL_ANIM 0x19, GARRISON 0x1a, THINK 0x1b`.
(The `do_job` switch prints `case MOVE_TO: case FLEE_TO:` as one grouped
label, which is why its textual order is not the value order. `ATTACK_TO =
2` vs `FLEE_TO = 4` is the one pair no literal pins — medium confidence; a
dump with an attack-move settles it.)

`Unit::do_job@00617a10` is the dispatcher: `ATTACK → do_attack`, `FOLLOW →
do_follow`, `GUARD → do_guard`, `ATTACK_TO → do_attack_to`, `MOVE_TO/FLEE_TO
→ do_move`, `GROUP_MOVE → do_group_move`, `GROUP_ATTACK → do_group_attack`,
`GROUP_ATTACK_TO → do_group_attack_to`, `GROUP_PATROL → do_patrol` (there
is no separate plain-patrol case), `ATTACK_GROUND → do_attack_ground`,
`CHANGE_FORM → do_form_change`, `STRAFE → do_strafe`, `NONE → do_idle`
(vtable `+0x184`).

### 1.5 The `AttackOrder` fields, what writes them

- `ox/whom/uid` — the target. `uid` is the target's `ObjectData::uid` at
  the moment of the order (`add_attack_order@005e5410`); `Unit::fight`
  compares it with the live object's `uid` on entry and **treats a
  mismatch as "no target"** (`ox = whom = -1` locally) — a recycled object
  index does not inherit the order.
- `mandatory` (`+0x1c`) — `add_attack_order` `param_4`. Set by a player's
  attack click (`Group::action_attack` passes its `param_3`, which
  `CommandPackage::process_attack@00949c30` hard-codes to **1**); cleared
  for targets the unit found itself (`find_nearby_target`'s `add_order`
  path passes `mandatory` only for AI siege on a city — COMBAT.md §12.2).
- `defensive` (`+0x1d`) and `def_x/def_y` — set in `add_attack_order`
  when the unit's combat stance is DEFENSIVE (1) and `param_5 == 0` (the
  order is *not* the explicit kind): `UnitData::find_def_pos@00608560`
  fills `def_x/def_y` from an existing DEFENSIVE attack action's post, else
  from the head move order's `x/y` if it carries flag `8`, else the unit's
  own quarter-tile centre (`div_3_table[x >> 4] * 0x30 + 0x18`). The post a
  DEFENSIVE unit returns to. With `param_5 != 0` (a player's attack)
  `defensive = 0, def = -1`.
- `in_range` (`+0x1e`), `ever_in_range` (`+0x1f`) — both 0 at creation;
  written only by `Unit::fight`: after its range test `in_range =
  is_in_range(...)` and, if in range, `ever_in_range = 1`. Read by `fight`
  in one place: a **packer** type (`unit_flags2 & 4`) that is not packed
  (`unit_masks & 0x80000` clear), out of range now but `ever_in_range` —
  its target walked out of its reach once it had it — **drops the order**
  (`kill_current_order`) rather than chasing. For every other type the two
  flags are bookkeeping only (and they reach the gamelog).
- `new_ord` (`+0x20`) — 1 at creation (`add_attack_order` writes it again),
  cleared by `fight` the first time it actually attacks (right after
  `set_attacking`). Read by `fight` on entry: a `recharging` unit that is not
  a cavalry archer **returns 0 at once if `new_ord == 0`** (it is mid-reload
  on a target it has already struck), and only the very first frame of a
  fresh order proceeds past the reload gate — to turn, set the animation,
  and then also return. So `new_ord` is "has not struck yet".

`GroupAttackOrder` adds `temporary` (`+0x3c`), `oxxx/whosoever` (`+0x40/
+0x44`, the target actually fought this frame) — and **no code path
creates one**: `OrdersMemManager::get_obj(GROUP_ATTACK)` has no caller in
the export (only `do_group_attack@005e75a0` steps it, and the only
references to `GROUP_ATTACK` outside `get_type` are comparisons and the
`OrderNames` table). `Unit::do_group_attack` is dead code in the shipped
game. A group's attack is N `AttackOrder`s (§3.4).

---

## 2. The per-frame step

### 2.1 Where the step runs

`Unit::process` → `Unit::work@0060d180` → `Unit::do_job@00617a10(type,
order)`, with the order being the **head** of the list. `work` has three
gates before `do_job` that matter here:

- every 32nd frame (`(frame + o) & 0x1f == 0`) `unit_masks & 4` is cleared
  (the "pathing to a unit" bit `go_to_unit@005f78c0` sets on the whole
  squad);
- if the head order is not `ATTACK`: `unit_masks & 0x100` is cleared when
  the order carries flag 4 and is not `EXPLORE_TO`; and a **recharging melee
  unit** (`recharging != 0`, `type.max_range == 0`, not `unit_flags & 0x400`)
  **returns** unless the head order has flag 4;
- a caravan off its route is re-routed (`end_trade_route`).

### 2.2 `ATTACK` — `Unit::do_attack@005f1b80`, every frame

Let `A` be the attack order (vslot `+0x50`, `get_attack_order`), `T` its
target `(ox, whom)`.

1. **Carrier.** If the unit `is(AIRCRAFTCARRIER)` (0x15f): only every 16th
   frame (`frame & 0xf == 0`); a unit target → `add_strafe_order` on every
   plane aboard (walk the `inside_down` chain from `+0x28/+0x3e`) with the
   order's `mandatory`; a building target → `kill_current_order`. Return.
2. **Unarmed transport.** If `type.attack == 0` and `is(TRANSPORTBARGE)`
   (0x140): a `valid_target` → `find_attack_pos` (twice, the second with the
   "any" flag), else the target's own cell; `add_move_order(…, MOVE_TO,
   QUEUE_FIRST, flag4 = 1)` — the barge sails next to the target. Not valid →
   `kill_current_order`.
3. **Unarmed member of a group.** If `type.attack == 0` and `group >= 0`:
   `GroupData::get_max_range(group, is_worker)` — the group's largest
   `max_range()` — and the stand-off `r = (range + 2) × 0xc0`
   (`+0x18` each for supply and for worker; for a building target the
   footprint `(x_size + y_size) × 6` is added). If the target is farther than
   `r + 0x48` by `vector_dist`: project from the target toward the unit by
   `r`, `find_nearby_spot` there (`FILTER_NOT_ME`), and if the spot is valid
   and in the same `tregion` as the target: `kill_current_order` then
   `add_move_order(spot, MOVE_TO, QUEUE_FIRST, flag4 = 0)`. Otherwise
   `kill_current_order`. (A supply wagon "attacking" with the army walks to
   just behind the army's range and stops.)
4. **Cavalry-archer bookkeeping.** If `mandatory` and `unit_flags &
   0x200000`: `cavarch_o/cavarch_who/cavarch_uid` (`+0xa2/+0xa8/+0xa6`) =
   the target, `unit_masks2 |= 0x100` for a unit target (cleared for a
   building).
5. **The seam.** `mandatory` → `fight(ox, whom, 1, 0, 0)`. Not mandatory
   and a cavalry archer (`unit_flags & 0x400`) in STAND_GROUND (combat
   stance 2): `type.max_range = second_max_range` around one `fight(ox,
   whom, 0, 0, 0)` and back to 0 (COMBAT.md §8.5). Otherwise `fight(ox,
   whom, mandatory, 0, 0)`, and if it returns 1 (it struck) and the type is
   a cavalry archer, `cavarch_fight` with `recharging` forced to 0 for the
   call.

So **`Unit::fight` is called once per frame per attack order, with
`param_3 = mandatory` and `param_4 = param_5 = 0`**. `docs/COMBAT.md` §8.2
covers what `fight` does from the strike on; what it does *to the order* is
§2.3.

### 2.3 What `Unit::fight` does to the order (the order-management half)

In `fight(o, who, mandatory, p4, cavarch)`
(`~/ghidra-projects/decomp/funcs/Unit/fight@005fd4d0.c`), in the order the
branches are met:

- **Entry.** `uid` mismatch → target treated as none. "Bad target" =
  `unit_flags2 & 1` clear, or invalid, or the order has flag `0x10`. A
  `recharging` unit with a good target returns 0 unless `new_ord` (and then
  only for the one turn-and-animate frame; §1.5). Then `set_new_location`
  to the quarter-tile centre.
- **Invalid target** (`Object::valid_target == 0`, COMBAT.md §12.1), not the
  cavalry-archer call: a mandatory order on a unit target tries the
  target's **captain** (`get_captain`, vslot `+0xe4`) — if that is valid the
  order's `ox` is rewritten to the captain and the function continues with
  it (a squad member died, the squad lives). Otherwise, with `p4 == 0`: if
  `waiting < 5` and the owner's "targets lost this frame" counter
  (`leaders.list[who] +0x9f4`) is over 10, `waiting += 2`, `unit_masks2 |=
  0x10` and return (a throttle — the whole army does not re-search on one
  frame); else **`find_new_target(0, 0)`** (the full search; it rewrites or
  kills the order), `leaders[who] +0x9f4 += 1`, and if the head is still
  `ATTACK` and the unit is not recharging, `unit_masks2 |= 0x10`. Return 0.
  A target that is a capturable city (`valid_target`'s out-arg) with a land
  unit under a mandatory/army order beyond `city_capture_radius × 0xc0` →
  `add_move_order(halfway point, QUEUE_FIRST, flag4 = 0)` (+ `Army::charge`
  for an army) and return.
- **Opportunity check** (COMBAT.md §8.2 step 0): a captain under AI control
  rolls; `Object::check_target(o, who, duty 1, guarding 1, use_poor, …)` —
  fails → `kill_current_order`, `find_melee_target(-1)`, and if that left an
  `ATTACK` at the head and the unit is not recharging, `unit_masks2 |= 0x10`;
  return 0.
- **The 1/5 re-search** (not mandatory, `p4 == 0`, not recharging, captain,
  unit target): `Random::get % 5 == 0` or flag `0x10` → `find_new_target`;
  a different answer rewrites `ox/whom/uid` on the order; `poor_target`
  otherwise.
- **Range.** `in_range = is_in_range(o, who, my x, my y, …, &dist)` is
  written to the order; in range → `ever_in_range = 1`. Out of range: the
  unpacked-packer-that-once-had-it rule kills the order (§1.5). In range
  and a packer that is packed: `add_cast_order(UNPACK 0x28c, QUEUE_FIRST)`
  and return (siege unpacks where it stands) — unless an entrenched siege
  unit's target out-ranges it and a better position exists
  (`find_attack_pos`), in which case `add_move_order(pos, QUEUE_NEW)`.
  In range, unpacked siege, a *unit* target: `ATTACK_GROUND` is inserted
  `QUEUE_FIRST` at the target's current cell (`accuracy = (domain == sea)`,
  `attack_unit = 2`), `do_idle`, return 0 (COMBAT.md §8.2 step 1).
- **Out of range, DEFENSIVE** (`get_combat_stance() == 1`, not mandatory):
  if the order is `defensive` with a post, and the unit is at least
  `max(unit_defensive_respond_range, max_range()) × 0xc0` from the post:
  `find_new_target(0, 0)`; if the head is still the same `ATTACK` on the
  same target and the unit is not recharging: `kill_current_order`, and if
  nothing is left on the list, `add_move_order(def_x, def_y, MOVE_TO,
  QUEUE_NEW, flag4 = 0)` with **flag `8`** set on the new move. Return 0.
- **Out of range, otherwise — the chase.** `find_attack_pos(o, who, 0,
  &x, &y, 0)` (a legal cell within range of the target; `Unit::
  find_attack_pos@00601280`/`@00602e60`, not read); none, or a STAND_GROUND
  unit that is not `on_duty`, or a unit with `unit_masks & 0x2000000` and
  `unit_masks2 & 0x20000` clear → `find_new_target` and then, if the head is
  still this order, `kill_current_order` (and the DEFENSIVE-post move as
  above); `unit_masks2 |= 0x10`. Found → **`add_move_order(x, y, MOVE_TO,
  QUEUE_FIRST, flag4 = 0)`** — the chase is a plain `MoveOrder` rotated to
  the head above the attack, so the *next* frame `do_job` runs `do_move`
  (`docs/MOVEMENT.md`), and the attack order is reached again only when the
  move ends (arrival: `Unit::do_move` → `kill_current_order`) or something
  rotates it out. If the chosen cell is not the unit's own, the **action's**
  flag `0x10` is toggled (set if clear, cleared if set) and `do_idle` runs —
  flag `0x10` is what makes the next `fight` re-search instead of trusting
  the target. While chasing, `do_move` itself, every 4th frame of an
  `ATTACK` action that is not mandatory and while the owner has `repaths`
  budget, calls `find_new_target(0, 1)` (`do_move@005f7b30` lines 96–110) —
  the "walking to attack" search of COMBAT.md §12.4.
- **The strike** (COMBAT.md §8.2 steps 2–7) sets `new_ord = 0` right after
  `set_attacking`, and returns 1.

**When the order ends.** `kill_current_order(this, 0)` from: `do_attack`
(carrier/building, unarmed group member, transport with no valid target);
`fight` (check_target failure; packer rule; DEFENSIVE out of range; no
attack position; `find_new_target` with nothing found — it kills the order
itself, COMBAT.md §12); and the generic paths — `Unit::die`,
`close_orders` from any `QUEUE_NEW`, `Group::action_halt`
(`close_orders(0)` on every member). The *target's* death does not touch
the attacker's order: the attacker finds out on its next `fight` through
`valid_target` (not active) and takes the invalid-target branch. Out of
sight likewise: `valid_target_const` requires `is_seen(who)`.

`kill_current_order@005e2cb0` itself: for the move family it folds the
move's `facing`/`reversing` into `unit_masks & 2` and the group's `facing`
if the unit is the leader; GATHER detaches the gatherer; TRADE_ROUTE ends
the route; CAST_SPELL unpays; SPECIAL_ANIM type 1 removes the unit from the
building's `launching` list; then it pops the head node, `kill_current_path`
if the order was a pathed move (vslots `+0x14 is_move`, `+0x24 is_pathed`),
returns the object to `OrdersMemManager::give_obj`, `clear_partial_path`,
`update_action`.

### 2.4 `ATTACK_TO` — `Unit::do_attack_to@005f2320`

`do_move(order)` first (a full `MoveOrder` step, `docs/MOVEMENT.md`). Then,
if the head is still this order and `(o + frame) % 15 == 0`: an armed
non-supply unit → `find_melee_target(-1, 0, 0, 1, 0)` — which adds an attack
order `QUEUE_FIRST` above the attack-move when it finds one (COMBAT.md §12.4);
under an army's `+0x2c` flag a unit more than 6 units from its slot skips the
search. Unarmed/supply → `do_attack_to_pause@005f22a0`: a group member
whose group `is_attacking_near` sets `MoveOrder::pause = 0xf` (it waits
fifteen frames while the others fight). The attack-move is a move that
looks around every 15 frames; the engagement is an ordinary `AttackOrder`
stacked on top, and when that dies the attack-move resumes.

### 2.5 `ATTACK_GROUND` — `Unit::do_attack_ground@005f1410`

`can_attack_ground` (type) else kill; a cell whose owner is at peace with me
and `attack_unit == 0` → kill. Facing = `find_angle` to the cell, ±90° for a
`GUN` (`unit_flags & 0x40`) broadside. Not in range (`is_in_range(cell)`) and
`attack_unit == 0`: pick the spot at `dist − big_radius` clamped into
`[min_range × 0xc0 + 0x90, max_range × 0xc0 − 0x30]` toward the cell,
`find_nearby_spot`, in range from there → `add_move_facing_order(spot,
angle, MOVE_TO, QUEUE_FIRST, flag4 = 0)`; else the "invalid order" message
and kill. Recharging → idle animation, return. `attack_unit`: 2 → set to 1
and fire; 1 → kill the order and `do_idle` (the siege-at-unit insert from
§2.3 fires exactly once). `set_attack(-1, -1)`; a packed packer unpacks
instead; `unit_masks |= 0x11000`; flag `0x80` on the order; `CHAR_ATTACK1/2`;
`fire_ammo(-1, -1)` if `fire_proj` (COMBAT.md §9: ground fire has no target
to home on). The tail (the reload) was not read.

### 2.6 `GUARD` — `Unit::do_guard@005e5c70`

Created by `add_guard_order@005e3e40(ox, whom, dx, dy, queue, …)`: target
and its `uid`, `guard_x/guard_y` = the target's position now (or mine if no
target), `dx/dy` = the offset asked for (`Group::action_guard@006fcd30`
passes `-1, -1` for a player's guard click: every member of the group gets
its own `GuardOrder` on the same target), `idle = retry = 0`, flag 4.

Per frame, on an active unit target: `retry` counts down if set and returns.
A target that is off the map, or idle (`is_moving` vslot `+0xd8` is 0):
**every 16th frame `(o + frame + 8) & 0xf == 0` → `find_melee_target(-1,
0, 0, 1, 0)`** (the guard's own engagement, with the `guarding` argument
that limits it to `unit_guard_respond_range × respond × 0x60` of the post —
COMBAT.md §12.2) and return; every 16th frame `(o + frame) & 0xf == 0` →
`idle += 1` and return. For a building target (`+0x1c` vslot nonzero):
within `attack_dist ≤ 0x600` nothing; farther, a captain makes a one-unit
`Group` and `Group::action_move_near(spot near the building, QUEUE_NEW,
MOVE_TO)` — a plain move. For a unit target: the guard point is the
target's position plus `dx` rotated by the target's facing (`sinx/cosx`,
reversed if the target's `unit_masks & 2`), validated with `invalid_loc`,
else `find_nearby_spot` (`0xc0..0x180`, `0x60`) or the target's own cell;
`guard_x/guard_y` updated; facing = the target's facing when it moves, else
`find_angle` to it (a worker/supply/hero guarding under `unit_masks &
0x40000` copies the facing). **Not on that quarter-tile** → `idle = 0`,
`add_move_facing_order(quarter-tile, facing, ATTACK_TO kind, QUEUE_FIRST,
flag4 = 0)`, the new move's `timer = 0x1e × max(1, Manhattan tile
distance)`, then `do_move` *this same frame*; if still guarding afterwards,
idle animation and `retry = Random::get % 3 + 6` (6–8 frames before the
next reposition — **one `game_random` draw per reposition**). On the tile:
turn to the facing if needed, `idle += 1`; a packed packer with auto-stance
(`get_packer_stance == 0`) unpacks (`add_cast_order(0x28c)`) after `idle ≥
0x1e` for `is(0x7b)` types else `0x46`. An inactive target → idle
animation, `kill_current_order`.

So a guard is a follow-with-offset whose engagement is the periodic
`find_melee_target` — it never calls `fight` itself.

### 2.7 `FOLLOW` — `Unit::do_follow@005e65d0`

`add_follow_order@005e3f60(ox, whom, queue, …)`: `ox/whom/uid` the target,
`oxx/whose/uid2` its transport if it is inside one (`+0x28/+0x3e`) else the
target itself; flag 4. Per frame: if the followed object is off the map and
inside something, the order swaps to follow the container (and remembers the
original in `oxx/whose`); if the original reappears (its `uid` matches) it
swaps back; else the container's captain. Not seen (`is_seen(my owner)`,
vslot `+0x48`) → kill. Distance `d = vector_dist` to it; the standoff `s =
clamp(los × 0x180 − k, 0x180, 0x600)` where `k = los × 0x60` if the target
is slower than me else `los × 0x300 / 5`, `k` doubled if it `is_moving`; if
`d > s + 0xc0`: project from the target toward me by `s`, `find_nearby_spot`
(fallbacks: the projected point, then a ring `s..` around the target at its
facing, then its own cell), `add_move_facing_order(spot, the target's
facing, MOVE_TO, QUEUE_FIRST, flag4 = 0)` and `do_move` this frame. Else
idle animation. If the target changed identity and is gone → `work` again
(vslot `+0x188`) on the rewritten order; nothing left → kill.

### 2.8 `GROUP_PATROL` (and `PATROL`) — `Unit::do_patrol@005f1910`

`add_patrol_order@005e4560(x1, y1, x2, y2, id, leader o, who, …, queue)`
creates a **`GroupPatrolOrder`** (`get_obj(GROUP_PATROL)` — `PatrolOrder`
alone is never instantiated), with `x_pos/y_pos` sized to two waypoints
(`[0]` = here, `[1]` = the click; `Group::action_patrol@007030c0` appends
further clicks with `make_valid` + a write at `x_pos.length` when the unit
already has a patrol), `waypoint = 0`, `GroupOrder` `oxx/whose` = the
leader, `id = (group.id + frame × 10) × 100 + group.order_num`, flag 4.
Per frame: a unit **not in a group**: `waypoint = (waypoint + 1) mod
count`, and it inserts an `ATTACK_TO` `MoveOrder` `QUEUE_FIRST` to that
waypoint (flags 1/4/0x20 clear, `facing = -1`) — the patrol is a loop of
attack-moves, each leg engaging what it meets (§2.4); when the leg's move
dies the patrol order is at the head again and issues the next leg. The
**leader** of a group does the same through `Group::action_move_to(group,
next waypoint, QUEUE_FIRST, …, ATTACK_TO, …)` — a whole-group attack-move
per leg (§3); a **follower** just idles (its legs come from the leader's
group move). A carrier (`is(0x15f)`) with planes aboard scrambles them
(`Group::action_scramble`). `update_patrol_order@005e35e0(id)` finds the
patrol order with that id in a unit's list (or any, with `-1`).

### 2.9 Building attack orders — `Build::add_attack_order@00622ce0`

Not an order object at all: writes `attack_ox (+0x7c)`, `attack_who (+0x81)`
and sets/clears `build_masks & 4` ("explicit target"). `Build::do_attack`
(COMBAT.md §8.6) reads them; `Group::action_attack` with a building group
writes the same three fields directly and refuses mixed groups. The sim's
`order_building_attack` is already this.

---

## 3. The group orders — what a multi-unit right-click produces

### 3.1 The entry: `CommandPackage::process_*` → `Group::action_*`

A player's command arrives as a `Command` packet (`+0x0 command_type`,
then the payload) processed by `CommandPackage::process_all`; the
selection arrives first as a `GroupCommand {num, who, list[]}` →
`process_group@0094a0c0` builds a `Group` of those units (captains only)
and `Groups::push_group`es it; `this->group` is then the group every
following order packet in the package applies to. The ones in this
sub-area:

| packet (`types.txt`) | handler | → | logged as |
|---|---|---|---|
| `MoveToCommand {to_x, to_y, set_angle, angle, orders, queued, form, width, disembark}` (0x16) | `process_move_to@009497c0` | `Group::action_move_to(g, x, y, queued, set_angle, angle, orders, **1**, form, width, disembark)` | `GLOG_COMMANDMANAGER` detail 1: `Log::say(int_str 0x3188: x, y, queued, set_angle, angle, orders, disembark)` + `(0x319c: form, width)`; `SyncLogger::logToMemory` of the same |
| `MoveNearCommand {…, tolerance, …}` (0x1a) | `process_move_near@009495c0` | `Group::action_move_near` with the tolerance | same |
| `AttackCommand {ox, whom, ignore, queued}` (0x11) | `process_attack@00949c30` | unit target or none: `Group::action_attack(g, ox, whom, **1**, queued, ignore)` | `GLOG_COMMANDMANAGER` detail 1: `(0x314c: ox, whom, ignore, queued, frame)` |
| `SiegeAttackCommand {ox, whom, queued}` | `process_siege_attack@00949ae0` | `Group::action_siege_attack` → `action_attack(…, 1, queued, 0)` for the siege half, `action_guard` for the rest | |
| `AttackGroundCommand {to_x, to_y, queued}` | `process_attack_ground@009494a0` | `Group::action_attack_ground` | |
| `GuardCommand {ox, whom, queued}` | `process_guard@009478a0` | `Group::action_guard(g, ox, whom, queued, 0)` → `Unit::add_guard_order(u, ox, whom, -1, -1, queued, …)` per member | |
| `FollowCommand {ox, whom, queued}` | `process_follow@009479c0` | `Group::action_follow` → `Unit::add_follow_order` per member | |
| `PatrolCommand {to_x, to_y, queued}` | `process_patrol@00949380` | `Group::action_patrol` → `Unit::add_patrol_order` per member (planes: a move) | |
| `FormCommand {form, rotate, queued}` | `process_form@00949d90` | `Group::action_form` | |
| `HaltCommand` | `process_halt@00949140` | `Group::action_halt(g, 0)` → `close_orders(0)`, `clear_partial_path`, `update_action` per member | |

**This is where the harness's order stream enters**: a recorded game is a
sequence of these packets, and the `GLOG_COMMANDMANAGER` lines in a gamelog
(detail 1, so `COMMANDMANAGER=1` in `gamelog.ini`) are the same stream in
text. `orders` in `MoveToCommand` is the `OrderIndex` kind of the move
(`MOVE_TO` for a right-click, `ATTACK_TO` for an attack-move, `EXPLORE_TO`
for auto-explore); `queued` is the `QueuePos`; the `1` that `process_move_to`
passes is what becomes **flag 4** on every resulting order. The single-unit
scripted path (`cheat select` + right-click) goes through the very same
`MoveToCommand`: a one-member group. `Unit::go_to@005f7a50` (the engine's
own "walk there") also builds a one-unit `Group` and calls
`Group::action_move_to` — with `0` for the flag-4 argument.

### 3.2 `Group::action_move_near@00704990` — who gets which order

For a `MoveToCommand` (tolerance 0): `Group::action_move_to@0070fba0` is a
one-line forward to `action_move_near(…, 0, …)`. Then, in order:

1. Scenario `ignore_orders` filter; clamp the click into the world
   (`world.w × 0x300 − 1`).
2. The group is **split by domain**: members are sorted into a land group
   and a sea/air group (units inside a transport count as the transport); if
   both halves are non-empty and the group has no army (`+0x8 < 0`), two
   groups are pushed and each gets its own `action_move_near` (recursion),
   and the click's tile class decides which half is "land" (`tile & 0x30 !=
   0x20`).
3. `QUEUE_FIRST` is implemented as `set_up_insert` + `action_halt` +
   recursion with `QUEUE_NEW` + `finish_insert` (the old orders are spliced
   back behind the new one); the leader's `unit_masks & 0x100` survives.
4. A group whose leader is an air unit of domain 2 without `unit_flags &
   0x20` returns (those move by another path).
5. **The formation**: `form = param_9` (`MoveToCommand.form`), or the
   group's current `GroupData::get_form` when it is 9 or −1; `form_mod =
   param_10` (`width`), or `get_form_mod_option`. `get_loc`/`get_loc_to`
   (QUEUE_LAST) gives the group's present/last-ordered centre. With
   `QUEUE_NEW`, every member's orders are cleared (`Unit::clear_orders`; a
   packing packer keeps its last order). `Group::compute_form@00707c80
   (x, y, form, form_mod, …, &angle, centre)` → a `Form` (0xe98): it finds
   the leader (`GroupData::find_leader`, its slot in `FormData +0x34 idx`),
   decides the formation **angle** (`find_angle(centre → click)`, or the
   leader's facing when the group is already there, with the "reverse"
   flip when the new angle is more than 90° off), calls `Form::categorize`
   and `Form::compute`, and leaves in `FormData.to_x[128]/to_y[128]`
   (`+0x514/+0x714`) **one destination per member** and `off_x/off_y`
   (`+0x914/+0xb14`) the slot offsets; `FormData.reverse (+0xe88)` is
   passed down as the move's `facing`. (`Form::compute` itself was not read
   — the formation geometry is its own sub-area.)
6. **Per member** (index `i` in `list`, 0-based): the member's `form`/
   `form_mod` bytes are set (`UnitData +0xaa/+0xab`, except the four
   civilian-transport types 0x32–0x35); its slot destination `to_x[i]/
   to_y[i]` is clamped into the world; when `i > 0` and the slot is in a
   different `tregion` from slot 0 (the leader's), the unit is **taken out
   of the group** (`group = -1`) and re-slotted by `find_nearby_spot` for
   its own type near slot 0 (a unit that cannot reach the formation walks
   alone); a non-captain takes its captain's slot nudged by
   `find_nearby_spot(guy_spacing, ×4)`; a captain whose slot is an
   `invalid_loc` gets `find_nearby_spot(0xc0, 0x480)` around slot 0.
   `QUEUE_NEW` on a packing packer re-queues behind its pack. Then **the
   choice**:

   ```
   if (orders == MOVE_TO || orders == ATTACK_TO) && !is_modern_infantry(u)
      && !( (u.type.role & 0x10 && !(u.unit_masks & 0x40000))
            || group.num < 2
            || (u.unit_masks & 4)           /* pathing to a unit */
            || u.type.domain == 1           /* sea */
            || form == 9 )
        Unit::add_group_move_order(u, tile(to_x[i]), tile(to_y[i]),
             angle | (group.angles[i] << 24),
             id = (group.id + frame*10)*100 + group.order_num,
             slot = i, leader o, who, …, kind = orders, …,
             queue, flag4 = 1, facing = form.reverse, orig = click, flag0x20 = disembark)
   else
        Unit::add_move_facing_order(u, tile(to_x[i]), tile(to_y[i]),
             angle | (group.angles[i] << 24), kind = orders, flag1 = 1,
             queue, flag4 = 1, facing = form.reverse, orig = click, flag0x20 = disembark)
   ```

   `u.unit_masks & 0x400` is cleared after either. So: **a group of two or
   more land units that are not modern infantry, not `role & 0x10` types
   (the role bit the workers/caravans carry, by `action_move_near`'s other
   uses of it), and not in form 9 (the "no formation" choice) each receive
   a `GroupMoveOrder`** (`GROUP_MOVE`, or `GROUP_ATTACK_TO` when `orders`
   is `ATTACK_TO`) — one per unit, all sharing the `id`, all naming the
   same leader, each with its own slot index and its own slot destination.
   Everything else — a lone unit, a boat, modern infantry, workers — gets a
   plain `MoveOrder` to *its own slot*. A city rally (`local_40/local_44`
   — an army with the rally flag on a friendly city within 0x200) turns
   members into `add_garrison_order` or a move to the city instead.
7. `update_positions(leader)@00713810` — `GroupData::curr_x[i]/curr_y[i]`
   = the slot offsets `off_x[i]/off_y[i]` rotated by the leader's move angle
   (`sin_table`), the per-frame "where is my slot relative to the leader"
   table `do_group_move` reads.
8. Then (lines 896–1330 of the export, read in outline) the **leader's
   path** is computed once — `PathFinder::find_wpath` from the leader's cell
   to slot 0 — and **copied to every member**, translated by the member's
   slot offset (`to_x[i] − to_x[leader]`), each waypoint re-validated
   against the tile class (`& 0x30`) and `invalid_loc`; a member whose
   translated waypoint is bad gets its nearest valid cell. The group does
   not path N times; it paths once and offsets.

`Unit::add_group_move_order@005e4710(tx, ty, angle, id, slot, leader o,
leader who, _, _, kind, _, queue, flag4, facing, orig_x, orig_y, flag0x20)`:
`get_obj(GROUP_ATTACK_TO)` if `kind == 2` else `get_obj(GROUP_MOVE)`; the
`MoveOrder` half exactly as `add_move_facing_order` writes it (`x/y =
tile × 0x30 + 0x18`, `dest_x/dest_y` the same, `last = -1`, `off_x/off_y =
x mod 0x300`, `orig`, `facing`, `tolerance = 0`, flag 1 **set**); the
`GroupOrder` half `oxx/whose` = the leader, `id`, `form_id = slot`,
`group_angle = angle`.

### 3.3 `GroupMoveOrder` per frame — `Unit::do_group_move@005e79a0`

`G` = the order (vslot `+0x98`): a `MoveOrder` with `oxx/whose` = the
leader, `id`, `form_id` = my slot index, `group_angle`, `in_group`.

- `group == -1` (the unit left its group) → `ungroup_move_order(id, 0)`.
- **I am the leader** (`oxx/whose == me`): if my *action* is `ATTACK`,
  every 32nd frame `(frame + o) & 0x1f == 0`, and the action's target is
  within `0x900` → `Group::kill_group_move(group, id)` (the whole group
  drops the group move and falls back to its attack orders). Else
  **`do_move(G)`** — the leader moves exactly like a lone `MoveOrder`
  (`docs/MOVEMENT.md`: path following, `get_speed(x, y, 0)`, the step) —
  and if `do_move` returned non-zero (still going) and the head is still
  `G`: `group.new_speed = get_speed(x, y, 1)` (the uncapped speed),
  `group.speed = old new_speed`, `group.march = 0` (→ 1 if the owner has
  `leaders & 0x8000` and a general is near), `Group::update_positions`.
  `do_move` returned 0 (arrived or the move died): if my action is
  `ATTACK`/`GROUP_ATTACK` or `G` is `GROUP_ATTACK_TO`, and I `collide`d with
  a valid target → `Group::kill_group_move(id)` then `Group::distribute_attack
  (target)` (or `Group::action_attack(collide target, QUEUE_FIRST)` for a
  group attack-move); else if the head is still a `GROUP_MOVE`/
  `GROUP_ATTACK_TO` → `ungroup_move_order(id, 0)`.
- **I am a follower.** Sanity: the leader exists, is on the map, is in my
  group, has an order, and that order is a group order (vslot `+0x2c`) with
  my `id` (or is a `CHANGE_FORM`); otherwise the leader is lost. My index in
  `group.list` is written to `form_id` every frame (the list compacts when
  units die); not found → `group = -1`. Leader lost: if I am still in a
  group and more than `0x5ff` from the leader's cell, `Group::
  refresh_group_order@00713a50(id, me)` — **I become the leader**: the slot
  offsets (`off_x/off_y`) are re-based on my slot, every member's group
  order is rewritten to name me (`Unit::modify_group_order@005e3530`: same
  `id` → `oxx/whose` = me), `update_positions`; else `ungroup_move_order`.
  My action is `ATTACK` and its target is in range → `kill_current_order`
  (the group move is dropped for the fight). Otherwise my target this frame
  is **`leader.pos + group.curr_x[form_id] / curr_y[form_id]`** (the rotated
  slot offset), pushed as a one-entry path: if the leader's distance to that
  point is within `0x60` of mine or I am within `0x180` of it — the slot is
  reached — `in_group = 0`, `dest = (to_x, to_y)`, `ungroup_move_order(id,
  0)`; else heading = `find_angle(me → slot)`; if my heading and the
  leader's heading differ by less than 60° (`< 0x55555555` of `2^32`), I
  walk to the slot (`push` it, `dest_x/dest_y`, `in_group = 1`); else if
  the leader has a path and I was in the group, I aim at the leader's next
  waypoint plus my offset (and `in_group = 0` once within `0x60`); else
  (turning hard) I aim at the midpoint of (me, slot) — `uVar8 = 1`.
  **Speed**: heading agrees (`uVar8 == 0`): if `march == 0` or a general is
  present, `group.speed = group.new_speed = min(group.speed,
  UnitData::speed(me))` — **the group's speed becomes the slowest reporting
  follower's**; then `v = get_speed(x, y, 1) + min(v / 3, 9)` — **a follower
  walks up to a third faster than its own speed (capped at +9) to hold the
  slot**. Heading disagrees: `v = get_speed(x, y, 0) / 2` (the capped speed,
  halved — a follower cutting the corner goes slow). `invalid_loc` on the
  slot: within `0x300` Manhattan a flock of birds may be added (**one
  `Random::get` draw from `game_random`**) and `ungroup_move_order`. A
  cavalry archer fights while moving (`cavarch_fight`). **`move_step(this,
  G, v)`** — the seam into `docs/MOVEMENT.md`'s unit step, with the speed
  passed in rather than computed inside. `move_step` returning 0 (arrived)
  with the head still `G`: an attack-context collision → `kill_current_order`
  / `Group::action_attack(collide target, QUEUE_FIRST)`; else
  `ungroup_move_order(id, 0)`.

`Unit::ungroup_move_order@005fd140(id, sub)`: walks to the captain
(`sub == 0`), finds the group order with that `id` in the list, and
**replaces it in place with a plain `MoveOrder`** (`GROUP_MOVE` → `MOVE_TO`;
`GROUP_ATTACK_TO` → `ATTACK_TO`) copied field-for-field
(`MoveOrder::operator=`), `dest = 0`, `orig_x/orig_y = x/y`, flag 1 cleared
unless I was the leader, `kill_current_path` unless I was the leader; then
the head rotates to it; then the same for every subordinate down the
`o_down` chain. So a group move *degrades* into N independent moves to the
same slot destinations, one unit at a time, as each reaches its slot or
loses the leader.

`Unit::kill_group_move@005e3400(id)`: for every order in the list with
that group `id` that is a move and not `GROUP_ATTACK_TO`, clear flag 1,
`kill_current_path`, and `kill_current_order`. `Group::kill_group_move
@007123f0` applies it to every member. `Unit::kill_group_order@005e3310(id)`
kills any group order with the id.

**The group speed**: `Groups::process@006fa210` runs once a frame for one
slot of each player's 64 group slots (round-robin), purging dead members
and resetting `group.speed = new_speed = UnitData::speed(leader)`;
`Group::compute_speed@00707f80` does the same on demand; `leader_report_speed
@007137f0(v)` = `speed = new_speed; new_speed = v; march = 0`; `report_speed
@00713bb0(v)` = `min`. `UnitData::get_speed(x, y, 0)@00608720`
(MOVEMENT.md §3) caps a unit with **no action** (`get_action() == NULL`) at
`group.speed`. A player's group move carries flag 4 so it *is* the action
and the cap does not apply to the group move itself — the formation holds
together by the follower's `+v/3` boost, and **the leader is not slowed by
the group**; followers catch up. (Medium confidence — whether
`GroupData::find_leader` picks the slowest unit was not read.)

### 3.4 `GroupAttackOrder` and `GROUP_ATTACK` — dead; `Group::action_attack` is N attack orders

`Group::action_attack@00712490(o, who, mandatory, queue, ignore)`: a
building group sets the three `Build` fields per building (§2.9). Unit
groups: `QUEUE_FIRST` via `set_up_insert`/`action_halt`/recursion as moves
do; a capturable city → `Build::check_capture` and a group *move* to it
(`action_move_to(…, MOVE_TO, 1, …)`); then three passes over the members
(`local_14 = 0..2`, by domain), skipping members the `ignore` mask excludes
(bit 1: `is(…)` types; bit 2: `is_special`; bit 4: siege `+0x10c`), and per
member: a plane gets a strafe retarget; a unit whose current action is
already a **mandatory** `ATTACK` on this target with fewer than three orders
queued and not in range keeps it; a packing packer: `clear_orders`, an
`UNPACK`/`PACK` cast if the range says so, then `add_attack_order(o, who,
QUEUE_LAST, mandatory, 1)`; a target that is a building and a unit that
cannot attack it gets `add_move_order(target cell, QUEUE_LAST)` +
`check_capture`; and the main path: **`add_attack_order(unit, o', who',
queue, mandatory, flag4 = 1)`** — where `o'` is the clicked target unless
`mandatory == 0`, in which case `find_melee_target(min(dist + 0xc0,
unit_respond_range × 0x240), …)` from the unit picks something nearer and
the click is only the fallback. `group.order_num += 1` at the end. An
unreachable building target (`find_attack_pos` from the leader fails) only
prints the red "invalid order" message for the console owner; the orders
are still given. **So a right-click attack with ten units selected is ten
`AttackOrder`s, each stepping through `do_attack → fight` on its own, with
no group coordination beyond `Group::distribute_attack` when a group move
ends on a target.**

`Group::distribute_attack@00713390(o, who)`: over the members, those whose
head order `is_attack` (vslot `+0x18`): in range of `(o, who)` → counted,
else their order's target is cleared to −1; if fewer than two are in range,
every attacking member gets `kill_current_order` + `add_attack_order(o, who,
QUEUE_FIRST, first member's mandatory, first member's flag 4)`.

### 3.5 Can v1 treat a group move as N independent moves plus an offset?

Almost — and the engine itself degrades to exactly that through
`ungroup_move_order`. What the independent-moves model gets **right** with
no extra work: the destinations (one per unit, from the formation's slot
table, which is its own sub-area), the path (one path, offset per unit —
§3.2 step 8), the end condition per unit, and the fact that a single
selected unit, any boat, modern infantry, a worker, or a "no formation"
group is *already* N independent `MoveOrder`s in the original. What it gets
**wrong**, per frame, for a land formation of ≥ 2:

1. **Followers do not walk their own path to their slot.** They walk toward
   `leader.pos + rotated offset` recomputed every frame (§3.3), so they
   track the leader's *current* position, not the destination; the
   formation bends around corners with the leader and re-forms when he
   stops. With independent moves each unit heads for its slot's destination
   from the start — right at arrival, wrong in transit.
2. **Speed.** Followers run `get_speed(…,1) + min(v/3, 9)` when heading
   with the leader and `get_speed(…,0)/2` when cutting across; the group
   speed is the min over followers and the `get_speed(…,0)` cap applies to
   later non-action orders. Independent moves run the plain pipeline.
3. **Leader loss and hand-over** (`refresh_group_order`): the next unit
   becomes leader and the offsets re-base — the other units' targets change.
4. **The attack-move interaction**: `do_group_move` kills the whole group's
   move when the leader's attack target comes within `0x900`, and hands a
   collision to `distribute_attack`/`Group::action_attack`.
5. **Determinism-visible side effects**: the follower's `Random::get` for a
   flock on an invalid slot advances the `game_random` stream the combat
   uses (COMBAT.md §10).

For the diff harness the honest position is: the per-frame positions of
followers in a group move **will not match** under the N-independent-moves
model; the arrival positions will; and the RNG stream diverges on the
first invalid-slot flock. A faithful `GroupMoveOrder` is one struct
(`MoveOrder` + leader + id + slot + `in_group`) and one function
(`do_group_move`, above) on top of the `MoveOrder`/`move_step` the sim
already has — the genuinely new inputs are `GroupData` (`curr_x/curr_y` per
slot, `speed/new_speed`, `list`, `march`) and the formation's
`to_x/to_y/off_x/off_y` from `Form::compute`.

### 3.6 `FormOrder`, `AirOrder`, `StrafeOrder` — named only

`FormOrder` (`CHANGE_FORM`, `MoveOrder + newform + delay`) is stepped by
`Unit::do_form_change@005e8670` (not read); `get_action` skips it as it
skips non-explicit moves. `StrafeOrder : AttackOrder, AirOrder` is the
plane's attack (`do_strafe@005eab00`), `AirOrder {cruising_alt, sharp_turn,
old, returning}` the air-movement base; `Unit::do_attack` on a carrier fans
out `add_strafe_order`. Not this sub-area.

---

## 4. The seams, by name

| order | per-frame function | calls into | with |
|---|---|---|---|
| `ATTACK` | `Unit::do_attack` | `Unit::fight(ox, whom, mandatory, 0, 0)` — COMBAT.md §8.2, `fight.rs::process_unit_combat/fight` | the chase is `add_move_order(find_attack_pos(), MOVE_TO, QUEUE_FIRST, flag4=0)` → next frame `do_move` (MOVEMENT.md); `find_new_target` on an invalid target; `kill_current_order` to end |
| `ATTACK_TO` | `Unit::do_attack_to` | `do_move(order)` then every 15th frame `find_melee_target(-1,…,1,0)` (COMBAT.md §12.4) | an engagement is an `ATTACK` `QUEUE_FIRST` on top |
| `ATTACK_GROUND` | `Unit::do_attack_ground` | `is_in_range(cell)`, `fire_ammo(-1,-1)` (COMBAT.md §9) | `attack_unit` 2→1→kill for the siege-vs-unit insert |
| `GUARD` | `Unit::do_guard` | every 16th frame `find_melee_target(-1,…,guarding=1)`; `add_move_facing_order(ATTACK_TO kind, QUEUE_FIRST)` + `do_move` to reposition | never `fight` directly |
| `FOLLOW` | `Unit::do_follow` | `add_move_facing_order(MOVE_TO, QUEUE_FIRST)` + `do_move` | standoff from `los` |
| `GROUP_PATROL` | `Unit::do_patrol` | leg = `ATTACK_TO` `MoveOrder` `QUEUE_FIRST` (lone) or `Group::action_move_to(…, ATTACK_TO)` (leader) | |
| `GROUP_MOVE` / `GROUP_ATTACK_TO` | `Unit::do_group_move` (+ `do_group_attack_to@005e74e0` = the same, then the `ATTACK_TO` 15-frame search) | leader: `do_move(order)`; follower: `move_step(this, order, v)` with `v` from §3.3 | ends by `ungroup_move_order` → a plain `MoveOrder` in place |
| a building's target | `Build::process/do_attack` | COMBAT.md §8.6 | `Build::add_attack_order` = three fields |

What the sim has today (`crates/sim/src/fight.rs::process_unit_combat`,
`lib.rs`): `combat::State {target, mandatory, stance, recharging, …}` is the
`AttackOrder` collapsed to its target and `mandatory`; `order_attack` is
`add_attack_order(QUEUE_NEW, mandatory = 1)`; the chase is `movement.dest =
pos_of(target)` (a straight line — no `find_attack_pos`, no inserted
`MoveOrder`, no flag-0x10 re-search toggle); the invalid-target path is
`retarget(None)` + `think_attack` (no captain hand-over, no `waiting`
throttle); there is no `def_x/def_y` return, no `in_range/ever_in_range`
(the packer rule), no `new_ord` (the first-frame-only turn while
recharging), no `ATTACK_TO`, `GUARD`, `FOLLOW`, `PATROL`, and no order
list. `Job {Build, Repair, Garrison}` is the list's other half. The missing
piece for this sub-area is the **list** and its head-rotation semantics
(`QUEUE_FIRST` inserts *above* the attack and the attack resumes when the
move dies), which is what makes the chase, the guard's reposition, the
patrol's legs and the group move's degradation all the same mechanism.

---

## 5. Where it sits in the frame, and what the gamelog writes

`Unit::process` (per unit, in object order, `docs/ATTRITION.md` has the
frame) decrements `recharging`/`full`, then `work` → `do_job(head)`. So an
attack order's `fight` and a move order's `do_move` are the same slot of the
frame, one per unit, and an inserted `QUEUE_FIRST` move takes effect on the
*next* frame's `do_job` (`do_guard`/`do_follow` are the exception: they call
`do_move` on the freshly inserted move in the same frame). `Groups::
process` runs after the units (one group slot per player per frame).
Attrition and the building clock do not read orders.

**The gamelog** (`UnitData::log_data@0060b070`, the `UNITS` category; the
unit body is at detail 3 after `ObjectData::log_data`, so **`UNITS=3`** —
the recipe's setting — prints the orders): after the unit's fields and its
`PATHDATA` stack, `OrderList::log_data@00730070` writes `length N` and then,
per order, `type <OrderIndex>`, `metric <node metric>`, and the order's own
`log_data` — a nested `BEGIN <NAME>` block per class in the hierarchy,
fields under lower-case field names. From a real dump (`Logs/gamelog.txt`,
2026-08-20):

```
   length 2
   type 6
   metric 0
   BEGIN BUILDORDER
    BEGIN TARGETORDER
     BEGIN UNITORDER
      flags 4
     ox 2006
     whom 1
     uid 12
   type 3
   metric 0
   BEGIN EXPLORETOORDER
    BEGIN MOVEORDER
     BEGIN UNITORDER
      flags 1
     x 41784
     y 15816
     angle 1003094016
     dest 0
     tolerance 0
     pause 0
     retry 0
     attempts 0
     timer 0
     facing -1
     dest_x 38712
     dest_y 15816
     last_x -1
     last_y -1
     coll_x 38257
     coll_y 15816
     orig_x -1
     orig_y -1
     off_x 312
     off_y 456
```

and a `GATHERORDER` (`type 7`) prints `TARGETORDER {UNITORDER {flags}, ox,
whom, uid}` then `tx ty build_type wait goto_build non_flat_gather dist_mod
been_there`. The block names are the class names upper-cased
(`ATTACKORDER`, `GUARDORDER`, `FOLLOWORDER`, `PATROLORDER`, `GROUPORDER`,
`ATTACKGROUNDORDER`, `FORMORDER`, `ATTACKTOORDER`, `GROUPATTACKTOORDER`)
except three in mixed case — `GroupMoveOrder`, `GroupAttackOrder`,
`GroupPatrolOrder`. By the `log_data` bodies (key strings come from the
internal string table, so their spelling is known only where a dump shows
it; the field order and count are from the code):

- `UNITORDER`: `flags`.
- `TARGETORDER`: `UNITORDER`, then `ox whom uid`.
- `ATTACKORDER`: `TARGETORDER`, then `mandatory defensive in_range
  ever_in_range new_ord def_x def_y` (seven keys, that order — none of the
  existing dumps has an `ATTACKORDER` block; no unit in them was ever
  ordered to attack).
- `GUARDORDER`: `TARGETORDER`, then `guard_x guard_y idle retry` and two
  more (`dx dy` by key length 4/5).
- `FOLLOWORDER`: `TARGETORDER` only (`oxx/whose/uid2` are not logged).
- `MOVEORDER`: `UNITORDER`, then `x y angle dest tolerance pause retry
  attempts timer facing dest_x dest_y last_x last_y coll_x coll_y orig_x
  orig_y off_x off_y` (the dump's order; the decompiled `this[-1]`
  arithmetic is shifted, trust the dump).
- `GROUPORDER`: `UNITORDER`, then five keys: `id form_id` and three more
  (an 11-char, a 2-char and a 7-char key — `group_angle`, `ox`/`oxx`,
  `whose`? — spelling open).
- `GroupMoveOrder`: `MOVEORDER`, `GROUPORDER`, then `in_group`.
- `GroupAttackOrder`: `whosoever oxxx temporary`, `ATTACKORDER`, `GROUPORDER`.
- `ATTACKGROUNDORDER`: `UNITORDER`, then `accuracy attack_unit att_x att_y`.
- `PATROLORDER`: `UNITORDER`, `x_pos` and `y_pos` as `SimpleArray<Coord>`
  blocks, then `waypoint`.
- `FORMORDER`: `MOVEORDER`, then `newform delay`.

The **command stream** is logged separately under `GLOG_COMMANDMANAGER`
(detail 1): `process_move_to` writes `to_x to_y queued set_angle angle
orders disembark` and `form width`; `process_attack` writes `ox whom ignore
queued frame`. With `COMMANDMANAGER=1` in `gamelog.ini` a played game's
every click is in the text log — the order stream the harness needs
without a recording.

---

## 6. Confidence, what is not established, checks

**High** (read in full, cross-checked against two or more functions):
the hierarchy and field offsets (§1.1); `add_attack_order`'s writes and
the `uid` rule; `do_attack`'s five branches and the `fight` call signature;
`fight`'s writes to `in_range/ever_in_range/new_ord` and its
`kill_current_order`/`add_move_order` exits; `do_guard`, `do_follow`,
`do_patrol`, `add_patrol_order`'s `GroupPatrolOrder`; `do_group_move` in
full; `ungroup_move_order`, `kill_group_move`, `modify_group_order`,
`refresh_group_order`, `distribute_attack`; the `CommandPackage` entry and
its arguments; the `action_move_near` per-member choice (quoted); `Group::
action_attack`'s per-member `add_attack_order`; `GroupAttackOrder` never
constructed; the log block names and `OrderList`'s `length/type/metric`
framing (from a dump); flag 4 as the explicit-action bit.

**Medium**: `ATTACK_TO`/`FLEE_TO` (2 vs 4); flag bits `0x01/0x20/0x80`
beyond "who sets them"; the group speed's effect on the leader (§3.3 last
paragraph — `find_leader` unread); `do_attack_ground`'s tail; `find_def_pos`'s
flag-8 source; the `role & 0x10` reading of the group-move predicate.

**Not established**: `Form::compute` / `Form::categorize` (the slot
geometry); `Group::action_move_near` lines 896–1330 (the leader path and
its offset copy, read in outline only — how a bad translated waypoint is
repaired); `Unit::do_form_change`; the spelling of the `ATTACKORDER`/
`GROUPORDER`/`GUARDORDER` log keys (needs a dump); `find_attack_pos` (two
overloads at `00601280`/`00602e60`, called but not read);
`GroupData::find_leader`; `Group::is_attacking_near`; what `MoveOrder::pause
= 0xf` does inside `do_move`.

**Behavioural checks worth running** (each one logged run with
`UNITS=3 COMMANDMANAGER=1`):
1. Select two land units, right-click: the dump should show one
   `GroupMoveOrder` block per unit with the same `id`, `form_id` 0 and 1,
   and `orders 1` in the command line — pins `MOVE_TO = 1` and the
   group-vs-plain predicate. Then one unit alone: `MOVEORDER` only.
2. Attack-move (A + click) with one unit: the command line's `orders`
   value pins `ATTACK_TO` (2 or 4) and the block name `ATTACKTOORDER`.
3. A right-click attack on an enemy unit: the `ATTACKORDER` block's seven
   keys and their spelling; `mandatory 1`, `new_ord 1` on the first frame,
   `0` after the first strike; `in_range` flipping as the target moves.
4. Guard a unit, then walk it away: the `GUARDORDER` keys, `retry` in
   `6..8` after each reposition, `idle` counting by sixteenths of a second.
