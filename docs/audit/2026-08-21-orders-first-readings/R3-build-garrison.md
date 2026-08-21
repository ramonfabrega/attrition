# R3 — BuildOrder, RepairOrder, GarrisonOrder (Board/AwaitBoard briefly), and `come_out`

First reading, 2026-08-20, from the full decompile export
(`~/ghidra-projects/decomp`), `types.txt`, `vtables.txt`, and one logged run
(`Logs/gamelog.txt`, the latest, `UNITS=3 BUILDS=6`). Everything about the
building's side of these orders — the construction clock, `do_construct`,
`repair_damage`, the `do_garrison` gates, `go_inside`/`come_out`'s placement —
is already in `docs/CITIES.md` (§3.3, §6.4–6.7, §9.3) and is cited rather than
repeated. What is new here is the **order** side: the structures, the queue
they live in, how the builder gets to the site and what it does when it is
done, the auto-tasking that hands a citizen its next order, and what the log
writes. Where this reading disagrees with `CITIES.md` it says so (§7).

---

## 1. What it is

### 1.1 The structures

All five are `TargetOrder`s, and `TargetOrder` **virtually** inherits
`UnitOrder` (the constructors set a `_vbtable_` at `+0x4` and the `UnitOrder`
vtable at the tail). Layout, from `types.txt` plus the constructors
(`BuildOrder::BuildOrder@004820c0`, `RepairOrder::@00482330`,
`GarrisonOrder::@00484850`, `BoardOrder::@00482190`,
`AwaitBoardOrder::@00482260`):

