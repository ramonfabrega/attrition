# R2 — `MoveOrder` and the path seam (first reading)

How a unit is told to go somewhere, what it keeps while it goes, how the
per-frame step decides between "walk straight", "ask the pathfinder" and
"give up", and where it hands off to `Unit::move_step` (`docs/MOVEMENT.md`).
Pathfinding proper (`PathFinder::astar_path` and friends) is treated as a
black box with a documented interface; collision (`detect_unit_collision`,
`resolve_unit_collision`) the same. Group moves are noted only where they
share `MoveOrder`'s machinery.

**How this was established.** `types.txt` for the layouts, `vtables.txt` for
the slots, the decompile export for every function named below, the PE bytes
for the log labels (`MoveOrder::log_data` writes its keys as UTF-16 constants;
all twenty were read back from `.rdata`), `llvm-objdump` for the one register
argument the decompiler dropped (`find_angle`'s in `Unit::add_move_order`),
and the bottle's `gamelog-run4-…` for two real `MOVEORDER` blocks — one on the
frame a citizen's move order was queued, one on the frame it first moved —
which confirm the label set, the field values the reading predicts, the
first-moving-frame rule and the "near move never calls the pathfinder" claim.

**Confidence.** High on the structure, the lifecycle, the planning decision
tree in `do_move`, the arrival rules, the log format and the harness check.
Medium on the collision-adjacent branches of `do_move` (read, summarised, not
re-derived) and on the PathFinder entry points' *exits* (read to the point
where `astar_path` is called; what it pushes is the next mechanic). Low on the
semantics of `UnitOrder::flags & 4` and `& 0x20` (named by their readers only).

---

## 1. What it is

### 1.1 `UnitOrder` and the virtual base

`UnitOrder` (`types.txt:19204`, size 8) is `{ +0 vptr, +4 char flags }`. Every
concrete order derives from it **virtually**: `MoveOrder` (size 0x5c) lays out
as `+0 vbptr, +4..0x50 fields, +0x50 vtordisp, +0x54 UnitOrder { vptr, flags }`
— `MoveOrder::MoveOrder@00488a10` writes `this+0x54 = UnitOrder::vftable`,
`this+0x58 = 0` (flags) and stores `vbase_off - 0x54` at `+0x50`. That is why
every `MoveOrder::*` body in the export is typed with `this[-1].field` —
Ghidra's `this` is the `UnitOrder` subobject at `+0x54`, so a printed
`this[-1].X` is the real field at `offset(X) − 8`. All offsets below are the
PDB's, after that correction (the `log_data` keys, read from the PE, agree
with it field for field).

Virtual slots used here (`vtables.txt:33597`, `MoveOrder::vftable @ 00b4a12c`,
and the `UnitOrder` base at `32355`):

| slot | meaning | evidence |
|---|---|---|
| `+0x4` | `clear` | `OrdersMemManager::get_obj@00730ac0` calls it on every pooled order it hands out |
| `+0xc` | `log_data` | |
| `+0x10` | `get_type` → `OrderIndex` | `MoveOrder`'s is COMDAT-folded onto `StrafeOrder::is_air` (a `return 1`): `MOVE_TO == 1` |
| `+0x14` | `is_move` | `UnitOrder::is_move_attack@0047ff00` = slot 0x14 ‖ slot 0x18; `MoveOrder`'s returns 1 |
| `+0x18` | `is_attack` | |
| `+0x20` | `is_target` | `Unit::work@0060d180` gates `check_target_path` on it |
| `+0x24` | `is_pathed` = `flags & 1` | `MoveOrder::is_pathed@00488980` |
| `+0x28` | `is_fleeing` = `flags & 2` | `MoveOrder::is_fleeing@004889a0` |
| `+0x3c` | `get_target_order` | |
| `+0x40` | `get_move_order` | every `do_move` entry: `pMVar3 = (order->vslot 0x40)()` |
| `+0x44/0x48/0x4c` | `get_attack_to / explore_to / flee_to` | `add_move_facing_order` |
| `+0x98` | `get_group_move_order` | `vtables.txt:33293` |
| `+0xb8` | the `MoveOrder` part of any move-family order | `kill_current_order` reads `+0x28 facing` through it; `Unit::work` reads `+4,+8` |

`OrderIndex` values pinned by this reading: `NONE 0, MOVE_TO 1, ATTACK_TO 2,
EXPLORE_TO 3, FLEE_TO 4, GATHER 7, ATTACK 10, CAST_SPELL 0xe, TRADE_ROUTE 0xf,
CHANGE_FORM 0x12, GROUP_MOVE 0x13, GROUP_ATTACK 0x14, GROUP_ATTACK_TO 0x15`
(`Unit::add_move_facing_order@005e55c0` maps its kind 1..4 onto the four
`get_obj(MOVE_TO/ATTACK_TO/EXPLORE_TO/FLEE_TO)`; `Unit::repath@005e29b0` and
`Unit::kill_current_order@005e2cb0` test the literal set `{1,2,3,4,0x12,0x13,
0x15}` where `resolve_unit_collision@005f9d30` tests the named set
`{MOVE_TO, ATTACK_TO, EXPLORE_TO, FLEE_TO, CHANGE_FORM, GROUP_MOVE,
GROUP_ATTACK_TO}`; `kill_current_order` pairs 7 with `remove_gatherer`, 0xf
with `end_trade_route`, 0xe with `unpay_cast_costs`; `Unit::do_job@00617a10`
lists the names in value order). `BUILD_AT` is 5 or 6 — not needed here.

### 1.2 `MoveOrder` — the fields, from their writers and readers

`types.txt:19084`. All `Coord`s are position units (1/192 tile, 1/768 world
cell), angles are the binary angle of `docs/MOVEMENT.md`.