| offset | `TargetOrder` (size 0x20) | `GarrisonOrder` (size 0x24) |
| --- | --- | --- |
| `+0x0` | vftable (TargetOrder's own virtuals: `get_target`, `target_exists`, `is_targeted`, `get_target_order`) | same |
| `+0x4` | vbptr | same |
| `+0x8` | `int ox` — the target's object number | same |
| `+0xc` | `int whom` — the target's owner | same |
| `+0x10` | `ushort uid` — the target's `ObjectData::uid` at order time | same |
| `+0x14` | — | `int search` |
| `+0x18` | `UnitOrder` subobject: vftable | (at `+0x1c`) |
| `+0x1c` | `UnitOrder::flags` (char) | (at `+0x20`) |

`BuildOrder`, `RepairOrder`, `BoardOrder`, `AwaitBoardOrder` add **no fields**
(all size 0x20); they differ only by `get_type`: `BUILD_AT`
(`BuildOrder::get_type@00482b50`), `REPAIR` (`@004827a0`), `BOARD_SHIP`,
`AWAIT_BOARD` (`@004828f0`); `GarrisonOrder::get_type@004847e0` → `GARRISON`.

Constructor state: `ox = whom = -1`, `uid = 0xffff`, `flags = 0`,
`search = 0`. `TargetOrder::clear@0047ff80` (called on a **recycled** order by
`OrdersMemManager::get_obj@00730ac0`, vtable slot `+0x4`) resets the same
three and `flags = 0`; `GarrisonOrder::clear@00484820` also sets
`search = -1` — the constructor's `0` and the clear's `-1` differ, but
`add_garrison_order` always writes the field, so nothing reads the default.

`UnitOrder::flags` (`+0x4` of the subobject): the only bit this sub-area
writes is **bit 2 (value 4)**, set from the last argument of every
`add_*_order` (`param_4`/`param_5`, below); I call it the **forced** bit. It
is read by `do_repair` (a building under attack is repaired only by a forced
order), by `Unit::work@0060d180` (a unit with `recharging != 0` steps only a
forced order; `unit_masks & 0x100` is cleared when the current order is
forced), and passed through by `action_swarm_around`. `add_board_order` sets
it unconditionally. MoveOrders show `flags 1` in the dump — that bit is the
move family's, not this reading's.

**`OrderIndex` values** (the enum is not in the export; derived from
`Unit::do_job@00617a10`'s case order and the literals the functions compare):
`NONE 0, MOVE_TO 1, ATTACK_TO 2, EXPLORE_TO 3, FLEE_TO 4, (5 unused by
do_job), BUILD_AT 6, GATHER 7, BOARD_SHIP 8, AWAIT_BOARD 9, ATTACK 10, FOLLOW
11, GUARD 12, REPAIR 13, CAST_SPELL 14, TRADE_ROUTE 15, STRAFE 16, AIR_PATROL
17, CHANGE_FORM 18, GROUP_MOVE 19, GROUP_ATTACK 20, GROUP_ATTACK_TO 21,
GROUP_PATROL 22, ATTACK_GROUND 23, AIR_ATTACK_GROUND 24, SPECIAL_ANIM 25,
GARRISON 26, THINK 27`. Pinned by the dump: `type 6` precedes every
`BUILDORDER`, `type 7` every `GATHERORDER`, `type 1`/`type 3` the
`MOVEORDER`/`EXPLORETOORDER` blocks; `kill_garrison_order` tests `0x1a`.

**Vtable slots on a `UnitOrder*`** (how every caller reaches the fields;
`vtables.txt` lists the order classes only partially, so these are by use):
`+0x4 clear`, `+0xc log_data`, `+0x10 get_type`, `+0x14 is_move` (true for the
MoveOrder family), `+0x18 is_attack`, `+0x20 is_targeted`, `+0x24
is_pathed`, `+0x3c get_target_order`, `+0x40 get_move_order`, `+0x60
get_build_order`, `+0x64 get_board_order`, `+0x68 get_await_board_order`,
`+0x6c get_gather_order`, `+0x70 get_repair_order`, `+0x7c get_cast_order`,
`+0x80 get_trade_order`, `+0xa8 get_spec_anim_order`, `+0xac
get_garrison_order`; the `const` twins sit `+0x78` higher (`+0xb4` target,
`+0xb8` move, `+0xd8` build, `+0xe4` gather, `+0xf4` cast — `MoveOrder`'s
listing shows `get_move_order` at both `+0x40` and `+0xb8`). All of the
`get_*_order` accessors return the order's own address, which is why every
consumer reads `ox/whom/uid` at `+8/+0xc/+0x10` of whatever it got back.

### 1.2 The queue — `OrderList` at `UnitData +0xc8`

`LinkListBase<UnitOrder*, uchar, RecycledOrderNode>` inside `OrderList`:
`current_data +0xcc`, `current_metric +0xd0`, `current_node +0xd4`, `length
+0xd8`, `head_node +0xdc`. A circular doubly-linked ring of
`RecycledOrderNode {next +0, prev +4, data +8, metric +0xc}`.

- `add@0046d5a0` links the new node **before `head`** (at the ring's tail)
  and then makes it `head`. `remove_current@0046d620` unlinks `current_node`,
  advances `current` to its `next`, and if the removed node was the head,
  `head = head->next`.
- **The executing order is `head->prev`** — every reader (`Unit::work`,
  `UnitData::order_type@00616e80`, `get_action@00608450`, `update_action`,
  `kill_current_order`) first does `current_node = head->prev`. So the queue,
  in execution order, is `head->prev, head->prev->prev, …, head`.
- `QueuePos`: **QUEUE_LAST** = `add` (the new order is newest; it runs last);
  **QUEUE_FIRST** = `add` then `head = head->next` (`add_build_order`,
  `add_garrison_order`: `*(&this->field_0xdc) = **(&this->field_0xdc)`), which
  makes the new node `head->prev` — it runs next; **QUEUE_NEW** =
  `unit_masks &= ~0x4000000; path.length = 0; close_orders(0);
  clear_partial_path; update_action` (clears everything — `close_orders@005e37f0`
  kills the current order while its type is not NONE, i.e. all of them) and
  then `add`.
- `Group::action_swarm_around` implements QUEUE_FIRST differently: it detaches
  the list (`set_up_insert`), `action_halt(0)`, re-enters itself with
  QUEUE_NEW, and re-appends the old list (`finish_insert`) — so the new
  move+build pair goes in front of everything that was queued.
- `OrdersMemManager::get_obj(type)@00730ac0` pops a recycled order of that
  type (calling its `clear`) or allocates; `kill_current_order@005e2cb0`
  gives it back (`give_obj`).

### 1.3 Who creates them

| | caller | args | pos, forced |
| --- | --- | --- | --- |
| `Unit::add_build_order@005e5210` | `Group::action_swarm_around@0070fbe0` (the UI/AI group build action and every re-approach), `Unit::check_build_order@00603470`, `Wall::process@00640450` (AI builder recruitment), `Unit::come_out@00617c10` (rally point on an unfinished own site) | `(o, who, pos, forced)` | swarm: `QUEUE_LAST, forced-through`; check_build_order: `QUEUE_FIRST, 1`; Wall::process: `QUEUE_NEW, 0`; come_out: `QUEUE_LAST, 0` |
| `Unit::add_repair_order@005e4ff0` | `Group::action_repair@007020c0`, `action_swarm_around` (REPAIR), `come_out` (rally point on a damaged own building) | `(o, who, pos, forced)` | swarm `QUEUE_LAST`; come_out `QUEUE_LAST, 0` |
| `Unit::add_garrison_order@005e4080` | `Group::action_garrison@00700490`, `Unit::do_garrison` (re-order to a sibling building), `come_out` (rally point on a building that accepts the unit) | `(o, who, search, pos, forced)` | do_garrison: `QUEUE_FIRST, same forced`, `search = 1`; come_out: `QUEUE_LAST, 0`, `search = 0` |
| `Unit::add_board_order@005e4d10` | transport logic (not read) | `(ship o, who, pos, _)` | `forced` always set |
| `Unit::add_await_board_order@005e4c80` | transport logic (not read) | `(unit o, who, _, forced)` | always appended (`pos` ignored) |

`add_build_order` / `add_repair_order`, after filling `ox/whom/uid` (uid =
`objects[whom][ox].uid`, or `0xffff` if either index is negative) and the
forced bit: if the unit's tile **region** (`WorldData::get_tregion`) differs
from the target's (`add_repair_order` also requires the target tile's
`WData.flags & 0x30 != 0x20`), and the player's transport level — `leader_flags
& 0x100 → 3, & 0x200 → 2, else (leader_flags >> 10) & 1` — is at least
`UnitData::transport_type(this)` and `can_ever_transport(this)`, set
`unit_masks |= 0x800000` ("needs a transport"; the movement/transport reader's
flag). Both set `unit_masks |= 0x400` ("has been a builder" — read by
`think_peasant`, below) and end with `update_action`.

---

## 2. The per-frame step

Every order is stepped from `Unit::work@0060d180` → `do_job(type, order)`
(`@00617a10`), with `type` = `get_type()` of `head->prev`. Before `do_job`,
`work` (a) sets `current = head->prev`, (b) runs `update_action` and, if the
action is targeted (`+0x20`) and the target's `uid` no longer matches the
order's, kills the order (`repath; kill_current_order`) — **a BuildOrder whose
site has been replaced dies here, before `do_build` ever sees it**; (c) every
16 frames (`(frame + o) & 0xf == 0`) for a targeted action, `check_target_path`
— which for a **building** target does nothing (it only re-paths to unit
targets: `check_target_path@005e22d0` gates on `target->is_unit` (`+0x8`)).
`work` is called from `Unit::process` after attrition and before the move
step (`docs/MOVEMENT.md`); the sim's `Sim::tick` already places the job step
there.

### 2.1 `Unit::do_build(order)@005eebf0`

`b = get_build_order(order)`; `o = b.ox`, `who = b.whom`, `T = objects[who][o]`.

```
1  o < 0 or who < 0 or not T.is_active (SubObjectData flags & 1):
       kill_current_order(0)
       if a next order exists and its type != NONE: return        # carry on with the queue
       build_done(o, who, UNIT_GATHER_RESPOND_RANGE × 192); return
2  T already active (WallData::is_active, flags & 4 — finished before I arrived):
       kill_current_order(0)
       if order_type() != NONE: check_build_order(); return        # §2.2
       if not (T is OILPLATFORM)
          and (unit_masks & 0x40000 [AI-controlled] or worker_stance ∉ {0, 1}):  build_done(...); return
       if who != my owner: build_done(...); return
       if not T.type.is_gather_building (BuildType vtbl +0x90) or T is UNIVERSITY: build_done(...); return
       add_gather_order(o, QUEUE_NEW, 0); deselect me if the selection has > 1; group = -1; return
3  not adjacent_to(o, who) (Object::adjacent_to@00651f40, §3.1)
   or (my tile is inside T's footprint (WallData::covers_tile) and T is not a FARM):
       f = order.flags & 4;  kill_current_order(0)
       Group g = {me};  g.action_swarm_around(o, who, QUEUE_FIRST, BUILD_AT, f)   # §2.3: move + build re-queued
       return
4  set_anim(T is FARM ? CHAR_SOW : CHAR_BUILD, 0, 1); face the site (find_angle → set_angle if it changed)
   unit_masks & 1 → return                                           # CITIES §3.3 reads this as "decoy"; the dump's builder has it clear
5  amount = ACCEL_CONSTRUCT (100); if not (Koreans with KOREAN_BUILD_UNDER_FIRE) and T.is_under_attack: amount = amount / 4 (toward zero)
   done = T.get_wall().do_construct(amount)                          # CITIES §3.3 — the seam
   if not done: return
6  kill_current_order(0);  rest = order_type()
   if rest != NONE:
       if not T.type.is_gather_building or T is UNIVERSITY or group < 0: check_build_order(); return
       Group::normalize(group);  if count(COUNT_PEASANTS) < 2: check_build_order(); return
   if (T is OILPLATFORM or (not AI and worker_stance ∈ {0,1})) and who == my owner
      and T.type.is_gather_building and T is not UNIVERSITY:
       add_gather_order(o, QUEUE_NEW, 0); deselect; group = -1; return     # the finished farm/mine/woodcutter/oil well gets its builder
   if rest == NONE: build_done(o, who, UNIT_GATHER_RESPOND_RANGE × 192); return
   check_build_order()
```

`UNIT_GATHER_RESPOND_RANGE × 0xc0` — tiles to position units. `worker_stance`
= `UnitData::get_worker_stance@006109f0` → `stance` (`UnitData +0xb1`) if the
type has a stance, else 0. The stance numbers, from their uses in this
sub-area: **1 and 2 build and repair, 0 and 1 gather** (so 1 is the normal
citizen; 0 = gather-only, 2 = build-only); the enum names are not in the
export (`WORKER_STAND` is the no-stance fallback and equals 0).

In the logged run (§5) the builder stood adjacent at frame 176, the move
order was gone at 177 (anim/facing set), `do_construct` first ran at 178
(site `flags 1 → 3`, `job_counter 100`), and completion at frame 327 left the
AI builder with a `GATHERORDER` on the farm it had just built — via
`build_done → find_gather_spot` (step 6, AI branch), `flags 0`.

### 2.2 `Unit::check_build_order@00603470` — the queue of build orders after one finishes

Walks the queue from `head->prev` while the order is `BUILD_AT` or `MOVE_TO`
(a move is skipped unless it is **forced** or the last node — then stop; the
decompile's loop variable is stale in one branch, so the skip rule is
medium confidence). Every `BUILD_AT` it reaches is **popped** (`repath;
kill_current_order`); if its target still exists and is not active, the
site's `o` is collected; collection stops after a site that is a city
(object `flags & 0x20` — the bit `DATALAYER.md` found on the starting city)
or whose type answers `BuildType +0xfc` / `+0x100` (not resolved).

Then: one site → `add_build_order(site, me, QUEUE_FIRST, 1)`. Several →
`Objects::find_units(SEARCH_FRIENDLY, within UNIT_BUILD_RESPOND_RANGE × 192,
FILTER_BUILDREPAIR)` (the result list is `objects +0x214`, count `+0x208`),
count for each collected site the friendly units whose **action** is
`BUILD_AT` on it, swap the least-crowded site to the front, and re-add all of
them with `QUEUE_FIRST, 1` in reverse — so they run least-crowded first, then
in their original order, all **forced**, all ahead of whatever else was
queued.

### 2.3 `Group::action_swarm_around(o, who, pos, BUILD_AT | REPAIR | GATHER, forced)@0070fbe0` — the walk

This is the only way a builder or repairer moves to its target; `BuildOrder`
owns no `MoveOrder` and calls no `go_to`. Per member of the group (land and
sea domains in two passes; air is collected and given a plain
`action_move_to`):

- skip a member that is busy (`UnitData::is_busy`), or carrying peasants, or
  — when the group is heading to a gather building under a BUILD_AT — already
  gathering (`local_30`, the "spare peasants" count);
- `R = min(x_size, y_size) × 0x60 + 0x30` (the site's footprint at
  `BuildType +0x234/+0x238`; a dead target uses the **unit** type's
  `+0x244`); **halved for a FARM** under BUILD_AT;
- `UnitType::find_nearby_spot(site x, y, R, …, FILTER_NOT_ME, …)` → a free
  spot around the site; for BUILD_AT the spot is then pushed `0x30` further
  from the site's centre on each axis and re-validated with radius 0 (taken
  if free);
- `Unit::add_move_facing_order(spot as UCoord (div_3_table[c >> 4]), angle,
  mode = (BUILD_AT ? 1 | (is_human ? 0 : 2) : 1), pathed = 0, pos, forced…)`
  — `mode` is the fifth argument (`Unit::work` passes `3` there), not the
  order's `flags`; in the dump the order this creates is an
  **`EXPLORETOORDER` wrapping a `MOVEORDER`** (type 3), so the "facing move"
  is the explore-to variant of the move family (the move reader's). For `pos == QUEUE_LAST` and a member
  whose current action is `GATHER` and `forced != 0`, the pos becomes
  `QUEUE_NEW` (the gather is pre-empted);
- then `add_build_order(o, who, QUEUE_LAST, forced)` (or `add_repair_order`),
  preceded by an `add_cast_order(spell 0x293)` when the unit can cast it
  (the flamethrower/whatever — not read), and for BUILD_AT the site's
  `build_masks &= ~0x2000`.

So after `do_build` step 3 the list is `[ExploreTo(spot) → BuildOrder →
…old queue]`; `do_job` runs the move until it completes (its own rule, R2),
then the BuildOrder is `head->prev` again and `do_build` re-tests adjacency.
A move that falls short (the spot was taken) just swarms again.

The units the group passes over as "carrying peasants"/castable are the
transport and Kremlin/flamethrower exceptions — noted, not read.

### 2.4 `Unit::build_done(o, who, range)@00603bf0` — what a builder does next

```
if length > 1: return                                    # more orders queued: they run
AI (unit_masks & 0x40000):
    find_build_spot() or find_repair_spot() or (starting_resources != 8 and find_gather_spot(range)); return
stance ∈ {1,2} and find_build_spot(): return
stance ∈ {0,1}: if find_gather_spot(UNIT_GATHER_RESPOND_RANGE × 192): group = -1; deselect; return
stance ∈ {1,2} and find_repair_spot(): return
if o ≥ 0 and who == me and T.type.is_gather_building and T isnt UNIVERSITY and T isnt OILPLATFORM
   and BuildData::num_gatherers(T) < T.gather_max (BuildData +0x80): add_gather_order(o, QUEUE_NEW, 0)
```

**`find_build_spot@00603e20`**: range = `UNIT_BUILD_RESPOND_RANGE × 192`
(`× 384` for stance 1 or 2); `Objects::find_builds(SEARCH_FRIENDLY, range,
FILTER_CONSTRUCT)`; keep own sites whose `build_masks & 0x20` (under attack,
stage 2) is clear; if any: count builders per site (friendly units whose
action is BUILD_AT on it, within the same range), pick the **fewest-builders**
site (ties → first), `swarm_around(site, QUEUE_LAST, BUILD_AT, 0)`; return 1.

**`find_repair_spot@00604320`**: AI players only at difficulty ≥ 2; range as
above; `ObjectsData::find_any_building(SEARCH_FRIENDLY, range,
FILTER_DAMAGED, FILTER_NOT_UNDER_ATTACK)`; the tile's territory owner must be
nobody, me, or a mutual ally (`diplos` both 2); `swarm_around(b, QUEUE_LAST,
REPAIR, 0)`; return 1.

**`find_gather_spot(range)@005f5170`** is the economy reader's; the shape:
scan my buildings — active, `is_gather_building`, not neutralized, UNIVERSITY
iff I am a scholar; with a free gather slot (`num_gatherers < gather_max`) or
one I am already at; same tile region; a citizen standing in a city skips
buildings of *another* city unless the city balance allows; within `range`
(ignored if ≤ 0 or for scholars); value = Σ over available goods of a
stockpile term, `× 500 / (dist/192 + 2)`; best → `add_gather_order(b,
QUEUE_LAST, 0)`.

### 2.5 The idle citizen — where an unordered citizen's BuildOrder comes from

`do_job(NONE)` → `Unit::do_idle@0060dcd0` → `check_idle@006032c0` (the idle
counter `UnitData::idle +0xb0`: `0xff → 1`; otherwise `+1` every frame while
≤ 1 and then only on frames with `(frame + o) & 0xf == 0`; `Unit::work` zeroes
it whenever the unit has an order) → `Unit::think@005f6e40` → gates
(`leader_flags & 2` must be set; `leaders +0x4 & 2` clear; `unit_masks &
0x1000000` clear; the `idle > 2 → only every 16 frames` cadence; the target
re-acquire and think_attack branches first) → `is_worker` →
**`think_peasant(0)@005f5760`**:

```
T = AI ? 1 : by LeaderOptions[who] +0x8: 1→7, 2→12, 3→17, 4→32, 5→62, else 2
if idle < T: return 0;  if idle != T and (idle − 2) % 5 != 0: return 0
AI peasant: think_civilian_transport(1) first
not a scholar and (unit_masks & 0x400 or stance ∈ {1,2}) and find_build_spot(): return 1
stance ∈ {0,1} (or no stance type and stance 0): find_gather_spot(AI ? −1 : UNIT_GATHER_RESPOND_RANGE × 192) → deselect; human: group = −1; return 1
not a scholar: human with (0x400 or stance ∈ {1,2}) and find_repair_spot(): return 1
unit_masks &= ~0x400
AI: region/scout logic, then find_repair_spot()
```

`do_think_order@005e5bf0` (`THINK`) = kill itself, then `think_peasant(1)`
for citizens/scholars (no cadence gate). The option at `LeaderOptions +0x8`
is the per-player idle-citizen delay (name not in the export).

### 2.6 `Wall::process@00640450` — the AI's builder recruitment (for completeness)

Every 32 frames (`(frame + o) & 0x1f == 0`), a site that is not active and
whose owner is **not human** (`LeaderData::is_human@006ec170` = `leader_flags
& 4`): wanted = `max(4, helpers)` (a wonder wants more, by rival wonder
progress); if `helpers < wanted`: `ObjectsData::find_unit(SEARCH_FRIENDLY,
0xf00 = 20 tiles, FILTER_TYPE PEASANTS (0x32), FILTER_NOT_BUSY)` →
`add_build_order(site, QUEUE_NEW, 0)`. The same function resets `helpers`
and `build_masks & 0x800` every frame (CITIES §3.3).

### 2.7 `Unit::do_repair(order)@005ee420`

`r = get_repair_order(order)`; `set_anim(CHAR_REPAIR)`. In order:

```
T.damage == 0 → kill; if not AI: (who == me and order_type() == NONE and stance < 2 and T.is_build
                                   and T.type.is_gather_building and T isnt UNIVERSITY*) → add_gather_order(o, QUEUE_NEW, 0)
                      AI: find_repair_spot();  return
who != me and not mutual allies → (same as above: kill + …)
not T.is_build (vtbl +0x1c) → kill; not T.is_active (finished) → kill
T.is_under_attack and not forced → kill;  tile territory owner ∉ {nobody, who, ally of who} → kill
not adjacent_to → f = forced; kill; Group{me}.action_swarm_around(o, who, QUEUE_FIRST, REPAIR, f); return
unit_masks & 1 → return
period, helpers++, amount, the price, T.repair_damage(amount)   — exactly CITIES §9.3
```

(*the `is(…)` argument is lost in the decompile; UNIVERSITY by analogy with
`do_build` — a listing check would settle it.) `period < 1` repairs
`T.damage` at once. The cost check uses `float` in the original (CITIES §9.3
notes it).

### 2.8 `Unit::do_garrison(order)@005e6b80`, `kill_garrison_order`, `go_inside`

The gates are CITIES §6.4, verbatim; the order-side facts it does not state:

- `g = get_garrison_order(order)`; the helicopter/AIRBASE branch kills the
  garrison order and adds a **strafe** order (`QUEUE_FIRST`, `temporary = 1`),
  or if the airbase is full a `MoveOrder` to it (`QUEUE_FIRST`) plus "full"
  feedback.
- **Not adjacent**: `add_move_order(spot, 1, 0, QUEUE_FIRST, 0, …)` with the
  spot from `find_nearby_spot` on the ring `min(x_size, y_size) × 0x60 + 0x30`
  (a DOCK: `+0x1b0 .. +0x330`), retried relaxed; **the GarrisonOrder stays
  in the list behind the move** — no swarm, no re-add. If no spot is found
  the order is killed.
- Admitted: `go_inside(captain, o, who, 0)`; if the target `is_build` (vtbl
  `+0x20`) `options->rebuild = 1` (UI); then
  `kill_garrison_order(captain, 0)@005e2bd0`: from the captain down the
  `o_down` chain, each unit whose action is `GARRISON` is `repath`ed and
  killed (the chain is walked only while the unit is a captain or was reached
  through `o_down`).
- Full: with `g.search != 0` and a city whose `city_flags & 1` is set,
  `find_garrison_build(city, who)@00605040` — the chain from `city +0x8`
  through `build +0x74`, active, with room for `num_inside + control_cost`,
  `can_garrison`, nearest by `vector_dist` — then `kill` and
  `add_garrison_order(b, who, search = 1, QUEUE_FIRST, forced)`. Else "full"
  feedback and kill.

`Unit::go_inside(o, who, chain)@0061a2e0`: climb to the captain (while not a
captain and `chain == 0`, `this = objects[o_up]`); `Object::insert_inside`;
deselect a non-plane; **if the container `is_unit` (vtbl `+0x18`) and
`uber_size > 1`: QUEUE_NEW-style clear of the orders** — i.e. a squad boarding
a *transport* drops its orders (CITIES §6.5 says "entering a building"; §7);
followers down `o_down` recurse with `chain = 1`; scholars
(`SCHOLARS`/`SCHOLARSKOREAN`) are moved to the container and get
`CHAR_DEFAULT`.

### 2.9 Board / AwaitBoard, briefly

`do_board@005ed1f0`: `CHAR_DEFAULT`; `check_meet_ship(o, who) == 0` → kill,
and if the ship `can_carry` me → `go_inside(ship)`. (The positive branch of
`check_meet_ship` — the approach — is the transport reader's.)
`do_await_board@005ed040` (on the **ship**): if the awaited unit is not me and
(same owner and alive and its action is `BOARD_SHIP` (8) on *this* ship) →
keep waiting (return); otherwise (the unit's board order targets me but it is
not same-owner → `repath; kill` **its** order), and kill my await order.

### 2.10 `Unit::come_out(chain)@00617c10` — the auto-tasking on leaving

Placement is CITIES §6.5. After `set_new_location`: an AI unit (`leader_flags
& 4` clear) that is not a citizen/scholar and not a plane gets a QUEUE_NEW-style
clear of its orders; a `NUCLEARMISSILE` (`is(0x13b)`) leaving with a targeted
first order bumps `leaders +0xa38` (a launch counter). Then the **rally
point**: the container is a
building with gather points (`Build +0xbc` list, `num_gather` at `+0xcc`,
`gather_inside` flag); each `GatherPoint {x|o +4, y|who +8, kind +0xc}` with
`kind 3` = an object (o, who) (also copied as a hotkey group), else a point.
For each point in turn (a waypoint chain: a point that yields no task becomes
a `move_facing_order(QUEUE_LAST)` / `Group::action_move_to(MOVE_TO)` for a
squad and the loop continues), with `B = find_any_building_at(point tile,
FILTER_SEEN)`:

- `B` found, I am a **citizen** (`PEASANTS`/`PEASANTSKOREAN`) and `B.is_build`:
  not active and same owner → `add_build_order(B, QUEUE_LAST, 0)`;
  `damage == 0` and the point is an object-kind and `B` active, gather
  building, not UNIVERSITY, same owner → `add_gather_order(B, QUEUE_LAST, 1)`;
  damaged → `add_repair_order(B, QUEUE_LAST, 0)`.
- a **scholar** (`SCHOLARS`/`SCHOLARSKOREAN`): object-kind point, `B` active,
  gather building and UNIVERSITY, same owner → `add_gather_order(B,
  QUEUE_LAST, 1)`.
- a **caravan** and `B.is_trade` (`+0x24`) and not enemy → `add_trade_order(B,
  QUEUE_LAST, 0)`.
- otherwise `B` alive, `is_build` (`+0x20`), active, ally, `garrison_limit >
  0`, `can_garrison`, object-kind point → `add_garrison_order(B, 0,
  QUEUE_LAST, 0)` for me and every unit down my `o_down` chain.
- an enemy `B` and I can attack (`UnitType +0x1e8`): object-kind → `attack`
  (`QUEUE_LAST, 1, 1`) for me and the chain; else a scout (`+0xf4 == 5`) →
  move/`action_move_to`; a unit whose type cannot (`+0x10c`) → `ATTACK_TO` /
  a plain move; else `attack (QUEUE_LAST, 1, 0)`.
- no building: `find_unit_with_radius` at the point → an enemy unit → attack
  orders as above; else `MOVE_TO`, or `ATTACK_TO` when the target tile holds
  a `0x1ab`/`0x1ac`/`DOCK` type.

At the end an AI unit (`0x40000`) that is not a caravan/merchant/fur trapper/
special is `add_to_army` on a frame chosen by `game_random`. A land citizen
leaving an `OILPLATFORM` is put into a fresh `TRANSPORTBARGE` first
(`same_damage`, `insert_inside`, `come_out(0)` of the barge). `Setup::
place_unit@005abca0` creates the starting units with `init_unit` →
`come_out(0)`; `Setup::build_units@005aafc0` (another reader's) then gives
starting citizens `add_gather_order(pre-placed building, QUEUE_NEW, 0)` when
`starting_resources != 8` — **the human's citizens walking to objects
2001–2005 carry GatherOrders (type 7, flags 0), not BuildOrders**; the dump
confirms it (§5). The only BuildOrders in the opening are the AI's (`flags 4`,
from its own issuing, not from `Setup`).

---

## 3. The seams

| this sub-area calls | in | when | expects |
| --- | --- | --- | --- |
| `Object::adjacent_to(o, who)@00651f40` | combat (`attack_dist`, COMBAT.md §13.1 / `combat::attack_dist`) | every `do_build`/`do_repair`/`do_garrison` frame | both alive; `attack_dist(o, who, my x, y) < 0x60` (96 = half a tile, edge to edge on the quarter-tile grid). A sea unit that is a non-transport (ship type `+0x218 == 1`, my type `+0x2b4 & 0x10` clear) instead uses `< BOAT_GARRISON_MAX_DISTANCE (+0x180 if my type's +0x248 < 4)` |
| `Wall::do_construct(amount)` | `build.rs`/`city.rs::do_construct` (CITIES §3.3) | once per adjacent builder per frame, units before buildings | `true` on the frame the site completes |
| `Build::repair_damage(amount)` + `repair_period`/`repair_amount` and the price | `build.rs` (CITIES §9.3) | once per adjacent repairer per frame | — |
| the garrison gates, `go_inside`, `kill_garrison_order` | `garrison.rs` (CITIES §6) | per frame | `GarrisonRefused::NotAdjacent` → a `MoveOrder` **in front of** the garrison order, everything else kills it |
| `action_swarm_around` → `find_nearby_spot` + `add_move_facing_order` (ExploreTo) | movement (R2) | on every non-adjacent `do_build`/`do_repair` frame | the move runs to completion by its own rule; then the BuildOrder is current again |
| `add_gather_order`, `find_gather_spot`, `BuildData::num_gatherers`, `gather_max` | economy (R-gather) | on completion (step 6), in `build_done`, `think_peasant`, `come_out` | — |
| `Objects::find_builds / find_units / find_any_building` | the object search (combat's target search family) | `find_build_spot`, `find_repair_spot`, `check_build_order`, `Wall::process` | lists in `objects +0x214` / count `+0x208` |

`ACCEL_CONSTRUCT`, `UNIT_BUILD_RESPOND_RANGE`, `UNIT_GATHER_RESPOND_RANGE`,
`KOREAN_BUILD_UNDER_FIRE`, `KOREAN_REPAIR`, `BOAT_GARRISON_MAX_DISTANCE`
are the constants; the two ranges are tiles, multiplied by `0xc0` at use
(`0x180` = 2 tiles for stance 1/2 in the spot searches).

---

## 4. Where it sits in the frame

`Game::do_frame` → `Leaders::process_all` → `Objects::process_all` (units and
buildings interleaved by object index; every unit's `Unit::process` runs
`work` → `do_job` → `do_build`/`do_repair`/`do_garrison`; a building's
`Wall::process` resets `helpers`) → `Objects::inc_time` (`Wall::inc_time` →
`update_hits`). CITIES §3.3's logged run fixed "all units before any
building" for the builder count; the sim's `Sim::tick` runs buildings first
and the job step inside the unit loop after attrition — the comment in
`lib.rs` explains the choice. Order creation by the UI/AI happens in the
command stream before `process_all` (the cheat/`Group::action_*` path,
ORACLE.md); `come_out`'s orders are added when the unit is ejected, i.e.
inside the building's `process_ejection`/`train` (CITIES §6.6), so they are
stepped from the next unit that runs — the same frame if the unit's index is
higher.

---

## 5. What the gamelog writes

`UnitData::log_data@0060b070` → `OrderList::log_data@00730070` (at
`UNITS ≥ 3`, which the latest run had): the field `length N`, then **from
`head` forward in `next` order** — so the newest order first and the
**executing one last** — per node: `type <get_type>`, `metric <node byte>`,
then the order's own block:

```
BEGIN BUILDORDER            (or REPAIRORDER / BOARDORDER / AWAITBOARDORDER / GARRISONORDER)
 BEGIN TARGETORDER
  BEGIN UNITORDER
   flags 4
  ox 2006
  whom 1
  uid 12
 [search 0]                 (GARRISONORDER only; after the TARGETORDER block)
```

(`BuildOrder::log_data@00482ad0`, `TargetOrder::log_data@0047f3a0`,
`UnitOrder::log_data@0047f960`, `GarrisonOrder::log_data@004846f0`; the keys
are 2/4/3/6-char literals at `0xb3f94c/0xb479e4/0xb479f0/0xb48ec4`, which the
dump renders as `ox`, `whom`, `uid`, and — by length — `search`.) Each
`log_data` closes its block (`Log` slot `+0x10`) before the parent writes its
own fields, which the text shows only by indent: `ox/whom/uid` sit one level
shallower than `flags`. **`DATALAYER.md`'s "a field belongs to the innermost
open block regardless of indent" rule mis-files them under `UNITORDER`**; the
reader needs "a field shallower than the open block's own indent closes it".

The walk→build→done sequence, AI citizen `who 1 o 1` → site `o 2006`
(constr_time 15000, a gather building), `gamelog.txt`:

| frame | unit's list (log order: newest → current) | site |
| --- | --- | --- |
| 1 | `GATHERORDER` on 2001 (`flags 0`) — from `Setup::build_units` | — |
| 2 | `BUILDORDER {flags 4, ox 2006}`, **`EXPLORETOORDER`/`MOVEORDER` to (41784,15816)** current; `unit_masks 0x40408`; an 8-entry `PATHDATA` stack | `flags 1` |
| 3–176 | same; the path is consumed; `orders_x/y` = the swarm spot | `flags 1` |
| 177 | `BUILDORDER` alone, unit at the spot, `angle 0` → `892141568` next frame | `flags 1` |
| 178–326 | `BUILDORDER`; builder anim | `flags 3`, `job_counter` +100/frame, `recharging` counts up |
| 327 | `GATHERORDER {flags 0, ox 2006}` (`build_done` → `find_gather_spot`) ; `unit_masks 0x4000a` | `flags 7`, `job_counter 0` |

The human's citizens (`who 0`, `o 1…`): `GATHERORDER {flags 0, ox 2001}` from
frame 1, a `MOVEORDER` (`flags 0` then `1`, `dest 1`) queued in front at frame
3 by the gather step, arrival and the move gone at frame 20. No BuildOrder.

---

## 6. What the sim has, and what the order-driven version must do

`crates/sim/src/lib.rs`: `Unit::job: Option<Job>` with `Job::{Build(usize),
Repair(usize), Garrison(usize)}`; `Sim::order_build/order_repair` (`lib.rs
1374–1381`) and `garrison::order_garrison` set it; `city.rs::process_unit_job`
(`~1444`) steps Build/Repair — `adjacent(corner, xs, ys, pos)` (a tile-based
stand-in), else `movement.dest = Some(bpos)`; `do_construct(at, amount)`;
repair via `build::repair_period/repair_amount`; `garrison.rs::
process_garrison_job` (`~256`) steps Garrison. One job, no queue, no
completion behaviour, no auto-tasking.

To replace `Job` with the order:

1. **A per-unit order list** (front = executing; `QUEUE_NEW` clears,
   `QUEUE_FIRST` pushes front, `QUEUE_LAST` pushes back), orders carrying
   `{target: (owner, o), uid, forced}`; `Garrison` adds `search`. The
   `orders_x/y` seam (`update_action`) is the move reader's.
2. **Adjacency** = `combat::attack_dist(site, unit) < 96` (replace the
   tile-based `adjacent`).
3. **`do_build`** per §2.1: site gone → pop, `build_done` if nothing queued;
   site active → pop and the step-2 gather rule; not adjacent → pop, push
   front `[ExploreTo(spot) , Build(same forced)]` with the spot from the
   swarm ring (`min(xs, ys) × 96 + 48`, FARM halved, nudged `+48` outward,
   `find_nearby_spot` is movement's); adjacent → `do_construct`, on `true`
   pop and the step-6 tree (`add_gather_order` = economy's, `check_build_order`
   = the queued-builds re-sort of §2.2, `build_done` of §2.4).
4. **`do_repair`** per §2.7 with the existing `repair_period/amount`; the
   walk the same way with `REPAIR`.
5. **`do_garrison`**: the existing gates; `NotAdjacent` → push **front** a
   `Move(ring spot)` and keep the order; admitted → `go_inside` and kill every
   `GARRISON` order down the chain; full + `search` → `find_garrison_build`
   and re-order `QUEUE_FIRST`.
6. **Auto-tasking** (`think_peasant` cadence + `find_build_spot`/
   `find_repair_spot`/`find_gather_spot`, `come_out`'s rally rules) is what
   makes citizens act without an order stream; it needs the object searches
   (`find_builds`, `find_any_building`, `find_units` with the BUILDREPAIR /
   CONSTRUCT / DAMAGED filters) and the `idle` counter and `leader_flags &
   2` / `LeaderOptions +0x8` inputs.
7. **The log**: `OrderList` emitted newest-first with the current last, the
   nested blocks above, so the harness can diff `type/ox/whom/uid/flags` per
   frame.

---

## 7. Confidence, what is not established, checks

**High**: the layouts (ctor + `types.txt` + the dump's field order); the
queue's direction (`add`, `remove_current`, every `head->prev` reader, and
the dump's `[BUILD, EXPLORETO]` with the ExploreTo executing); `do_build`'s
decision tree and the completion rule (the 327-frame trace matches the AI
branch); the walk being `action_swarm_around` → ExploreTo + re-queued
BuildOrder; `adjacent_to = attack_dist < 0x60`; `build_done`,
`find_build_spot`, `find_repair_spot`, `think_peasant`'s gates and cadence;
the `do_repair` and `do_garrison` order-side gates; the log format.

**Medium**: the `OrderIndex` numbering beyond the literals the dump pins
(`1, 3, 6, 7`) — it follows `do_job`'s case order and is consistent with every
literal met (`8, 9, 0xa, 0xc, 0xd, 0xe, 0xf, 0x10, 0x12, 0x13, 0x15, 0x19,
0x1a`), but the enum itself is not exported; the `check_build_order` skip rule
for moves (the decompile's loop variable is stale); the exact
`find_nearby_spot` argument meanings (movement's); the `come_out` rally rules
for the non-citizen branches (read once through a heavily mangled function);
the worker-stance numbering's names.

**Not established / open**:

- `do_repair`'s `is(…)` literal in the post-repair gather rule (lost in the
  decompile; UNIVERSITY by analogy) — one listing look.
- `BuildType +0x90` (`is_gather_building`, by every use), `+0xfc`, `+0x100`
  (the `check_build_order` stop rule), `+0x78` (the cost per good),
  `UnitType +0x1e8` (can attack), `+0x218` (domain), `+0x2b4 & 0x20`
  (helicopter) — named by use, not by symbol.
- `unit_masks & 1` (the step-4 guard; CITIES calls it decoy) and `& 0x400`
  ("was a builder"; set by the two adders, cleared by `think_peasant`) — the
  bits are in no exported enum.
- The `flags 4` on the AI's opening BuildOrder: which AI call issues it
  (`Group::action_swarm_around(…, forced = 1)` from the build-order script?) —
  the AI reader's.
- **A correction candidate for `docs/CITIES.md` §6.5**: `go_inside` clears a
  squad's orders when the *container is a unit* (vtbl `+0x18`, true for
  `Unit`, false for `Build`), i.e. boarding a transport, not entering a
  building. Worth a second look at the COMDAT bodies (`Buffer::
  is_pending_load` = `return 1`, `Window::get_button` = `return 0` is the
  reading here) before amending.
- `DATALAYER.md`'s reader rule and the order blocks (§5) — the harness will
  mis-nest `ox/whom/uid` unless the shallower-field-closes-block rule is
  added; checkable against the dump in a minute.

**Behavioural checks worth running** (each a short logged drive):

1. *The swarm ring and the re-approach*: order a citizen to a site from
   behind an obstacle with `BUILDS=6 UNITS=3` and read `orders_x/y` and the
   `EXPLORETOORDER` destination against `min(xs, ys) × 96 + 48 (+48)` — pins
   the spot rule and whether a short move swarms again.
2. *Two queued sites*: shift-queue two build orders, finish the first, and
   read the list after completion — settles `check_build_order`'s re-sort
   and the forced bit (`flags 4`) it writes.
3. *The idle citizen*: a fresh citizen next to an unfinished own site with
   no order: the frame it picks up a `BUILDORDER` (expected `idle == 2`, i.e.
   the third idle frame, with the default option) and its `flags 0`.
4. *Garrison walk*: the `MOVEORDER` in front of the `GARRISONORDER`, and
   `search` after a "full" redirect.