| off | field | what it is |
|---|---|---|
| `+0x4` `x`, `+0x8` `y` | the destination, **snapped to the centre of its 48-unit cell**: `add_move_facing_order` takes `UCoord`s and stores `u*0x30 + 0x18`. `add_move_order@00616ed0` converts a `Coord` with `div_3_table[v >> 4]` (`= v/48`). The log confirms: a scout's `x 45048` is `938*48+24`; a citizen's `x 4008 = 83*48+24`. |
| `+0xc` `angle` | the facing to apply on arrival. `add_move_order` computes `find_angle(x − pos.x, y − pos.y)` — the direction from the unit to the target *at order time* (the decompiler prints `find_angle(param_3,param_4)`; the listing at `00616ed6..616f21` loads `ecx = x − x_internal`, `edx = y − y_internal` — `find_angle` is register-called). `Group::action_move_near@00704990` passes the group's facing with a per-slot byte packed into the top byte. Consumed by `move_step` on the final waypoint when this is the only order, and by `update_action` as `dest_angle` (§3.4). |
| `+0x10` `dest` | **"I have a current waypoint"**: set to 1 when `do_move` takes the stack top into `dest_x/dest_y`; cleared to 0 by arrival (`move_step`), by `go_around_building` failure, by `resolve_unit_collision`, by a collision detected on `coll_x/coll_y`, and by `ungroup_move_order`. |
| `+0x14` `tolerance` | **never written by anything read here** except `clear`/`operator=`; `UnitData::tolerance (+0x60)` is the live one (§2.4). Logged 0 throughout run4. |
| `+0x18` `pause` | frames to stand still: `resolve_unit_collision` sets `Random::get(0,0xffff) % 9 + 1` on a mutual collision with a non-flagged partner; `do_move` decrements and returns without stepping while non-zero (§2.6). |
| `+0x1c` `retry`, `+0x20` `attempts` | the modern-infantry entrench/pack wait: every 128 frames (phase `o*0x11 + frame`) a modern-infantry unit with no general sets `retry = guy0+0x78`, `attempts = −3`; each later frame `retry--`, and when it reaches 0 `attempts += 3`; `attempts` then counts down by one a frame. While `retry != 0` the unit does not step (§2.2). Zero for everything else. |
| `+0x24` `timer` | a self-destruct: `do_move` decrements it and at `1` kills the order and re-runs `Unit::work` the same frame. No writer found in the move family (it is set on *gather* orders by `do_move`'s GATHER branch via slot `0x6c`+0x24 — a different struct at the same offset). Zero in every logged move. |
| `+0x28` `facing` | passed in by the caller (−1 from `add_move_order`, the group's `+0xe88` from `action_move_near`); `kill_current_order` reads it (`−1 < facing` → after a non-forced kill, `unit_masks` bit 2 is set to `facing != 0` with a `reversing` twist, and the group's `+0x48` byte). Not used by the step. |
| `+0x2c` `dest_x`, `+0x30` `dest_y` | the current waypoint — **what `move_step` walks toward**. Initialised to `x,y`; rewritten from the stack top each time `dest` goes 0→1; moved by `find_path`'s pull-back and by `resolve_unit_collision`'s side-step. |
| `+0x34` `last_x`, `+0x38` `last_y` | the position at which the last straight-line plan was made: `find_path` sets them to the unit's position after a successful `go_around_building` detour; `do_move` sets both −1 when it takes a fresh target; a stack top equal to `last` (and not final) is popped before re-planning. Initialised −1,−1. |
| `+0x3c` `coll_x`, `+0x40` `coll_y` | where the suspended path search would resume; `do_move`'s pending-search branch probes `detect_unit_collision(coll)` every other frame (§2.1). Zero otherwise. |
| `+0x44` `orig_x`, `+0x48` `orig_y` | the un-snapped point the caller asked for (`action_move_near` passes the click point; `add_move_order` passes its `p8,p9`, −1 in every caller read). **Not reset by `clear` or the constructor** — only written by the adders and `reset_move_orders@005fd080` (which copies `x,y` back into them). Informational here. |
| `+0x4c` `off_x`, `+0x4e` `off_y` (short) | `x mod 0x300`, `y mod 0x300` — the destination's offset inside its world cell. `go_around_building` and `find_tpath` use `off % 0xc0` (the offset inside the *tile*) to place detour waypoints off-centre; `find_tpath` loads `pathfinder.offx/offy = off % 0xc0 − 0x60`. Log: `off_x 504` for `x 45048` (`45048 = 58*768 + 504`). |

`MoveOrder::clear@004889c0` zeroes everything but `orig_*`, sets `facing = −1`
and `UnitOrder::flags = 0`; the constructor does the same. `OrdersMemManager::
get_obj` calls `clear` on a recycled order, so a new move order starts clean
except for `orig_*` (which the adder overwrites anyway).

**`UnitOrder::flags` bits** (as read by the move family):

- `1` — *pathed*: "the top segment of `UnitData::path` is mine". Set by
  `do_move` after planning (`*pbVar1 |= 1`), cleared on the final waypoint,
  passed in by the adder (`add_move_facing_order` p5; `action_move_near`
  passes 1 because it has just emptied the stack; `check_target_path` passes
  0). `kill_current_order` pops the current order's path segment only when
  `is_move && is_pathed` (`kill_current_path@005e31d0`: pop until an entry with
  `flags & 1` is popped).
- `2` — *fleeing*; only `FleeToOrder` sets it (not read here).
- `4` — set from `add_move_facing_order` p7 / `add_group_move_order` p13;
  `UnitData::get_action@00608450` and `Unit::update_action@0060a870` treat a
  move order **without** bit 4 as a transit leg and look past it for the
  "action" order beneath — so bit 4 marks a move that is itself the action.
  `Unit::work` clears `unit_masks & 0x100` when the current order has it.
  `UnitData::invalid_loc@00607c30` also reads the *path* top's `flags & 4`
  (a different flag word — see §1.4).
- `0x20` — from p11; `move_step` on the final waypoint: a type with
  `unit_flags & 0x20` (flyer) carrying passengers and this bit set ejects them
  and kills the order (an air-transport unload). Not modelled.

### 1.3 `ExploreToOrder`, `FleeToOrder`, `AttackToOrder`, `GroupMoveOrder`

`ExploreToOrder` and `FleeToOrder` are `MoveOrder` with no extra fields (size
0x5c; `ExploreToOrder::ExploreToOrder@00481ec0` / `FleeToOrder@00481f50` call
`MoveOrder::MoveOrder` and only replace the vftable); their `log_data` opens
`EXPLORETOORDER` / `FLEETOORDER` and delegates to `MoveOrder::log_data` (the
scout's block in §5 is exactly that). `AttackToOrder` is also 0x5c.
`do_job` sends `MOVE_TO` and `FLEE_TO` to `do_move`; `EXPLORE_TO` to
`do_explore_to`, `ATTACK_TO` to `do_attack_to` (other readers). `GroupMoveOrder`
(size 0x78, `in_group +0x68`) extends the same 0x50 bytes with the group's
leader `o/who` (+0x54/+0x58), a group-move id (+0x5c), slot index (+0x60),
+0x64, `in_group` — `add_group_move_order@005e4710` fills them and `Unit::
do_group_move@005e79a0` calls `do_move` for the leader and `move_step` for the
followers (R-group's area; `ungroup_move_order@005fd140` converts a
`GROUP_MOVE` back into a `MOVE_TO` by `MoveOrder::operator=` and sets `dest = 0`,
`orig = x,y`).

### 1.4 The unit-side state: `UnitData` (`types.txt:43325`)

| off | field | role here |
|---|---|---|
| `+0x58 dest_angle` | written by `update_action`: the first move order's `angle`, else the unit's `angle`. The body turns toward it when idle (`docs/MOVEMENT.md`). |
| `+0x60 tolerance` | the **current waypoint's** tolerance, copied from the `PathData` when the waypoint is taken; raised on collision (§2.4). The arrival radius. |
| `+0x68 unit_masks` bit `8` | "**a straight line to `dest_x/dest_y` has been verified, go**" — set by `find_path == 0`, cleared when a new waypoint is taken, when `move_step` enters a tile that turns out invalid, and by `go_around_building` failure. It is the switch between the planning half and the stepping half of `do_move`. (Logged: `unit_masks 268435466 = 0x10000000|8|2` on the first moving frame.) |
| `+0x70 orders_x`, `+0x74 orders_y` | written only by `update_action` (and `Unit::init`): the first move order's `x,y` — "where this unit is going to be" — or the unit's position when the first order is not a move. Range checks use it (`check_target_path@005e22d0` → `is_in_range(…, orders_x, orders_y, …)`). Logged at `UNITS=3` (`tools/gamelog/aim.py` already reads it). |
| `+0x88 collide` | a collision counter: `do_move` resets it to 0 when a probe of `coll_x/y` comes back clear, `++` per resumed search; `resolve_unit_collision` `++`; read by the `find_tpath`-vs-`find_upath` choice (§2.5). |
| `+0xaf path_recursion` | `find_path`'s recursion depth: zeroed by `do_move` before each `find_path`, `++` per call (saturating at 255), `go_around_building` is refused at 10 and sets it to 10 on failure. Logged `path_recursion 1` after one straight-line check. |
| `+0xb8 Stack<PathData> path` | `{ list, size, length, increment }` (`types.txt:40517`); `init` gives 10 slots, `increment` 10. **The top is `list[length−1]`**; `peek` on an empty stack returns `list[0]` (stale data — callers guard with `length`). |
| `+0x104..0x114 openlist, openlistrefs, closedlist, validlist, blocklist` | a *suspended* A* search. `openlist != 0` means one is pending; `clear_partial_path@005e3920` returns them to the recyclers and zeroes `temp_x/y, tol, offset, endx/y, traversed, avoid_land/sea` (+0x118..+0x148). |
| `+0x120 avoid_x/y` | set −1,−1 by `do_move` just before `move_step`; `find_path` writes the point it could not reach when the pulled-back goal collapses onto the unit. |

`PathData` (`types.txt:40517`): `{ Coord to_x, to_y; int tolerance; int flags }`.
Flags as read here: `1` = the **final** waypoint of an order's segment (the
goal itself); `2` = a collision side-step (`resolve_unit_collision` pushes
one; `move_step` may snap through a collision on it; `find_upath` compacts
runs of them); `4` = **turn in place before walking to this one** (`move_step`
per `docs/MOVEMENT.md`; `do_move` sets it on a waypoint in a different terrain
region from the unit; `find_path` sets/clears it on re-pushed detour points by
comparing the two tiles' water bit; `invalid_loc` treats a top with it as
"may cross"); `8` = a `go_around_building` mid-detour point; `0x10` = a block to
`resolve_block@005fccc0` (a building's tile: own → mark it, allied → set a
diplomacy bit, enemy → `add_attack_order`); `0x20` = a caravan road
waypoint (`do_move`'s `TRADE_ROUTE` branch, `caravan_step` and `verify_road`).

### 1.5 The order list

`OrderList` (+0xc8, size 0x1c) is `{ vptr; LinkListBase<UnitOrder*> }`:
`+0xcc current_data, +0xd0 current_metric, +0xd4 current_node, +0xd8 length,
+0xdc head_node` (`types.txt:43471`). `RecycledOrderNode = { next, prev, data,
metric }` (`:43479`). `LinkListBase::add@0046d5a0` links the new node *before*
`head_node` and then **makes it the head**, so with the list circular the
logical order runs by `prev` from `head->prev`: `update_order@006179d0`,
`get_order@0060b030`, `get_action` all start at `head->prev` (the oldest =
current order) and advance by `prev`. `QUEUE_FIRST` (`add_move_facing_order`)
is `head = head->next` after the add, which rotates the new node to the front.
`length` is what `do_move`/`move_step` test as `orderlist +0x10 == 1` ("this
move is the only order"). **`OrderList::log_data@00730070` starts at the
current order and walks by `next` after a first advance, so the log lists the
orders in reverse execution order — the last one printed is the current
order** (§5; for two orders `[MOVE (current), GATHER]` it prints `GATHER`
then `MOVE`).

### 1.6 Lifecycle

**Created** by `Unit::add_move_facing_order@005e55c0` (the only constructor
path; `add_move_order@00616ed0` is a thin wrapper that snaps and computes the
angle). Its `QUEUE_NEW` branch first does `unit_masks &= ~0x4000000;
path.length = 0; close_orders(0); clear_partial_path(); update_action()` —
a new move wipes the path stack and every existing order. Then it takes a
pooled order of the kind, fills `x,y,angle,dest=0,dest_x/y=x/y,last=−1,
off_x/y, orig, facing, pause=retry=timer=0, flags 1/4/0x20`, appends it,
rotates it to the front for `QUEUE_FIRST` (also `clear_partial_path`), and
calls `update_action`. Two side rules in the same function: a `QUEUE_LAST`
`MOVE_TO` for a type with `role & 0x10` becomes an `EXPLORE_TO` and sets
`unit_masks |= 0x4000000` (`Unit::work` later rewrites it in place); an
`ATTACK_TO` while the unit's current order is `ATTACK` on a non-sea target
re-adds that attack `QUEUE_FIRST` after the move.

The player's right-click reaches it through `Group::action_move_to@0070fba0 →
action_move_near@00704990`, which for a group of fewer than two units calls
`add_move_facing_order(ux, uy, angle, MOVE_TO, pathed=1, qpos, p8, group+0xe88,
click_x, click_y, p11)` (line ~792) and for a larger land group
`add_group_move_order`. `Unit::go_to@005f7a50` (used by `go_to_unit`/
`go_to_city` when farther than `0x480 = 6 tiles`) builds a one-unit temporary
group and goes through the same `action_move_to` with `QUEUE_NEW`. The other
in-engine callers of `add_move_order` are the target orders' transit legs:
`check_target_path`, `do_attack`, `do_gather`, `do_garrison`, `do_trade`,
`do_cast`, `fight`, `repair_damage`, `target_opportunity`, `come_out`,
`check_meet_ship`, `do_non_flat_gather` — all with `QUEUE_FIRST`, most with
`pathed = 0`.

**Stepped** from `Unit::process → Unit::work@0060d180 → do_job(type, order)
→ do_move@005f7b30` once a frame while it is the head of the list (§4).

**Finished** by `kill_current_order(this, 0)` — from `move_step` on the final
waypoint, from `do_move` on a dead/unreachable goal, from `repath` when a
target order decides its transit legs are stale — which pops the current
order's path segment if it was pathed, `clear_partial_path`, `update_action`.

---

## 2. The per-frame step — `Unit::do_move@005f7b30`

`mo = order->get_move_order()`. In order, each frame:

### 2.1 Before planning

1. **Cavalry-archer fire on the move** (`unit_flags & 0x200000` with an attack
   value): `cavarch_fight` every 32 frames unless the combat stance is 5 or the
   action is `ATTACK` without `unit_flags & 0x400`. Combat's; not read further.
2. **A suspended search** (`openlist != 0`): the frame count since
   `collide_frame` gates an `ATTACK` retarget (`find_new_target`) every 4
   frames, a `GATHER` "close enough" (`vector_dist < 0x120`) that parks the
   unit (`avoid_x/y = x,y`, resets the gather order's fields, kills the move);
   then every other frame from 4 on, `detect_unit_collision(coll_x, coll_y,
   1,1,0,0,1) == 0` → `dest = 0; clear_partial_path; flags |= 1; collide = 0;
   return 0` — the blocker has gone, re-plan next frame; otherwise every 4
   frames `repaths[who]++`, `collide++`, `dest = 0`, and
   `PathFinder::find_upath_restore(path, who, o)` resumes the A* with limit
   `300 / repaths²`; if still suspended return 0; if it has been more than 5
   frames since `collide_frame`, `collide = 0`. **Returns 0 — no step while a
   search is pending.** (How a search becomes suspended is inside
   `astar_path` with `saving = 1`; not read.)
3. **`timer`**: if `> 0`: at 1 → `kill_current_order(0); Unit::work();
   return 0`; else `timer--`.
4. **The action under the move.** `act = get_action()` (the first order that
   is not a transit leg). If it is `ATTACK` (10): the target's object is
   resolved through slot `0x50` (`+8 ox, +0xc whom, +0x10 uid, +0x1c` a
   char); if alive with the same uid — for a ranged type (`max_range != 0`):
   when the target is not a building, not a flank (`flanking` / the
   `0x2aaaaaaa` = 60° cone), **in range, and no collision at the unit's own
   spot** → `kill_current_order` (the move has done its job); a melee type
   every 16 frames may `find_melee_target` and `change_target` to a closer
   in-range unit; with a building target: in range, valid, no collision →
   kill the move. If the target is gone and the unit is within `0x481` of the
   (register-passed) point and not `unit_flags & 0x10` → `repath(); return 0`
   (`repath` pops the leading transit legs so the target order re-issues).
   If it is `TRADE_ROUTE` (0xf): a caravan whose endpoints are negative →
   kill (`LAB_005f82a3`). These are combat/economy seams; summarised only.

### 2.2 The entrench wait (`retry` / `attempts`)

`if (retry != 0) { if (--retry == 0) attempts += 3; return 0; }` — no step.
Then the 128-frame modern-infantry check described in §1.2 sets
`retry = guy0+0x78, attempts = −3, set_anim(CHAR_PACK)` and returns 0.
Then `if (attempts != 0) attempts--`.

### 2.3 Planning: the first time, or with an empty stack

```
if (!mo->is_pathed() || path.length == 0) {
    push { mo->x, mo->y, tol 0, flags 1 }                       // the goal, final
    r = PathFinder::find_wpath(&pathfinder, &path, pos.x, pos.y, who, o)   // @00688fc0
    if (r < 1) { if (path empty || top != that goal) push it again;
                 r = PathFinder::find_tpath(..., &path, pos.x, pos.y, who, o)  // @006897d0
                 if (r < 1 && top != goal) push it again }
    flags |= 1                                                     // pathed
    if (path.length > 10) return 1                                 // long path: step next frame
}
```
So **every fresh move calls `find_wpath` exactly once**; on open ground within
one world cell it returns with the stack still `[goal]` (§3.2).

### 2.4 Taking a waypoint (`dest == 0`)

```
top = path[length-1]      (list[0] if empty — stale; callers keep it non-empty here)
dest = 1; unit_masks &= ~8; dest_x/y = top; UnitData::tolerance = top.tolerance
if (top.flags & 1) or TRADE_ROUTE:
    TRADE_ROUTE with top.flags & 0x20: caravan road bookkeeping (caravan_step, verify_road, clear 0x20 on the stack)
if !(top.flags & 1):
    if get_tregion(top tile) != get_tregion(unit tile): pop; push top with flags |= 4   // turn in place first
c = detect_unit_collision(top, 0,0,0,0,0)
if c: if (top.flags & 1) and action is TRADE_ROUTE/GATHER/ATTACK/BUILD_AT → kill the order (LAB_005f8851)
      else if the collider's order is not a move: t = collider.type.big_radius * 3;
           if UnitData::tolerance < t: tolerance = t; pop; push top with tolerance t    // give up short of a parked unit
if vector_dist(dest − pos) <= UnitData::tolerance:                                    // already there
    dest = 0; pop; if !(flags & 1) return 1
    if orderlist.length == 1: set_angle(mo->angle, 0)
    flags &= ~1; kill_current_order(0); return 0
if (top.flags & 0x10) and resolve_block(): return 1
```

### 2.5 The speed, and the straight-line check

`speed = get_speed(pos, 0)`; `× ai_speed` if > 1; `× 5/4` (truncating toward
zero, `(5s + (5s<0 ? 3 : 0)) >> 2`) for modern infantry — exactly as
`docs/MOVEMENT.md` states.

```
if !(unit_masks & 8):
    path_recursion = 0
    r = find_path(dest_x, dest_y)                                  // §3.1
    if r == 0: top = peek; if top == pos: masks &= ~8
               else: masks |= 8; dest = 1; dest_x/y = top; tolerance = top.tolerance
    elif r == 2: return 1
    if masks & 8: goto STEP
    // the straight line is not enough: ask the pathfinder
    if invalid_loc(tile of mo->x,y, 1,0,0,0,1) and orderlist.length > 1: kill twice (LAB_005f82a3), return 1
    n = Random::get(game_random, 0, 0xffff)                        // *** consumes the game RNG ***
    thr = n%5==2 ? 0x600 : n%5==0 ? 0x1800 : 0xf00                 // 2, 8 or 5 world cells, Manhattan
    if |dest − pos|_manhattan > thr:
        before = path.length; r = find_wpath(&path, who, o) (the 4-arg wrapper @00688e10, from the unit's position)
        if r > 0 and length unchanged: r = 0;   wflag = 1
    else:
        top = peek; if !(top.flags & 1) and top == last_x/y: pop
        if collide == 0: pop while !(top.flags & 0x21) and top.tolerance < 0x60;  r = find_tpath(&path, who, o)
        else:            r = find_upath(&path, who, o, 0)
        if r > 0 and length unchanged: r = 0;   wflag = 0
    if r == 0:
        if path.length == 0: goto TAKE
        top = peek
        if !(top.flags & 1): pop; (if a dropped register flag & 4 and more remain: pop the next and re-push it with flags |= 4); dest = 0; return 1
        pop; flags &= ~1; fallthrough to KILL
    elif r != -1:
     TAKE: last_x/y = −1; dest = 1; top = peek
        if (top.flags & 0x10) and resolve_block(): return 1
        path_recursion = 0; r2 = find_path(top)
        if r2 == 0: masks |= 8  elif r2 == 2: return 1
        if !(masks & 8): if wflag: return 1; resolve_block(); return 1
        top = peek; dest_x/y = top; tolerance = top.tolerance; goto STEP_IF_MOVING
     KILL: kill_current_order(0); if order_type is ATTACK or BUILD_AT: kill_current_order(0) again; return 1
```

Read as a rule: a far-off goal (more than 2/5/8 cells, drawn at random) is
planned on the **world-cell grid** (`find_wpath`, `astar_path` step `0x300`),
a nearer one on the **tile grid** (`find_tpath`, step `0xc0`) unless the unit
has been colliding (`collide != 0`), when it is planned on the **48-unit grid**
(`find_upath`, step `0x30`). A result of `−1` (goal off the map) kills the
order and, if the order beneath is an `ATTACK` or `BUILD_AT`, that one too.
A result of 0 with a non-final top drops that waypoint and tries again next
frame. A positive result, or a 0 on an empty stack, takes the top as the new
target and verifies the straight line to it with `find_path` **the same
frame** — so the unit steps on the frame the path is computed.

### 2.6 Stepping

```
if (unit_masks & 8):
    if pause != 0: pause--; if type not in {ATTACK_TO, GROUP_ATTACK_TO} or type.attack != 0: return 1
                   set_anim(CHAR_DEFAULT); return 1                // standing still
STEP: avoid_x/y = −1,−1; return move_step(mo, speed)
```
(`STEP_IF_MOVING` returns 1 if `masks & 8` is still clear.)

### 2.7 Arrival, in `move_step@005faf30`

The handoff. `move_step(mo, step)` reads `mo->dest_x/dest_y` as the target and
the stack top's `tolerance`/`flags` (by index — the top is not popped), calls
`set_angle(find_angle(dest − pos), 0)` first (so `UnitData::angle` is written
every step; the guy's facing is what then turns), does the turning and the
step exactly as `docs/MOVEMENT.md` gives them, and then:

- **Partial step** (`step < manh`): after moving, if
  `|dest − pos|_manhattan > UnitData::tolerance` → `return 1` (still going).
  Else **arrived**: `dest = 0; pop`; if the popped entry is not final (`flags
  & 1 == 0`) → `return 1` (next frame `do_move` takes the next waypoint, §2.4,
  and `masks & 8` is cleared there so `find_path` re-checks the new leg); if
  final: `if orderlist.length == 1: set_angle(mo->angle, 0)`; `flags &= ~1`;
  (the air-unload case); **`kill_current_order(0); return 1`**.
- **Full step** (`manh <= step`, the Manhattan snap): `detect_unit_collision
  (dest)`; if clear and the new tile is valid: `set_anim` (`CHAR_DEFAULT` if
  the unit is already on it and has at most one order), `set_new_location
  (dest)`; `dest = 0; pop` (collision: `collide_o/who = −1`, `dest = 0; pop`,
  default anim). If the popped entry is not final → `return 1`. Else the
  facing: `if orderlist.length == 1 or the action is GATHER: set_angle(mo->angle, 0)`;
  `flags &= ~1`; `kill_current_order(0); return 1`.
- A step into a tile `invalid_loc` says is blocked: `unit_masks &= ~8; return 0`
  — next frame `do_move` re-plans from §2.5. A collision on a partial step
  with `tolerance` already raised: `UnitData::tolerance = 2 × manh` (the
  give-up short of `docs/MOVEMENT.md`), otherwise `resolve_unit_collision`.

**Two arrival tests, two metrics.** Before stepping, `do_move` tests the
Euclidean `vector_dist(dest − pos) <= tolerance`; after a partial step
`move_step` tests the Manhattan `|dx|+|dy| <= tolerance`. With tolerance 0
(every goal, every tile/48-grid waypoint's last entry) only the exact snap
arrives; a world-cell waypoint (`tolerance 0x180`) or a tile waypoint
(`0x60`) is "reached" when the unit passes within that Manhattan distance,
which is how a pathed unit cuts its corners.

**What is returned.** `do_move` returns 0 for "nothing to do this frame" and
1 for "did something" (including stepping and killing); `do_group_move` uses
the value to decide whether to update the group's positions.

---

## 3. The seams

### 3.1 `Unit::find_path@005fb910` — the straight-line verifier

Called with a waypoint `(x, y)` and returns **0 = walk straight (or a detour
was pushed and verified)**, **1 = cannot / do not — plan**, **2 = abort this
frame**.

```
if type is a flyer (unit_flags & 0x20): return 0
if invalid_loc(my tile, 1,0,0,0,0): find_nearby_spot; set_new_location(there, 1, 0); return 1   // teleport off a bad tile
if (x,y) == pos: return 0
mo = update_order()->get_move_order()
if cell-Manhattan(pos, goal) > 4 and !invalid_loc(goal tile, 1,1,0,0,0): return 1    // far and reachable: the pathfinder's job
ang = find_angle(goal − pos); spd = max(get_speed(pos,0), 3); sx = sinx(ang, spd); cy = cosx(ang, spd)
path_recursion++ (saturating 255)
while invalid_loc(goal tile, 0,0,0,0,0):                                              // pull the goal back toward us
    clamp sx, cy to the remaining dx, dy
    if goal == mo->dest_x/y: mo->dest_x -= sx; mo->dest_y += cy
    if path not empty and top == goal: rewrite top the same way
    goal -= (sx, −cy); if sx == 0 and cy == 0: break
if domain == 2 (air): return 0
dx, dy = goal − pos; if both 0: avoid_x/y = goal; return 1
march from pos toward goal in steps of spd (components clamped to the remainder):
    each time the march enters a new tile: if invalid_loc(tile, 0,0,0,0,0):
        if path_recursion < 10: go_around_building(last good point, ang, spd)    // pushes 1–3 waypoints
            if it did not give up (path_recursion != 10):
                w = the first detour point (the top); pop what it pushed into a side stack, skipping one that equals pos
                if w is not pos, not mo->last, not the goal, and pos != mo->last:
                    r = find_path(w)                                              // recurse toward the detour
                    if r == 0: if !(masks & 8): re-push the side stack onto path (fixing flag 4 by the water bit of each pair of tiles), mo->last = pos, re-push the top likewise, masks |= 8
                               return 0
                    if r == 2: return 2
        return 1                                                                 // blocked, no detour: plan
    if the remainder is within one step on both axes: return 0                  // clear
    if the Manhattan remainder grew since last time: return 0
return 0
```
`go_around_building@005fc350` is a tile-walk along the blocking tile's edge
(sideways in the axis the march was not moving on, both directions, bounded by
the map) that pushes up to three `PathData`s — the turn-in point (`flags 0`),
a midpoint (`flags 8`) and the target tile centre (`flags 0`) — each placed
`off_x/off_y % 0xc0 / 2 − 0x30` from the tile centre; if the detour would be
longer than `0x300` or nothing works it sets `mo->dest = 0; masks &= ~8;
path_recursion = 10`. It is the only thing besides the pathfinder and
collision that pushes waypoints. Not modelled here; its trigger is an
invalid tile on the straight line, which open ground never has.

**On open, flat ground** (every tile the march crosses returns
`invalid_loc == 0`), `find_path` returns 0 for any goal within 4 world cells
(Manhattan, on the cell grid `div_3_table[v >> 8]`), and 1 beyond. That is
the whole reason the near move never touches `PathFinder`.

### 3.2 `PathFinder` — the interface, not the internals

All five entry points share the globals `pathfinder` (`PathFinderData` at
`+0x40`: `pathing_unit +0x54, sx/sy +0x58/5c` = the unit's tile, `offx/offy`,
`army +0x70, iroquois +0x74, worker +0x78, limit +0x80, saving +0x84,
scouting +0x94`) and the static `grouppath`. They take the unit's own
`Stack<PathData>`, **pop the goal off its top, and push back what the unit
should walk, top first**; the return value is the stack length, `0` for
"no path" (the goal is popped and not returned unless it was final, then
`−(flags & 1)` in `find_wpath`'s A* case), `−1` for "goal off the map" (the
stack is emptied).

| entry | grid / `astar_path` step | start | what it does before A* |
|---|---|---|---|
| `find_wpath(Stack*, Coord x, Coord y, who, o)` @00688fc0; the 4-arg wrapper @00688e10 passes the unit's position | world cells, `0x300` | explicit | same cell or flyer → push goal back, return length. Else (for leaders without flag 4: walk from the goal toward the start in `0x180`/`0x30` steps until `get_tregion` matches and, for sea, `invalid_loc(…,0,1,1,1,0)` is clear; for flag-4 leaders: a `was_seen` fog walk down the stack) → push goal; **cell-Manhattan < 3 → return length**. Else push `{goal cell centre, tol 0x180, flags 0}`, `{start cell centre, tol 0}`, set `scouting` (explore orders of `role & 0x10` types), `army` (a non-worker, non-attacking unit with no attack value, or `is_supply`, off `world+0x134 & 0x100` cells), `worker`; `astar_path(0x300, 0)`; 0 → pop, return `−(flags & 1)`; else length. Clears `army/worker/iroquois`, `kill_lists`. |
| `find_tpath(Stack*, x, y, who, o)` @006897d0; wrapper @00688e60 | tiles, `0xc0` | explicit | not `saving`: same tile or flyer → push goal (tol 0), return. Else walk from the goal toward the start until `get_tregion` matches and `valid_tcoord`; if within `0x60` on both axes without that and the goal is not final → return 0. Then tile-Manhattan < 2 → push goal, return; else push goal, `{goal tile centre, tol 0x60 (0x180 kept if it had it), flags 0}`, `{start tile centre, tol 0}`; `offx/offy = mo->off % 0xc0 − 0x60` when the current order is a move; `astar_path(0xc0, 0)`; on failure and not `saving`, pop a non-final top. |
| `find_upath(Stack*, x, y, who, o, anti)` @00682f30; wrapper @00688eb0 sets `limit = 500 / repaths[who]²` (halved with `anti`) | 48-unit cells, `0x30` | explicit | same 48-cell → push goal (tol 0), return; walk from goal toward start on the 48-grid with `get_tregion` + `valid_ucoord`; within `0x18` → return 0 unless final; 48-Manhattan < 2 → push goal, return; else push goal, goal 48-centre, start 48-centre; `astar_path(0x30, anti)`; on failure (not `saving`): pop a non-final top, then **kill the unit's current order** unless it is a move whose `mo->retry != 0`; on success with more than three entries: drop a top equal to the unit's position, **compact consecutive `flags & 2` side-steps that are collinear or 48 apart**, re-push. |
| `find_upath_restore(Stack*, who, o)` @00688f40 | as above with `saving = 1`, `limit = 300 / repaths²` | unit pos | resumes the suspended search |
| `find_wpath_army` @00683730 | `army = 1`, into `grouppath` | | the group's |
| `find_road(...)` @00688a40 | caravan roads between two buildings | | not the move order's |

`repaths[who]` (`GameDaemon`) is a per-player pressure counter: halved each
frame (`GameDaemon::process_all@00732700`, to 0 below 3), incremented per
pathfinder call under collision (`do_move` §2.1, `resolve_unit_collision`),
and it shrinks `find_upath`'s node limit quadratically — a player whose units
are all colliding gets shallower searches.

**What a straight-line stand-in must return to be faithful on open flat
ground.** For a goal in the unit's own world cell, or within two cells
(cell-Manhattan < 3): `find_wpath` leaves the stack as `[goal{flags 1, tol 0}]`
and returns 1 — a stub that does exactly that is *exactly* right, and since
`find_path` then verifies the line and takes over, the pathfinder is not
consulted again. Beyond two cells it pushes `{goal-cell centre, tol 0x180}`
and whatever `astar_path` adds between the start-cell centre and it — on open
ground presumably the chain of cell centres, each with tolerance `0x180`,
which the unit then reaches by passing within 384 Manhattan of each; a stub
returning `[goal]` would make the unit walk the straight line instead, which
is the same path only when the straight line passes within 384 of every
cell centre A* would pick (it does not in general — the chain hugs the grid).
It would also skip the `Random::get` in §2.5 only if it *also* made
`find_path` return 0 — the RNG draw happens on the `do_move` side whenever
`find_path` returns 1, pathfinder or not, so a faithful stub must leave that
draw in place. And it would get wrong: the `find_tpath` refinement when the
nearer branch is taken (tile centres, tolerance `0x60`), the `find_upath`
branch under collision, `tregion` crossings (flag 4, turn in place), and the
`−1`/kill exits. What it would not get wrong: the first frame's step (the
first waypoint's direction is within the start cell either way only when the
goal is near; for a far goal the first leg is toward the *next cell centre*,
not the goal — so even the first step differs beyond two cells).

### 3.3 Collision (`detect_unit_collision@00617060`, `resolve_unit_collision@005f9d30`)

`detect_unit_collision(x, y, a, b, c, d, e)` returns non-zero when the unit's
collision circles at `(x, y)` overlap another unit's (or, with some flags, a
building); `do_move` probes the waypoint with it before taking it and
`move_step` probes every step. `resolve_unit_collision(x, y)` is what a
failed step falls into: it may kill the order (arrived at a wall it was
building; a transport to board; in range of the target), attack the blocker
(`add_attack_order QUEUE_FIRST`), wait (`pause = rand % 9 + 1`), side-step
(push `{tile corner, tol 0, flags 2}` and point `mo->dest_x/y` at it), or,
with `path.length != 0` and `repaths[who] < 16`, pop the stack down to a real
waypoint (tolerance ≥ `0x60`, or no `flags & 2`, or one `CollCheck::
collide_here` is happy with), snap the unit to its 48-cell centre
(`set_new_location(…, 1, 0)`) and call `find_upath(…, anti = action is ATTACK)`;
on 0 it sets `mo->dest = 0`. It bumps `collide` and `collide_frame` (the
`unit_masks |= 0x40` "yield" is set for a mutual collision between two moving
own units under the `repaths` threshold). The one thing the move order needs
from it is that it only ever *pushes* waypoints and writes `dest/dest_x/y/
pause`; the next `do_move` handles everything else.

### 3.4 `update_action@0060a870` — `orders_x/y` and `dest_angle`

Runs at the end of every `add_*_order`, `kill_current_order`, `close_orders`,
and at several points in `do_move`. It writes `orders_x/y = pos; dest_angle =
angle`; then if the first order is not a move, leaves them; else walks the
leading move orders (those with `is_move` and without `flags & 4`, and any
`CHANGE_FORM`), writing `orders_x/y = mo->x/y; dest_angle = mo->angle` for
each, and once more for a final move that is the action itself. It returns
the action order (or 0 if the list ends in transit legs). So `orders_x/y` is
the *last* leading move's destination and `dest_angle` its angle — the log
shows `dest_angle` equal to the queued move's `angle` on the frame the move is
queued, before the unit has turned (`angle 1431655765`, `dest_angle
−381943808` at frame 3; both `−381943808` at frame 4).

### 3.5 `set_angle@00605400` / `set_new_location@005f8d20`

`Unit::set_angle(a, _, snap)`: writes `UnitData::angle = a` unconditionally,
toggles `unit_masks & 2` when the change is more than a quarter turn
(and flips the group's `+0x48` for its leader), then `Guy::set_angle(guy0, a,
snap)` — `docs/MOVEMENT.md`'s snap flag. `move_step` calls it with snap 0 for
the heading every step and for `mo->angle` on arrival. `set_new_location(x,
y, a, b)` moves the unit (and the body's `des_x/y`), returning 0 when refused;
`move_step` returns 1 on a refusal without arriving.

---

## 4. Where it sits in the frame

`Game::do_frame@00591ef0`: `Leaders::process_all` → `NetDaemon::process_all`
(the order stream, several times) → `Leaders::strategy_all` (AI) →
`GameDaemon::process_all` (halves `repaths`) → `Armies::process_all` →
`NetDaemon::process_all` → **`Objects::process_all`** → `Objects::inc_time`.
Inside `Unit::process`, `Unit::work` (vslot `0x188`) runs at the bottom —
after `process_attrition` and the supply check — and the guys after it
(`docs/MOVEMENT.md`, "Where movement sits in the frame"). So a move order
applied from the stream for frame *F* is stepped by `do_move` in frame *F*'s
`Objects::process_all`, after that frame's attrition, and the position logged
at the end of *F* already carries the first step. `Unit::work`
itself: picks the head order, does the `MOVE_TO` → `EXPLORE_TO` rewrite for
`unit_masks & 0x4000000`, and for every non-move head order every 16 frames
(phase `o`) runs `check_target_path` on a target order (which may `repath`,
`add_move_order`, or kill); it clears `unit_masks & 0x10` at the end.

`do_move` is reached for `MOVE_TO` and `FLEE_TO` from `do_job`; `do_attack_to`
and `do_explore_to` own their own entry but share everything from §2.3 on
(they call `do_move` or reuse its pieces — not read here); `do_group_move`
calls `do_move` for the leader. The construction clock, `fight` and the rest
are reached only after this move order is killed and the order beneath
becomes the head.

---

## 5. What the gamelog writes

`UnitData::log_data@0060b070` opens detail 3 (`0x28))(3)`) right after the
object base, writes `angle, dest_angle, trench_angle, tolerance, orders_x,
orders_y, …`, and at the end `Stack<PathData>::log_data(&path)`,
`OrderList::log_data(&orderlist)`, then the guys. At `UNITS=3`:

```
   BEGIN STACK<TYPE>              Stack<PathData>::log_data@0046fda0 — only when length != 0:
    size 10
    length 3
    increment 10
    BEGIN PATHDATA                 PathData::log_data@0046fb70, bottom of the stack first
     to_x 45024
     to_y 19680
     tolerance 0
     flags 1                       <- the goal (final), pushed first
    BEGIN PATHDATA
     to_x 45816 ... tolerance 384 flags 0
    BEGIN PATHDATA
     to_x 45048 to_y 18168 tolerance 384 flags 0     <- the top = current waypoint
   length 1                        OrderList::log_data@00730070: the list length (no BEGIN of its own)
   type 3                          per order: get_type()
   metric 0                        the node's metric byte
   BEGIN EXPLORETOORDER            the order's own log_data (EXPLORETOORDER / FLEETOORDER wrap MOVEORDER)
    BEGIN MOVEORDER                MoveOrder::log_data@00487e70
     BEGIN UNITORDER
      flags 1                      UnitOrder::log_data@0047f960
     x 45048
     y 19704
     angle 1913782272
     dest 0
     tolerance 0
     pause 0
     retry 0
     attempts 0
     timer 0
     facing 0
     dest_x 45048
     dest_y 19704
     last_x -1
     last_y -1
     coll_x 0
     coll_y 0
     orig_x 45024
     orig_y 19680
     off_x 504
     off_y 504
   length 2                        <- PtrArray<Guy>
```
(`gamelog-run4-gunpowder-nubian-leaders9.txt:7712-7760`, the AI scout.) The
twenty keys are `x y angle dest tolerance pause retry attempts timer facing
dest_x dest_y last_x last_y coll_x coll_y orig_x orig_y off_x off_y`, in that
order — read back from the PE at the addresses the decompile cites
(`0xb3fc34 …`), which also settles the `this[-1]` offset shift. Three reader
traps: an empty stack writes only `BEGIN STACK<TYPE>` and the next `length`
line is the *order list's*; `STACK<TYPE>` is a template name shared with
every other `Stack<T>` (the guys' `size/length/increment` follow the orders);
and **the orders are listed in reverse execution order** — the last one
printed is the current order (the citizen block in §6 lists `GATHERORDER`
then `MOVEORDER`; the move is the head, as `orders_x = 4008` — the move's
`x` — confirms through `update_action`).

The position fields the harness already diffs (`docs/DATALAYER.md` §3) are
`x_internal/y_internal` at level 0; `rondata::gamelog` does not yet read
`MOVEORDER`, `PATHDATA`, `orders_x/y`, `dest_angle`, `unit_masks` or
`path_recursion`, all of which are present at `UNITS=3` and are the natural
next things to link (`orders_x/y` and the stack are the cheapest way to feed
the simulation the order without a recorded-game parser).

---

## 6. The check the harness will run

A citizen ordered to a point on open ground, 25 position units a frame.

**What determines the first moving frame.** The frame on which `do_job`
first dispatches the `MOVE_TO`. For an order applied from the stream
(`NetDaemon::process_all` runs before `Objects::process_all`), that is the
same frame: `do_move` plans, verifies and steps in one call (§2.3–§2.6). For
a move queued *by another order during `Unit::work`* — the starting citizens'
`do_gather`, which `add_move_order`s `QUEUE_FIRST` while the gather order is
the one being stepped — it is the **next** frame, because `do_job` has already
dispatched. run4 shows exactly that: at the end of frame 3 citizen `o 1`
(`4248, 28680`) has `[MOVE_TO x 4008 y 28296 dest 0 flags 0]` under its
gather order, an empty path stack, `idle 1`, `dest_angle` already the move's
angle and `angle` still `0x55555555`; at the end of frame 4 it is at
`(4234, 28659)` — a step of `(−14, −21)`, `vector_dist 25` — with `angle =
dest_angle = −381943808` (= `find_angle(−240, −384)`, 32° west of north,
turned instantly from a standstill per `docs/MOVEMENT.md`), `dest 1`,
`dest_x/y 4008/28296`, `UnitOrder flags 1`, `unit_masks 0x1000000A` (bit 8:
straight line verified; bit 2: `set_angle`'s quarter-turn toggle),
`path_recursion 1`, and the stack `[{4008, 28296, tol 0, flags 1}]` — length
one. The lag between the click and the frame the order is *applied* is the
order stream's, not this mechanic's (not established here).

**The path's shape on open ground.** The goal is 240 × 384 units away —
1.25 × 2 tiles, in the adjacent world cell. `find_wpath` returns with the
stack `[goal]` (cell-Manhattan 1 < 3); `find_path` marches the line in
25-unit steps, every tile valid, returns 0; `masks |= 8`. From then on each
frame is `do_move → (masks & 8) → move_step(goal, 25)`: a straight line at
`find_angle`'s quantised heading, the `sinx/cosx` components with the table's
rounding (`−14, −21` here, Manhattan 35 for a Euclidean 25), until
`manh <= 25` snaps it onto `(4008, 28296)` exactly; that frame pops the final
entry, applies `set_angle(mo->angle)` if the move is the only order (here it
is not — the gather is beneath, but the `GATHER` action also gets the
facing), clears `flags & 1`, and `kill_current_order` — the gather order
becomes the head the same frame and runs its own step the *next* frame. No
`PathFinder` call, no RNG draw, no waypoint but the goal. The simulation's
`Sim::order_move(unit, dest)` + `process_movement` (`crates/sim/src/lib.rs:1457`)
already produces this trajectory for a dest the unit can reach straight; what
it lacks is the order record (so the facing on arrival, the kill, and the
order beneath), the 48-grid snap of the destination, and the "next frame"
rule for orders queued from inside the step.

**Beyond two world cells** the same citizen would draw one `Random::get`
(`§2.5`) and, above the drawn threshold, walk a chain of cell-centre waypoints
with tolerance 384 — so a far move is the first thing that needs `PathFinder`,
and the first thing that diverges if the RNG stream is not in step.

---

## 7. Confidence, what is not established, behavioural checks

**High.** The struct and its offsets (PDB + the log labels read from the PE +
the two real blocks); the constructor and `clear`; `add_move_facing_order`'s
fills including the 48-snap and `off_*`; `add_move_order`'s angle (listing);
the order list's direction and the log's newest-first order (`LinkListBase::
add`, `OrderList::log_data`, confirmed by `orders_x` in the citizen block);
`update_action`'s outputs; `do_move`'s planning decision tree (§2.3–§2.6) and
both arrival rules; `find_path`'s 0/1/2 contract and its 4-cell rule; the
three `find_*path` wrappers' grids, their same-cell / <3-cell early returns,
and what they push before `astar_path`; `kill_current_order`'s path pop;
`repaths`; the first-moving-frame rule (decompile + log); the 25-unit
straight line (log).

**Medium.** The ATTACK/TRADE_ROUTE branches of §2.1 (summarised from one
reading; combat's reader should own the in-range test); `go_around_building`'s
geometry (read once, not re-derived — its trigger is absent on open ground);
`resolve_unit_collision` (summarised); `do_group_move`'s use of `do_move`.

**Not established.**

- What `astar_path` pushes on open ground between the start-cell centre and
  the goal-cell centre (the chain and its tolerances) — the next mechanic; §3.2
  says what a stub must and cannot reproduce.
- `UnitOrder::flags & 4` and `& 0x20`: named by their readers (a move that is
  the action itself; air unload). Which callers set them, and why `Unit::work`
  clears `unit_masks & 0x100` on them, is group/air territory.
- The leader flag `leaders[who] & 4` that switches `find_wpath` and
  `invalid_loc` onto `was_seen` (fog-respecting pathing) — human vs AI is the
  guess.
- What sets a search to `saving` (the suspended-search state `openlist != 0`)
  and how `coll_x/coll_y` get filled — both inside `astar_path`.
- `MoveOrder::tolerance` (+0x14) has no writer found; `UnitData::tolerance`
  is the live one. Whether anything reads +0x14 beyond `log_data`/`operator=`.
- `timer`'s writer for a move order (none found; the only `+0x24 = 1` writer
  targets the gather order).
- The group's packed top byte in `angle` (`action_move_near`): what the slot
  byte does on arrival for a single unit (for the citizen it was 0).
- `UnitData::angle` is written with the *heading* by `move_step`'s first
  `set_angle`, yet `docs/MOVEMENT.md` logged a facing 1–2° off the heading
  while walking; `Guy::set_angle`/`do_turn` may write it back. The citizen
  case cannot tell (instant turn). Flagged for the movement document, not
  changed here.
- The order stream's delay from click to applied frame.

**Behavioural checks worth running** (each a logged run at `UNITS=3`):

1. *The far-move chain.* Order a citizen 10+ cells across open ground; read
   the `PATHDATA` entries on the first moving frame and the per-frame
   positions. Settles the open-ground `astar_path` output (the tolerance-384
   chain) and whether the first step points at the next cell centre.
2. *The RNG draw.* Same run, two seeds: the 2/5/8-cell threshold is drawn on
   the first `do_move` of any move that `find_path` refuses; a move between
   2 and 8 cells should switch between `find_wpath` and `find_tpath` across
   seeds (visible as cell-centre vs tile-centre waypoints).
3. *Arrival facing.* One unit, one move order, no others: on the arrival
   frame `angle` should equal the order's `angle` (the direction from the
   start point to the goal at order time) — `set_angle(mo->angle)` with
   `orderlist.length == 1`. Then the same with a second order queued behind
   (`length 2`): the facing should *not* be applied.
4. *The stream lag.* `cheat select` + right-click at a known frame; the first
   frame with `dest 1` gives the applied frame; the step is on that frame.
