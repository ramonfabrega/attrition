# Orders

How a unit is told what to do, what it keeps while it does it, and how one
frame of "doing it" runs. The eleventh mechanic, and the one every earlier
mechanic was secretly waiting for: movement's `move_step` is called from a
move order, the construction clock from a build order, `fight` from an attack
order, the gatherer count from a gather order — and until now the simulation
reached each of them through a stub (`Job`, `movement.dest`, `combat.target`)
rather than through the thing the original has, which is a **per-unit order
list** with three enqueue modes, one order stepped per frame, and a lifecycle
every mechanic above hangs off.

**Scope.** The `UnitOrder` hierarchy and the 27 live order kinds; the order
list, its orientation, the three `QueuePos` modes, adding, killing, clearing;
where in the unit's frame the current order is stepped and what an idle unit
does instead; the move order end to end — the destination snap, the path
stack, the planning decision tree, the two arrival tests, the facing on
arrival, the straight-line verifier — with the pathfinder as a named seam; the
build, repair and garrison orders and how a builder gets to its site; the
gather order — a registration on the building, the walk, the farm and the
wood/ore choreography; the attack order and the part of `fight` that manages
it; the guard, follow, patrol and attack-move orders; the group move and what
a multi-unit right-click produces; the start of a game — what exists at frame
0 and which orders the starting units carry; the spatial queries every walk
ends in; what the gamelog writes for all of it.

**Not in it.** `PathFinder` itself (`astar_path` and the three grids — the next
mechanic; §4.6 gives its interface and what a stand-in gets wrong); the
formation geometry (`Form::compute`); the AI that issues orders (`Leader::
plan_strategy`, `Group::action_*` as the AI's callers — phase 5); transports,
boarding and the civilian-transport request (`BoardOrder`, `AwaitBoardOrder`,
`think_civilian_transport`, surveyed only); casting, trade routes, strafing,
air patrols and the special-animation order (named, not read); the unit's body
(`Guy::move`, `docs/MOVEMENT.md`); the construction clock, the garrison gates
and the attack step themselves (`docs/CITIES.md`, `docs/COMBAT.md`).

**How this was established.** Symbol names, struct layouts and field
offsets from `game/sbl/rise.pdb`; behaviour from the decompile export under
`~/ghidra-projects/decomp` (`tools/ghidra/`). Seven readers took one
sub-area each and read their functions in full; four claims were settled in
the PE bytes or the listing (`get_starting_citizens`' jump table,
`add_move_order`'s register-passed `find_angle` arguments, the `QueuePos`
values in `add_move_facing_order`, and `MoveOrder::log_data`'s twenty keys,
which also settle the `this[-1]` offset shift, §1.1), and two real gamelogs
at `UNITS=3` confirmed the list orientation, the log format, the citizen's
first moving frame and a 327-frame build trace. **The function-by-function
inventory of what each reader read is in `docs/JOURNAL.md`, 2026-08-29.**
Nothing is transcribed; see `docs/DECISIONS.md` entry 7.

**Status.** First reading, implementation, blind second reading and all
seven adjudications are landed: the verdicts and the counts are in
`docs/audit/2026-08-21-orders.md`, the story and the reports' location in
`docs/JOURNAL.md` (2026-08-21). What the implementation leaves as inputs
is §13.

**Confidence.** High for the structures, the list's orientation and the three
enqueue modes, the dispatch and its place in the frame, `do_move`'s planning
tree and both arrival rules, `find_path`'s contract, `do_build`/`do_repair`/
`do_garrison`'s decision trees, the gather registration and `num_gatherers`'
three terms, `do_gather`'s branches and the non-flat machine's timers,
`do_attack`'s five branches and what `fight` writes to the order, the
`CommandPackage` entry and the group-vs-plain predicate, the start-of-game
sequence and the id scheme, and the log format — and, after the second
reading, the `OrderIndex` values (the PDB enum), `check_build_order`'s cursor
walk (the listing), and `flags & 4`'s meaning. Medium for the collision
branches of `do_move` and `resolve_unit_collision` (read once, summarised),
`go_around_building`'s geometry (**raised to high 2026-08-26**: §4.6.1 was
re-derived and pinned against the original's own path stack),
`think`'s cadence table (several bodies
lost), the group speed's effect on the leader, `Leader::produce_building`'s
scoring (read in shape — take the starting sites from the dump, do not
reimplement). Low for the semantics of `UnitOrder::flags` bits
`0x8/0x10` beyond their writers, and the `0x28b` cast block in `work`
(not credible as decompiled).

**What it changed elsewhere.** `docs/COMBAT.md` §10 calls `GameAccess::rnd` "a
second, unsynchronised stream"; it is `Random::get(game_random, 0, 0xffff) % n`
— the sync stream (§6.4). `docs/CITIES.md` §6.5's "`go_inside` clears a
squad's orders on entering a building" is the *transport* case: the clear is
gated on the container being a unit (§5.8). `docs/DATALAYER.md`'s "citizens
walk to the pre-placed sites" is right and its mechanism is not a build
order: the sites are complete at frame 0 and the citizens carry gather orders
(§9). `docs/DATALAYER.md`'s block-nesting rule mis-files `ox/whom/uid` under
`UNITORDER` (§11). `docs/ORACLE.md`'s open "order stream" item closes:
`COMMANDMANAGER=1` logs every click (§8.1, §11.3). `docs/MOVEMENT.md`'s open
question on the facing a walking unit logs is flagged, not changed (§4.5).

---

## 1. The order system

### 1.1 `UnitOrder`, and the virtual base every order shares

`UnitOrder` (`types.txt:19204`, size 8) is `{ +0 vptr, +4 char flags }` and
nothing else. **Every concrete order derives from it virtually**; MSVC lays a
derived order out as `vbptr`, its own fields, a 4-byte `vtordisp`, then the
`UnitOrder` sub-object at the tail. Two consequences, both traps:

- `types.txt` lists an unnamed 0xc-byte tail on every order struct that is
  *not* a field (`TargetOrder +0x14..+0x1f` is the vtordisp, the base's vptr
  and `flags`).
- Methods compiled with `this` at the virtual-base sub-object
  (`MoveOrder::*`, `GatherOrder::log_data`/`clear`, `TargetOrder::*`) print
  every field in the export as `this[-1].<wrong name>`: a `MoveOrder` field
  printed as `this[-1].X` is the real field at `offset(X) − 8`; a
  `GatherOrder::log_data` field is shifted by two. The log keys read back
  from the PE settle the real order (§11). Calibrate on `TargetOrder::
  log_data@0047f3a0`: `this[-1].whom + 4` is the vbptr dereference, so
  `this = A + 0x18` and `this[-1].uid` is really `ox`.

The hierarchy, from the constructors (each writes `…::vftable_for_…` or calls
its base by name):

```
UnitOrder                                  (virtual base of everything; {vptr, flags})
├─ TargetOrder      {+0x8 ox, +0xc whom, +0x10 uid}                           size 0x20
│  ├─ AttackOrder   {+0x14 def_x, +0x18 def_y, +0x1c mandatory, +0x1d defensive,
│  │                 +0x1e in_range, +0x1f ever_in_range, +0x20 new_ord}       size 0x30
│  │  ├─ StrafeOrder        : AttackOrder, AirOrder                            size 0x54
│  │  └─ GroupAttackOrder   : AttackOrder, GroupOrder  — never constructed (§7.9)
│  ├─ GuardOrder    {+0x14 dx, +0x18 dy, +0x1c guard_x, +0x20 guard_y, +0x24 idle, +0x28 retry}
│  ├─ FollowOrder   {+0x14 oxx, +0x18 whose, +0x1c uid2}
│  ├─ GatherOrder   {+0x14 tx, +0x18 ty, +0x1c build_type, +0x20 wait, +0x24 goto_build,
│  │                 +0x25 non_flat_gather, +0x26 dist_mod, +0x27 been_there}  size 0x34
│  ├─ TradeOrder    {+0x14 oxx, +0x18 whose, +0x1c started, +0x20 loaded, +0x24 uid2}
│  ├─ BuildOrder, RepairOrder, BoardOrder, AwaitBoardOrder  (no fields of their own)
│  ├─ GarrisonOrder {+0x14 search}                                             size 0x24
│  └─ CastOrder     {+0x14 x, +0x18 y, +0x1c paid, +0x20 spell}
├─ MoveOrder        {+0x4 x, +0x8 y, +0xc angle, +0x10 dest, +0x14 tolerance, +0x18 pause,
│                    +0x1c retry, +0x20 attempts, +0x24 timer, +0x28 facing, +0x2c dest_x,
│                    +0x30 dest_y, +0x34 last_x, +0x38 last_y, +0x3c coll_x, +0x40 coll_y,
│                    +0x44 orig_x, +0x48 orig_y, +0x4c off_x, +0x4e off_y (shorts)}  size 0x5c
│  ├─ AttackToOrder, FleeToOrder, ExploreToOrder  (no fields of their own)
│  ├─ FormOrder     {+0x50 newform, +0x54 delay}
│  └─ GroupMoveOrder : MoveOrder, GroupOrder {+0x54 oxx, +0x58 whose, +0x5c id,
│                      +0x60 form_id, +0x64 group_angle, +0x68 in_group}      size 0x78
│     └─ GroupAttackToOrder : GroupMoveOrder
├─ PatrolOrder      {+0x4 x_pos, +0x20 y_pos (SimpleArray<Coord>), +0x3c waypoint}
│  └─ GroupPatrolOrder : PatrolOrder, GroupOrder
├─ AttackGroundOrder {+0x4 att_x, +0x8 att_y, +0xc accuracy, +0x10 attack_unit}
├─ GroupOrder       {+0x4 oxx, +0x8 whose, +0xc id, +0x10 form_id, +0x14 group_angle}
├─ AirOrder         {+0x4 oxx, +0x8 whose, +0xc cruising_alt, +0x10 sharp_turn, +0x14 old, +0x18 returning}
└─ ThinkOrder       (the base and nothing else)
```

Constructor defaults that matter: `TargetOrder` `ox = whom = −1`, `uid =
0xffff`; `AttackOrder` everything 0 except `def_x = def_y = −1` and
**`new_ord = 1`**; `MoveOrder` everything 0 except `facing = −1`;
`GatherOrder` `goto_build = 1`, the rest 0/−1; `GarrisonOrder` `search = 0`
(its `clear` writes −1, but every adder writes the field). `clear()` (vslot
`+0x4`) is the constructor's twin — `OrdersMemManager::get_obj` calls it on
every recycled order. (An earlier draft claimed `MoveOrder::clear` was an
exception that left `orig_x/orig_y` alone; the second reading found the
constructor does not write them either — `clear` and
`MoveOrder::MoveOrder@00488a10` write the identical 18 slots. R1.)

**The vtable** (`UnitOrder::vftable @ 00b474f0`, 74 slots): `+0x4 clear`,
`+0x8 walk_data`, `+0xc log_data`, **`+0x10 get_type() → OrderIndex`** (pure
in the base — there is *no per-frame virtual*; the step is `Unit::do_job`'s
switch, §2.3), `+0x14 is_move`, `+0x18 is_attack`, `+0x1c is_move_attack` (=
the previous two or-ed), `+0x20 is_targeted`, `+0x24 is_pathed` (`flags & 1`),
`+0x28 is_fleeing` (`flags & 2`), `+0x2c is_group`, `+0x30 is_air`, `+0x34
is_patrol`, `+0x38 print_details`, then the down-casts in two banks. The
first bank is the **`update_*_order`** set (non-`const`): `+0x3c
update_target_order`, `+0x40 update_move_order`, `+0x44/+0x48/+0x4c`
attack-to/explore-to/flee-to, `+0x50 update_attack_order`, … `+0x60 build`,
`+0x6c gather`, `+0x70 repair`, `+0xac garrison`, `+0xb0 think`. The second
bank, `+0xb4`–`+0x124`, is the **`get_*_order`** `const` twins (`+0xb8 move`,
`+0xe4 gather`, `+0xcc attack`). The twins are *not* uniformly `+0x78` apart:
the `get_*` bank declares `patrol` (`+0xc8`) before `attack` (`+0xcc`) where
the `update_*` bank has `attack` (`+0x50`) before `patrol` (`+0x5c`), so
attack's twin is `+0x7c` higher and patrol's `+0x6c` — settled from the PDB's
own `UnitOrder` field list (`LF_FIELDLIST 0x1A0D7`, each `LF_ONEMETHOD`
carrying its `vftable offset`; audit R1-14, third pass 2026-08-23). Every
`get_*_order`/`update_*_order` returns the order's own address, which is
why consumers read `ox/whom/uid` at `+8/+0xc/+0x10` of whatever came back. A
slot that prints as `Window::get_button` is the COMDAT fold of `return 0`,
and a slot that prints another class's method (`StrafeOrder::is_air` at a
`MoveOrder` `+0x14`) is the fold of `return 1`.

### 1.2 `OrderIndex` — the 27 live kinds

**The enum is in the PDB** — `llvm-pdbutil dump --types
--type-index=0x1E22 game/sbl/rise.pdb` enumerates all 28 values outright
(`QueuePos` is `0x216F`). The table below was originally rebuilt from three
places that agree — the `switch` in `OrdersMemManager::get_new_order@00730550`
(case number → constructor and `malloc` size), every class's `get_type` body,
and the names Ghidra prints for the `OrderIndex` comparisons in
`Unit::work`/`do_job` — and pinned by the dump (`type 1` precedes every
`MOVEORDER`, `3` every `EXPLORETOORDER`, `6` every `BUILDORDER`, `7` every
`GATHERORDER`); **the second reading then found the enum itself and every
value matched** (`docs/audit/2026-08-21-orders.md` R1). The pool
`OrdersMemManager::order_lists` is `SafeRecycler<UnitOrder>[28]`, one per
value.

| value | name | class | size |
|---|---|---|---|
| 0 | `NONE` | — | `do_job` → `do_idle`; `close_orders` stops on it |
| 1 | `MOVE_TO` | `MoveOrder` | 0x5c |
| 2 | `ATTACK_TO` | `AttackToOrder` | 0x5c |
| 3 | `EXPLORE_TO` | `ExploreToOrder` | 0x5c |
| 4 | `FLEE_TO` | `FleeToOrder` | 0x5c |
| 5 | (`PATROL`) | `PatrolOrder` exists, **never constructed**: `get_new_order` has no case 5, `do_job` none, `add_patrol_order` asks for `GROUP_PATROL` | — |
| 6 | `BUILD_AT` | `BuildOrder` | 0x20 |
| 7 | `GATHER` | `GatherOrder` | 0x34 |
| 8 | `BOARD_SHIP` | `BoardOrder` | 0x20 |
| 9 | `AWAIT_BOARD` | `AwaitBoardOrder` | 0x20 |
| 10 | `ATTACK` | `AttackOrder` | 0x30 |
| 11 | `FOLLOW` | `FollowOrder` | 0x2c |
| 12 | `GUARD` | `GuardOrder` | 0x38 |
| 13 | `REPAIR` | `RepairOrder` | 0x20 |
| 14 | `CAST_SPELL` | `CastOrder` | 0x30 |
| 15 | `TRADE_ROUTE` | `TradeOrder` | 0x34 |
| 16 | `STRAFE` | `StrafeOrder` | 0x54 |
| 17 | `AIR_PATROL` | `AirPatrolOrder` | 0x68 |
| 18 | `CHANGE_FORM` | `FormOrder` | 0x64 |
| 19 | `GROUP_MOVE` | `GroupMoveOrder` | 0x78 |
| 20 | `GROUP_ATTACK` | `GroupAttackOrder` — never constructed (§7.9) | 0x54 |
| 21 | `GROUP_ATTACK_TO` | `GroupAttackToOrder` | 0x78 |
| 22 | `GROUP_PATROL` | `GroupPatrolOrder` | 0x64 |
| 23 | `ATTACK_GROUND` | `AttackGroundOrder` | 0x20 |
| 24 | `AIR_ATTACK_GROUND` | `AirAttackGroundOrder` | 0x48 |
| 25 | `SPECIAL_ANIM` | `SpecialAnimOrder` | 0x2c |
| 26 | `GARRISON` | `GarrisonOrder` | 0x24 |
| 27 | `THINK` | `ThinkOrder` | 8 |

The **move family** — what `kill_current_order`, `work`, `repath` and
`resolve_unit_collision` treat together — is `{1, 2, 3, 4, 18, 19, 21}`: the
classes with a `MoveOrder` base. `NONE = 0`, `PATROL = 5` and
`NUM_UNIT_ORDERS = 28` are the enum's own names; `ATTACK_TO = 2` and
`FLEE_TO = 4` are pinned by it, and corroborated three further ways —
`UnitData::is_fleeing@0046efa0` tests `get_type() == 4`,
`add_move_facing_order@005e55c0:66` maps its kind argument 2 →
`get_obj(ATTACK_TO)` and 3 → `get_obj(EXPLORE_TO)`, and the `get_new_order`
jump table. **No behavioural check is needed** (second reading, R1).

### 1.3 `UnitOrder::flags`

One byte. From its writers and readers:

| bit | meaning | evidence |
|---|---|---|
| `0x01` | **pathed** — "the top segment of the unit's path stack is this move's" | `MoveOrder::is_pathed`; set by `do_move` after planning, cleared on the final waypoint; `add_move_facing_order` takes it from its 5th argument (`Group::action_move_near` passes 1, `check_target_path` 0); `kill_current_order` pops the path segment only if `is_move && is_pathed` |
| `0x02` | **dead** — nothing writes it | `MoveOrder::is_fleeing@004889a0` really is `flags & 2`, but **no code in the image writes the bit** and nothing calls vslot `+0x28`. The live predicate is `UnitData::is_fleeing@0046efa0` = `order_type() == FLEE_TO`, read by `PathFinder::calc_cost` and `Unit::target_opportunity`. Two adjudicators reached this independently (R1, R2); R2 adds that the only `or byte ptr [reg+4], 0x2` sites in the image are in `Ammo::init*` |
| `0x04` | **the action bit** — this order is an intent, not a transit leg | `UnitData::get_action@00608450` / `Unit::update_action@0060a870` walk past a move order *only if it lacks this bit* (§3.3); set from the last argument of every `add_*_order` — `CommandPackage::process_move_to` passes 1 for every player move, `Group::action_attack` 1 for every player attack, `add_think_order`/`add_spec_anim_order`/`add_guard_order`/`add_follow_order`/`add_patrol_order` set it unconditionally, while the engine's own inserted transit moves (`fight`'s chase, `do_gather`'s walk, `do_guard`, `do_follow`, `go_to`) pass 0; `Unit::work` lets a recharging melee unit step only an order carrying it, and clears `unit_masks & 0x100` on one; `do_repair` repairs a building under attack only under one. The dump: the AI's opening `BUILDORDER` has `flags 4`, the starting citizens' Setup `GATHERORDER` `flags 0`, the gather step's inserted `MOVEORDER` `0` then `1` |
| `0x08` | a DEFENSIVE unit's "this move is my post" | set by `fight` on the move that sends a DEFENSIVE unit back; `UnitData::find_def_pos` reads it (§7.2) |
| `0x10` | an attack order's "re-target requested" | toggled by `fight` when the chase cell it chose is not the unit's own; read on entry as "bad target" → `find_new_target` (§7.2) |
| `0x20` | `MoveToCommand.disembark` → `add_move_facing_order`'s 11th argument | **Two** readers, not one: `move_step@005faf30:333, :387` (the flyer branch — carrying passengers, it unloads them on the final waypoint and kills the order) **and** `PathFinder::astar_path@00683770:447`. Not modelled |
| `0x80` | an attack-ground order has fired | cleared by `add_attack_order`/`add_attack_ground_order`, set by `do_attack_ground` |

The three first readers named `0x04` three ways — "forced", "automatic/
engine-inserted", "explicit player action" — from three sets of callers.
The mechanics are one: the bit makes the order *the action* in
`get_action`'s walk, and it is carried by orders issued as intents (a player's
click, the AI's `Group::action_*`, a think) and absent from the transit legs
the engine inserts in front of them.

**Second reading (R1) — this reading confirmed, by a reader nobody had
cited.** `Group::set_up_insert@0070e520:25` is the cleanest statement of the
bit's meaning in the whole image: a group insert copies **only** the orders
that carry it. `Unit::land_plane@005e9950:69` is a clearer. The blind reader
arrived at "explicitly ordered" from `do_repair`'s under-attack abandon,
which is the same claim from the other end. Bit `0x80`'s setter is
`do_attack_ground@005f1410:204`; a **reader** for it is still missing.

### 1.4 The list — `OrderList` at `UnitData+0xc8`

`OrderList : LinkListBase<UnitOrder*, uchar, RecycledOrderNode>` (size 0x1c):
`+0xc8` vptr, `+0xcc current_data`, `+0xd0 current_metric`, `+0xd4
current_node`, `+0xd8 length`, `+0xdc head_node`, `+0xe0 ordered` (zeroed,
never otherwise written). `Unit` has no fields of its own (`Unit` and
`UnitData` are both 0x158), so every `field_0xcc..0xdc` the decompiler prints
on a `Unit*` is one of these. Nodes are `RecycledOrderNode {next, prev, data,
metric}` from a global recycler; `metric` is written 0 on `add`, logged,
serialised, and never read.

**The list is a circular doubly-linked ring whose `head_node` is the newest
order and whose current order is `head->prev`, the oldest.**
`LinkListBase::add@0046d5a0` splices the new node before `head` and makes it
`head`; every accessor (`Unit::update_order@006179d0`, `UnitData::get_order`,
`order_type`, `get_action`, `update_action`, `work`, `kill_current_order`,
`close_orders`) first sets `current_node = head->prev`. `remove_current@
0046d620` unlinks the cursor's node, advances the cursor to `next`, and if the
removed node was `head`, `head = head->next`. (One clause so a
re-implementation does not have to re-derive it: the cursor is left on
`head_node`, **not** on the successor, and every caller re-runs the tail
positioning before reading again. R1.) `length` is what `do_move`/
`move_step` test as "this move is the only order" (`== 1`).

So in execution order the queue is `head->prev, head->prev->prev, …, head`;
`add` appends to the *back* of the queue; and the front is rotated by moving
`head` one step along `next`.

### 1.5 `QueuePos` — the three enqueue modes

Values settled in the listing of `Unit::add_move_facing_order@005e55c0`
(`cmp eax,2 / jne` guards the clearing path; the decompile prints that clear
as unconditional and cannot be right), names from the decompile, the UI
mapping from `Options::picked_spot@00721c40`:

| value | name | UI | effect |
|---|---|---|---|
| 0 | `QUEUE_FIRST` | ctrl-click | `add`, then **`head = head->next`** — the new order is now `head->prev`, the current one; the old current resumes when it is killed. (`add_think_order` does this unconditionally, ignoring its argument.) |
| 1 | `QUEUE_LAST` | shift-click | `add` — runs after everything queued |
| 2 | `QUEUE_NEW` | plain click | `unit_masks &= ~0x4000000; path.length = 0; close_orders(0); clear_partial_path(); update_action();` then `add` — everything is killed, oldest first, with each kind's teardown (§3.2), and the new order is alone |

The engine's own inserted orders use `QUEUE_FIRST` almost everywhere (22 of
the 25 `add_move_order` call sites) — a transit leg in front of the intent —
and `QUEUE_NEW` where it means to replace the intent. `QUEUE_LAST` is the
player's shift-click, `come_out`'s rally orders and `find_gather_spot`'s.
`Group::action_swarm_around` and `action_move_near` implement `QUEUE_FIRST`
for a *group* differently: detach the list (`set_up_insert`), `action_halt`,
recurse with `QUEUE_NEW`, re-append the old list (`finish_insert`) — the new
orders go in front of everything that was queued.

### 1.6 The pool — `OrdersMemManager`

One global, `ordmgr`, 28 `SafeRecycler<UnitOrder>`, each a clean and a dirty
stack. `get_obj(kind)@00730ac0` pops the kind's clean pool (calling `clear()`)
or `get_new_order`s; every `add_*_order` starts here. `give_obj@00730bb0`
clears and pushes on the dirty pool; `kill_current_order` ends here.
`cycle@00730e20` moves dirty to clean **once a frame, from `Game::do_frame`
after `frame++`** — an order freed this frame is reusable next frame, never
this one. `init` pre-allocates 30 of each. An allocation detail with one
observable: an order's fields are whatever `clear()` leaves, which is why
every `clear` was read beside its constructor.

---

## 2. The unit's frame, and the step

### 2.1 `Game::do_frame@00591ef0`

In order: `GameLog::begin_frame` (the `[Start Frame]` dump at the current
`frame`); player speed; the scenario script (the general-powers half only
from frame 1); **`Leaders::process_all`** (income first —
`docs/ECONOMY.md`); **`NetDaemon::process_all`** (the command stream is
applied here, before any object moves); the rush-rule message;
`NetDaemon::process_all` **twice**; **`Leaders::strategy_all`** (the AI,
unless `semaphore[1] & 8`); `GameDaemon::process_all` (halves `repaths`,
§4.6); `Armies::process_all`; `NetDaemon::process_all` again;
**`Objects::process_all`** (every unit's order step, `move_step`, `fight`,
attrition, the building clocks); `Objects::inc_time`;
`GraphicEvents::process`; `NetDaemon::process_all` a fifth time;
`Leaders::end_process_all`; `Achieve::capture_data`;
`Leader::process_event_frame`; **`frame = frame + 1`**;
`OrdersMemManager::cycle`; `Roads::scan_and_kill_stray_roads`; `frame %
15 == 0 → tick++`; the auto-save (`frame != 0`); **`GameLog::end_frame`**
(the `[End Frame]` dump).

**Second reading (`docs/audit/2026-08-21-orders.md`, F1, F3).** The first
`NetDaemon::process_all` is **before** the rush-rule message, not after, and
there are **five** calls in the frame, not two — the points at which a click
or an AI `Group::action_*` reaches a unit. The only `frame == 0` branches in
`do_frame` itself are the general-powers script and the auto-save.

`Objects::process_all@0065dce0` walks players in the order `(frame + i) % 10`
for `i = 0..9` — **the player processed first rotates with the frame** — and
within a player the unit array in index order, then buildings (`≥ 2000`),
then walls (`≥ 3000`); dead objects only have `hold_frames` decremented.

**Frame numbering in the gamelog.** `GameLog::check_accept@009309a0` opens a
`FRAME n` block with `game->frame`, and `end_frame` runs *after* the
increment. So the `[Start Game]` dump is the state after `Setup::build_game`
and before any `do_frame` (frame 0), and **`FRAME n` under `[End Frame]` is
the state after n passes of `do_frame`**, the n-th pass having run with
`game->frame == n − 1`.

### 2.2 `Unit::process@00610bc0` — where the order is stepped

Per alive unit, in order, with the gates:

1. Countdowns: `recharging`, `full`, `waiting`, `healing` each decrement if
   non-zero.
2. The decoy / caster / aircraft upkeep (a decoy's `suffer_attrition(frame % 7
   == 0)` — `docs/ATTRITION.md`'s second entry; aircraft fuel and heal; a
   caster's spells and mana).
3. A carrier unit's own training queue.
4. `process_healing` — unconditionally.
5. **If not inside anything** (`inside_up < 0`): every 16 frames phased by `o`
   `process_cloak`; every 32 frames `process_attrition`; if `attrition != 0`
   and `(frame + o) % attrition == 0`, `process_supply` or `suffer_attrition(1)`
   — `docs/ATTRITION.md`'s cadence block, unchanged; then **`work()`** (vslot
   `0x188`) — the order step; then `Guy::process` for every body.
   **Else** (garrisoned or boarded): clear the moving/fighting masks; every 32
   frames phased by `who` an AI citizen inside a city may `come_out`; **no
   order is stepped while inside** — a garrisoned unit's orders are frozen,
   not killed.

So the ordering `docs/MOVEMENT.md` states — healing, cloak, attrition, the
supply check and the bleed, *then* the order step, then the bodies — is what
the function does, and what this reading adds is that **every order, not only
movement, is stepped at that one point**: a build order's clock tick, an
attack's strike, a gather's walk all run after the unit has bled for the
frame.

### 2.3 `Unit::work@0060d180` → `do_job@00617a10` — the step

`work`, in order:

1. `order = update_order()` (the oldest); `type = order ? get_type() : NONE`.
2. Every 32 frames phased by `o`: `visible = 0` unless `SubObjectData.flags &
   0x80`; `unit_masks &= ~4` (the "pathing to a unit" bit `go_to_unit` sets on
   a squad).
3. If `type != ATTACK`: `SubObjectData.flags &= 0x7f`; if `unit_masks & 0x100`
   and the order has the action bit and is not `EXPLORE_TO`, clear `0x100`;
   and a **recharging melee unit** (`recharging != 0`, `type.max_range == 0`,
   not `unit_flags & 0x400`) **returns here** unless the order has the action
   bit — it finishes a player's move but not an engine-inserted leg.
   `unit_flags & 0x400` is the **melee-and-ranged** flag: `UnitType::init@
   0061ab50:359` zeroes the range field `+0x1fc` when it is set and reports
   "Improper use of melee-and-ranged unitflag", which also confirms `+0x1fc`
   as the range (second reading, R1).
4. Caravan bookkeeping (a caravan off its route → `end_trade_route`).
5. Move-family pre-step: a `MOVE_TO` with `unit_masks & 0x4000000` is removed
   and re-added as `EXPLORE_TO` (the deferred conversion `add_move_facing_order`
   set up for `role & 0x10` types) — and it is **stepped as an `EXPLORE_TO` in
   the same frame**: the listing at `0x60d712` re-runs `update_order` and
   `get_type` into the locals `do_job` is then called with (R1, which also
   settles that `do_job`'s second argument is always the current order, never
   the action); unless the unit is an unpacked merchant
   kind, **every 16 frames phased by `o`**: `action = update_action()`, and if
   it `is_targeted` and is not `AWAIT_BOARD`: a `GUARD` repaths every 64,
   any other targeted action → `check_target_path@005e22d0` (which re-paths
   only to *unit* targets — a building target is left alone). Then an
   action-bit move that is the only order: if within `block_radius`
   (`ptype+0x240`) of its target object the move is reset to the unit's
   position and, if the unit's tile equals the target's, killed; else a cast
   of `0x28b` is queued (the decompiled flow here is not credible — §14).
6. Non-move orders with `SubObjectData.flags & 8` (the "freshly ordered" bit
   `check_idle` sets): merchants/fur trappers set the leader's `0x2000000`;
   clear the bit.
7. An open path search (`openlist`) on a non-move order → `clear_partial_path`.
8. **Target liveness**: `action = update_action()`; if targeted: `(ox, whom)`
   through `get_target_order`; `ATTACK` → `set_in_danger` on self and a unit
   target; **if the target's `uid` (`ObjectData+0x30`) differs from the
   order's** — it died or its slot was reused — `STRAFE` resets its target,
   an `is_attack` action that is current clears its target, and then
   **`repath(); kill_current_order(0)`** — a build order whose site was
   replaced, a gather order whose building died, dies here before its `do_*`
   ever runs. A `TRADE_ROUTE` action tests its second endpoint the same way.
9. `do_launch` if carrying air; `safe--`.
10. **If `type != NONE`: `idle = 0`** (`UnitData+0xb0`); the `0x1000` mask for
    `unit_flags & 0x40000` types.
11. A recent collision (`collide_frame` within 4 frames) on a non-move order
    → `detect_boat_collision`.
12. **`do_job(type, order)`** — a `switch` on `OrderIndex`, one call per case:
    `NONE → do_idle` (vslot `0x184`); `MOVE_TO, FLEE_TO → do_move`; `ATTACK_TO
    → do_attack_to`; `EXPLORE_TO → do_explore_to`; `BUILD_AT → do_build`;
    `GATHER → do_gather`; `BOARD_SHIP → do_board`; `AWAIT_BOARD →
    do_await_board`; `ATTACK → do_attack`; `FOLLOW → do_follow`; `GUARD →
    do_guard`; `REPAIR → do_repair`; `CAST_SPELL → do_cast`; `TRADE_ROUTE →
    do_trade`; `STRAFE → do_strafe`; `AIR_PATROL → do_air_patrol`; `CHANGE_FORM
    → do_form_change`; `GROUP_MOVE → do_group_move`; `GROUP_ATTACK →
    do_group_attack`; `GROUP_ATTACK_TO → do_group_attack_to`; `GROUP_PATROL →
    do_patrol`; `ATTACK_GROUND → do_attack_ground`; `AIR_ATTACK_GROUND →
    do_air_attack_ground`; `SPECIAL_ANIM → do_spec_anim`; `GARRISON →
    do_garrison`; `THINK → do_think_order`.
13. Post-step: the stacking separation for `unit_flags & 0x20` types (push two
    same-type units within `0x180` apart by half their separation each);
    `unit_masks &= ~0x10`.

**One order is stepped per unit per frame, and it is the one at the front
when `work` starts.** When a `do_*` kills its order, the next one is not
started until the following frame's `work`; when a `do_*` inserts a move
`QUEUE_FIRST` (the gather walk, the chase), that move runs from the *next*
frame. Two `do_*` are the exception and call `do_move` on the move they just
inserted, in the same frame: `do_guard` and `do_follow` (§7.5, §7.6). An
order applied from the command stream, by contrast, is stepped in the **same
frame** it arrives — `NetDaemon::process_all` runs before `Objects::
process_all` — so a player's move order steps, and the position logged at the
end of that frame already carries the first step (§4.8).

### 2.4 Idle — `do_idle`, `check_idle`, `think`

`Unit::do_idle@0060dcd0` (the `NONE` case): `set_anim(CHAR_DEFAULT)` unless a
hero mid-attack-animation; `collide = 0`; **`check_idle()`; `think()`**. So
`think` runs every frame the list is empty; its own cadence is inside.

**The idle counter** — `UnitData::idle` (`+0xb0`, a byte) — `Unit::check_idle@
006032c0`: `0xff → 1`; if `idle > 1` and `(o + frame) & 15 != 0` leave it;
else `idle += 1`. `work` zeroes it whenever an order exists. So from the frame
the list empties: 0 → 1 → 2 on two consecutive frames, then +1 every 16
frames phased by `o`. Side effects on it: the entrench (`americans_marine_
entrench`), cloak particles, and on the first idle frame `SubObjectData.flags
|= 0x18` (the "freshly idle" bits; `0x8` is what `work` step 6 reads), setting
the leader's `0x2000000` for fishermen/merchants/fur trappers.

**`Unit::think@005f6e40`** (343 lines; the branch conditions are legible,
several bodies are dropped by the decompiler).

**The global cadence gate, found by the second reading (R1) and missing from
the first.** Before everything below except the captain mirror and the
citizen mask-clear, `think@005f6e40:87` returns when

```
(SubObjectData.flags & 0x10) == 0 && idle > 2 && ((o + frame) & 15) != 0
```

— so **from the third idle frame on a unit thinks once in sixteen**, phased
by `o`, and only the "could not reach" bit (§6.3) restores every-frame
searching. This changes when an idle citizen re-picks a job and when an idle
soldier runs its target search, and it was a live divergence in
`crates/sim/src/orders.rs` until it was landed. Then, in order:

1. A non-captain whose captain's action is `ATTACK` with a valid target:
   `add_attack_order(target, QUEUE_NEW, …)` — squads follow the captain's
   fight.
2. Citizens (`PEASANTS`/`PEASANTSKOREAN`): `unit_masks &= 0x87ffffff`, leader
   flags `|= 0x80000`.
3. **Auto-attack** (unless `unit_masks & 0x100`): on `idle == 1` or every 32
   frames phased by `o`, a captain with a remembered `near_o/near_who` that is
   alive and in range → `add_attack_order(near, QUEUE_NEW, 0, 0)`; then for
   fighting kinds `think_attack` (`docs/COMBAT.md` §8.1) or `think_merchant`.
4. **Workers** (`ObjectData::is_worker`): **`think_peasant(0)`** — §5.9;
   nonzero **ends the think** (`005f7195`), so a citizen just given a job
   never reaches 5.
5. Caravans → `think_caravan`; rare collectors — merchants *and* fishing
   boats — ~~(AI only)~~ **for a human** (`leader_flags & 4`, the same bit
   `Leader::new_rare@006d9e70` reads; `docs/MERCHANT.md` §4) on `idle == 1`
   or every 32 frames → `do_gather(search)` and then
   `unpack_merchant(this, 4)`; AI specials → `think_spellcaster`,
   `think_scout`; everyone on `idle == 1`/32: fishermen → `think_fish`;
   merchants on `idle == 1`/128 → `think_merchant` (`docs/MERCHANT.md`);
   carriers → `think_carry`;
   then the tail (`005f7615`): a supply wagon or hero → `add_to_army`, a
   scout or spy with no army → `think_scout`, anything else returns
   (`docs/SCOUT.md` §2).

**The "everyone on `idle == 1`/32" in step 5 is a gate of its own, and it is
the second of two.** It stands between the human block's `unit_masks &
0x40000` exit and `think_fish`, and it returns:

```
if (idle != 1 && ((o + frame) & 31) != 0) return;
```

So the whole of step 5 from `think_fish` down — the tail included — runs on a
unit's **first** idle frame and then once in thirty-two, phased by `o`. The
mod-16 gate above only decides whether the function is entered at all; this
one decides the tail, and the two are not the same period. A unit on its
*second* idle frame passes the first (`idle > 2` is false) and fails this one.

**That one frame was East Indies' word.** This crate carried only the mod-16
gate until 2026-09-01, so run56's AI scout — which arrives, goes idle on 2664
with `idle == 1` and takes a whole `think_scout` both sides agree on — ran a
*second* whole `think_scout` on 2665, where the original runs none: with
`o == 0` the original's next think is 2688, and then 2720, 2752, … exactly.
Nineteen draws that were nobody's. `scout::tests::an_idle_scout_thinks_on_
its_first_frame_and_then_once_in_thirty_two` is the guard, and it fails on the
old code with the scout thinking on every frame of `idle <= 2` and then every
sixteenth.

So an idle soldier does its target search on its first idle frame and every
32 frames after, phased by `o`; an idle citizen runs `think_peasant(0)` every
frame, with its own gate on `idle` (§5.9) — `think_peasant` sits at
`LAB_005f7179`, **above** the tail's gate, which is why the citizen keeps its
own cadence and the scout does not. For a unit nobody orders and nothing
approaches, the observable per-frame writes are `idle` and `collide = 0`.

**`ThinkOrder`** exists for one reason: `Unit::do_gather` queues one (three
sites) when a gather order cannot proceed — `kill_current_order(0);
add_think_order()` — and `Unit::add_think_order@005e3df0` (argument ignored:
`flags |= 4`, `add`, `head = head->next`) makes it current. `do_think_order@
005e5bf0` kills it; if the list is now non-empty that order runs next frame;
else a citizen or scholar runs **`think_peasant(1)`** — the forced job search,
without waiting for the idle timer. The fall-through set is the four type
indices `0x32..0x35` (`PEASANTS, PEASANTSKOREAN, SCHOLARS, SCHOLARSKOREAN`);
note that `think`'s own citizen branch above is `0x32`/`0x33` **only** (R1).
`check_idle` has one more effect worth recording: at `idle == 4` it re-faces
an entrenched unit.

---

## 3. Adding, killing, and the action

### 3.1 `Unit::add_*_order` — one shape, 21 functions

Read in full for `add_move_facing_order@005e55c0` and confirmed by grep on the
rest — **with two exceptions the second reading found (R1, R3), because the
grep behind "21" matched the `QUEUE_NEW` prologue rather than the
`QUEUE_FIRST` tail**:

- **`add_repair_order@005e4ff0` has no `QUEUE_FIRST` branch at all.** After
  the list add at `0x5e51f0` come only `update_action` and `ret 0x10`, where
  `add_build_order` has an explicit `cmp [ebp+0x10],0` → `clear_partial_path;
  head = head->next`. It also overwrites its own `where` slot at `0x5e50e7`,
  so no later test is possible. No caller passes `QUEUE_FIRST`, so nothing
  observable turns on it — but a re-implementation should not invent the
  branch.
- **Six sites bypass the `add_*` façade entirely** and call
  `OrdersMemManager::get_obj` directly: `fight@005fd4d0:556`
  (`ATTACK_GROUND`), `unpack_merchant@006038e0:26`, and
  `check_meet_ship@00604550:239, :285, :324` (`MOVE_TO`).

The common shape, then:

1. If `queued == QUEUE_NEW`: `unit_masks &= ~0x4000000; path.length = 0;
   close_orders(0); clear_partial_path(); update_action()`.
2. `get_obj(kind)`, down-cast, fill the fields, `LinkListBase::add`.
3. If `queued == QUEUE_FIRST`: `clear_partial_path()`, `head = head->next`.
4. `update_action()`.

Adding never touches the current order's state except through
`clear_partial_path` and `update_action` — a `QUEUE_FIRST` insert suspends the
current move with its path intact (`flags & 1` stays) and it resumes when the
inserted order is killed. `add_build_order`/`add_repair_order` also set
`unit_masks |= 0x400` ("has been a builder"; `think_peasant` reads it) and,
like `add_gather_order`, request a civilian transport (`unit_masks |=
0x800000`) when the target's tile region differs from the unit's and the
player's transport level allows it.

### 3.2 `Unit::kill_current_order(silent)@005e2cb0`

Tears down the current order. `silent != 0` is reached from **exactly one
call site** — `Unit::close@0060ee50:601`, under `SubObjectData.flags & 1` —
i.e. only when the unit is being destroyed (R1). `unit_masks &= ~0x20000`;
then per kind:

- **move family**: `facing = mo->facing` — and `MoveOrder::facing` **does**
  have a reader, here: `kill_current_order@005e2cb0` fetches it through the
  order vtable slot `+0xb8` (`0x5e3018 mov eax,[eax+0x28]`), which is why a
  grep for the field name finds nothing (R2). If `facing ≥ 0 && !silent &&
  playing`: set or clear `unit_masks & 2` by it (`reversing()` is asked
  `unit->angle − order->angle`) and,
  if the unit leads a group, write it to the group's `+0x48` — the "face the
  way you were told" carry-over (`docs/MOVEMENT.md`'s reversed bit).
  **This paragraph was right, and it sat here unjoined to
  `docs/GROUPS.md` §6.3 for a day while that document called the mirror's
  predicate its sharpest open question** (2026-08-26). What was missing
  was the *ordering*: the `QUEUE_NEW` clear that fires this runs **before**
  the next `compute_form`, so this write is not a carry-over into some
  later frame — it is the flag the very next layout reads.
  `Sim::hand_back_facing` implements it.
- **`GATHER`**: leader flags `|= 0x2000000` (the economy's re-plan hint); if
  playing and the target is a live building of the same owner,
  **`Build::remove_gatherer(build, o)`** — killing a gather order is what
  removes the unit from the building's chain (§6.1); `unit_masks &=
  0x87ffffff`; a citizen's hold-doober removed.
- **`TRADE_ROUTE`**: `end_trade_route`. **`CAST_SPELL`**: a paid cast refunds
  (`unpay_cast_costs`) unless `silent`. **`SPECIAL_ANIM`** type 1: leave the
  building's `launching` list.

Then: reposition, `remove_current`; if the removed order `is_move && is_pathed`
→ `kill_current_path` (pop the path stack until an entry with `flags & 1` is
popped); `give_obj`; `clear_partial_path`; `update_action`.

`close_orders(silent)@005e37f0` is `while (head && current->get_type() != NONE)
kill_current_order(silent)` — it empties the list oldest first, so every
teardown fires in queue order. `clear_orders@005e3860` is the `QUEUE_NEW`
clear without the add (29 callers: death, capture, garrison, the group
halt). `reset_move_orders` writes every move's `orig_x/y = x/y`;
`modify_group_order(id, …)` and `kill_group_order(id)` are the group's (§8.4).

### 3.3 `update_action` / `get_action` — "what am I doing" vs "where am I going"

`Unit::update_action@0060a870` writes `orders_x/y` (`UnitData+0x70/+0x74`) =
the unit's position and `dest_angle` (`+0x58`) = `angle`; then walks from the
current order forward over every **plain move** — `is_move()` and without the
action bit — and every `CHANGE_FORM`, copying each move's `x, y, angle` into
`orders_x, orders_y, dest_angle` as it goes, and stops at the first order
that is neither, returning it (or `NULL` when the queue is moves only).
**The terminating order also writes** `orders_x/orders_y/dest_angle` when it
`is_move()` and either it is the last node or it carries the action bit
(`update_action@0060a870:47`) — without that clause the mechanism could not
produce the `dest_angle` the gamelog shows for a player's move, which is how
the second reading caught it (R1). `crates/sim/src/orders.rs` already does
this.
`UnitData::get_action@00608450` is the same walk without the writes.

So `orders_x/y` is the final destination of the leading run of transit moves
(what the UI flags, what `check_target_path`'s range test uses), `dest_angle`
the last such move's angle — the log shows `dest_angle` equal to the queued
move's `angle` on the frame the move is queued, before the unit has turned —
and **the action** is the first intent: an `ATTACK` behind its chase, a
`BUILD_AT` behind its approach, a `GATHER` behind its walk, a move that
carries the action bit. It runs at the end of every `add_*`, every kill, and
at several points inside `do_move`.

---

## 4. The move order

### 4.1 The fields, from their writers and readers

All `Coord`s are position units (192 a tile, 768 a world cell); angles are
`docs/MOVEMENT.md`'s binary angle.

**Diffed, 2026-08-26.** This table was read off the PE and compared with
nothing for two days. `run29_s_move_orders_match_the_field_table_row_for_row`
in `rondata::diff` now walks **every move order of run29's four states** —
79 of them — and asserts the rows that claim a shape: the destination is
snapped to its 48-unit cell centre; `off_x`/`off_y` are `x mod 0x300`,
`y mod 0x300` (the offset *inside the world cell*, and **not** a formation
slot — the name collision with `GroupData::off_x` has misled a reader
before); `tolerance`, `pause`, `retry`, `attempts` and `timer` are 0
throughout; `dest_x/dest_y` is the path stack's **top** whenever `dest` is
1; the bottom of the stack carries the goal flag; the `pathed` bit is set
exactly when the unit has a stack at all; and `orders_x/orders_y` is the
current move's own `x, y`. `coll_x/coll_y` is the one row with no
invariant — 17 of the 79 carry a blocker's position, which is
`detect_unit_collision` working.

| field | what it is |
|---|---|
| `x, y` | the destination, **snapped to the centre of its 48-unit cell**: `add_move_facing_order` takes `UCoord`s and stores `u*0x30 + 0x18`; `add_move_order@00616ed0` converts a `Coord` with `div_3_table[v >> 4]` (`= v/48`). The log confirms: a scout's `x 45048 = 938*48+24`, a citizen's `x 4008 = 83*48+24`. |
| `angle` | the facing to apply on arrival. `add_move_order` computes `find_angle(x − pos.x, y − pos.y)` — the direction from the unit to the target *at order time* (register-called; settled in the listing at `00616ed6..616f21`). `Group::action_move_near` passes the formation angle with a per-slot byte in the top byte. Consumed by `move_step` on the final waypoint when the move is the only order (or the action is `GATHER`), and by `update_action` as `dest_angle`. |
| `dest` | **"I have a current waypoint"**: 1 when `do_move` takes the stack top into `dest_x/dest_y`; 0 on arrival (`move_step`), on `go_around_building` failure, after `resolve_unit_collision`, on a collision at `coll_x/coll_y`, and from `ungroup_move_order`. |
| `tolerance` | **never written** except by `clear`/`operator=`; `UnitData::tolerance (+0x60)` is the live one. Logged 0 throughout. |
| `pause` | frames to stand still: `resolve_unit_collision` sets `Random::get(0,0xffff) % 9 + 1` on a mutual collision; `do_attack_to_pause` sets 15; `do_move` decrements and does not step while non-zero. |
| `retry`, `attempts` | the modern-infantry pack/entrench wait: every 128 frames (phase `o*0x11 + frame`) a modern-infantry unit with no general sets `retry = guy0+0x78`, `attempts = −3`; each later frame `retry--`, at 0 `attempts += 3`; `attempts` then counts down one a frame. While `retry != 0` the unit does not step. Zero for everything else. |
| `timer` | a self-destruct: `do_move` decrements it and at 1 kills the order and re-runs `work` the same frame. Set by `do_guard` on its reposition move (`0x1e × max(1, tile Manhattan)`); zero in every logged plain move. |
| `facing` | the caller's (−1 from `add_move_order`; the formation's `reverse` from `action_move_near`); read by `kill_current_order` (§3.2). Not used by the step. |
| `dest_x, dest_y` | **the current waypoint — what `move_step` walks toward.** Initialised to `x, y`; rewritten from the stack top each time `dest` goes 0 → 1; moved by `find_path`'s pull-back and by `resolve_unit_collision`'s side-step. |
| `last_x, last_y` | the position at which the last straight-line plan was made (`find_path` after a successful detour); −1 when a fresh target is taken; a stack top equal to `last` (and not final) is popped before re-planning. |
| `coll_x, coll_y` | **the refused step point**, not the blocker's (run10 frame 123) and not `astar_path`'s: `detect_unit_collision@00617060` writes it. `do_move` re-probes it every other frame while a search runs. `docs/COLLISION.md` §4.3. |
| `orig_x, orig_y` | the un-snapped point the caller asked for (`action_move_near` passes the click; `add_move_order` passes −1, −1). Not reset by `clear`. Informational. |
| `off_x, off_y` | `x mod 0x300`, `y mod 0x300` — the destination's offset inside its world cell; `go_around_building` and `find_tpath` use `off % 0xc0` (inside the *tile*) to place detour waypoints off-centre. |

`ExploreToOrder`, `FleeToOrder`, `AttackToOrder` are `MoveOrder` with no extra
fields; their `log_data` opens their own block and delegates. `do_job` sends
`MOVE_TO` and `FLEE_TO` to `do_move`, `EXPLORE_TO` to `do_explore_to`,
`ATTACK_TO` to `do_attack_to` (§7.4), which share everything from planning on.

### 4.2 The unit-side state

| `UnitData` | role |
|---|---|
| `+0x58 dest_angle`, `+0x70 orders_x`, `+0x74 orders_y` | `update_action`'s outputs (§3.3) |
| `+0x60 tolerance` | the **current waypoint's** tolerance, copied from its `PathData` when taken; raised on collision. The arrival radius. |
| `+0x68 unit_masks` bit `8` | "**a straight line to `dest_x/dest_y` has been verified, go**" — set by `find_path == 0`, cleared when a new waypoint is taken, when `move_step` enters a tile that turns out invalid, and by `go_around_building` failure. The switch between the planning half and the stepping half of `do_move`. (The log's `unit_masks 0x1000000A` on the citizen's first moving frame is this bit, plus `2` from `set_angle`'s quarter-turn toggle.) |
| `+0x88 collide` | a collision counter: reset when a probe of `coll_x/y` comes back clear, `++` per resumed search and in `resolve_unit_collision`; chooses between `find_tpath` and `find_upath` (§4.4). |
| `+0xaf path_recursion` | `find_path`'s recursion depth: zeroed before each `find_path`, `++` per call (saturating 255), `go_around_building` refused at 10. Logged `1` after one straight-line check. |
| `+0xb8 Stack<PathData> path` | `{list, size, length, increment}`, 10 slots to start. **The top is `list[length−1]`**; `peek` on an empty stack returns `list[0]` — stale; callers guard with `length`. |
| `+0x104..0x114 openlist, openlistrefs, closedlist, validlist, blocklist` | a *suspended* A\* search; `openlist != 0` means one is pending; `clear_partial_path@005e3920` returns them and zeroes the search scratch. |
| `+0x120 avoid_x/y` | −1,−1 before each `move_step`; `find_path` writes the point it could not reach. |

`PathData` (`types.txt:40517`): `{Coord to_x, to_y; int tolerance; int flags}`.
Flags: `1` = **the final waypoint of an order's segment** (the goal); `2` = a
collision side-step; `4` = **turn in place before walking to this one**
(`move_step`, per `docs/MOVEMENT.md`; set on the goal when it lies in a
different terrain region) — and it **also relaxes `invalid_loc`**, letting the
leg cross terrain the unit would otherwise refuse; `8` = a
`go_around_building` mid-detour point, which **suppresses the collision test
entirely** (both R2); `0x10` = a block to
`resolve_block` (a building's tile: own → mark, allied → a diplomacy bit,
enemy → `add_attack_order`); `0x20` = a caravan road waypoint.

### 4.3 Lifecycle

**Created** by `Unit::add_move_facing_order@005e55c0` — the only constructor
path; `add_move_order@00616ed0` is a thin wrapper that snaps the destination
and takes the angle **to the point it was handed, not to the snap of it** —
the listing subtracts the unit's position from the *arguments* and only then
indexes `div_3_table`, so an unsnapped point (every farm cell, every pasture
walk) gives a heading up to 24 units an axis off the destination
(`docs/SYNC.md` §3.12). It fills `x, y, angle, dest = 0, dest_x/y = x/y, last = −1, off_x/y,
orig, facing, pause = retry = timer = 0`, the `flags` bits `1/4/0x20` from
three arguments; two side rules: a `QUEUE_LAST` `MOVE_TO` for a **`role & 0x10`** type
(`UnitTypeData +0x2c8` — *not* `unit_flags2`, as §2.3 step 5 already had it
right; R1 — and the bit is `is(SCOUT)` on land, `is(BARK)` at sea:
`docs/GROUPS.md` §6.6 step 6) becomes `EXPLORE_TO` and sets
`unit_masks |= 0x4000000`; and a
`QUEUE_NEW` attack-move (`mode == 2`) while the current order is `ATTACK`
re-adds that attack `QUEUE_FIRST` after the move, so an attack-move issued
mid-fight keeps the fight. The second reading corrects that last rule twice
(R2): there is **no liveness test** on the target, and sea targets are
**kept**, under `has_objmask(0x80000000)`.

The player's right-click reaches it through `CommandPackage::process_move_to`
→ `Group::action_move_to@0070fba0` → `action_move_near@00704990`, which for a
group of one calls `add_move_facing_order(ux, uy, angle, MOVE_TO, pathed = 1,
queue, action = 1, form.reverse, click, disembark)` and for a larger land group
`add_group_move_order` (§8). `Unit::go_to@005f7a50` (used by `go_to_unit`/
`go_to_city` when farther than `0x480`) builds a one-unit group and goes
through the same `action_move_to` with `QUEUE_NEW` and action = 0. The
in-engine transit legs — `check_target_path`, `do_attack`, `do_gather`,
`do_garrison`, `do_trade`, `do_cast`, `fight`, `repair_damage`,
`target_opportunity`, `come_out`, `check_meet_ship`, `do_non_flat_gather` —
call `add_move_order`, all with `QUEUE_FIRST`, most with `pathed = 0`.

**Stepped** by `do_move@005f7b30` once a frame while it is current.
**Finished** by `kill_current_order(0)` — from `move_step` on the final
waypoint, from `do_move` on a dead or unreachable goal, from `repath` when a
target order decides its transit legs are stale.

### 4.4 `Unit::do_move@005f7b30` — the per-frame step

`mo = order->get_move_order()`. Each frame, in order:

**Before planning.**

1. Cavalry-archer fire on the move (`cavarch_fight` every 32 frames;
   combat's).
2. **A suspended search** (`openlist != 0`): an `ATTACK` action retargets
   every 4 frames; a `GATHER` action within `vector_dist < 0x120` parks the
   unit (`avoid_x/y = pos`, resets the gather fields, kills the move); else
   every other frame from 4 on, `detect_unit_collision(coll_x, coll_y, …) ==
   0` → `dest = 0; clear_partial_path; flags |= 1; collide = 0; return 0` (the
   blocker has gone — re-plan next frame); otherwise every 4 frames
   `repaths[who]++`, `collide++`, `dest = 0`, `PathFinder::find_upath_restore`
   resumes the A\* with limit `300 / repaths²`. **No step while a search is
   pending.**
3. **`timer`**: `> 0`: at 1 → `kill_current_order(0); work(); return 0`; else
   `timer--`.
4. **The action under the move** (`get_action()`): if `ATTACK` and the target
   alive with the same `uid` — a ranged type with a non-building, non-flank,
   in-range target and no collision at its own spot → `kill_current_order`
   (the move has done its job); a melee type every 16 frames may
   `find_melee_target` a closer in-range unit; a building target in range,
   valid, no collision → kill the move; a non-mandatory `ATTACK` action every
   4th frame under `repaths` budget → `find_new_target(0, 1)` (`docs/COMBAT.md`
   §12.4's "walking to attack" search). A target that is gone and the unit
   within `0x481` of its point → `repath(); return 0`. A `TRADE_ROUTE` with
   negative endpoints → kill.

**The entrench wait.** `if (retry != 0) { if (--retry == 0) attempts += 3;
return 0; }`; the 128-frame modern-infantry check (§4.1); `if (attempts != 0)
attempts--`.

**Planning — the first time, or with an empty stack.**

```
if (!mo->is_pathed() || path.length == 0) {
    push { mo->x, mo->y, tol 0, flags 1 }                       // the goal, final
    r = PathFinder::find_wpath(&path, pos, who, o)              // §4.6
    if (r < 1) { re-push the goal if it is gone;
                 r = PathFinder::find_tpath(&path, pos, who, o); re-push if gone }
    flags |= 1                                                   // pathed
    if (path.length > 10) return 1                              // a long path: step next frame
}
```

So **every fresh move calls `find_wpath` exactly once**; on open ground within
two world cells it returns with the stack still `[goal]`.

**Taking a waypoint** (`dest == 0`):

```
top = path[length−1]
dest = 1; unit_masks &= ~8; dest_x/y = top; UnitData::tolerance = top.tolerance
if (top.flags & 1 or the action is TRADE_ROUTE) and get_tregion(top tile) != get_tregion(unit tile):
        pop; push top with flags |= 4, tolerance = 0                  // turn in place first
c = detect_unit_collision(top, …)
if c: if (top.flags & 1) and the action is TRADE_ROUTE/GATHER/ATTACK/BUILD_AT → kill the order
      else if the collider's order is not a move: t = collider.type.big_radius * 3;
           if UnitData::tolerance < t: tolerance = t; pop; push top with tolerance t    // give up short of a parked unit
if vector_dist(dest − pos) <= UnitData::tolerance:                                    // already there (Euclidean)
    dest = 0; pop; if !(flags & 1) return 1
    if orderlist.length == 1: set_angle(mo->angle, 0)
    flags &= ~1; kill_current_order(0); return 0
if (top.flags & 0x10) and resolve_block(): return 1
```

**Second reading (R2) — the region check's guard was inverted.** The first
reading had it run for a *non*-final waypoint; it runs for the **goal**, or
for a `TRADE_ROUTE` leg, and it also zeroes the tolerance. The turn-in-place
is what a unit does before crossing into a different terrain region at the
*end* of a leg, not in the middle of one.

**The collision test is the block's teeth, and it is diff-backed** (item 63).
It is `detect_unit_collision`'s third call site — the only one that runs
*before* a step — and it fires **once per leg**, on the frame `dest` goes
0 → 1. Its kill arm is what makes a worker whose destination is taken stand
still instead of walking into it; run10's frames 199–201 are the case, and
`docs/COLLISION.md` §5.1 and §8 carry it.

**The speed, and the straight-line check.** `speed = get_speed(pos, 0)`, `×
ai_speed` if > 1, `× 5/4` truncating toward zero for modern infantry — exactly
as `docs/MOVEMENT.md` states. Then:

```
if !(unit_masks & 8):
    path_recursion = 0
    r = find_path(dest_x, dest_y)                                  // §4.5
    if r == 0: top = peek; if top == pos: masks &= ~8
               else: masks |= 8; dest = 1; dest_x/y = top; tolerance = top.tolerance
    elif r == 2: return 1
    if masks & 8: goto STEP
    // the straight line is not enough: ask the pathfinder
    if invalid_loc(tile of mo->x,y, 1,0,0,0,1) and orderlist.length > 1: kill twice, return 1
    n = Random::get(game_random, 0, 0xffff)                        // *** the sync RNG ***
    thr = n%5==2 ? 0x600 : n%5==0 ? 0x1800 : 0xf00                 // 2, 8 or 5 world cells, Manhattan
    if |dest − pos|_manhattan > thr:
        r = find_wpath(&path, who, o)   (from the unit's position); wflag = 1
    else:
        top = peek; if !(top.flags & 1) and top == last_x/y: pop
        if collide == 0: pop while !(top.flags & 0x21) and top.tolerance < 0x60;  r = find_tpath(&path, who, o)
        else:            r = find_upath(&path, who, o, 0)
        wflag = 0
    (r > 0 with the length unchanged counts as 0)
    if r == 0:
        if path.length == 0: goto TAKE
        top = peek
        if !(top.flags & 1): pop (and flag the next for a turn-in-place if needed); dest = 0; return 1
        pop; flags &= ~1; fallthrough to KILL
    elif r != −1:
     TAKE: last_x/y = −1; dest = 1; top = peek
        if (top.flags & 0x10) and resolve_block(): return 1
        path_recursion = 0; r2 = find_path(top)
        if r2 == 0: masks |= 8  elif r2 == 2: return 1
        if !(masks & 8): if wflag: return 1; resolve_block(); return 1
        top = peek; dest_x/y = top; tolerance = top.tolerance; goto STEP_IF_MOVING
     KILL: kill_current_order(0); if the order beneath is ATTACK or BUILD_AT: kill it too; return 1
```

Read as a rule: a goal farther than the drawn threshold (2, 5 or 8 world
cells, Manhattan — **one `Random::get` from the sync stream on the first
`do_move` of any move `find_path` refuses**) is planned on the **world-cell
grid** (`find_wpath`, step `0x300`), a nearer one on the **tile grid**
(`find_tpath`, `0xc0`) unless the unit has been colliding, when it is planned
on the **48-unit grid** (`find_upath`, `0x30`). `−1` (goal off the map) kills
the order and, if the order beneath is an `ATTACK` or `BUILD_AT`, that too. A
`0` with a non-final top drops that waypoint and tries again next frame. A
positive result, or `0` on an empty stack, takes the top as the new target
and verifies the straight line to it with `find_path` **the same frame** — the
unit steps on the frame the path is computed.

**Stepping.**

```
if (unit_masks & 8):
    if pause != 0: pause--; (idle animation for an unarmed attack-move); return 1
STEP: avoid_x/y = −1,−1; return move_step(mo, speed)
```

`do_move` returns 0 for "nothing to do this frame" and 1 for "did something"
(stepped, killed, re-planned); `do_group_move` reads the value.

### 4.5 Arrival — the handoff into `move_step@005faf30`

`move_step(mo, step)` reads `mo->dest_x/dest_y` as the target and the stack
top's `tolerance`/`flags` by index (the top is not popped), calls
`set_angle(find_angle(dest − pos), 0)` first — so `UnitData::angle` is
written with the heading every step — does the turning and the step exactly
as `docs/MOVEMENT.md` gives them, and then:

- **Partial step** (`step < manh`): after moving, if `|dest − pos|_manhattan >
  UnitData::tolerance` → `return 1` (still going). Else **arrived**: `dest = 0;
  pop`; a popped entry that is not final → `return 1` (next frame `do_move`
  takes the next waypoint and, `masks & 8` being clear, `find_path` re-checks
  the new leg); final: `if orderlist.length == 1: set_angle(mo->angle, 0)`;
  `flags &= ~1`; (the air-unload case); **`kill_current_order(0); return 1`**.
- **Full step** (`manh ≤ step`, the Manhattan snap): `detect_unit_collision
  (dest)`; if clear and the tile valid: `set_anim`, `set_new_location(dest)`;
  `dest = 0; pop`. **The leg is popped even when the destination turns out to
  be occupied** (R2) — the unit does not keep re-aiming at a taken cell. Not final → `return 1`. Final: `if orderlist.length == 1 or
  the action is GATHER: set_angle(mo->angle, 0)`; `flags &= ~1`;
  `kill_current_order(0); return 1`.
- A step into a tile `invalid_loc` says is blocked: `unit_masks &= ~8; return
  0` — next frame `do_move` re-plans. A collision on a partial step with
  `tolerance` already raised: `UnitData::tolerance = 2 × manh` (the give-up
  short of `docs/MOVEMENT.md`), otherwise `resolve_unit_collision`.

**Two arrival tests, two metrics.** Before stepping, `do_move` tests the
Euclidean `vector_dist(dest − pos) ≤ tolerance`; after a partial step
`move_step` tests the Manhattan `|dx|+|dy| ≤ tolerance`. With tolerance 0
(every goal, every tile/48-grid waypoint's last entry) only the exact snap
arrives; a world-cell waypoint (`0x180`) or a tile waypoint (`0x60`) is
"reached" when the unit passes within that Manhattan distance — how a pathed
unit cuts its corners.

**The facing on arrival** is `mo->angle` — the direction from the start point
to the goal *at order time* — applied only when the move was the unit's only
order or the action beneath is a gather; a unit with a second order queued
keeps its heading — and **the two arms differ by exactly that gather
clause**, which is diff-backed on run10's frames 110 and 116
(`docs/SYNC.md` §3.12). `UnitData::angle` is the heading and `GuyData::
angle` the facing, `do_turn` the only writer of the second
(`docs/MOVEMENT.md`, "Two angles"); `set_angle(mo->angle, 0)` passes **0**
for the snap flag, so it moves the heading and not the facing, and the body
swings onto the order's angle over the frames after the unit stops.

### 4.6 `Unit::find_path@005fb910` — the straight-line verifier, and the pathfinder seam

Called with a waypoint and returns **0 = walk straight (or a detour was
pushed and verified)** or **1 = cannot/do not — plan**. A third value, `2`
("abort this frame"), is written in the body but **cannot be returned**: the
sole `mov eax,2` at `0x5fc345` sits downstream of the `cmp eax,2` on the
*recursive* call at `0x5fc127`, so nothing produces it at the base case and
`do_move`'s two `r == 2` arms are **dead code** (second reading, R2; the
first reading documented `2` as live).

```
the domain is not 2 (air) → 0        // the gate is `domain != 2`, not a flyer test (R2)
if invalid_loc(my tile, 1,0,0,0,0): find_nearby_spot; set_new_location(there, 1, 0); return 1   // off a bad tile
if (x,y) == pos: return 0
if cell-Manhattan(pos, goal) > 4 and !invalid_loc(goal tile, 1,1,0,0,0): return 1          // far and reachable: the pathfinder's
ang = find_angle(goal − pos); spd = max(get_speed(pos,0), 3); sx = sinx(ang, spd); cy = cosx(ang, spd)
path_recursion++
while invalid_loc(goal tile, 0,0,0,0,0): pull the goal back toward us by (sx, cy), rewriting mo->dest_x/y and the stack top if they are it
air → 0;  goal == pos → avoid_x/y = goal; return 1
if manh(goal − pos) <= spd: return 0                                       // the loop is never entered
march, while spd < manh(goal − pos):                                        // (third reading, 2026-08-23)
    if manh grew since the previous iteration: return 0
    ang = find_angle(goal − pos)                    // RECOMPUTED EVERY STEP from the current remainder
    sx = sinx(ang, spd); if |dx| < |sx|: sx = dx    // each axis CLAMPED to its remainder — no overshoot
    cy = cosx(ang, spd); if |dy| < |cy|: cy = −dy
    pos += (sx, −cy)
    if the step enters a new tile and invalid_loc(tile, 0,0,0,0,0):
        if path_recursion < 10: go_around_building(last good point, ang, spd)       // pushes 1–3 detour waypoints
            if it did not give up: w = the first detour point; if w is not pos/last/the goal: r = find_path(w)
                if r == 0: re-push the detour, last = pos, masks |= 8; return 0
                if r == 2: return 2
        return 1                                                                     // blocked, no detour: plan
    recompute dx, dy; if |dx| <= |sx| and |dy| <= |cy|: return 0   // within one ACTUAL step, both axes (<=)
return 0
```

**On open, flat ground** (every tile the march crosses returns `invalid_loc ==
0`), `find_path` returns 0 for any goal within 4 world cells (Manhattan, on
the cell grid `div_3_table[v >> 8]`) and 1 beyond. That is the whole reason a
near move never touches `PathFinder`.

**The pull-back is not a detail, and it was the simulation's last unimplemented
line here (2026-08-26).** `while invalid_loc(goal tile, 0,0,0,0,0)` walks the
*goal* back toward the unit one `(sinx, cosx)` step at a time — each component
only ever clamped down to its own remainder, never re-aimed — and rewrites
`mo->waypoint` and the path stack's top wherever they are the goal, so the
pulled-back point is what the unit is really walking to from then on. It is
what puts a **woodcutter at the edge of the forest** rather than inside it:
forest refuses `invalid_loc`, so a citizen ordered at a forest tile is
silently re-aimed at the last open point short of it, and the march that
follows never enters a tile that would ask `go_around_building` for a detour.
Without it the sim marched into the forest, gave up, and paid `do_move`'s grid
draw (`docs/SYNC.md` §6, item 28) on a walk the original planned for free.
Two further exits belong to the same block and are now modelled: the goal
walking all the way back onto the unit sets `avoid = goal` and returns 1, and
the loop breaks when both components have been clamped to zero.

`crates/sim/src/orders.rs`, `Sim::find_path` — which now takes the move order
by `&mut` for exactly this reason, and the goal by value, because the
detour's recursive call passes a waypoint of its own rather than
`mo->dest_x/dest_y`. `go_around_building@005fc350`, which the pull-back
cannot stand in for, is §4.6.1 and landed 2026-08-26.

~~**The march as written above has a fixed point, and it hung the simulation
(2026-08-22).**~~ **Settled, 2026-08-23, by the pathfinder reading**
(`docs/PATHFINDER.md` §9): the fixed point was a mistranscription, twice
over. The original recomputes `find_angle` from the *current* remainder on
every iteration (the transcription hoisted it out of the loop), and clamps
each component to its axis' remainder (`|sinx| > |dx| → step_x = dx`), so an
axis that closes stays closed and the next angle points wholly along the
other axis. The exit tests are: loop only while `spd < manh`; return 0 when
`manh` grew; return 0 when the remainder is within one **actual clamped
step** on both axes (`<=`). With `spd >= 3` the dominant `sinx/cosx`
component is ≥ 2 after truncation, so a no-progress step is unreachable and
the original needs no guard. The citizen case that hung us — `(60, 1600)` at
`spd 25` — walks: y closes exactly, the angle re-aims due east, x closes,
return 0, no `find_wpath` draw. The sim's interim "return 1 on no progress"
guard (and its stated sync divergence) is retired; the march is now
transcribed as above, and the soak that found the hang stands guard over the
rewrite. `go_around_building@005fc350` is §4.6.1; its trigger — an invalid
tile on the straight line — open ground never has.

### 4.6.1 `Unit::go_around_building@005fc350` — the detour, and how `find_path` accepts it

**Established 2026-08-26 from the decompile, and confirmed against the
original's own path stack**: run20's unit `1/1` at frame 1 ends with
`[{(41640, 39384), tol 0, flags 1}, {(40644, 39036), tol 0, flags 0}]`, and
the simulation now produces those two entries exactly — coordinates,
tolerances and flags. The second is this function's output, and the `36`
that separates `40644` from the tile centre `40608` is the `off % 0xc0 / 2`
skew below, so a detour placed at the tile centre fails that check.
Confidence: **high** for the geometry and the pushes, which the dump pins;
**medium** for the diagonal case and the give-up exits, which no capture on
disk reaches. It supersedes the first reading's "medium — geometry" (§0).

`go_around_building(last good point, ang, spd)` is called from the march
(§4.6) when the next step would enter an invalid tile and
`path_recursion < 10`. It returns nothing: **`path_recursion == 10` on
return is the refusal**, and the caller reads it.

It **recomputes the blocked point itself** as `last + (sinx(ang, spd),
−cosx(ang, spd))` — *unclamped*, where the march clamps each component to
its remainder, so the two need not be the same point. That is the
original's, not a slip.

The step that failed crossed a tile edge, so the unit has a **lane** (the
row or column it is still standing in, from `last`'s tile) and a **blocked
lane** (the one the blocked point is in). Which axis the walk runs along:

- `ltx == btx` and `lty != bty` (a vertical crossing) → walk **horizontally**,
  `step = (sign(sx), 0)`.
- `ltx != btx` and `lty == bty` (a horizontal crossing) → walk **vertically**,
  `step = (0, sign(sy))`.
- Both differ (a diagonal). `|Δtx| + |Δty| > 2` gives up. Otherwise, if the
  corner `(ltx, bty)` is clear the walk is vertical; if it refuses but
  `(btx, lty)` is clear the walk is horizontal; if both refuse and
  `spd > 1`, **recurse with `spd − 1`** — a shorter step may land
  orthogonally instead — and at `spd == 1` fall back on the dominant
  component, `|sx| < |sy|` choosing the vertical walk.

Then the same walk runs **both ways** from the blocked point, one tile
(`0xc0`) at a time. At each offset it asks two tiles: `A`, the unit's own
lane at that offset, and `B`, the blocked lane at it. Off the map is that
direction's failure. `A` refusing stops the walk, and the direction
succeeds only if `B` is clear there; otherwise the walk continues while `B`
refuses and stops successfully the moment it does not.

Of the two results: the one that succeeded, or, if both did, the one whose
Manhattan distance to `mo->dest_x/dest_y` is smaller — forward on a tie.
Either is discarded in favour of the other if it resolves to the **unit's
own tile centre**. Neither succeeding gives up, and so does a winner more
than `0x300` from `last` in either axis.

The pushes, bottom first, are one or three:

- Only when the unit's own lane **refuses at the found offset**: the
  **turn-in point**, the found tile in the blocked lane; then a
  **`flags 8` midpoint**, the mean of the turn-in centre and the centre of
  the tile one step *back* along the walk in the unit's own lane, biased
  one unit down on each axis where the turn-in centre is the larger.
- Always: **the target**, in the unit's own lane — at the found offset when
  the lane was clear there, one tile back along the walk when the turn-in
  pair was pushed.

Every one of them is placed at `tile*0xc0 + 0x30 + (off % 0xc0) / 2` rather
than the tile centre `tile*0xc0 + 0x60`, from the order's own `off_x/off_y`
(§4.1) — so a formation's units do not all aim at the same point.

Giving up is `mo->dest = 0; unit_masks &= ~8; path_recursion = 10`.

**What `find_path` does with what was pushed**, and it is half the mechanic.
The stack length is taken before the call. On return with
`path_recursion != 10` the pushed entries are lifted off one at a time into
a scratch stack; an entry that lands exactly on the unit is **dropped** and
the next one down becomes the candidate. Six degenerate cases then refuse
without recursing: the candidate or the entry below it being the unit's
position or `mo->last_x/last_y`, the unit already standing where it last
planned, and a candidate equal to the (pulled-back) goal.

Otherwise **`find_path` recurses on the candidate**. A non-zero return
refuses — and the lifted entries are simply discarded, so the stack is back
where it started and `do_move` pays the grid draw. A zero return, with
`unit_masks & 8` still clear (an inner detour may already have set it),
puts them back in their original order, each entry's **`flags & 4`
recomputed against the one that will sit above it**: set when exactly one
of the two tiles is ocean (`(mask & 0x30) == 0x20`), cleared otherwise, so
a leg crossing the shore turns in place before it walks. Then
`mo->last = the unit's position`, the new top gets the same treatment
against the unit's own tile, and `unit_masks |= 8`.

`crates/sim/src/orders.rs`: `Sim::go_around_building`, `edge_walk`,
`detour_mid`, `detour_gave_up`, `detour_verified`, `shore_flagged`. The
checks are `diff::tests::run20_s_frame_1_spends_the_farm_s_ambience_pair`
(the original's own stack, entry for entry) and, without the install,
`cities_tests::a_line_that_clips_a_building_detours_instead_of_re_planning`
and `a_wall_with_no_way_round_gives_up_and_pays_the_grid_draw`. Both were
made to fail on purpose before landing — the first by neutering the
function to give up, the second by pinning the skew at `0x30`, which moves
the run20 waypoint from `(40644, 39036)` to `(40608, 39072)`.

**What it does not establish.** No capture on disk reaches the diagonal
case, the `spd − 1` recursion, the turn-in pair (run20's walk finds its own
lane clear), the `0x300` cut-off, or the shore's `flags & 4` — those are
read, not diffed. The check that would settle the first three is a
`UNITS=3` window on a game with a unit ordered diagonally into a building
corner; the shore one wants a walk along a coast.

**The pathfinder's interface** (the internals are the next mechanic). All
entries share the global `pathfinder` (`PathFinderData`: `pathing_unit,
sx/sy` = the unit's tile, `offx/offy`, `army, iroquois, worker, limit, saving,
scouting`) and take the unit's own `Stack<PathData>`: **they pop the goal off
its top and push back what the unit should walk, top first**; the return is
the stack length, `0` for "no path" (the goal is popped and not returned
unless final — then `−(flags & 1)` in `find_wpath`'s A\* case), `−1` for "goal
off the map" (stack emptied).

| entry | grid / `astar_path` step | before A\* |
|---|---|---|
| `find_wpath(Stack*, x, y, who, o)@00688fc0` (4-arg wrapper `@00688e10` from the unit's position) | world cells, `0x300` | same cell or flyer → push the goal back, return length. Else (leaders without flag 4: walk from the goal toward the start in `0x180`/`0x30` steps until `get_tregion` matches and, for sea, `invalid_loc(…,0,1,1,1,0)` is clear; flag-4 leaders: a `was_seen` fog walk) → push goal; **cell-Manhattan < 3 → return length**. Else push `{goal cell centre, tol 0x180, flags 0}`, `{start cell centre, tol 0}`, set `scouting`/`army`/`worker`; `astar_path(0x300, 0)`; 0 → pop, return `−(flags & 1)`; else length. |
| `find_tpath(Stack*, x, y, who, o)@006897d0` (wrapper `@00688e60`) | tiles, `0xc0` | same tile or flyer → push goal (tol 0), return. Walk from the goal toward the start until `get_tregion` matches and `valid_tcoord`; within `0x60` on both axes without that and not final → 0. Tile-Manhattan < 2 → push goal, return; else push goal, `{goal tile centre, tol 0x60 (0x180 kept), flags 0}`, `{start tile centre, tol 0}`; `offx/offy = mo->off % 0xc0 − 0x60`; `astar_path(0xc0, 0)`; on failure pop a non-final top. |
| `find_upath(Stack*, x, y, who, o, anti)@00682f30` (wrapper `@00688eb0` sets `limit = 500 / repaths[who]²`, halved with `anti`) | 48-unit cells, `0x30` | same 48-cell → push goal, return; walk on the 48-grid with `get_tregion` + `valid_ucoord`; within `0x18` → 0 unless final; 48-Manhattan < 2 → push goal; else goal, goal 48-centre, start 48-centre; `astar_path(0x30, anti)`; on failure pop a non-final top **and kill the unit's current order** unless a move with `retry != 0`; on success with more than three entries drop a top equal to the unit's position and compact collinear `flags & 2` side-steps. |
| `find_upath_restore(Stack*, who, o)@00688f40` | `saving = 1`, `limit = 300 / repaths²` | resumes the suspended search |
| `find_wpath_army@00683730`, `find_road@00688a40` | | the group's; the caravan's |

`repaths[who]` (`GameDaemon`) is a per-player pressure counter: halved each
frame (to 0 below 3), incremented per pathfinder call under collision, and it
shrinks `find_upath`'s node limit quadratically.

**What a straight-line stand-in must return to be faithful on open flat
ground.** For a goal in the unit's own world cell or within two cells
(cell-Manhattan < 3), `find_wpath` leaves the stack `[goal{flags 1, tol 0}]`
and returns 1 — a stub that does exactly that is *exactly* right, and since
`find_path` then verifies the line and takes over, the pathfinder is not
consulted again. Beyond two cells it pushes `{goal-cell centre, tol 0x180}`
and whatever `astar_path` adds between the start-cell centre and it — on open
ground presumably the chain of cell centres, each with tolerance `0x180`,
which the unit reaches by passing within 384 Manhattan of each; a stub that
returns `[goal]` makes the unit walk the straight line instead, which is the
same path only when the line passes within 384 of every centre A\* would pick
(it does not in general — the chain hugs the grid — and even the first step
differs, since the first leg is toward the *next cell centre*, not the goal).
The stub must still leave the **`Random::get` draw** in place — it happens on
the `do_move` side whenever `find_path` returns 1, pathfinder or not. And it
gets wrong the `find_tpath` refinement on the nearer branch (tile centres,
tolerance `0x60`), the `find_upath` branch under collision, `tregion`
crossings (flag 4), and the `−1`/kill exits.

### 4.7 Collision — see `docs/COLLISION.md`

`detect_unit_collision@00617060` and `resolve_unit_collision@005f9d30` have
their own document. Three call sites touch this one: `move_step`'s per-step
probe (§4.5, `COLLISION` §5), `do_move`'s waypoint test (§4.4, `COLLISION`
§5.1), and the recovery, which only ever *pushes* waypoints and writes
`dest`/`dest_x/y`/`pause` — the next `do_move` handles the rest.

### 4.8 The worked example — the dump's citizen

A citizen ordered to a point on open ground, 25 position units a frame
(`gamelog-run4`, citizen `who 0 o 1`). At the end of frame 3 it stands at
`(4248, 28680)` with `[MOVE_TO x 4008 y 28296 dest 0 flags 0]` queued in front
of its gather order, an empty path stack, `idle 1`, `dest_angle` already the
move's angle and `angle` still `0x55555555`. At the end of frame 4 it is at
`(4234, 28659)` — a step of `(−14, −21)`, `vector_dist 25` — with `angle =
dest_angle = −381943808` (`= find_angle(−240, −384)`, turned instantly from a
standstill per `docs/MOVEMENT.md`), `dest 1`, `dest_x/y 4008/28296`, `UnitOrder
flags 1`, `unit_masks 0x1000000A`, `path_recursion 1`, and the stack
`[{4008, 28296, tol 0, flags 1}]`. The goal is 240 × 384 units away — in the
adjacent world cell: `find_wpath` returned with `[goal]`; `find_path` marched
the line in 25-unit steps, every tile valid, returned 0; `masks |= 8`; from
then on each frame is `do_move → move_step(goal, 25)`, a straight line at
`find_angle`'s quantised heading with the `sinx/cosx` components (`−14, −21`,
Manhattan 35 for a Euclidean 25), until `manh ≤ 25` snaps it onto `(4008,
28296)` exactly (frame 20 in the log); that frame pops the final entry,
applies `set_angle(mo->angle)` because the action is `GATHER`, clears `flags &
1`, kills the move — and the gather order is current again the same frame and
runs its own step the *next*. No `PathFinder` call, no RNG draw, no waypoint
but the goal. The move was queued by `do_gather` at the end of frame 3
(§6.3) — an order inserted inside the step runs the next frame — which is
the three quiet passes `docs/DATALAYER.md` saw.

---

## 4.9 The row, the jump, and what a diff now backs (2026-08-31)

**The whole `MOVEORDER` row is compared** (`OrderMismatch::Move` in
`rondata::diff`), field for field against every dumped move of both maps.
Until this item only `coll_x/coll_y` was, and the cost of that is the
lesson: East Indies' `1/4` was booked for three days as an order-list
*length* at frame 168, because the two sides had picked different farm
tiles on **167** and no comparison read the field that said so. §4.1's
table had been diff-backed since 2026-08-26 for run29's four states; this
is the same table asserted on every frame of a 1,850-frame capture.

Three of the row's fields are **reported and do not score**, each for a
stated reason, exactly as `UnitOrder::flags` is:

- `dest` — the original clears it on arrival, on a `go_around_building`
  failure, after `resolve_unit_collision` and on a collision at
  `coll_x/coll_y`. Three of the four are the pathfinder seam's own timing.
  `dest_x/dest_y` is compared on the frames `dest` says it is live, and the
  path stack scores outright.
- `last_x/last_y` — written only by a *successful detour*, the same seam.
- `facing` — the formation mirror, whose sign is an open question
  (`docs/GROUPS.md` §6.3).

**`goto STEP` lands past the pause check**, and it is worth stating as a
rule rather than leaving in §4.4's pseudocode. The straight-line check's
success jumps to `STEP`; only the re-plan's `TAKE` comes back through
`STEP_IF_MOVING`, and only a unit that *already* held `unit_masks & 8` at
entry reaches the check at all. So **a unit that re-verifies its line this
frame steps this frame, and its collision `pause` does not tick.**

run10's `1/2` is the worked example, and the dump carries every field of
it. Frame 572: `resolve_unit_collision` writes `pause 3`, `coll_x/coll_y
40752/17630` and `dest 0`, and the unit stands at `(40728, 17640)`. Frame
**573**: `dest` is 1 again, `pause` is **still 3**, and the unit has moved
to `(40728, 17615)` — the fresh waypoint was verified and stepped in the
same frame. Frames 574, 575, 576: `pause` 2, 1, 0 with the position
unchanged, the three still frames. 577 steps.

**`coll_x/coll_y` is written into the order by the probe, not by the
step**, and `move_step` goes on writing its own fields through the same
pointer afterwards — so a port that steps on a copy has to take the pair
back or its next store undoes the probe. The arm that shows it is the
blocked stand while a turn is still owed (`docs/COLLISION.md` §4.3, §5),
and it was worth Great Lakes' whole order score: 791 → **1374**, with the
tick score and every unit's parting frame unmoved (item 115).

Ticking the pause on 573 costs one frame for the rest of that walk, and it
was worth **209 frames of the headline**: Great Lakes' ticks 572 → 781 and
its player 1 573 → 782, East Indies' 536 → 1373 orders. Both maps' tick
scores now sit one frame past their own word parting, which is what a
capture that holds position for exactly as long as it holds the stream
looks like.

---

## 5. Build, repair, garrison — and what a citizen does next

### 5.1 The structures

`BuildOrder`, `RepairOrder`, `BoardOrder`, `AwaitBoardOrder` are bare
`TargetOrder`s (`ox, whom, uid`; `get_type` differs); `GarrisonOrder` adds
`search`. `uid` is the target's `ObjectData::uid` at order time (`0xffff` if
either index is negative) — the liveness key `work` step 8 compares.

| adder | callers → `(pos, action)` |
|---|---|
| `Unit::add_build_order@005e5210(o, who, pos, action)` | `Group::action_swarm_around` (`QUEUE_LAST`, passed through), `check_build_order` (`QUEUE_FIRST, 1`), `Wall::process`'s AI recruiter (`QUEUE_NEW, 0`), `come_out` (`QUEUE_LAST, 0`) |
| `add_repair_order@005e4ff0` | `Group::action_repair`, `action_swarm_around`, `come_out` (`QUEUE_LAST, 0`) |
| `add_garrison_order@005e4080(o, who, search, pos, action)` | `Group::action_garrison`, `do_garrison`'s redirect (`QUEUE_FIRST`, `search = 1`), `come_out` (`QUEUE_LAST, 0`, `search = 0`) |

### 5.2 `Unit::do_build(order)@005eebf0`

`T = objects[whom][ox]`:

```
1  o < 0 or who < 0 or T not alive:
       kill; if a next order exists (type != NONE): return
       build_done(o, who, UNIT_GATHER_RESPOND_RANGE × 192); return
2  T already active (finished before I arrived):
       kill; if order_type() != NONE: check_build_order(); return
       if not OILPLATFORM and (AI-controlled (unit_masks & 0x40000) or worker_stance ∉ {0,1}): build_done; return
       if who != my owner: build_done; return
       if not T.type.is_gather_building or T is UNIVERSITY: build_done; return
       add_gather_order(o, QUEUE_NEW, 0); deselect; group = −1; return
3  not adjacent_to(o, who) (§10) or (my tile is inside T's footprint and T is not a FARM):
       f = order.flags & 4; kill
       Group{me}.action_swarm_around(o, MY OWN who, QUEUE_FIRST, BUILD_AT, f); return   // §5.4: the walk
4  set_anim(FARM ? CHAR_SOW : CHAR_BUILD); face the site
   unit_masks & 1 → return
5  amount = ACCEL_CONSTRUCT (100); T under attack and not (Koreans with KOREAN_BUILD_UNDER_FIRE): amount /= 4
   done = T.wall.do_construct(amount)                                                    // docs/CITIES.md §3.3
   if not done: return
6  kill; rest = order_type()
   if rest != NONE:
       if not gather building or UNIVERSITY or group < 0: check_build_order(); return
       Group::normalize(group); if count(COUNT_PEASANTS) < 2: check_build_order(); return
   if (OILPLATFORM or (not AI and stance ∈ {0,1})) and same owner and gather building and not UNIVERSITY:
       add_gather_order(o, QUEUE_NEW, 0); deselect; group = −1; return                  // the finished farm gets its builder
   if rest == NONE: build_done(o, who, UNIT_GATHER_RESPOND_RANGE × 192); return
   check_build_order()
```

`worker_stance` = `UnitData::get_worker_stance@006109f0` → `stance`
(`UnitData+0xb1`) if the type has one, else 0; **1 and 2 build and repair, 0
and 1 gather** (1 is the normal citizen).

**Steps 2 and 6 are `not AI`, and this crate read them as stance alone.**
Both gather arms here — and §5.6's — are `OILPLATFORM or
(`unit_masks & 0x40000` clear and worker_stance ∈ {0,1})`: **only a human
builder adopts the site it has just finished.** The AI's goes back through
`build_done`, whose own arm (§5.5) searches afresh and need not pick that
site at all. What hid it for a month is in the journal. run10's
frame 167 is the case that tells the two apart: the AI's `1/1` finishes a
farm and the original sends it to the Woodcutter's Camp instead.
**Diff-backed 2026-08-27** (item 47) — the building, the tile `(212, 93)`
and `dist_mod 4` all agree on the frame.

**The walk is not the order's.** `BuildOrder` owns no `MoveOrder` and calls no
`go_to`: a non-adjacent `do_build` kills itself and `action_swarm_around`
re-queues `[ExploreTo(spot) → BuildOrder]` in front of everything (§5.4); the
move runs to completion by its own rule, then the build order is current
again and re-tests adjacency. A move that falls short swarms again.

### 5.3 `Unit::check_build_order@00603470` — the queue of sites after one finishes

Walks the queue **with a cursor** while the order is `BUILD_AT` or `MOVE_TO`.
A move is **stepped over, not removed** — `current_node = current_node->prev`
— and the walk stops on a move that carries the action bit or is the newest
node. (The first reading hedged this as "the decompile's loop variable is
stale in one branch; medium"; the hedge is wrong and is withdrawn. The
listing is unambiguous: `0x6036df test byte ptr [ecx+eax+4], 4; jne` then
`0x6036e6 cmp eax,[esi+0xdc]; je`, then the advance at `0x6036fd`, with no
call between. The two arms differ only in which node the advance starts
from — the cursor for a move, the re-tailed head after a kill — which is what
made the decompiler's locals look stale. R3 C2.) Because the loop's own gate
is `BUILD_AT || MOVE_TO`, a swarm's **`EXPLORE_TO` ends the scan**. Every `BUILD_AT` reached is popped (`repath; kill`) — its liveness test is
the **full** `target_exists`, `ox ≥ 0 && whom ≥ 0 && obj->flags & 1 &&
obj->uid == order.uid`, the `uid` included, unlike `do_build`'s (R3 C3) — and
if the target still exists and is not active its `o` is collected. Collection
stops after a site that is a city or answers two unresolved type slots, and
**the site it stops on does not get killed**: the `break` jumps out before
the `repath; kill`, so that site ends the pass with its original order still
queued *and* a fresh one re-added — a duplicate the next `do_build` disposes
of through its "already finished" branch (R3 C5, a finding neither reading
made). Then: one site → `add_build_order(site, QUEUE_FIRST,
1)`; several → `Objects::find_units(SEARCH_FRIENDLY, UNIT_BUILD_RESPOND_RANGE ×
192, FILTER_BUILDREPAIR)`, count per site the friendly units whose action is
`BUILD_AT` on it — **excluding the counting unit itself** (`:132`
`this_00->o != this->o`; `find_build_spot`'s otherwise identical count does
*not* exclude self — R3 C6, F2) — swap the least-crowded site to the front, re-add all with
`QUEUE_FIRST, 1` in reverse — least-crowded first, then original order, all
with the action bit, all ahead of whatever else was queued.

### 5.4 `Group::action_swarm_around(o, who, pos, BUILD_AT|REPAIR|GATHER, action)@0070fbe0`

Per member (land and sea in two passes; **an air member gets nothing at
all** — not a plain move, as the first reading had it): skip a busy member;
**a carrier of peasants is kept, not skipped** (the first reading inverted
this); and — for a gather building under `BUILD_AT` — one already gathering
is skipped **only when `spare != 0`**. The legal `what` values are
`BUILD_AT` and `REPAIR` **only**: a `GATHER` reaches
`Error::report("ILLEGAL SWARM AROUND ORDER")`. All four are the second
reading's (`docs/audit/2026-08-21-orders.md` R3 S3–S5, S8), and each was a
clause the first reading had backwards. `R = min(x_size, y_size) × 0x60 + 0x30` (the footprint
from `BuildType +0x234/+0x238`; a dead target uses the unit type's), **halved
for a FARM** under `BUILD_AT`; `find_nearby_spot(site, R, …, FILTER_NOT_ME)`
(§10) → a free spot around the site, and for `BUILD_AT` the spot is pushed
`0x30` further from the site's centre on each axis and re-validated with
radius 0; `add_move_facing_order(spot, angle, mode = BUILD_AT ? 1 | (human ? 0
: 2) : 1, pathed 0, pos, action…)` — in the dump the order this creates is an
**`EXPLORETOORDER`** (type 3). **That `angle` is `find_angle(site − spot)`,
from the spot the ring returned and *before* the `BUILD_AT` nudge** — the
builder arrives facing what it will build — and **not** the ring's own sweep
bearing, `find_angle(unit − site)`. The decompiler prints both calls with the
same locals (the pair travels in `ecx`/`edx`); the listing settles it
(`7103f3`, `710415`, `71021a`). Diffed, §4.9; for `pos == QUEUE_LAST` on a member whose
action is `GATHER` with `action != 0`, the pos becomes `QUEUE_NEW` (the gather
is pre-empted); then `add_build_order(o, who, QUEUE_LAST, action)` (or
`add_repair_order`), and for `BUILD_AT` the site's `build_masks &= ~0x2000`.

Four additions from the second reading (R3 S1, S6, S9, S10): a `QUEUE_FIRST`
swarm is a **re-entry that halts the group first**; a barge or a carried unit
has its footprint substituted; the scratch group closes with an
`action_move_to`; and the members' spots deconflict **through
`find_nearby_spot`'s own occupancy test**, not through any geometric
spreading — which is why two builders never need a formation.

### 5.5 `Unit::build_done(o, who, range)@00603bf0` — what a builder does next

```
if length > 1: return                                     # more orders queued: they run
AI (unit_masks & 0x40000): find_build_spot() or find_repair_spot() or (starting_resources != 8 and find_gather_spot(range)); return
stance ∈ {1,2} and find_build_spot(): return
stance ∈ {0,1}: if find_gather_spot(UNIT_GATHER_RESPOND_RANGE × 192): group = −1; deselect; return
stance ∈ {1,2} and find_repair_spot(): return
if o ≥ 0 and who == me and T is a gather building, not UNIVERSITY, not OILPLATFORM, and num_gatherers(T) < gather_max:
    add_gather_order(o, QUEUE_NEW, 0)
```

**`find_build_spot@00603e20`** (§5.10 is the two searches it makes): range
`UNIT_BUILD_RESPOND_RANGE × 192`, `× 384` on `get_worker_stance` 1 or 2 —
a type with no worker stance type reads 0 and never doubles;
`find_builds(SEARCH_FRIENDLY, range, 0x200, FILTER_CONSTRUCT)`; keep own
sites whose `build_masks & 0x20` is clear; if any: count the builders on
each from a second search, `find_units(…, range, 0x200,
FILTER_BUILDREPAIR)`, keeping a unit whose **action** is a `BUILD_AT` on
one of them and counting it once (the scan `break`s on its first match);
take the fewest — ties → first, the min-search being a strict `<` up from
index 0 — and `swarm_around(site, QUEUE_LAST, BUILD_AT, 0)`; return 1.

`find_repair_spot@00604320` is §5.10 too; `find_gather_spot` is §6.6.

### 5.6 `Unit::do_repair(order)@005ee420`

`set_anim(CHAR_REPAIR)`; then:

```
T.damage == 0 → kill; not AI: (same owner, no next order, stance < 2, T a gather building, not UNIVERSITY*) → add_gather_order(o, QUEUE_NEW, 0); AI: find_repair_spot(); return
who != me and not mutual allies → as above
not a building → kill;  not active (unfinished) → kill
under attack and not (order.flags & 4) → kill;  tile territory owner ∉ {nobody, who, ally} → kill
not adjacent_to → f = flags & 4; kill; Group{me}.action_swarm_around(o, who, QUEUE_FIRST, REPAIR, f); return
unit_masks & 1 → return
period, helpers++, amount, the price, T.repair_damage(amount)        — exactly docs/CITIES.md §9.3
```

(\*the `is(…)` literal is lost in the decompile; UNIVERSITY by analogy with
`do_build` — one listing look.)

### 5.7 `Unit::do_garrison(order)@005e6b80`, `kill_garrison_order`

The gates are `docs/CITIES.md` §6.4 verbatim. The order-side facts it does not
state: the helicopter/AIRBASE branch kills the garrison order and adds a
strafe (`QUEUE_FIRST`) or, if the airbase is full, a `MoveOrder` to it.
**Not adjacent**: `add_move_order(spot, 1, 0, QUEUE_FIRST, 0, …)` with the spot
from `find_nearby_spot` on the ring `min(x_size, y_size) × 0x60 + 0x30` (a
DOCK: `0x1b0..0x330`), retried relaxed; **the garrison order stays in the
list behind the move** — no swarm, no re-add; no spot → kill. **Admitted**:
`go_inside(captain, o, who, 0)`, then `kill_garrison_order(captain, 0)@
005e2bd0` — from the captain down the `o_down` chain, each unit whose action
is `GARRISON` is `repath`ed and killed. **Full**: with `search != 0` and a city
whose `city_flags & 1` is set, `find_garrison_build(city, who)@00605040` (the
city's building chain, active, with room for `num_inside + control_cost`,
`can_garrison`, nearest by `vector_dist`) → kill and `add_garrison_order(b,
who, search = 1, QUEUE_FIRST, action)`; else kill.

### 5.8 `go_inside`, and `come_out`'s rally orders

`Unit::go_inside(o, who, chain)@0061a2e0`: climb to the captain; `Object::
insert_inside`; deselect; **if the container `is_unit` and `uber_size > 1`: a
`QUEUE_NEW`-style clear of the orders** — a squad boarding a *transport* drops
its orders; `docs/CITIES.md` §6.5 says "entering a building", and the gate is
the container being a unit (the vslot at `+0x18`; a `Build`'s is the folded
`return 0`). Followers recurse with `chain = 1`.

`Unit::come_out(chain)@00617c10`: placement per `docs/CITIES.md` §6.5 — for a
unit inside nothing (the starting citizens, §9), `find_nearby_spot` around its
own position with `min = block_radius` (`ObjectTypeData+0x240`), `max =
block_radius + UNIT_DISEMBARK_DISTANCE`, `FILTER_NOT_ME`, and `set_new_location`
there: **the citizen steps off the building's footprint to the nearest free
spot beside it** (§10 for the candidate order). An AI unit that is not a
citizen/scholar/plane then clears its orders. Then the **rally point** (the
building's `GatherPointList`, §6.1 — `action 3` = an object `(o, who)`, else a
point): for each point, `B = find_any_building_at(tile, FILTER_SEEN)`:

- a **citizen** and `B` a building: not active and same owner →
  `add_build_order(B, QUEUE_LAST, 0)`; `damage == 0`, an object-kind point, `B`
  active, a gather building, not UNIVERSITY, same owner →
  `add_gather_order(B, QUEUE_LAST, 1)`; damaged → `add_repair_order(B,
  QUEUE_LAST, 0)`;
- a **scholar**: object-kind, `B` active, a gather building and UNIVERSITY,
  same owner → `add_gather_order(B, QUEUE_LAST, 1)`;
- a **caravan** and `B.is_trade` and not enemy → `add_trade_order`;
- otherwise `B` alive, a building, active, ally, `garrison_limit > 0`,
  `can_garrison`, object-kind → `add_garrison_order(B, 0, QUEUE_LAST, 0)` for
  me and my `o_down` chain;
- an enemy `B` and I can attack → attack orders (`QUEUE_LAST, 1, 1`) for me
  and the chain; a scout → a move; a type that cannot attack it →
  `ATTACK_TO`/a move;
- no building: an enemy unit at the point → attack; else `MOVE_TO`, or
  `ATTACK_TO` when the tile holds a `0x1ab`/`0x1ac`/DOCK type.

A point that yields no task becomes a `move_facing_order(QUEUE_LAST)` and the
loop continues — the waypoint chain. At the end an AI unit is `add_to_army`
on a frame chosen by `game_random`; a land citizen leaving an OILPLATFORM is
put into a fresh `TRANSPORTBARGE` first.

### 5.9 The idle citizen — `think_peasant`

`Unit::think@005f6e40` → `is_worker` → **`think_peasant(0)@005f5760`**:

```
T = AI ? 1 : by LeaderOptions[who] +0x8: 1→7, 2→12, 3→17, 4→32, 5→62, else 2       # the per-player idle-citizen option
if idle < T: return 0;  if idle != T and (idle − 2) % 5 != 0: return 0
AI (0x40000) and TypeIndex 0x32/0x33 — the base type, so no scholar: think_civilian_transport(1); 1 → return 1
not a scholar and (unit_masks & 0x400 or stance ∈ {1,2}) and find_build_spot(): return 1
stance ∈ {0,1} (or no stance type and 0): find_gather_spot(AI ? −1 : UNIT_GATHER_RESPOND_RANGE × 192) → deselect; human: group = −1; return 1
not a scholar: human with (0x400 or stance ∈ {1,2}) and find_repair_spot(): return 1
unit_masks &= ~0x400
AI: SCOUT §11.1's tail; find_repair_spot()
```

With the default option the gate passes at `idle == 2` — the third idle frame
— then every fifth increment (80 frames, phased by `o`). `think_peasant(1)`
from a `THINK` order has no gate. An AI-controlled citizen (`unit_masks &
0x40000`) uses `T = 1` and an unlimited gather range; both landed
2026-08-30, `docs/SYNC.md` §3.16. The colonist line landed
2026-09-01, word 3978 (`docs/TRANSPORT.md` §7, SYNC §3.23).

`Wall::process@00640450`: every 32 frames phased by `o` a
site that is not active and whose owner is not human wants `max(4, helpers)`
builders (more for a wonder) and recruits with `find_unit(SEARCH_FRIENDLY,
0xf00, FILTER_TYPE PEASANTS, FILTER_NOT_BUSY)` → `add_build_order(site,
QUEUE_NEW, 0)`. The same function resets `helpers` and `build_masks & 0x800`
every frame (`docs/CITIES.md` §3.3).

---

## 5.10 The searches a builder makes — `find_builds`, `find_units`, `find_repair_spot`

**The two searches, read 2026-09-03** (item 194).
`Objects::find_builds@0065a120` and `Objects::find_units@0065a620` each have
two paths and pick between them on cost: `n = (range + 0x2ff) / 0x300` — the
range in **cells**, rounded up, a cell being four tiles — and the circle path
runs while `circle_radius[n]` is no more than `game->num_def_builds` for the
one and `game->total_units` for the other. `num_def_builds` is a flat **200**
(`Game::init_data@0058dca0`, beside `num_def_units = 200`; it is the
per-player object-array size, which is why a building's `o` is `2000 + i`),
and `total_units` is the live count `Unit::init`/`Unit::close` keep. So the
build search is always the circle for any range a citizen uses, and the unit
search is not: `circle_radius[3]` is 45 and `circle_radius[6]` is 145, against
59 live units on run69's frame 2803.

The circle path walks `circle_x`/`circle_y` out to `circle_radius[n]` around
the searcher's own cell, takes each cell's object chain (`+8`/`+10` the head's
`(o, who)`, then `+0x2c`/`+0x2e` — the same `down`/`down_who` the dump
prints), and applies `Search::valid_search` and `Search::valid_filter`. **It
has no distance test at all**; the list path, which walks the per-leader
object arrays instead, has `vector_dist <= range`. The `0x200` flag is the
region gate — the cell's `+4` against the query point's — and on the list path
`find_units` computes the query cell as `div_3_table[pos >> 6]`, a **tile**
coordinate indexed into the cell grid, where every other site uses `>> 8`.
That is the original's own arithmetic and is not reproduced here.

`FILTER_CONSTRUCT` is arm 5 of `Search::valid_filter@0067dbb0`'s jump table
(the index is `filter − FILTER_TYPE`; the table is at `0067e57c` and the arm
at `0067dd54`, read from the PE because the decompiler prints the dispatch as
an indirect jump): `vtable+0xc` on the object, then `vtable+0x40` for what
`vtable+0x4c` is asked, **negated** — an object that is not active, which for
a building is a site still under construction. The polarity is settled by arm
6 next door, `FILTER_DAMAGED`, which is the same pair un-negated plus
`+0x24 damage != 0`. The `FilterIndex` enum is the PDB's:
`FILTER_ALL = 0`, `FILTER_TYPE = 1`, … `FILTER_CONSTRUCT = 6`,
`FILTER_DAMAGED = 7`, … `FILTER_BUILDREPAIR = 10`, … `NUM_FILTER = 24`.

**`find_repair_spot@00604320`**: a human
(`leader_flags & 4` set — `LeaderData::is_human@006ec170` is literally
`return leader_flags & 4`, which also settles §14's flag question) always
searches, at `UNIT_BUILD_RESPOND_RANGE × 192`, doubled to `× 384` on worker
stance 1 or 2; an AI searches at `× 192` flat and **only when its effective
difficulty ≥ 2** (per-leader in network mode, the lobby's otherwise), else
return 0 — the gate and the branch shape per audit R3 F3, third pass
2026-08-23. Then `find_any_building(SEARCH_FRIENDLY, range, FILTER_DAMAGED,
FILTER_NOT_UNDER_ATTACK)` whose tile's territory owner is nobody, me or a
mutual ally; `swarm_around(b, QUEUE_LAST, REPAIR, 0)`.

### 5.10 `UnitData::stance` — the byte that means four things

`UnitData +0xb1` is one signed byte and §5.5, §5.9 and §6.6 all gate on it,
but it is not one quantity: **what it indexes depends on the type carrying
it**. `GroupData::get_stance_option@0070bab0` sizes the option array from the
same enum — six options for a combat unit, four for a worker, two each for a
caster and a packer — so a stance is always an index into a list whose length
its type fixes, and comparing the byte across two types is meaningless.

`StanceTypes` is the PDB's enum, whole:

```
STANCE_COMBAT = 0   STANCE_WORKER = 1   STANCE_CASTER = 2
STANCE_PACKER = 3   NUM_STANCE_TYPES = 4   STANCE_NONE = -1
```

**Which kind a type carries** — `UnitTypeData::get_stance_type@0061d350`.
The order is the whole of it, and it is not the order the names suggest:

```
role & 0x10000 (military):  unit_flags2 & 4 ? PACKER : COMBAT
TypeIndex 0x32..0x35:       WORKER          # the citizen/scholar four
(unit_flags2 & 6) == 2:     CASTER
otherwise:                  NONE
```

The military arm's return is written as arithmetic —
`-(uint)((unit_flags2 & 4) != 0) & STANCE_PACKER` — which is a **mask, not a
conditional**: a military type that does not pack falls out at `0`,
`STANCE_COMBAT`, and never reaches `NONE`. And because the military test is
first, the caster arm only ever sees civilians.

**The byte a new unit is born with** — `Unit::init@00612100:282–309`:

```
COMBAT:  leader_options[who].buildings
WORKER:  (leader_flags & 4) == 0 ? (game.info.starting_resources == 8) + 1
                                 : leader_options[who].peasants
CASTER:  ~(leader_options[who].flags[0] >> 4) & 1
PACKER:  ~(leader_options[who].flags[0] >> 3) & 1
NONE:    0
```

`leader_flags & 4` is `LeaderData::is_human@006ec170`, which this document
already leans on above. So the worker arm is **inverted from what the name
suggests**: the human takes their own `peasants` option and the *AI* takes
the lobby's `(starting_resources == 8) + 1`. `Unit::init` reads
`game->info.starting_resources` raw and not `get_starting_resources(who)`, so
the asymmetric-teams row under `GAME_RULES == 8` never reaches it.

**`LeaderOption`** is `LeaderOptions::list[who]`, 0x20 bytes, ten slots, and
`LeaderOptions::log_data@006f1480` prints four of its fields under the
executable's own names (`0xae1628`, `0xae1630`, `0xae156c`, `0xae1668`, all
UTF-16): `who +0x0`, `peasants +0x4`, `peasants_wait +0x8` (the idle option
this document tabulates below), `buildings +0xc`, then a `BitMask` whose
inline storage is `+0x1c`. `LeaderOptions::init@006f1d40` lays down
`peasants 0`, `peasants_wait 2`, `buildings 0`, and a mask cleared and then
given **bits 1 and 3**. Bit 4 stays clear, which is why a caster is born at 1
and a packer at 0 out of two adjacent bits of one byte. The only writers are
`CommandPackage::process_leader_options@009441d0` — a player's click — and
`ScenarioFuncSet::set_auto_peasant_level@009ff620`, which bounds the value to
`0..3`, writes `peasants`, and pushes it onto every `0x32`/`0x33` unit and
every `WORKER`-kind building the player already owns.

**And then the trainer overwrites it** — `Build::train@0062f9b0:86–101`:

```
if get_stance_type(new unit) == get_stance_type(this building):
    Unit::set_stance(new unit, this->stance, 0)
```

A building carries its own stance at `Build +0x7e`, from
`Build::init@00629740:92–113`, which is the switch above with two arms
rewritten:

```
WORKER:  ((leader_flags & 4) == 0 && starting_resources == 8) ? 2
                                                             : leader_options[who].peasants
CASTER:  1                       # flat, not the complemented bit
```

`BuildTypeData::get_stance_type@006396c0` gives the building's kind: a
building that trains nothing is `NONE`; a city is `WORKER`, a fort `CASTER`,
`SIEGEFACTORY`/`FACTORY` (`0x1ae`/`0x1af`) `PACKER`, `MARKET`/`UNIVERSITY`/
`AIRBASE` (`0x1b4`/`0x1a4`/`0x1bf`) and `MISSILESILO` (`0x208`) `NONE`, and
every other trainer `COMBAT`. The city and fort tests walk `FROM`
(`is_city` = `is(VILLAGE, 0)`, `is_fort` = `is(FORTX, 0)`); the five type
indices are tested by **identity** and do not.

**That one difference between the two worker arms is the whole finding.** An
AI's *starting* citizens go through `Unit::init` alone and are born at **1**;
every citizen it *trains* comes out of a city, whose kind is also `WORKER`,
and takes the city's **0**. A human's are 0 either way. Two captures print
exactly that, on two maps:

| block | `0/1..0/5` | `1/1..1/5` | `0/0`, `1/0` | `who 8` |
| --- | --- | --- | --- | --- |
| run68 and run69, frame 0 | 0 | **1** | 1 | 0 |
| run68, frame 6595 (`1/6..1/15` trained) | 0 | 1 | 1 | 0 → and the trained ones **0** |

The scouts at `0/0` and `1/0` are the caster arm (`TypeIndex 0x45`,
`unit_flags2 & 6 == 2`) and gaia's objects are the `NONE` arm. run69's leader
flags say which player is which: leader 0 is `176160775` (`…111`, bit 2 set)
and leader 1 `176160787` (`…10011`, clear).

**Coverage.** The unit and building switches, the two stance-type functions
and `Build::train`'s override are **diff-backed**: `init_stance` is asserted
against every unit of both maps' first blocks
(`init_stance_is_the_original_s_on_both_maps_first_blocks`) and the trained
half against run68's 135-block window, where `stance` is now one of the
compared fields. Not established: what `buildings +0xc` is named for, since
`Unit::init` reads it for a **combat unit**; which option bit is which beyond
3 and 4; the `starting_resources == 8` arms of both switches, which no lobby
on disk reaches; and the two writers, neither of which is modelled — give the
sim either one and `Sim::build_stance`'s computed byte must become a stored
field.

---

## 6. The gather order

### 6.1 A registration, not a carry loop

Rise of Nations has no gatherer carrying a load back. Income is paid by the
economy from the building's **gatherer chain**: `BuildData+0x70 gather_down`
(the head, −1 empty) → `UnitData+0x92 gather_down` → … . `Unit::add_gather_
order` pushes the unit onto that chain **the moment the order is issued**,
`kill_current_order` pops it when the order dies, and the economy counts the
chain — `BuildData::calc_gather@0062d360` calls **`num_gatherers(this, 1, 1)`**,
which is: `count_inside(PEASANTS)` for an OILPLATFORM and `count_inside(
SCHOLARS)` for a UNIVERSITY (garrisoned gatherers), plus every chain member for
which `is_gathering_at(this, arrived = 1)` holds — a citizen/scholar on the
map whose **action** is a `GATHER` on this building **with `been_there`
set** — skipping decoys (`unit_masks & 1`).

**`is_gathering_at@00608880` matches on `get_action()`** — §3.3's walk —
not on the front of the list, and it is load-bearing. `do_gather` and `do_non_flat_gather` insert their walks as
`QUEUE_FIRST` moves *without* the action bit, so during every walk-out and
walk-back leg the front order is a `MOVE` and only `get_action` still finds
the `GATHER`. Under the "first order" shorthand a woodcutter would stop
counting the moment it set off — and be **pruned out of its own chain by
`check_gatherers`**. Two further details: the
inside-the-building match is **scholar-only** (`ptype[4] ∈ {0x34, 0x35}`;
G17), and `num_gatherers`' second argument is both the decoy filter **and** a
`count_inside` mode selector, `COUNT_TYPE + 2` (G13). This is `Site::gatherers`' real
source in `docs/ECONOMY.md`; the count is **`|{u in chain : u.first_order is
GATHER(this) and u.been_there and not a decoy}| + inside(PEASANTS on a
platform / SCHOLARS in a university)`**. ~~Every UI/AI caller uses
`num_gatherers(0, 0)`.~~ — **corrected 2026-09-03 (196): all but
`Animal::think_farm_animal`, which asks `(1, 0)`** (`docs/SYNC.md` §3.6).

The chain: `Build::add_gatherer@0062f640(o, who)` — same owner;
`num_gatherers(0,0) < gather_max` (`BuildData+0x80`); the unit exists, is on
the map, is one of `PEASANTS 0x32, PEASANTSKOREAN 0x33, SCHOLARS 0x34,
SCHOLARSKOREAN 0x35` — four **literal** type-index comparisons, not a lineage
`is`, so a modded citizen-alike outside those four indices can never join a
chain (R4 G9) — not already in the chain → `check_gatherers()` then
**push-front**. The capacity test runs **before** the prune, so a chain full
of stale entries refuses a new gatherer until something else prunes it
(R4 G10). `remove_gatherer@0062f8d0(o)`: unlink (walking stops at the
first dead unit); its second parameter is unread, and unlike
`check_gatherers` it clears the removed unit's `gather_down` in **both**
arms — `check_gatherers` clears it only on a head drop, so
`UnitData::gather_down` is not a reliable "am I registered" field; only the
walk is (R4 G11, G12). `check_gatherers@0062f710`: prune every member that is dead,
not `is_gathering_at(this)` or off the map. `is_gathered_by(o)`: membership.
`all_gathering@0062f570`: every member's first order is a gather with
`goto_build == 0` and `wait ≥ 0` (all out at their tiles); an empty chain is
true. `gather_max` comes from `BuildTypeData::max_gatherers@0063c430` — **1 for
a flat type** (`build_flags & 0x10000000`), else `calc_gather`'s slot output
(`docs/ECONOMY.md`'s open item).

`gather_from` (`BuildData+0x98`, a `MiningList`) is filled once, at building
creation, by `Build::find_gather_tiles@00623350` — and **shuffled off the
sync stream** as it is. Specified, with its draw count, in
`docs/ECONOMY.md`, "The gather list, and its shuffle";
`do_non_flat_gather` ranks tiles by `i >> 2`, so that ordering is the order
the ground is worked in.

**`GatherPoint`/`GatherPointList` (`BuildData+0xb8`) are the rally point**, not
gathering slots: `GatherPoint {x, y, action}`, `action 3` = an object `(o,
who)`; written by `Group::action_gather_point`, `action_city_gather`,
`ScenarioFuncSet::set_rally`, `Build::train`; read only by `come_out` (§5.8).
Logged as `GATHERPOINT { X, Y, ACTION }` inside `BUILDDATA`.

### 6.2 `Unit::add_gather_order@0061a5c0(o, pos, action)`

1. `QUEUE_NEW` → the clear (§3.1). `unit_masks &= ~0x400`.
2. `get_obj(GATHER)`: `ox = o, whom = who`, `uid = objects[who][o].uid`,
   `build_type = its TypeIndex`, `tx = ty = −1`, `non_flat_gather = dist_mod =
   0`.
3. **If the target's type `is_gather_type`** (BuildType vslot `0x90`,
   `build_flags & 0x40`): **`Build::add_gatherer(this.o, this.who)` — the unit
   joins the chain now, before it has moved**. Then if the type is not flat
   (vslot `0x94`) and not a UNIVERSITY: `non_flat_gather = 1`, `dist_mod = MINE
   ? 10 : 4`. (`non_flat_gather` is write-only: set here, logged, never read.)
4. `flags` bit 4 ← `action`; the civilian-transport request if the regions
   differ; `orderlist.add`; `update_action`.

Callers: `Setup::build_units` `(site, QUEUE_NEW, 0)` (§9); `Unit::build_done`,
`do_build` step 2/6, `do_repair` `(o, QUEUE_NEW, 0)`; `come_out` `(rally_o,
QUEUE_LAST, 1)`; `find_gather_spot` `(best, QUEUE_LAST, 0)`; `Group::
action_gather` `(o, QUEUE_LAST or QUEUE_NEW, 1)` (the UI and the AI).

### 6.3 `Unit::do_gather(order)@005ef2a0` — the walk, the arrival, the stand

Shorthand: `b` the building, `n = b.num_gatherers(0,0)`, `max = gather_max`,
"kill" = `kill_current_order(0)`, "think" = `add_think_order()`, "move(x, y)"
= `add_move_order(x, y, 1, 0, QUEUE_FIRST, 0, …, −1, −1)` — acted on the next
frame.

```
go.whom != who                                → kill; return
b not a live building, or not activated       → kill; return
if !go.been_there and !b.is_gathered_by(u.o):                  # the retry for a unit refused at issue
    if n < max: (OILPLATFORM or a sea unit → arrival check); b.add_gatherer; if still not in → FAIL
    else FAIL
    FAIL: kill; think; if b is flat and !b.covers_tile(u.tile): move(b.x, b.y); return
if build_type == FARM: b.city < 0 → FAIL; farm = b[+0x78]
if build_type ∈ {MINE, WOODCUTTER}:
    every 128 frames phased by o*4: if n > max → kill; think; return
    do_non_flat_gather(go); return                                 # §6.4
if build_type != FARM:                                              # university, oil well, oil platform, …
    if !adjacent_to(ox):
        d = min(x_size, y_size)*0x60 + 0x30
        same domain (or cannot transport): find_nearby_spot(b, d, −1, 0, angle, NOT_ME, …) fails → kill; flags |= 0x10; return
        else (a boat): aim at the platform itself
        if !b.is_gathered_by(u.o) and not OILPLATFORM → been_there = 0; return
        move(spot); return
else (FARM):
    if !b.covers_tile(u.tile):
        regions differ and cannot transport → kill; flags |= 0x10; return
        n > max → kill; flags |= 0x10; return
        land unit and !b.is_gathered_by(u.o) → been_there = 0; return
        move(b.x, b.y); return                                     # the farm's centre, 48-snapped by the adder
ARRIVED:
if !been_there:
    n > max → kill; flags |= 0x10; return
    been_there = 1; leader_flags |= 0x2000000                      # the economy's dirty flag
    scholar → go_inside(ox, whom, 0); check_gatherers; kill; return
    citizen at an OILPLATFORM → (a carried citizen arrives as its barge) go_inside; kill; return
    otherwise check_gatherers                                       # and stay
every frame from here:
    AI citizen, every 256 frames phased by (o + frame + who), diff > 1, b damaged, not under attack, in a city
        → add_repair_order(ox, whom, QUEUE_NEW, 0); return
    build_type != FARM (the OILWELL): one worker a frame bumps b.recharging (build_masks & 0x800, docs/CITIES.md §1.3);
        set_anim(CHAR_FARM); face 0x40000000; set_new_location(b.x + 0x120, b.y, 1, 1); return
    FARM: §6.5
```

The two gates are the object's alive bit and `WallData::is_active`: an
unfinished or dead building kills the order. `SubObjectData::flags |= 0x10`
marks "could not reach" (`think` reads it). A unit that joined but is not in
the chain by the time it arrives (someone pruned it) gets `been_there = 0` and
waits without moving. University and oil platform gatherers **garrison** and
the order ends; `num_gatherers` then counts them through `count_inside`.
`wait, tx/ty, goto_build, dist_mod` are the non-flat machine's; nothing here
reads them.

### 6.4 `Unit::do_non_flat_gather@005f0170(go)` — wood and ore

Two sides, by `goto_build`. `W = is(WOODCUTTER)`; `rnd = Random::get
(game_random, 0, 0xffff)`.

```
if been_there: one worker a frame bumps b.recharging
u.group = −1
goto_build == 0:                                                   # out at / heading to the tile
    unit_masks &= ~0x78000000; if !been_there: been_there = 1; dirty
    if wait < 0:                                                   # "return to the camp"
        find_nearby_spot(b, d = min(x_size,y_size)*0x60+0x30, …) fails → set_anim(CHAR_CHOP_WOOD); wait = 20; return
        set_anim(CHAR_DEFAULT); move(spot); goto_build = 1; wait = 32; unit_masks |= W ? 0x8000000 : 0x20000000; return
    a = guy0.cur_anim; a == 0x1d → return                          # mining: nothing at all
    a == 0x19: wait--; if wait != 0 → return;  wait = all_gathering() ? −1 : 300 + rnd % 100; return   # chopping
    T = (tx*0xc0 + 0x60, ty*0xc0 + 0x60)                           # the tile centre
    if vector_dist(u, T) < 0x140:                                  # arriving at the tile
        wait--; if wait == 0: all_gathering() → wait = −1; return  # LAB_005f0ef1, before the facing
                             else wait = 100 + rnd % 50
        face T; add the tree/ore effect if none; set_anim(W ? CHAR_CHOP_WOOD : CHAR_MINE_ORE); return
    set_anim(CHAR_DEFAULT)
    find_nearby_spot(T, 0xc0, 0x100, 2, angle, NOT_ME, …) fails, or the spot is u.pos or avoid_x/y:
        tx = ty = −1; goto_build = 1; wait = −1; if dist_mod: dist_mod--; return          # give the tile up
    !b.is_gathered_by(u.o) → been_there = 0; return
    move(spot); unit_masks |= W ? 0x10000000 : 0x40000000; return
goto_build == 1:
    wait ≥ 0:
        adjacent_to(ox): wait--; been_there = 1 (+dirty); wait < 0 → return; face the camp; set_anim(W ? CHAR_DUMP_WOOD : CHAR_DUMP_ORE); return
        !b.is_gathered_by(u.o) and land → been_there = 0; return
        regions differ and can transport → move(b.x, b.y); return
        find_nearby_spot(b, d, …) ok → move(spot); return
        !been_there and vector_dist(u, b) > 0x600: find_nearby_spot(b, 0x600, …) fails → kill; return; move(spot); return
        wait = −1; return
    wait < 0: choose a tile
        if !been_there: been_there = 1; dirty
        set_anim(CHAR_DEFAULT)
        if tx < 0 or ty < 0 or !has_gather_access(tx, ty, who, 1, 0):
            dm = dist_mod; (a tiny mountain: dm = min(dm, 3)); best = 9999999
            for i in 0..count: (x, y) = gather_from[i]
                if tdata[y][x].mask & 0x4000 and has_gather_access(x, y, who, 1, 0):
                    score = max(3, vector_dist(|x − b.tx|, |y − b.ty|)) * dm + (i >> 2)
                    if score < best: best, pick = score, i
            nothing → return
            move pick to the back of gather_from; go.tx, go.ty = pick
        wait = 400 + rnd % 200; goto_build = 0
        !W: unit_masks |= 0x40000000; wait = 1000000; return                                  # a miner stays out
        unit_masks |= 0x10000000; return
```

In plain terms: a woodcutter alternates 400–599 frames at a tile
(re-rolling 300–399 on the chop branch until every peer is out) with a walk
back to the camp for a 32-frame dump animation; a miner walks out once and
stays; `all_gathering` staggers the return trips. **Three sync-stream
draws** — `% 100 + 300` at `+0xcc3`, `% 50 + 100` at `+0xdad`, `% 200 + 400`
at `+0x54b`. `WorldData::has_gather_access@
006b4e50(tx, ty, who, 1, 0)`: the tile has `mask & 0x8000`, its `wdata` owner
is −1/`who`/an ally, and at least one orthogonal neighbour is neither terrain
class `(mask & 0x30) == 0x20` nor itself a resource tile (`& 0x4000`).

**Five lines the implementation has drifted from, each caught by a
differential check and never by a reading** (`docs/JOURNAL.md`, 2026-08-26,
2026-08-27, 2026-08-30 and 2026-08-31). As rules:

- **`guy0.cur_anim` is read before the tile is, and it decides the
  frame.** The two wait branches above are not one branch: the chopping
  guy's is `% 100 + 300` at `+0xcc3` and it is where a woodcutter spends
  every reroll after its first frame at the tile; the arrival's is
  `% 50 + 100` at `+0xdad` and run39 does not reach it once in 1,850
  frames. Merging them and rolling the arrival's formula puts a
  woodcutter's clock at a third of the original's, and **a draw count
  cannot see it** — both sites draw exactly once. A miner's `CHAR_MINE_ORE`
  returns before everything, so its `wait` never moves at all.

- **The camp arrival** is `wait--`, `been_there`, `wait < 0 → return`, face,
  `CHAR_DUMP_*`, and no third `set_anim` — the listing at
  `5f0b5e`–`5f0b89` holds the two dumps and nothing else.
- **`set_anim(CHAR_DEFAULT)` runs before** the tile approach's
  `find_nearby_spot` (`+0xfd4`).
- **`unit_masks & 0x78000000` is what a carrying walk is**, and the four
  `|=` above are its only writers (`Unit::think:82` and
  `kill_current_order:107` clear it). Reading `goto_build` instead makes a
  citizen's first walk to its camp a `WALK_WITH_WOOD` and costs the
  arrival stand at the end of it — `docs/ANIM.md` §4.4.
- **A walk goes in *front* of the order that issued it, and the order is
  still the order.** The function holds `go` for its whole body, so the
  `goto_build = 1` / `wait = 32` written after `add_move_order(…, 1, …)`
  land on the gather order, not on the move now ahead of it.
- **The tile choice's filter is not optional** (`005f0575`): a candidate is
  scored only if it still carries `mask & 0x4000` *and* `has_gather_access`
  holds for it, and when nothing passes, the function returns before
  `wait = 400 + rnd % 200` — no tile, and no draw. Without it a woodcutter
  is sent to the tile in the middle of its own forest, which has no
  orthogonal neighbour to stand on; the `else` arm re-chooses on the same
  test, so a held tile that loses its access is given up too (`005f0655`).

The three `CHAR_DEFAULT` sites are marked (`anim::SITE_STAND_GATHER`
`+0x10f`, `SITE_STAND_TILE` `+0xfd4`, `SITE_STAND_RETURN` `+0xb99`) and so
are the three draws (`SITE_TILE_WAIT` `+0x54b`, `SITE_WORK_WAIT` `+0xcc3`,
`SITE_ARRIVE_WAIT` `+0xdad`). The order's whole row — `tx`, `ty`, `wait`,
`goto_build`, `been_there`, `dist_mod` — is diffed against the dump's on
every frame (`OrderMismatch::Gather`), which is what makes the next drift a
failure rather than a reading.

Two diffs stand on this section.
`run39_s_woodcutters_reroll_on_the_chop_branch_not_the_arrival_s` is the
sharper: every rise in a dumped `wait` over run39's 1,850 frames must be
produced by the value the trace's draw on that frame returned, **under that
site's own formula** — 24 rerolls, 19 at `+0x54b` and 5 at `+0xcc3`, none
at `+0xdad` — and then the whole row against the record, ~~16,152 fields
to frame 897~~ **26,094 to 1,686** (2026-08-31; items 106 and 109 each
walked past that wait without touching it). It needs no simulation for its
first half, so it names the sites from the original alone.

The dump's woodcutter citizen (§4.8): frame 1 — `goto_build 1, wait 0`,
adjacent, `wait → −1`, `been_there = 1`; frame 2 — a tile (`wait = 400 +
rnd % 200`, `goto_build = 0`); frame 3 — outside `0x140` of the tile
centre, `find_nearby_spot(T, 0xc0, 0x100)`, `move(spot)`; frame 4 — the
move steps. That is its `(4008, 28296)`.

## 6.5 The farm (`do_gather`, FARM, arrived)

`ft` is `FarmStruct+0xbd` — 1 is the **pasture**, which grows no crop, is
skipped by `Farms::inc_time` and carries five animals of owner 9
(`docs/SYNC.md` §3.6).

```
ft = FarmsData::get_farm_type(farm)                    # farm_type & 1
ANIMAL_FARM: set_anim(CHAR_SOW); every 256 frames phased by (o*7 + frame + who):
    move((cx + 1 + GameAccess::rnd(xs/2)) * 0xc0 + 0x60, (cy + 1 + GameAccess::rnd(ys/2)) * 0xc0 + 0x60); return
dx, dy = u.tile − b.tile_corner(); out of the footprint → dx = dy = 1
state = farms.farm_data[farm].status[dx][dy]           # NOT [dy][dx] — see below
0: guy0.cur_anim != '$' → as 1;  else → new tile
1: set_anim(CHAR_SOW); Farms::grow(farm, dy, dx); return
2: guy0.cur_anim != '#' → set_anim(CHAR_REAP); Farms::snip(farm, dy, dx); return;  else → new tile
3: set_anim(CHAR_REAP); return
new tile: move((cx + GameAccess::rnd(xs)) * 0xc0 + 0x60, (cy + GameAccess::rnd(ys)) * 0xc0 + 0x60)
```

**`guy0.cur_anim` is the guy's live byte, and it is what decides the
switch** (2026-08-31, item 84). The test is
`*(char *)(**(int **)&this->field_0xf4 + 0x9c)` at `005eff5e` and
`005eff8a` — `UnitData::guys[0]`, then `GuyData +0x9c cur_anim`, named by
the type record. It is not a flag this branch keeps: **anything at all**
that plays an animation on the farmer between two farm frames changes what
the switch reads. A walk, an idle, and — the case that pays — the blocked
stand `Unit::move_step+0x823` plays before its give-up tests
(`docs/COLLISION.md` §5), which sets the byte **without the body moving a
unit**.

That is the whole of run33's word from 780 to 986. The AI's `1/4` stands
on farm cell 7 reaping it; `Farms::inc_time` decays the cell to empty
under it on 777; on 778 it is an empty cell under a reaper, so the switch
sends it to a new tile; on 779 its walk is blocked and the stand replaces
the reap. On 780 the original's cell 0 arm reads a byte that is no longer
`'$'` and sows. A model that remembers "this farmer was reaping" until it
takes a step re-picks that cell for ever — two draws every other frame,
and a farm that never falls under `Farms::inc_time`'s five-empty gate
(`docs/SYNC.md` §3.3), so a seventh farm draw a frame on top. Both
symptoms are one byte.

**The modulus is the type's own footprint** — `ObjectType::x_size`
(`+0x234`) and `y_size` (`+0x238`), 4 and 4 for the farm, lost by the
decompiler because `GameAccess::rnd` takes it in `ecx`. All four loads are
in the listing: `005efff9`/`005f0004` for the crop, `005efdd8`/`005efde5`
for the pasture, which halve each — so the herder gets one of the *inner*
four tiles, hence its `+ 1`, and the farmer any of the sixteen. run13 had
measured 4 off the dump's goals first (`docs/SYNC.md` §4.1).

**`GameAccess::rnd@0043cca0(n)` is `Random::get(game_random, 0, 0xffff) % n`**
— the sync stream, and `n ≤ 1` answers 0 *without a draw*; `docs/COMBAT.md`
§10 calls it unsynchronised and is wrong. `Farms::grow@008d91c0` sets `state = 1` and adds **`0.005f` to a
`float[4][4] percent`** until it reaches `1.0f`, then `state = 2`; `snip`
turns 2 into 3; what regrows 3 → 0 (`Farms::process`, presumably) was not
read, so a float accumulator sits upstream of the sync RNG.

**The clock is 101 frames, not 200** (`docs/SYNC.md` §3.3).
`Farms::inc_time@008d8600` adds a *second* `0.005f` to every growing cell
each frame, so a farmed cell takes two adds a frame, and `0.005f` in single
precision first passes `1.0f` on the **201st**: `grow`'s add on frame 100
is that one, the cell ripens under a still-sowing farmer, and the next
frame is the "new tile" above with its two draws. `'#'` is the **sow** animation
(index 35, the one every farmer shows) and `'$'` the reap, so case 2's
"not `'#'`" is *not sowing*; regrowth from 3 is `−0.01f` a frame, 101
frames to empty. The clock is `farms.rs`, and a **pasture has
none**: `inc_time` skips it, so a herder run through the crop switch would
reach the ripening on the two hundredth frame rather than the hundredth.

**The cell is `status[dx][dy]`, index `dx·4 + dy`, not the transpose.**
`FarmStruct` holds `float[4][4] percent` at `+0x8` and `uchar[4][4]
status` at `+0xac`, and three addressings name the same byte:
`do_gather`'s switch at `005eff54` reads `(dx + farm·0x30)·4 + 0xac + dy`;
`Farms::grow(farm, dy, dx)@008d91c0` — note the argument order — writes
`(farm·0x30 + dx)·4 + 0xac + dy`, and `snip` the same; `inc_time` sweeps
`4·dx + dy`, `dx` inner. The dump prints them in that order too.

**Coverage**, all diff-backed. The animation byte:
`an_empty_farm_cell_is_sown_unless_the_guy_is_still_reaping` (two sims
differing in that byte alone, made to fail on the flag it replaced) and
`run33_s_long_trace_says_where_the_word_parts`, whose word moves 780 →
986 with it. The cell index and the clock:
`run12_and_run13_s_farm_records_are_the_original_s_cell_for_cell` (every
cell of run12's frames 1–3 and run13's 95–104 against `Farms::log_data`)
and `run14_s_frames_match_the_trace_draw_for_draw` (every re-target of
that capture — 101, 199, 201, 211, 217, 218, 220, 241 — draw for draw).
The pasture arm: `a_pasture_herder_walks_only_on_its_own_256_frame_phase`
against run39's record — 42 dumped steps in four runs, each opening the
frame after a phase frame, and all seven phase frames spending the pair in
the trace (`docs/SYNC.md` §3.15).

## 6.6 `Unit::find_gather_spot(range)@005f5170`

Walk the owner's buildings: keep those that exist, are active, `is_gather_
type`, not neutralised, university-iff-scholar, with `num_gatherers(0,0) <
gather_max` **or `is_gathering_at` this building already** (the unit's own
action, not membership of the gatherer chain), in the unit's `tregion`; a
citizen standing in a city skips a building of *another* city when its own
city's population is `< 2`, **or** when `mypop <= otherpop + 2` — i.e. it
crosses to the other city's building only when its own city is more than two
more crowded than that one (`find_gather_spot@005f5170:108`; `CityData
+0x5a free` plus `+0x5c gatherers` is AI bookkeeping and an input to
`crates/sim`); within `range` if `> 0` — **a scholar ignores `range`
entirely**, the caller's value being kept only `if (!bVar2)` where `bVar2` is
the `SCHOLARS`/`SCHOLARSKOREAN` test (R4 G42) — score as below; take the max
with a **strict** `best < score` from a starting `0`, so the first maximum
wins a tie and a building scoring zero or less is not a candidate at all; if
not already gathering there → `add_gather_order(best, QUEUE_LAST, 0)`.
Integer division throughout.

**The score is cap headroom, not rate** — read as "the leader's per-good
rate" here until 2026-08-29, and it is not:

```
value = 0
for g in the six goods:                      # iVar7 = 0x30, 0x34 … 0x44
    if not type_avail(g, 1):        continue
    if not building.is(best_gather_type(g)): continue
    if over_cap[g] != 0:            continue # +0x4c, key 0x8932
    value += resource_cap[g] - income[g]     # +0x30 key 0x1281, +0x94 key 0x90236
    if has_tribe_bonus(0x16) and g != KNOWLEDGE:
        value += 0x640                       # dutch_interest_cap × 16, a literal here
score = value * 500 / (dist / 0xc0 + 2)
```

The three fields are `LeaderDataEncrypt`'s, all written by
`Leader::do_gather` (`docs/ECONOMY.md`), the first two in sixteenths — so
the term is **the unused part of the commerce cap** for the good this
building gathers, and the citizen goes to whichever good the player is
furthest from maxing out. `over_cap[g]` is nonzero only where that good's
income was clamped this frame, which drops an already-maxed good's buildings
out of the search rather than merely scoring them low. `dist / 0xc0` buckets
by the tile, so ties are the common case and the numerator decides them.
The story is in `docs/JOURNAL.md`, 2026-08-29.

## 6.7 How a gather order ends

`kill_current_order` on a `GATHER` (§3.2) → `Build::remove_gatherer`. The chain
also self-prunes: `check_gatherers` on every `add_gatherer` and on arrival;
`num_gatherers` ignores a member whose first order is no longer a gather at
this building, so a citizen re-tasked without a kill stops counting at once.
Building destroyed: `do_gather` kills on `!exists || !is_active`; `work` step
8 kills on a `uid` mismatch once the slot is reused. Full: `add_gatherer`
refuses; `do_gather` kills and queues `THINK` → `think_peasant(1)` →
`find_gather_spot`.

---

## 6.8 `Unit::think_fish@005f4c60` — where an idle fishing boat goes

Established 2026-09-01 from the decompile, the listing and the PE, and
**diff-backed on three frames of run54/run58** — 4462, 4871 and 4948, the
whole life of the AI's first Fisherman `1/14`. It is the head of `think`'s
tail (§2.4 step 5) and nothing below it had ever been built; it is what
East Indies' word sat on at 4462.

`Unit::think` reaches it at `+0x6a6` behind `is(0x13d, 0)` — the
`FISHERMEN` lineage — after the mod-32 tail gate, and a `1` back ends the
think. Everything below (`think_merchant`, `think_carry`, `add_to_army`,
the scout tail) is therefore unreachable for a fishing boat.

**The head, and who skips it.** If the type packs (`unit_flags2 & 4`) and
the unit is **not** packed (`unit_masks & 0x80000`) — `calc_gather` is
§6.10:

```
if ((o + frame) & 0x3ff) != 0: return 0            # once in 1,024, phased by o
r = calc_gather(…, &n, …)
unit_masks = n ? unit_masks | 0x20 : unit_masks & ~0x20
if r != 0: return 0                                # it can still gather here
```

A **packed** boat skips the whole block and searches every time it is
called. That matters more than it looks: `Unit::init@00612100:376` gives
`unit_masks |= 0x80000` to every type with `unit_flags2 & 4`, so a
Fisherman is **born packed** and searches on its first idle frame — run58's
`1/14` comes out of the Dock on 4461 and searches on 4462, where the
1,024-frame gate would have refused it (`14 + 4462` is not a multiple of
1,024).

**The walk.** `region = get_tregion(unit.tile)` — the *alternate* one, so a
boat on a coastal half-land cell answers its **sea** region — then the 289
offsets of `move_x/move_y[0 .. 0x121]` around the unit's own **cell**
([`crate::world::MOVE_289`]), in table order. A candidate is kept when all
three hold:

- `WData.flags & 0x100 == 0` — not a coastal half-land cell;
- `WData.land` is 1 or 2 (`SANDY` or `OCEAN`);
- `WData.region == region` — the raw field, against the unit's `get_tregion`.

Then, per kept cell, in this order:

```
good = find_good_at(cell, who, 0, 0)               # the cell's chain terminator
if good >= 0 and WData.down >= 0 and WData.down_who < 8 and WData.down != o:
    continue                                       # somebody else's claim; no draw
num  = good >= 0 ? 1_000_000 : 0
dist = max(1, |cx − x| + |cy − y|)                 # Manhattan, in cells
score = num / dist + rnd(60) + i                   # `i` is the ring index itself
if best_score < score: best, best_score = i, score
```

Three things in that line are load-bearing and all three are diff-backed:

- **`+ i`.** With no good anywhere every numerator is zero, so the score is
  `rnd(60) + i` and the *last* accepted index all but always wins. That is
  frame 4462: 54 accepted cells, 54 draws, and the winner is index **288**.
- **`move_y[288]` is `−16`.** The last entry of ring 8 is stored with the
  wrong `y` — `(−8, −16)` where the square wants `(−8, −7)` — so an empty
  sea sends a fishing boat sixteen cells north, out of the square it just
  searched. `1/14` is at cell (57, 55) on 4462 and its move order is to
  (49, 39). `rondata::pe` checks the whole table against the executable.
- **`OIL` is excluded, twice.** `find_good_at` refuses `TypeIndex::OIL` by
  name, and `Objects::init_good@00653f30` never wrote an oil patch into a
  cell's chain in the first place — it returns before the terminator write.
  On 4871 the boat is at (49, 39) with a Fish at (46, 37), distance 5, and
  an **Oil** at (44, 39), distance 5 too and at a *higher* ring index; if
  oil counted, the `+ i` term would have sent it to the oil. It goes to the
  fish.

**The answer.** `(dx, dy) = move[best]`. If it is `(0, 0)` — the unit's own
cell, which wins whenever the unit is standing on a good, since `dist`
floors at 1 and `1_000_000` beats every jitter — then a packed boat casts
`add_cast_order(−1, −1, −1, −1, 0x28c, QUEUE_FIRST, 0)` and returns 1, and
an unpacked one returns 0. Otherwise it is a **`QUEUE_NEW` move** to the
target cell's centre, exactly `add_move_order`'s shape: the angle from the
unsnapped centre, the destination snapped to the quarter-tile
(`cell × 768 + 408`), `tolerance 0`, `facing −1`, and the `PATHED`/`ACTION`
flags cleared. run58's block 4463 prints it field for field.

**The unpack is renamed on the way in.** `add_cast_order@005e4a60` rewrites
`0x28c` before it builds the order: `is(MACHINEGUN)` → `0x28e`; the exact
ids `MERCHANT`/`MERCHANTDUTCH`/`FURTRAPPER` → `0x290`; `is(FISHERMEN)` →
**`0x292`**; otherwise it stands. run58's block 4949 prints `spell 658`,
which is `0x292`. (`0x28b`, the pack, has the mirror-image rewrite to
`0x28d`/`0x28f`/`0x291` and no caller here.)

**The unreachable half.** After the loop the listing tests
`cmp edx, 0x121` on the best *index* and, if it is greater, replaces it
with `rnd(0x79) + 0x19`. `edx` is initialised to **0** at `5f4d2b` and only
ever assigned a loop index, so nothing can reach that draw. It is dead code
in the shipped build, and it is why `think_fish` has one draw site rather
than two.

### 6.8.1 Coverage

| claim | backed by |
| --- | --- |
| the cadence gates, the accept predicates, the score, `+ i`, the winner | **diff** — run58 frames 4462 (54 draws, `Unit::think_fish+0x27a`), 4871 (177) and 4948 (161), and the move orders and cast the dump prints on 4463, 4872 and 4949 |
| the head's `calc_gather` gate | **diff** — run54's frame 5106, four draws to this crate's 165 before it landed (§6.10) |
| `WData.down` is the live chain's head, not a snapshot | **diff** — run54's frame 5285, 177 accepted cells against this crate's 178 |
| `move_x`/`move_y[0 .. 0x121]`, the typo at 288 | **the PE**, checked on every run (`rondata::pe::tests`) |
| a packing type is born packed | reading (`Unit::init:376`) — and the 4462 search is only possible with it |
| `0x28c` → `0x292` | **diff** — run58 block 4949, `spell 658` |
| oil is in no cell's chain | reading (`Objects::init_good`), and the 4871 choice is what a wrong answer would have moved |
| the `best > 0x120` fallback is dead | the listing (`5f4d2b`, `5f4f0f`) |

**What is not established.**

- ~~**`UnitData::calc_gather@00609180`**~~ — the head's "can I still gather
  where I stand" test. **§6.10 since 2026-09-01.** run54's boat reaches the
  head on frame **5106**, standing on its fish: the original spends four
  draws there and this crate spent 165, walking a 17 × 17 it had no business
  walking. With the head answering the word went **5106 → 5285**, and with
  the claim test below it **5285 → 5376**.
- ~~**What the cast does.**~~ §6.9, and it moved East Indies' word from
  4950 to 4988.
- **The chain, whole.** `find_good_at` walks the cell's object chain in the
  original; here the terminator is a field, which is exact only because
  `Objects::init_good` is the only writer of one (`docs/COLLISION.md` §3
  and item 48's chain remain the general case). The **claim** half of the
  same chain is landed: `WData.down`/`down_who` is now the live unit chain's
  head, falling back to the loaded snapshot for the buildings and goodies
  this crate does not thread. run54's frame **5285** is what named it — a
  deployed boat *is* its cell's `down`, so the second Fisherman's search
  must refuse the fish the first is sitting on, and with the snapshot alone
  it accepted 178 cells where the original accepted 177. The word went
  **5285 → 5376**.
- **The second Fisherman's route.** Both boats' *sea* paths part from the
  original's on the frame they are planned — 4464 and 4870 — two cells
  north over the first half of a nineteen-cell staircase. That is the
  pathfinder, not this; `run58_s_five_thousand_frames_stand_where_the_
  original_s_do` pins both frames.

---

## 6.9 `Unit::do_cast@005ebfe0` — the craft table, and the deploy

Established 2026-09-01 from the decompile and `craftrules.xml`, and
**diff-backed on run58's whole 5,200 frames**: the AI Fisherman `1/14`'s
`spell_time` climbing 1 … 39 on frames 4950 … 4988, its `unit_masks`
`786440 → 262152` on 4989, and its `mylos` `4 → 6` on the same frame. It is
what East Indies' word sat on at 4950; §6.8 is what queues the order.

### The table

`craftrules.xml` is `GameAccess::spelltypes`: **55 `CRAFT` records**, in
file order, at `TypeIndex` `0x275 … 0x2ab`. `TypeData::is_spell_type` is
that range and nothing else, and a craft index outside it makes `do_cast`
substitute `0x296` (`Morph`) for the *job time* while keeping the real one
for the cast. The columns this crate reads:

| column | what reads it |
| --- | --- |
| `JOB_TIME` | `SpellTypeData::get_job_time@00675800`, in frames |
| `FLAGS` | letters `a`..`m` as bits `0..12`; `& 0xe` (`b` units, `c` buildings, `d` an area) is what makes a craft **targeted** |
| `FROM` / `FROM2` | the caster lineages, already read for `is_castable`'s head and for the caster bit (`docs/DATALAYER.md`) |

The four pack rows and the four unpack rows are the pairs `add_cast_order`
rewrites into (§6.8, "The unpack is renamed on the way in"), and their job
times are the deploy's whole cost:

| pair | `FROM` | `JOB_TIME` |
| --- | --- | --- |
| `0x28b` / `0x28c` | Catapult | 80 |
| `0x28d` / `0x28e` | Machine Gun | 50 |
| `0x28f` / `0x290` | Merchant | 148 |
| `0x291` / **`0x292`** | Fishermen | **40** |

`get_job_time` adjusts **nine** of the fifty-five and none of them is one
this crate issues: `0x27d` Entrench takes the French tribe bonus and
Antipater's rate; `0x275` Bribe and `0x27f` Informer halve under
`SPIES_CRAFT_FASTER`; `0x28b`/`0x28c` take the Turkish bonus, Napoleon's, a
half for two type masks and a quarter for a third; `0x28d`/`0x28e` halve
for one mask; `0x280`/`0x281` halve under a tribe bonus and flatten to 10
for one. `0x28a` and `0x292` are named by no arm, so the record's own field
is the answer — which is why run58's deploy is exactly forty frames.

### The untargeted arm, in the order it spends its frame

`do_cast`'s first branch is `spell_flags & 0xe`; a craft that carries none
of the three takes this half. Then:

1. `SpellType::pay_cast_costs`, once per order, on the order's own `+0x1c`
   ("paid"). Refused, the order dies. Every craft issued here has empty
   `COST`, `COST2` and `MANA`.
2. **On the first frame only** (`spell_time == 0`) the animation, and the
   state test that goes with it. `is_pack` → the caster must **not** be
   packed (`unit_masks & 0x80000`) or the order dies here, then
   `set_anim(CHAR_PACK, 0, 1)`; `is_unpack` → it must **be** packed, then
   `set_anim(CHAR_UNPACK, 0, 1)`; anything else → `CHAR_DEFAULT`, which is
   the transport's. One call site, so all three chains are the trace's
   `Guy::set_anim+0x97a < Unit::do_cast+0xc89`.

   **A rare collector unpacking is re-seated in between**
   (`005eca9c`–`005ecb0d`), and it is three calls in this order:

   - `UnitData::good_merchant_spot(tile)` on the caster's **own** tile —
     the same two-by-two and `calc_gather` §3.1's ring walk uses. A `0`
     kills the order, plays `S_INVALID_ORDER` to its owner and returns.
   - `Unit::set_new_location(div_3_table[(x ^ 0x63637) >> 4] * 0x30 + 0x18,
     …, 1, 1)` — the **unit-cell** centre of where it stands, 48 units to a
     side. A merchant walks to a tile corner and stops on a cell centre, so
     the point is usually the one it already holds; what the call is *for*
     is its third argument, which teleports every tracked crew figure onto
     its offset with the driver's facing (`docs/MOVEMENT.md`, "Who writes
     it, and when", last row).
   - `Unit::set_angle(guy 0's angle)` — the unit's **heading** set to its
     own facing, which also rewrites the crew's `des` from that angle.

   `UnitData::is_rare_collector@0046fae0` is the gate, and it is
   `is_merchant`'s three ids — `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER` —
   with `is(FISHERMEN)` behind them, so a **fishing boat takes this path
   too**: run58's `1/14` deploy passes `good_merchant_spot` on its own
   ocean tile.
3. On that same frame, and **for `0x28a` alone**, the shore test:
   `find_nearby_spot` for a barge within `unit_board_distance`, and no
   water means the order dies (`docs/TRANSPORT.md` §6).
4. `spell_time += 1`; below `get_job_time` it returns and the order stands.
5. `SpellType::cast@00676ce0` — a switch behind an `is_castable` that has
   to answer **3**. For `0x28c`/`0x28e`/`0x290`/`0x292` that is
   `is_map_unit` and the packed bit still set; for `0x28a` it is
   `can_transport`.
6. `kill_current_order` — **for every craft but `0x28a`**. The transport is
   the exception because `cast_transport` has already moved the order list
   onto the new boat, and the boat kills the cast there.

### `SpellType::cast_unpack@006709c0`

For a caster whose `TypeIndex` is not `0x3d`, `0x3e` or `400` — the two
merchants and the fur trapper — it is one state change and its
consequences: `unit_masks &= ~0x80000`, then `update_los` (`+0x160`),
`update_speed` (`+0x174`), `Unit::update_gpiece` and a `set_new_location`
on the unit's own position. The merchant arm ahead of it is a different
thing: `good_merchant_spot`, a snap to the tile corner, `set_blocked_at` on
the four tiles under the trader and `leader_flags |= 0x2000000`.

**The line of sight is the visible half.** `Unit::update_los@0060e4d0`
clamps a packed unit to four tiles — `if (mylos > 3) mylos = 4` at
`0060e638`, gated on the type's `unit_flags2 & 4` **and** either
`unit_masks & 0x80000` or `UnitData::is_packing@0060aa60` (the current
order is a cast whose spell `is_pack`). A caster that fails that gate falls
through to the merchants' fixed `epoch + 4` rather than skipping both. This
crate recomputes `mylos` on every read, so clearing the bit *is* the
update, and run58's `4 → 6` on frame 4989 is `LOS 4` plus one Science
epoch's `SCIENCE_LOS 2` with the clamp lifted.

### 6.9.1 Coverage

| claim | backed by |
| --- | --- |
| `0x292`'s `JOB_TIME` is 40, and the clock is one step a frame | **diff** — run58's `spell_time` 1 … 39 on 4950 … 4988, and `unit_masks 786440 → 262152` on 4989 |
| the order survives the wait, and dies after the cast | **diff** — the same forty frames, and `1/14` idle again from 4990 |
| `cast_unpack` clears `0x80000` | **diff** — `run58_s_five_thousand_frames_stand_where_the_original_s_do` compares the bit on **94,935** unit-frames, none wrong |
| the packed `mylos` clamp | **diff** — the same test, 94,338 unit-frames with the one known cache row (item 35) |
| the 55 rows are `0x275 … 0x2ab` in file order | the file, and `0x292`'s `FROM Fishermen` against §6.8's `add_cast_order` rewrite |
| `is_castable` must answer 3, and its pack/unpack cases | reading (`00675bc0`) — and the 4989 cast is what a wrong answer would have moved |
| the pack arm, and `CHAR_PACK` | reading; nothing in any capture packs |
| the rare collector's re-seat, and that it snaps the crew | **diff** — run75's block 6145 has `1/24`'s crew figure on `(40706, 14716)` at its driver's angle the frame the unpack starts, four frames before it could have walked there, and the re-seat took Great Lakes' word 6151 → **6463** (item 210) |
| `good_merchant_spot` gating that re-seat | reading — no capture has it refuse; run58's Fisherman and run53's Merchant both pass it |

**What is not established.**

- **The targeted half**, everything behind `spell_flags & 0xe`: the range
  walk, `is_valid_target`, the `find_nearby_spot` approach and the
  `add_move_order(QUEUE_FIRST)` it issues, the cloak and the message
  window. Nothing here issues one, and `do_cast` kills such an order rather
  than pretending. *Capture:* a spy craft, which needs a Spy.
- **`pay_cast_costs`**, which is modelled as "never refuses". True for
  every craft with empty `COST2` and `MANA`, which is the two this crate
  issues; a `MANA` craft would need the mana pool, and nothing keeps one.
- **The general's step.** Between 4 and 5 a caster whose type answers
  `+0x10c` and which `has_general(0, 0x162)` takes an **extra**
  `spell_time += 1` and bumps every guy's `+0x74` against its `+0x78`. It
  halves a pack's wait in practice and no capture has a general.
- **The captain give-back** for `0x28a`: a figure whose captain is itself
  casting hands the frame back. Every unit here is its own captain.
- **`update_speed`** moves no number on run58 (`myspeed 38` on both sides
  of the deploy). ~~And `update_gpiece` is item 152 — the boat's guy
  carries no piece at all.~~ **`update_gpiece` is modelled since
  2026-09-01** (`docs/ANIM.md` §3.4): the boat is born on its `-PACKED`
  piece, which is the entry carrying `CHAR_UNPACK`, and
  `Unit::update_gpiece@005e2920` swaps it for the plain one — the entry
  carrying `CHAR_PACK` — the moment `cast_unpack` clears the bit. Giving
  the deploy's animation a length is what let the wrap on 4988 be spent,
  and the word went **4988 → 5106**.
- **The merchant arm of `cast_unpack`**, and with it `good_merchant_spot`
  and the four `set_blocked_at` calls. *Capture:* a Merchant on a rare,
  which needs a rare in reach of the AI.
- **The non-spell-type arm** of `do_cast` — `LeaderData::current_upgrade`
  then `set_type`, and a `go_inside`/`come_out` pair when the unit stands
  on a `tile_mask & 3 == 3` cell with a friendly building. No craft index
  reaches it; it is what an order carrying a *unit* type in its spell slot
  would do, and nothing issues one.

---

## 6.10 `UnitData::calc_gather@00609180` — may I stay where I am?

Established 2026-09-01 from the decompile and the listing, and **diff-backed
on run54/run58's own word**: it is the head of §6.8 and the whole of
`Unit::do_gather@005fce20`, and it is what East Indies' long capture parted
on at **5106**.

It answers three things at once. Two are fields of the unit and both are in
every `UNIT` record the log prints:

- **`UnitData::rare`** (`+0x54`, the union with `air_alt` and
  `former_type`) — the `TypeIndex` of the good it found, `−1` when it looked
  and found none. `Unit::init@00612100:90` starts it at **0**, so "never
  asked" and "asked and failed" are different values.
- **`UnitData::good_obj`** (`+0x94`) — the `circle_x`/`circle_y` index the
  good was found at, in **tiles** from the unit's own. `Unit::init:281`
  starts it at `−1`.

The third is the return value, and it is not "there is a good": it is
**"there is a good and nobody else of my kind is sharing it"**.

**Who calls it.** Three gameplay callers. The first two are exclusive on the
packed bit and pass `param_7 = 1`; the third is the economy's and passes
**zero**, which turns on three things the other two never see (below):

- `Unit::think_fish@005f4c60`'s head — an **unpacked** packing type, once in
  1,024 frames (§6.8). A `1` back ends the think where it stands.
- `Unit::think@005f6e40:179` → `Unit::do_gather@005fce20` — a **packed** AI
  `is_rare_collector@0046fae0` (a merchant by id, or the `FISHERMEN`
  lineage), on `idle == 1` or once in 32. `do_gather` is nothing but this
  function plus the `unit_masks & 0x20` write, and its arm also bumps
  `idle` and, on a `1`, tries `unpack_merchant(4)`.

- `Leader::calc_gather@006ceee0` **step 6** → the same
  `Unit::do_gather@005fce20`, with `param_7 = 0`, `param_8 = 0` and the three
  outputs live: the six rates, the `BitMask<44>` of rares owned and a per-good
  tally. This is the income path, and it is specified in `docs/ECONOMY.md`
  (step 6, whole). `param_7 = 0` also turns on `calc_gather`'s **first** test,
  which the other two do not have: a packed fisherman or merchant answers 0
  before anything else happens.

The first two pass `param_7 = 1`, `param_8 = 1` and `(−1, −1)` for the
position, so what follows is that call and no other.
(`Unit::find_merchant_spot@00603ab0` passes a *probe* position;
`Options::describe` and `IFaceSelected::draw_unit_text` pass `param_7 = 0` and
are the interface.)

**The radius.** `ObjectTypeData::upgrade_level@00661090` walks the type's
`FROM` chain and counts the ancestors still in the starting type's own
lineage — equality, then the `is_list`, then `is_slow`, which is
`TechTree::is`. Then

```
r = merchant-by-id or is(FISHERMEN) ? level * 4 + 4 : level + 2
```

in **tiles**. The shipped `Fishermen` has `<FROM>none</FROM>`, so a fishing
boat searches `circle_radius[4]` — one cell each way.

**The two walks, both in tiles.** Both use the octagonal spiral
`circle_x`/`circle_y` (`circle_init@006817f0`, [`crate::ai_place::circle`])
around the unit's own **tile**, and both take the **first** tile that
answers rather than the nearest good — which is the same thing, since the
spiral is ordered by ring.

A tile qualifies on two tests and nothing else:

- the **surface field**: `(TData & 0x30) == 0x20` (ocean) for anything that
  is not a merchant by id, and `!= 0x20` for one that is. There is no third
  arm — the only non-merchant caller is the fishing boat, and fish are in
  the sea.
- `TData & 0x200`, [`crate::world::tile::AS_BUILDING`] — the bit the good's
  own object sets on the tiles it covers.

The good is then looked up in that tile's **cell**,
`ObjectsData::find_good_at@0065bec0(tx >> 2, ty >> 2, who, 0, 0)`. **That
mismatch of scales is why a boat standing on its fish answers ring 1 rather
than ring 0**: the fish marks its own tiles, the boat is on a different tile
of the same cell, so index 0 fails the `0x200` test and index 1 — the
spiral's first neighbour — carries it. run58's `1/14` prints `good_obj 1`
on every frame from 4992 to the end of the capture.

- **Block A** (`00609289`..`0060934e`) retries the remembered `good_obj`, on
  its own, before anything else. A hit keeps the index; a miss on a tile
  that passed the surface test clears it to `−1`; a tile of the *wrong*
  surface leaves it alone.
- **Block B** (`0060937d`..`00609585`) is the spiral, and it writes the
  index it stopped at.

**The crowd count, and the return value** (`LAB_00609573`, `006095b0`..
`0060985a`). Having found a good it walks the object chains of the
`circle_radius[2]` cells — **cells** now, not tiles — around the unit, and
counts the owner's other objects that are all of: live (`is_active`, vtable
`+0x8`), on the map (`is_on_map`, `+0xbc`), in the unit's own lineage
(`ObjectData::is` falls through to the type's `is(this->type, 0)`), not the
unit itself, and **not packed** — except that with `param_8 = 1` a packed
unit that `UnitData::is_unpacking@0060a4b0` accepts (its head order is a
craft whose spell `is_unpack`) counts anyway. The distance test is

```
vector_dist(|dx|, |dy|) < (their r + my r) * 0xc0
```

— the listing at `609761`..`609824`, where `0xc0` is 192, the tile.

Then:

```
if count == 0: return 1
n = count + 1
for i in 0..6: rates[i] /= n
if param_7 != 0: return 0        # both gameplay callers
*param_2 = 1; return 1
```

So the six gather rates `LeaderData::calc_rare@006e08d0` filled are shared
out among everyone standing on the good, and **a crowded gatherer is told to
move**: one other boat within eight tiles turns "stay" into "go", and
`think_fish` walks its 17 × 17 again.

**`unit_masks & 0x20` can never be set by either gameplay caller.**
`*param_2` is zeroed at the top and written `1` only after the `param_7`
test, which both callers fail. `think_fish` clears the bit outright;
`do_gather`'s wrapper sets it from the same always-zero out-parameter. No
`unit_masks` value in run33, run39, run53, run54 or run58 carries the bit,
which is what a grep of every dumped record says.

### 6.10.1 Coverage

| claim | backed by |
| --- | --- |
| the head's return value, and that a boat on its fish stays | **diff** — run54's long word: frame 5106 spends 165 draws here and **4** in the original, and with this the word passes it |
| the tile walk's scale, and `good_obj 1` for a boat on its own fish | **diff** — run58's `UNIT` record, `good_obj 1` on 210 frames from 4992 |
| `rare` is the good's `TypeIndex`, `−1` for a failed look | **diff** — the same 210 frames at `rare 6`, and 120 at `rare −1` from 4872 |
| `unit_masks & 0x20` is never set **by the `param_7 = 1` callers** | **the dumps** — no `unit_masks` in any of the five long captures carries it |
| the `param_7 = 0` form: the packed refusal, the rates, the rare mask | **diff** — run63's `myspeed 38 → 45` on frame 5552 and run59's two incomes (`docs/ECONOMY.md`, step 6) |
| the radius, `upgrade_level × 4 + 4` | reading (`00661090`), and the file's `<FROM>none</FROM>` on Fishermen |
| the crowd count and its distance test | the listing (`609761`..`609824`); no capture has two boats on one fish |
| `param_8`'s unpacking arm | reading (`0060a4b0`) |

**What is not established.**

- ~~**The rates.**~~ Landed 2026-09-02: `LeaderData::calc_rare@006e08d0`, the
  ally-territory flag that scales them (block A takes
  `LeaderData::is_ally@006edb50`; block B inlines a team comparison) and the
  division by `count + 1` are all in `crates/sim/src/rares.rs`, reached by the
  third caller above — the two named here really do discard the array.
  `docs/ECONOMY.md` step 6 is the specification, and run59's census is the
  diff: the AI's `income[food]` and `income[wealth]` were 160 sixteenths short
  apiece for 250 frames, and 160 is `10 × 16` — Fish's own two `BONUS_NUM`s.
- **The other two writers of `rare` and `good_obj`.** run58's `1/14` gets
  `rare −1` on **4872** — `Unit::think`'s rare-collector arm, which this
  crate does not model at all, including its `idle += 1` — and `rare 6,
  good_obj 1` on **4992**, which is the gather job (`Unit::do_job@00617a10`
  → `Unit::do_gather@005ef2a0`) and not this call. So the two fields are
  right *here* and cannot yet be compared against the dump end to end; the
  widening is booked with the arm.
- **The chain, whole.** The crowd count walks this crate's unit chain, where
  the original threads buildings and goodies through the same list
  (`docs/COLLISION.md` §3, item 48). Neither can be counted — the walk keeps
  only the owner's units of its own lineage — so the shape differs and the
  answer does not.
- **The `0x200` writer.** The tile bit arrives from a start dump's own
  `TData` and nothing in this simulation sets it when a good is placed by
  hand; a fixture must mark the tile itself.

---

## 7. The attack order, and the other combat orders

### 7.1 `AttackOrder` — the fields, what writes them

- `ox/whom/uid` — the target; `fight` compares `uid` with the live object's on
  entry and **treats a mismatch as no target** — a recycled index does not
  inherit the order.
- `mandatory` — `add_attack_order@005e5410`'s 4th argument: a player's attack
  click (`CommandPackage::process_attack` hard-codes 1); cleared for targets
  the unit found itself (`find_nearby_target`'s path passes it only for AI
  siege on a city, `docs/COMBAT.md` §12.2).
- `defensive`, `def_x/def_y` — set when the unit's combat stance is DEFENSIVE
  (1) and the order is *not* the explicit kind: `UnitData::find_def_pos@
  00608560` fills the post from an existing DEFENSIVE attack's, else from the
  head move's `x/y` if it carries flag 8, else the unit's own quarter-tile
  centre (`div_3_table[x >> 4] * 0x30 + 0x18`). A player's attack: `defensive
  = 0, def = −1`.
- `in_range`, `ever_in_range` — written only by `fight`: `in_range =
  is_in_range(…)` after its range test, and `ever_in_range = 1` when in range.
  Read in one place: a **packer** type (`unit_flags2 & 4`) that is not packed,
  out of range now but `ever_in_range` — its target walked out of its reach
  once it had it — **drops the order** rather than chasing. Bookkeeping for
  every other type.
- `new_ord` — 1 at creation; cleared by `fight` the first time it actually
  strikes. On entry a `recharging` non-cavalry-archer **returns at once if
  `new_ord == 0`**; only the first frame of a fresh order proceeds past the
  reload gate — to turn and set the animation — and then also returns. "Has
  not struck yet."

### 7.2 `Unit::do_attack@005f1b80` and what `fight` does to the order

`do_attack`, every frame: a carrier (every 16th frame, strafes its planes at
a unit target, kills on a building); an unarmed transport barge sails to
`find_attack_pos` (`add_move_order(QUEUE_FIRST, action = 1)`); an unarmed
member of a group walks to `r = (group's max range + 2) × 0xc0` behind the
army and kills the order; cavalry-archer bookkeeping; then **the seam**:
`fight(ox, whom, mandatory, 0, 0)` — once per frame per attack order — with
the cavalry-archer STAND_GROUND twist (`max_range = second_max_range` around
the call, `docs/COMBAT.md` §8.5).

`Unit::fight@005fd4d0`'s order-management half, in the order met
(`docs/COMBAT.md` §8.2 owns the strike):

- **Entry.** `uid` mismatch → no target. "Bad target" = `unit_flags2 & 1`
  clear, invalid, or the order has flag `0x10`. A `recharging` unit with a
  good target returns 0 unless `new_ord` (§7.1). `set_new_location` to the
  quarter-tile centre.
- **Invalid target** (`valid_target == 0`): a mandatory order on a unit tries
  the target's **captain** — valid → the order's `ox` is rewritten and the
  function continues (a squad member died, the squad lives). Otherwise: if
  `waiting < 5` and the owner's "targets lost this frame" counter (`leaders
  +0x9f4`) is over 10 → `waiting += 2`, return (a throttle — the army does not
  re-search on one frame); else **`find_new_target(0, 0)`** (rewrites or kills
  the order), `+0x9f4 += 1`. A capturable city beyond `city_capture_radius ×
  0xc0` under a mandatory/army order → `add_move_order(halfway, QUEUE_FIRST,
  0)`.
- **Opportunity check** (`docs/COMBAT.md` §8.2 step 0): an AI captain rolls;
  `check_target` fails → kill, `find_melee_target(−1)`.
- **The 1/5 re-search** (not mandatory, not recharging, captain, unit
  target): `Random::get % 5 == 0` or flag `0x10` → `find_new_target`.
- **Range.** `in_range = is_in_range(…)`; in range → `ever_in_range = 1`. Out
  of range: the packer rule. In range and a packed packer: `add_cast_order
  (UNPACK 0x28c, QUEUE_FIRST)`, return — unless an entrenched siege unit's
  target out-ranges it and `find_attack_pos` finds better, then
  `add_move_order(pos, QUEUE_NEW)`. In range, unpacked siege, a unit target:
  `ATTACK_GROUND` inserted `QUEUE_FIRST` at the target's cell (`accuracy =
  (domain == sea)`, `attack_unit = 2`), `do_idle`, return 0.
- **Out of range, DEFENSIVE** (stance 1, not mandatory): a `defensive` order
  with a post, the unit at least `max(unit_defensive_respond_range,
  max_range()) × 0xc0` from it → `find_new_target(0, 0)`; if the head is still
  this order and not recharging → kill, and if nothing is left →
  `add_move_order(def_x, def_y, MOVE_TO, QUEUE_NEW, 0)` with **flag 8**.
- **Out of range, otherwise — the chase.** `find_attack_pos(o, who, 0, &x, &y,
  0)` (a legal cell within range of the target; not read); none, or a
  STAND_GROUND unit not `on_duty`, or `unit_masks & 0x2000000` without
  `unit_masks2 & 0x20000` → `find_new_target` and, if the head is still this
  order, kill (+ the DEFENSIVE post move). Found → **`add_move_order(x, y,
  MOVE_TO, QUEUE_FIRST, 0)`** — the chase is a plain `MoveOrder` rotated above
  the attack; next frame `do_job` runs `do_move`, and the attack order is
  reached again when the move ends (arrival → `kill_current_order`) or
  something rotates it out. If the chosen cell is not the unit's own, the
  **action's** flag `0x10` is toggled and `do_idle` runs — what makes the next
  `fight` re-search instead of trusting the target. While chasing, `do_move`
  itself runs the every-4th-frame `find_new_target(0, 1)` (§4.4).
- **The strike** sets `new_ord = 0` right after `set_attacking`; returns 1.

**When the order ends**: `kill_current_order` from `do_attack` (carrier/
building, unarmed member, barge with no target), from `fight` (the
`check_target` failure, the packer rule, DEFENSIVE out of range, no attack
position, `find_new_target` with nothing found — it kills the order itself),
and from the generic paths (`die`, `close_orders` on any `QUEUE_NEW`, `Group::
action_halt`). The target's death does not touch the attacker's order: it
finds out on its next `fight` through `valid_target`; out of sight likewise
(`is_seen`).

### 7.3 `ATTACK_GROUND` — `do_attack_ground@005f1410`

`can_attack_ground` else kill; a cell at peace with me and `attack_unit == 0`
→ kill. Facing = `find_angle` to the cell, ±90° for a `GUN` broadside. Not in
range and `attack_unit == 0`: the spot at `dist − big_radius` clamped into
`[min_range × 0xc0 + 0x90, max_range × 0xc0 − 0x30]` toward the cell, `find_
nearby_spot`, in range from there → `add_move_facing_order(spot, angle,
MOVE_TO, QUEUE_FIRST, 0)`; else kill. Recharging → idle animation. `attack_unit`
2 → 1 and fire; 1 → kill and `do_idle` (the siege-at-unit insert fires once).
`set_attack(−1, −1)`; `unit_masks |= 0x11000`; flag `0x80`; `fire_ammo(−1, −1)`
if `fire_proj` (`docs/COMBAT.md` §9).

### 7.4 `ATTACK_TO` — `do_attack_to@005f2320`

`do_move(order)` first (a full move step). Then, if the head is still this
order and `(o + frame) % 15 == 0`: an armed non-supply unit →
`find_melee_target(−1, 0, 0, 1, 0)` — which adds an attack order
`QUEUE_FIRST` above the attack-move when it finds one (`docs/COMBAT.md`
§12.4); under an army flag a unit more than 6 units from its slot skips the
search. Unarmed/supply → `do_attack_to_pause`: a group member whose group
`is_attacking_near` sets `pause = 15`. The attack-move is a move that looks
around every 15 frames; the engagement is an ordinary `AttackOrder` stacked on
top, and when that dies the attack-move resumes.

### 7.5 `GUARD` — `do_guard@005e5c70`

Created by `add_guard_order@005e3e40(ox, whom, dx, dy, queue, …)`: target and
`uid`, `guard_x/guard_y` = the target's position now, `dx/dy` = the offset
asked (`Group::action_guard` passes −1, −1 for a player's click — every
member gets its own `GuardOrder` on the same target), `idle = retry = 0`,
action bit. Per frame, on an active unit target: `retry` counts down. A
target off the map or idle (`is_moving == 0`): **every 16th frame `(o + frame
+ 8) & 0xf == 0` → `find_melee_target(−1, 0, 0, 1, 0)`** (the guard's own
engagement, limited to `unit_guard_respond_range` of the post) and return;
every 16th frame `(o + frame) & 0xf == 0` → `idle += 1` and return. A
building target: within `attack_dist ≤ 0x600` nothing; farther, a one-unit
group `action_move_near(spot near it, QUEUE_NEW, MOVE_TO)`. A unit target: the
guard point is the target's position plus `dx` rotated by the target's facing
(`sinx/cosx`, reversed if its `unit_masks & 2`), validated with `invalid_loc`,
else `find_nearby_spot(0xc0..0x180, 0x60)` or the target's cell; `guard_x/y`
updated; facing = the target's when it moves, else `find_angle` to it. **Not
on that quarter-tile** → `idle = 0`, `add_move_facing_order(quarter-tile,
facing, ATTACK_TO kind, QUEUE_FIRST, 0)`, the new move's `timer = 0x1e ×
max(1, tile Manhattan)`, then **`do_move` this same frame**; if still guarding
afterwards, idle animation and `retry = Random::get % 3 + 6` — **one
sync-stream draw per reposition**. On the tile: turn if needed, `idle += 1`; a
packed packer with auto-stance unpacks after `idle ≥ 0x1e`/`0x46`. An inactive
target → kill. A guard is a follow-with-offset whose engagement is the
periodic `find_melee_target`; it never calls `fight` itself.

### 7.6 `FOLLOW` — `do_follow@005e65d0`

`add_follow_order@005e3f60`: `ox/whom/uid` the target, `oxx/whose/uid2` its
transport if inside one, action bit. Per frame: a followed object inside
something swaps to follow the container (and back when the original
reappears by `uid`); not seen → kill; `d = vector_dist`; the standoff `s =
clamp(los × 0x180 − k, 0x180, 0x600)` with `k = los × 0x60` if the target is
slower else `los × 0x300 / 5`, doubled if it `is_moving`; `d > s + 0xc0` →
project from the target toward me by `s`, `find_nearby_spot` (fallbacks: the
projected point, a ring at the target's facing, its own cell),
`add_move_facing_order(spot, the target's facing, MOVE_TO, QUEUE_FIRST, 0)`
and **`do_move` this frame**; else idle animation.

### 7.7 `GROUP_PATROL` — `do_patrol@005f1910`

`add_patrol_order@005e4560(x1, y1, x2, y2, id, leader, …, queue)` creates a
**`GroupPatrolOrder`** (`PatrolOrder` alone is never instantiated) with two
waypoints (`[0]` here, `[1]` the click; further clicks append), `waypoint =
0`, `id = (group.id + frame × 10) × 100 + group.order_num`, action bit. Per
frame: a unit **not in a group**: `waypoint = (waypoint + 1) mod count` and an
`ATTACK_TO` move `QUEUE_FIRST` to it — a patrol is a loop of attack-moves,
each leg engaging what it meets (§7.4); the **leader** of a group does the same
through `Group::action_move_to(group, next waypoint, QUEUE_FIRST, …,
ATTACK_TO)`; a **follower** idles (its legs come from the leader's group move).

### 7.8 A building's attack order

`Build::add_attack_order@00622ce0` is not an order object: it writes
`attack_ox (+0x7c)`, `attack_who (+0x81)` and sets/clears `build_masks & 4`;
`Build::do_attack` (`docs/COMBAT.md` §8.6) reads them. The simulation's
`order_building_attack` is already this.

### 7.9 `GroupAttackOrder` is dead

`OrdersMemManager::get_obj(GROUP_ATTACK)` has no caller in the export; only
`do_group_attack@005e75a0` steps it, and the only other references are
comparisons and the `OrderNames` table. A group's attack is N `AttackOrder`s
(§8.3).

---

## 8. The command stream and the group orders

### 8.1 The entry — `CommandPackage::process_*` → `Group::action_*`

A player's command arrives as a `Command` packet processed by `CommandPackage::
process_all`; the selection first, as a `GroupCommand {num, who, list[]}` →
`process_group@0094a0c0` builds a `Group` of those units (captains only); the
order packets that follow apply to it:

| packet | handler → | logged at `COMMANDMANAGER=1` |
|---|---|---|
| `MoveToCommand {to_x, to_y, set_angle, angle, orders, queued, form, width, disembark}` | `process_move_to@009497c0` → `Group::action_move_to(g, x, y, queued, set_angle, angle, orders, **1**, form, width, disembark)` | `to_x to_y queued set_angle angle orders disembark` + `form width` |
| `MoveNearCommand {…, tolerance}` | `process_move_near` → `action_move_near` | same |
| `AttackCommand {ox, whom, ignore, queued}` | `process_attack@00949c30` → `Group::action_attack(g, ox, whom, mandatory = **1**, queued, ignore)` | `ox whom ignore queued frame` |
| `SiegeAttackCommand`, `AttackGroundCommand`, `GuardCommand`, `FollowCommand`, `PatrolCommand`, `FormCommand`, `HaltCommand` | `action_siege_attack` (attack for the siege half, guard for the rest), `action_attack_ground`, `action_guard` → `add_guard_order(−1, −1)` per member, `action_follow`, `action_patrol`, `action_form`, `action_halt` → `close_orders(0)` per member | |

**This is where an order stream enters**: a recorded game is a sequence of
these packets, and the `GLOG_COMMANDMANAGER` lines of a gamelog (detail 1:
`COMMANDMANAGER=1` in `gamelog.ini`) are the same stream in text. `orders` in
`MoveToCommand` is the `OrderIndex` kind (`MOVE_TO` for a right-click,
`ATTACK_TO` for an attack-move, `EXPLORE_TO` for auto-explore); `queued` is
the `QueuePos`; the `1` is the action bit on every resulting order. The
scripted `cheat select` + right-click path (`docs/ORACLE.md`) goes through the
very same `MoveToCommand` as a one-member group; `Unit::go_to` builds a
one-unit group and calls `action_move_to` with `0` for the action bit.

### 8.2 `Group::action_move_near@00704990` — who gets which order

For a `MoveToCommand` (tolerance 0; `action_move_to` forwards). In order: the
scenario's `ignore_orders` filter; clamp the click into the world; **split the
group by domain** (land vs sea/air; units inside a transport count as the
transport) and recurse on each half; `QUEUE_FIRST` via detach/halt/`QUEUE_NEW`/
re-append; an air leader of domain 2 returns. **The formation**: `form` from
the packet or the group's current (9 = "no formation"), `form_mod` (width);
`QUEUE_NEW` clears every member's orders; `Group::compute_form@00707c80` → a
`Form` that finds the leader (`GroupData::find_leader`), decides the formation
angle (`find_angle(centre → click)`, or the leader's facing when already there,
with the reverse flip past 90°), runs `Form::categorize`/`compute` (not read)
and leaves **one destination per member** in `to_x[128]/to_y[128]` and the slot
offsets in `off_x/off_y`. **Per member** `i`: the slot destination clamped; a
member whose slot is in a different `tregion` from slot 0 is taken out of the
group and re-slotted by `find_nearby_spot` near slot 0; a non-captain takes
its captain's slot nudged; an invalid slot → `find_nearby_spot(0xc0, 0x480)`
around slot 0. Then:

```
if (orders == MOVE_TO || orders == ATTACK_TO) && !is_modern_infantry(u)
   && !( (u.type.role & 0x10 && !(u.unit_masks & 0x40000)) || group.num < 2 || (u.unit_masks & 4)
         || u.type.domain == 1 /* sea */ || form == 9 )
     add_group_move_order(u, tile(to_x[i]), tile(to_y[i]), angle | (group.angles[i] << 24),
          id = (group.id + frame*10)*100 + group.order_num, slot = i, leader, kind = orders,
          queue, action = 1, facing = form.reverse, orig = click, disembark)
else add_move_facing_order(u, tile(to_x[i]), tile(to_y[i]), angle | …, kind = orders, pathed = 1,
          queue, action = 1, facing = form.reverse, orig = click, disembark)
```

So **a group of two or more land units that are not modern infantry, not
`role & 0x10` types (~~workers, caravans~~ — **a `SCOUT` on land or a
`BARK` at sea**, settled by the type record in `docs/GROUPS.md` §6.6
step 6) and not in form 9 each receive a `GroupMoveOrder`** (`GROUP_MOVE`,
or `GROUP_ATTACK_TO` for an attack-move) — one per unit, a shared `id`, the
same leader, each its own slot and destination. A lone unit, a boat, modern
infantry, a scout, or a "no formation" group get plain `MoveOrder`s to their
own slots. Then `update_
positions(leader)` fills `GroupData::curr_x[i]/curr_y[i]` — the slot offsets
rotated by the leader's move angle — and (in outline) **the leader's path is
computed once** (`find_wpath` from the leader's cell to slot 0) **and copied
to every member translated by its slot offset**, each waypoint re-validated;
the group paths once and offsets.

### 8.3 `GroupMoveOrder` per frame — `do_group_move@005e79a0`

`G` = the order: a `MoveOrder` plus `oxx/whose` = the leader, `id`, `form_id` =
my slot, `group_angle`, `in_group`.

- `group == −1` → `ungroup_move_order(id, 0)`.
- **The leader**: if my action is `ATTACK` and every 32nd frame its target is
  within `0x900` → `Group::kill_group_move(group, id)` (the whole group drops
  the group move for its attacks). Else **`do_move(G)`** — the leader moves
  exactly like a lone `MoveOrder` — and if still going: `group.new_speed =
  get_speed(x, y, 1)`, `group.speed = old new_speed`, `update_positions`.
  `do_move` returned 0: an attack-context collision → `kill_group_move` then
  `Group::distribute_attack(target)`; else `ungroup_move_order`.
- **A follower**: sanity on the leader (exists, on the map, in my group, has a
  group order with my `id`); my index in `group.list` is rewritten to
  `form_id` every frame. Leader lost and I am more than `0x5ff` from its cell
  → `Group::refresh_group_order@00713a50(id, me)` — **I become the leader**
  (offsets re-based on my slot, every member's order rewritten to name me via
  `modify_group_order`); else `ungroup`. My action is `ATTACK` with its
  target in range → kill. Otherwise my target this frame is **`leader.pos +
  group.curr_x[form_id]/curr_y[form_id]`**, pushed as a one-entry path: the
  slot reached (the leader's distance to it within `0x60` of mine, or I am
  within `0x180`) → `in_group = 0`, `ungroup`; else heading = `find_angle(me →
  slot)`; within 60° of the leader's heading → walk to the slot (`in_group =
  1`); else if the leader has a path → its next waypoint plus my offset; else
  the midpoint. **Speed**: heading agrees: `group.speed = group.new_speed =
  min(group.speed, UnitData::speed(me))` — the group's speed becomes the
  slowest reporting follower's — then `v = get_speed(x, y, 1) + min(v / 3, 9)`
  — **a follower walks up to a third faster (capped +9) to hold its slot**;
  heading disagrees: `v = get_speed(x, y, 0) / 2`. An invalid slot within
  `0x300` Manhattan may add a flock of birds (**one sync-stream draw**) and
  `ungroup`. **`move_step(this, G, v)`** — the seam into `docs/MOVEMENT.md`'s
  unit step with the speed passed in. Arrived with the head still `G` →
  `ungroup` (or the attack-context hand-off).

`ungroup_move_order@005fd140(id, sub)`: finds the group order with that `id`
and **replaces it in place with a plain `MoveOrder`** (`GROUP_MOVE → MOVE_TO`,
`GROUP_ATTACK_TO → ATTACK_TO`) copied field-for-field, `dest = 0`, `orig =
x/y`, flag 1 cleared and `kill_current_path` unless I was the leader; then the
same for every subordinate down `o_down`. So a group move *degrades* into N
independent moves to the same slot destinations, one unit at a time.
`kill_group_move(id)`: every move with that `id` (not `GROUP_ATTACK_TO`) loses
flag 1, its path, and is killed. `Groups::process@006fa210` runs once a frame
for one slot of each player's 64 group slots, purging dead members and
resetting `group.speed = new_speed = UnitData::speed(leader)`. `UnitData::
get_speed(x, y, 0)` caps a unit **with no action** at `group.speed`; a
player's group move carries the action bit so the cap does not apply to it —
the formation holds by the follower's boost, and the leader is not slowed
(medium: `find_leader` unread).

### 8.4 N independent moves plus an offset — the verdict for v1

Almost — the engine itself degrades to exactly that through `ungroup_move_
order`. Right with no extra work: the destinations (one per unit, from the
formation's slot table, its own sub-area), the path (one, offset per unit),
the end condition per unit, and every case the original already treats as N
plain moves (a single unit, boats, modern infantry, workers, form 9). Wrong
per frame for a land formation of two or more: followers track the leader's
*current* position plus a rotated offset, not the destination (the formation
bends around corners and re-forms when he stops); the follower speed boost
and the cutting-corner halving; leader loss and hand-over; the attack-move
interaction; and the flock draw on an invalid slot. For the harness: the
per-frame positions of followers in a group move will not match under the
N-moves model; the arrival positions will; the RNG stream diverges on the
first invalid-slot flock. A faithful `GroupMoveOrder` is one struct and one
function on top of the `MoveOrder`/`move_step` the simulation has; the
genuinely new inputs are `GroupData` (`curr_x/y`, `speed/new_speed`, `list`,
`march`) and `Form::compute`'s slot table.

### 8.5 `Group::action_attack@00712490` — N attack orders

A building group sets the three `Build` fields per building (§7.8). Unit
groups: `QUEUE_FIRST` via detach/halt/recursion; a capturable city →
`check_capture` and a group move to it; then per member (three passes by
domain, skipping the `ignore` mask's kinds): a plane gets a strafe; a unit
whose action is already a mandatory `ATTACK` on this target with fewer than
three orders queued and not in range keeps it; a packing packer clears,
casts `UNPACK`/`PACK` if the range says so, then `add_attack_order(o, who,
QUEUE_LAST, mandatory, 1)`; a unit that cannot attack a building target gets
a move to its cell + `check_capture`; and the main path: **`add_attack_order
(unit, o', who', queue, mandatory, action = 1)`**, where `o'` is the clicked
target unless `mandatory == 0`, in which case `find_melee_target(min(dist +
0xc0, unit_respond_range × 0x240), …)` picks something nearer and the click is
the fallback. `group.order_num += 1`. **A right-click attack with ten units
selected is ten `AttackOrder`s, each stepping `do_attack → fight` on its own**,
with no coordination beyond `Group::distribute_attack@00713390` when a group
move ends on a target: members whose head order `is_attack` and in range of
`(o, who)` are counted, the rest have their target cleared; fewer than two in
range → every attacking member gets kill + `add_attack_order(o, who,
QUEUE_FIRST, first member's mandatory, first member's action bit)`.

**The verdict stands, and it now has a record to be diffed against**
(2026-08-26). run31 is the first dump holding `GroupMoveOrder`s
(`docs/GROUPS.md` §12.1): the order the simulation stands in for with a
plain `MoveOrder` is on disk, with its slot destination, the click it came
from, the leader it was laid out around and the member's slot index. What
the seam costs is therefore measurable now rather than argued, and the
~~thing to measure first is step 6's `angle + (angles[i] << 24)` against the
`MOVEORDER` base's own `angle`.~~ **First thing measured, 2026-08-26.** The
order's `angle` is the formation's own to the bit on all 945 of run31's
order blocks, and its `facing` (`MoveOrder +0x28`) is the mirror the layout
used — which turned out to be a *live input* rather than a record:
`Unit::kill_current_order` hands it back to the group when the order dies,
and that is the flag `compute_form` reads at the next click
(`docs/GROUPS.md` §6.3). The simulation carries both now —
`Sim::add_move_facing_order` puts the formation's bearing plus the slot's
packed byte into the order, and `MoveOrder::facing` carries the mirror out
with it — so what is left of this seam is the per-frame follower, not the
order. **The `pathed` argument left it too, 2026-08-26**: the group's
adders pass 1 (the table in §1.3), and now that `Group::action_move_near`
plans the whole chain itself (`docs/GROUPS.md` §6.7) the simulation passes
it too — so a group's move order is born with `flag::PATHED` set and a
stack under it, and `do_move` no longer re-plans it a frame later. The `angles[i] << 24` term is still all-zero in every run on disk (a
Line leans nowhere), so the byte's **sign** rides on the listing alone
(`docs/GROUPS.md` §13).

---

## 9. The start of a game

### 9.1 The sequence — `Game::run@00584590`

`GameLog::init`; `game_random->random_seed = info.seed` (the lobby seed —
`SEED` in `-config`, `Seed` in `rise.ini`); **`Game::init_rules_and_teams@
00589bb0`** — inside, in order: randomised `starting_technology == 9`,
`Map::set_map`, **`init_starting_resources`** (§9.4), `Leaders::init`,
`num_nations/num_players`, `init_teams`, `init_handicaps`, `init_tribes` (a
random tribe is `rand % 24`, from `game_random`); `init_console`,
`init_type_backup`; **`Game::init(0, is_scenario, 0)`** → `init_data`,
`World::init`, **`Setup::build_game`**, then `Setup::reinit_game_units` (every
alive object's `z` re-read from the terrain); `clock_init`, `solo_init`
(hotkey housekeeping); `playing = 1`; **`GameLog::begin_game`** — the `[Start
Game]` dump at `frame == 0`, **the state after `build_game` and before any
`do_frame`**; then `Game::loop` → `do_frame` (§2.1). There is no simulation
pass between `build_game` and `begin_game`.

### 9.2 `Setup::build_game@005ac190` — the world, then one empire per player

`random_seed = info.seed` again; **`frame = tick = 0`**; `world_pop/cities/
villages/total_units = 0`, the market tables (`market[k] = 0x32`, flux 15);
`Objects::clear`, `Groups::clear`; `world->seed = rand % 0xffff + 1`. The
random-map path: `Map::new_map` → `game_map->make(num_nations, …)` (the
map-maker places the start cells into `world->start_x/start_y` with
`game_random` draws, at least 5 then 3 cells from the edge), `Terrain::init`,
`mark_forest`. **The start-slot order**: `Game::start_list[k]` is the player
indices — a random permutation when the map randomises starts (`rand & 7`,
re-drawn until fresh), else identity; `Leader::init(who, tribe, who)` for every
active player; then **`build_empire(who, slot)`** for each, `start_index[who]
= k`; then `World::analyze_map`, **`compute_all_territory`**, `Herd::
create_units`, scripts. (The scenario path runs `ScenarioRead::import`
instead; CtW its own setup; out of scope.)

**Second reading (`docs/audit/2026-08-21-orders.md`, S1, S5).** Two
additions. `build_game` **re-seeds `game_random` from `info.seed`** at its
head, so `init_starting_resources` and the tribe roll (inside
`init_rules_and_teams`, §9.1) and the map, the permutation and every
`place_unit` draw are **two separate walks of the same seed**, not one
stream. And in a team game, unless `world +0x30 == 0x16`, **each player's
teammates are placed immediately after it**, at consecutive start
positions — which start cell a player gets is not `start_list` order alone.

**Counted (2026-08-24, run11's checksum trace — `docs/ORACLE.md`, "The
setup path's checksum trace is the RNG state").** On the harness's lobby
(seed 12345, Great Lakes, Small, two players): the random nation 1 draw;
the world seed 1; the map maker 3 + 5,568 + 1,016 + 3 + 1,824 + 231 +
2,920 inside `Map::make` and 173 in `make_rivers`, none in `Terrain::init`;
**the start permutation 8**; the AI's `Leader::init` 20 (the personality)
and the human's 0; **the two `build_empire`s 1,293**; `Herd::create_units`
44; nothing after. The state entering frame 0 is the trace's last record,
and `build_sim` installs it rather than modelling the walk.

**`build_empire@005abb80(who, slot)`**: `city = build_cities(who, slot,
starting_town)`, then `build_units(who, slot, city, starting_town)`; Spanish →
`leader_flags |= 0x1000`. **`build_cities@005ab910`**: `home_reg` = the start
cell's region; `starting_town == 0` or a CtW nomad → `−1` (no city); else
`o = find_free(who, 2000, 3000, &build_mark, −1)` (= 2000 on a fresh table),
`snap_center` of `VILLAGE` at `(start_x × 0x300, start_y × 0x300)`, **`Build::
init(who, VILLAGE, o, x, y, 0)`** (Chinese with the bonus → `TOWN`), **`Build::
activate(0, 0, 0)`** (`captured = 0, announce = 0, counted = 0`, `docs/CITIES.md`
§4) — **the starting city is a finished, active city at frame 0**;
`cities_built++` by hand; `compute_all_territory`; `starting_town == 3` →
`large_city_buildings`, `== 2` → `small_city_buildings`, then
`build_civ_specific`.

**The pre-placed buildings — `small_city_buildings@005aae10(who, city)`**,
each a `Leader::produce_building(leader, type, city.o, 0)`, in this order:
`WOODCUTTER`; then **Americans** with `AMERICANS_STARTING_FARMS > 0` → that
many `FARM`, else **Lakota** → none, else **three `FARM`**; then `LIBRARY`.
So the default is **`2001` woodcutter, `2002–2004` farms, `2005` library**.
`large_city_buildings`: `WOODCUTTER`, five `FARM` — **none for Lakota**, and
no American override on this list — four `TOWER`, `LIBRARY`. `build_civ_
specific@005ab760` adds a `MARKET` for Nubians or Dutch, `UNIVERSITY` for
Greeks (`GREEK_UNIVERSITY_EARLY > 1`), `TEMPLE` Koreans, `GRANARY` Egyptians,
`LUMBERMILL` French, `SENATE` Iroquois, each behind its power constant.

**Second reading (C2, C3).** The small list's tests nest Americans *outside*
and Lakota *inside* — no tribe holds both powers, so the order is not
observable, but the code's is as written above. The Large Town's five farms
are inside the Lakota exemption, which the first reading omitted (its §9.3
already subtracts the matching five citizens).

**Who chooses the site: `Leader::produce_building@006e1400` — the AI's own
placer, with `frame == 0` special cases.** Not the map-maker, not an offset
table. In shape: the anchor is the city's tile; a spiral over `circle_x/
circle_y[1 .. circle_radius[(get_radius(city) + 2) / 4])`; per cell in bounds,
`wdata.flags & 0x4000` clear, `check_building_wcoord`, `buildings_allowed`,
`is_ocean` matching the type's domain, `blocked_site` at the snapped centre
(`docs/CITIES.md` §2.6); **score: `FARM`/`MINE` → `find_friends` adjacent
same-type count `n` → `(n+2) × 1000`, else `4000/dist + rand % 500`** (a
sync-stream draw per candidate); **`WOODCUTTER` at frame 0: 1000 if `dist ≤
3`, 500 if 4, 250 beyond, times the cube of the forest count `blocked_site`
returned**; best wins; a second pass jitters within a 2 × 2 box with
`rand % 100` per unblocked sub-position (the woodcutter instead scans for the
most forest, no draw). **At frame 0 the creation is `Objects::init_build(who,
type, x, y, 0, −1)` then `Build::activate(0, 1, 0)` — the starting buildings
are complete and active, not construction sites.** (`frame != 0` is the AI's
live path: pay, `init_build`, pick the nearest idle citizen, `action_swarm_
around(BUILD_AT)`.) **The starting sites depend on the seed and the map; the
harness takes them from the dump and does not reimplement the placer.**

### 9.3 `Setup::build_units@005aafc0(who, slot, city, starting_town)` — the units and their orders

The start tile `(sx, sy) = (start_x[slot], start_y[slot])`, centre `(sx ×
0x300 + 0x180, …)`. **How many citizens**: `get_starting_citizens@005aaf50` is
a four-way jump table the decompiler could not recover; the PE bytes at
`0x5aaf50–0x5aafa6` settle it: `(n, ordered)` = **`(3, 0)` for Nomad, `(4, 0)`
for a city only, `(5, 2)` for a Small Town, `(10, 5)` for a Large Town**;
`starting_town ≥ 4 → 0, 0`. Then `starting_resources == 7 → n += 8`; `== 8 → n
+= 12`; Americans with `AMERICANS_STARTING_FARMS > 3` and a Small Town → `n +=
farms − 3`; Lakota `−3`/`−5`; Koreans `+ KOREAN_CITIZENS[0]`.

**The scout first** (`starting_town != 0`): `SCOUT` (or the tribe's graft),
`current_upgrade`, **`place_unit(who, city, type, centre)`**; Spanish:
`SPANISH_EXTRA_SCOUT` more, **plus one further scout when `info.reveal_map >
1`**; Dutch: two `MERCHANTDUTCH`; Greeks (and only with a city):
`GREEK_START_SCHOLARS` scholars, each `go_inside` the nearest own
`UNIVERSITY` if one exists. **`place_unit@005abca0`** — the scattered
placement: up to 60 tries (500 for a Nomad): `idx = rand % circle_radius[r]`
(a sync-stream draw per try) with **`r = 2` on a start with a city, and on a
Nomad start `r = 8` for the first unit, `r = 0x18` (24) once
`LeaderData::active != 0`**; the cell = the city's tile + `(circle_x[idx],
circle_y[idx])`; accept if in bounds, **same `region` as the anchor**,
`wdata.flags & 0x30 == 0`, land, `flags & 0x100` clear, **`wdata.down < 0`
(no object on the cell)**, the `tdata` word's bit 14 clear and low two bits
not 3, `flags` bit 15 clear → `Objects::init_unit(who, type, cell centre,
−1, −1, −1)` — **and return, with no `come_out`**. After 60 failures: no
city → `init_unit` at the point + `come_out(0)` (the only `come_out` path);
a city → `Build::train(city, type)`. **No order is given**: the scout, and
every Nomad/City-only citizen, starts idle.

**Second reading (U4, U6, U8, U15).** The ring radius above is the
correction: the first reading gave `circle_radius[2]` for every start, which
is right with a city and wrong for a Nomad. `come_out` is the exhausted
no-city path only. And under a Nomad start (`starting_town == 0` with no
city) **`has_tribe_bonus` returns 0 for every power** — a Nomad Greek gets no
scholars, a Nomad Spaniard no extra scouts, a Nomad Korean no extra citizens.

**The citizens**, `n` of them, index `i`, farm cursor `k` from 0:

- `starting_town < 2` → `place_unit` (scattered, idle).
- `starting_town ≥ 2`, in four steps, the first match winning:
  1. **`i < ordered` → `2001` unconditionally** — the woodcutter, with *no*
     type or capacity test; the only gate is `2001 < build_mark` and the
     object's alive bit at creation. **A dead `2001` sends the citizen to
     `place_unit` (idle), not to the farm scan.**
  2. `i ≥ ordered` → scan `t = 2002 + k, …`, stepping while the slot is
     alive and not a `FARM`, and stopping at the first `FARM`, the first
     dead slot, or `build_mark`; the cursor advances past it.
  3. **The stopping object** — the farm, or, when the scan found none, the
     last building it looked at — is accepted if it `is_active`,
     `is_gather_type` and is **not** a `UNIVERSITY`. That last guard is
     live, not vacuous: `UNIVERSITY` *is* a gather type, and the Greek one
     is produced last, so a citizen beyond the farm count (Koreans,
     `starting_resources == 7`) would otherwise be put to work in it.
  4. Otherwise `2001` again, behind the full four-way guard (`is_active`,
     `is_gather_type`, not a `UNIVERSITY`, `num_gatherers < gather_max`);
     else `place_unit` (idle).

  **For a target `t`: `o = Objects::init_unit(who, citizen, t.x, t.y, −1,
  −1, −1)` — created at the building's centre; `Unit::come_out(u, 0)` —
  steps off the footprint to the nearest free spot beside it (§5.8, §10);
  then, unless `starting_resources == 8`, `Unit::add_gather_order(u, t,
  QUEUE_NEW, 0)`** — which also `Build::add_gatherer`s it at once (§6.2).

  **Second reading (U10, U11).** Steps 1 and 3 are the corrections. The
  first reading read step 1 as "2001 if alive, else scan", which would send
  a citizen to a farm when the woodcutter is dead; and it read step 3 as
  applying only to a farm found, making the university guard look vacuous.
  Both readings got the *default* Small/Large outcome right — the last
  building on those lists is a `LIBRARY`/`SENATE`/`TEMPLE`, none a gather
  type, so the fallback is what fires — and both misdescribed the rule.

So for the default Small Town: citizens `1, 2` are created at the woodcutter
`2001` with `GATHER 2001`; `3, 4, 5` at farms `2002, 2003, 2004` with a `GATHER`
on each; the library `2005` and the city `2000` get nobody; Large Town: `1..5`
on the woodcutter, `6..10` on farms `2002..2006`. **That gather order is the
only thing in each citizen's list at frame 0**, and its first step (§6.3,
§6.4) is what moves the dump's citizens on frames 2–4 without anyone issuing
anything. The AI's scout carries no order; it moves because the AI orders it
on an early frame — an AI order, not a scripted start; `ai off` in the
console would pin it still.

**Second reading (F4) — the mechanism, corrected.** The first reading
attributed the scout's first move to `Leaders::strategy_all` on frame 0.
`strategy_all` does run then, and `Leader::plan_strategy` does skip its
cadence early-out at frame 0 and run the full sweep — but that sweep
**issues no order** (it contains no `action_*` or `add_*_order` call; it ends
in `check_orphaned_buildings`, `compute_sites(0)`, `production_step = 1`),
and the decision half `production_ai` only begins at frame 1, behind its own
human gate. The scout's first order comes from the **idle path**,
`Unit::think → think_scout` (§2.4). The observation stands; the attribution
does not.

**The id scheme.** `Objects::clear@0065d740` sets per player `unit_mark = 0`,
`build_mark = 2000`, `wall_mark = 3000`; `Objects::find_free@0065ad60(who,
base, limit, &mark, want)` with `want < 0` reuses a dead slot in `[base, mark)`
or allocates at `*mark` and increments. `init_unit` calls `find_free(who, 0,
2000, …)`, `init_build` `find_free(who, 2000, 3000, …)`. On a fresh table the
first unit is `o 0`, the first building `o 2000`, per player — scout `0`,
citizens `1..5`, city `2000`, the five buildings `2001..2005` in production
order. `init_unit` creates `uber_size` `Unit`s per call chained by
`o_up/o_down`; the captain is placed exactly at `(x, y)` by `Unit::init`
(`set_new_location`), followers `find_nearby_spot`-placed; each fresh
`Unit::init` does `close_orders(0); clear_partial_path; update_action` — **a
fresh unit's order list is empty**. A citizen's `uber_size` is 1.

### 9.4 `Game::init_starting_resources@0058a500`

For each of the six goods: `starting_resources == 12` → replaced by `rand %
12` first; `base = Constants::starting_goods[i]` (`+0x234`), or
`starting_goods[0]` for every good but **knowledge** when the setting is 7;
the lobby
row `starting_resources.list[setting]` gives `lo` (`+0x3c`) and `hi` (`+0x40`):
`lo == 0 → base / 2`; else `lo × base + (span × base > 1 ? rand % (span × base)
: 0)` — **a sync-stream draw per good whenever the row has a spread**; setting
8 → 99999. `Leader::init@006e3930:881–897` pays it: `bucket_add(t, starting[t])`
for every `type_avail` good; Barbarians' defenders `× (starting_resources2 +
1)`; a CtW nomad `× ctw_nomad_starting_res_x`; then **Persians** scale FOOD by
`(PERSIANS_BONUS_FOOD + 100)/100` and **Greeks** with `GREEK_DELAY_KNOWLEDGE`
have KNOWLEDGE zeroed. The lobby's start *age* is `docs/TECH.md`'s.

**Second reading (R2, R3).** The exempt good under setting 7 is index **3,
`KNOWLEDGE`** (`if (setting == 7 && i != 3)`), not wealth (index 2) — the
first reading named the wrong good. The constant is
`ctw_nomad_starting_res_x`, and the Persian and Greek terms were missing.

---

## 10. The spatial queries — `find_nearby_spot`, `invalid_loc`, `adjacent_to`, `covers_tile`

Every walk an order makes ends at a point `UnitType::find_nearby_spot@0061de70`
returns, and the function is **deterministic and RNG-free** (no `Random::get`
on the path, nor in the collision helpers). Settled in the disassembly where
the decompiler dropped the angle arithmetic and the `project` call.

**`find_nearby_spot`** — `this` is the unit *type*; returns **0 = found**
(`out` = the spot) or 1 (`out` = the input point). The parameter names are
the original's, from `llvm-pdbutil dump --symbols` (R7 N2):
`(x, y, to_x, to_y, min_radius, max_radius, radius_step, bias_angle, filter,
not_o, not_who, ignore_units, uber_unit, ox, whom, reg)`. Note that
**`Unit::find_nearby_spot` drops its `ox`/`whom`** — `come_out` passes a pair
into those dead slots (R7 N13).

- **Defaults**: `(min > 0 && max == 0) || max < 0` → `max = min + 4 ×
  big_radius` (`min + 0x240` when `big_radius == 0` and not `unit_flags &
  0x10`; a **squad** placement — `uber_unit != 0` — is instead
  `min + 0xc0 + 4 × (((uber_size − 1) × guy_spacing) / 2 + big_radius)`,
  which the first reading understated as "half a formation": it misses both
  the `× 4` and the `+ 0xc0`. R7 N3); `max < min → max =
  min`; `step ≤ 0 → (max − min) / 8` (truncating), at least 1. `min == 0 &&
  max == 0` is *not* a default: one ring of radius 0, one candidate — "is
  this exact point free", how the swarm's `+0x30` nudge is re-validated.
- **The candidate sequence**: rings `r = min, min + step, …, max` (the last
  clamped to `max` and searched); on each ring with `r > 0`, **31 bearings
  in the order `k = 0, 1, −1, 2, −2, …, 15, −15`** at `angle + k/16 turn`,
  plus a thirty-second when `|k| ≥ 8` — the fifteen sixteenths `0, ±1/16 …
  ±7/16`, then the odd thirty-seconds with `17/32` twice, `1/32` never and the
  direct opposite never; the point is `(x + sinx(a, r), y − cosx(a, r))`
  (`docs/MOVEMENT.md`'s table), **snapped to the centre of its 48-unit
  quarter-tile** (`div3(v >> 4) × 0x30 + 0x18`). A ring with `r == 0` has the
  one candidate `(x, y)`. The first candidate that passes wins.
- **The test, in order**: on the map; `region` (if asked) — the cell's
  `WData.region`, or `region2` for an ocean tile of a coastal cell; not a
  `0x4000` tile (air exempt); **not on `bo`'s footprint** (the corner exactly
  as `tile_corner`; a FARM is exempt for a citizen type — a peasant may stand
  anywhere on a farm); terrain class by domain — sea needs ocean (a warship
  also `!(T & 0x2400)`), land needs *not* ocean (**forest, road, mountain,
  cliff all pass here** — `invalid_loc` is the stricter predicate, and
  `find_path` later teleports a unit off a tile it rejects); collision —
  `nocoll` accepts anything; else `FILTER_NOT_ME`/`CAN_COLLIDE` with a real
  `(o, who)` — **the pairwise pair, `docs/COLLISION.md` §5.2** — use
  `Objects::find_collision` (land: `CollCheck::collide_here`
  with `new_block_radius` in quarter-tiles) **and** `find_ordered_collision`
  (a 9-cell walk against other units' `orders_x/y` — a spot another own unit
  is walking to is taken); any other filter uses `find_unit_with_radius`
  (reject iff `vector_dist ≤ other.big_radius + r_coll`) and the own-player
  ordered variant. A building's `new_block_radius` is 0, so only `0x4000`
  tiles and the footprint test keep units off it.
**The ocean the ring could stand in (2026-09-01).** The terrain-class test
above was **not implemented** — `find_nearby_spot` asked `World::accepts`,
which is a cell-bounds test the sweep's own bounds check has already made —
so a land unit was free to take a spot on an ocean tile. It cost nothing
until a swarm ring was drawn around a *coastal* site. run56 frame 2977:
player 1's citizen `o 11` is sent to build the AI's Dock at
`(44160, 41856)`, `R = 4 × 0x60 + 0x30 = 0x1b0`, base bearing
`find_angle(me − site) ≈ 311°`; the sweep's `k = 0` is refused by the
footprint and `k = +1` lands on tile `(228, 215)`, whose mask is `0x420` —
surface `0x20`, ocean. The original refuses it and takes `k = −1`, one
bearing later in the `0, 1, −1, 2, −2, …` order. With the test in place
this crate takes `k = −1` too, and unit `1/11` walks the original's own
line for the whole of run56. `docs/AI.md` §21; the reason it took until now
is that no capture on disk had ever put a swarm ring on a coast.

- **The `(-1, -1)` form** (2026-09-01): `Unit::do_cast` and
  `SpellType::cast_transport` are the two sites that pass `FILTER_NOT_ME`
  with `not_o = not_who = -1` — nobody is exempt, and the block, the
  domain and the radius defaults are the **type's** rather than a unit's.
  The bias is `0x55555555` and the radius `constants.unit_board_distance`,
  and what they are asking is "is there water a barge could be born on"
  (`docs/TRANSPORT.md` §6.1). `find_nearby_spot_type` in `orders.rs`.
  **And its collision half is not the pairwise pair** (2026-09-02): the
  flag that selects `find_collision`/`find_ordered_collision` is set at
  `0061deb0` only when `not_o` and `not_who` are *both* non-negative, so
  this form falls to `find_unit_with_radius` — reject iff `vector_dist <=
  other.big_radius + r_coll`, with `r_coll` the asking type's `+0x240` —
  and its ordered sibling is skipped outright because that one is guarded
  on `not_who >= 0`. `docs/COLLISION.md` §5.2.1 has the predicate and what
  reading it as the pairwise pair cost: East Indies' long word, 5819 →
  **6164**.
- **The base bearing** at every build/repair/gather/garrison call site is
  **`find_angle(me − target)`** (asm-confirmed at the swarm, garrison and
  gather sites): the sweep starts on the unit's own side of the target.
  `come_out` uses south (`0x80000000`), `find_path`'s teleport and `go_to`
  `0x55555555` (a third of a turn — arbitrary, not a sentinel), `init_unit`
  the unit's own angle. **`Animal::do_idle`'s far wander is `0x55555555`
  too** (2026-08-31, item 102), which is the one place the difference has
  been measured: reading it as the animal's facing put run39's `8/3` 180°
  out and cost 125 frames of East Indies' word — `docs/SYNC.md` §3.19,
  `docs/ANIM.md` §7. Where a call site's bearing is a literal it is worth
  reading as one; the natural reading is the wrong one at three of the six
  sites here.

| caller | centre | min | max | step | `bo` |
|---|---|---|---|---|---|
| `action_swarm_around` (build/repair/gather approach) | the site | `min(x_size, y_size) × 0x60 + 0x30` (halved for a FARM under `BUILD_AT`) | 0 → default | −1 → `/8` | the site (footprint refused) |
| the `+0x30` nudge | the nudged point | 0 | 0 | 0 | the site |
| `do_garrison` | the target | `size × 0x60 + 0x30` (a wider `+0x1b0..+0x330` ring for one type — the `is(…)` literal is lost) | −1 | 0 | — (then retried with `nocoll 1`) |
| `do_gather` | the building | `size × 0x60 + 0x30` | −1 | 0 | — |
| `do_non_flat_gather`: the camp; the tile | the camp; the tile centre | `d`; `0xc0` | −1; `0x100` | 0; `2` | — |
| `come_out`, no host | own position | `block_radius` | `block_radius + UNIT_DISEMBARK_DISTANCE` | 0 | — (then `nocoll 1`) |
| `come_out`, **unit** host | the host's position | the **host's** `block_radius` | that `+ UNIT_DISEMBARK_DISTANCE` | 0 | — (then `nocoll 1`); bearing is the **host's** `angle`, not south (`docs/TRANSPORT.md` §6.4) |
| `come_out`, building host | the host's position | the training ring, `0` while it dies | `+ (MAX − ) DISTANCE` | 0 | — (`docs/CITIES.md` §11) |
| `Animal::do_idle`, the far wander (`docs/ANIM.md` §7) | the herd centre | `0xc0` | −1 | 0 | — |

**`UnitData::invalid_loc(tx, ty, terrain_only, ignore_unseen,
ignore_buildings, pathfinder, ignore_domain)`** — the PDB's own names
(R7 V1) — a *tile* predicate,
**0 = valid**, 1 off the map, 2 terrain, 3 a warship on a `0x2400` tile, 4 a
`0x4000` structure tile (an enemy structure's tile counts as valid for an
armed unit under `c`). The five flags, by effect: `a` skip the structure test
(the "my own tile" checks); `b` fog-respecting for humans (the pathfinder);
`c` the enemy-structure rule; `d` the pathfinder's mode (a transport may
cross land, a `unit_masks & 0x800000` land unit may cross ocean, forced by
the path top's flag 4); `e` the same two gates without the rock/mountain
cell rule. A land unit gets 2 on ocean, forest (unless `unit_masks2 &
0x4000`), mountain and cliff tiles; **on an open flat map away from water,
forest, mountains and buildings it is always 0.**

**`Object::adjacent_to(o, who)`**: both active; `attack_dist(o, who, my x, y)
< 0x60` — 96 units, half a tile, edge to edge on the quarter-tile grid
(`docs/COMBAT.md` §13.1) — except a caller that is sea-domain, **`is_unit()`** and without
`unit_flags & 0x10`, which uses `BOAT_GARRISON_MAX_DISTANCE` (`+0x180` if
its `new_block_radius < 4`). The `is_unit()` gate is what keeps a
**sea-domain building** — a Dock is domain 1 — on the ordinary `< 0x60`
rule; without it the first reading's "sea-domain caller" would have loosened
it (R7 D2). **`WallData::covers_tile(tx, ty)`**: `tile_corner`
and `cx ≤ tx < cx + x_size && cy ≤ ty < cy + y_size` — `docs/CITIES.md` §2.2.
**`Unit::detect_unit_collision(x, y, …)`** returns 0/1 from
`CollCheck::collide_here` at `new_block_radius` then an order-aware 9-cell
scan; a handful of well-spaced units on open ground never trip it.

A correction for `docs/ATTRITION.md`: `WData.down/down_who` is the head of
the per-cell object chain (`ObjectData +0x2c/+0x2e` links) that every
collision query walks — a spatial index, not a territory claim.

---

## 11. What the gamelog writes

### 11.1 The order list, at `UNITS=3`

`UnitData::log_data@0060b070` opens detail 3 after the object base, writes
`angle, dest_angle, trench_angle, tolerance, orders_x, orders_y, …, idle,
unit_masks, unit_masks2, path_recursion, gather_down, …`, and at its end
`Stack<PathData>::log_data(&path)`, **`OrderList::log_data(&orderlist)`**, then
the guys. No order's `log_data` calls `set_detail`, so **the whole order list
appears at `UNITS=3`** and nothing of it below.

```
   BEGIN STACK<TYPE>              Stack<PathData>::log_data — only when length != 0
    size 10
    length 3
    increment 10
    BEGIN PATHDATA                 bottom of the stack first
     to_x 45024
     to_y 19680
     tolerance 0
     flags 1                       <- the goal (final), pushed first
    BEGIN PATHDATA … tolerance 384 flags 0
    BEGIN PATHDATA … to_x 45048 to_y 18168 tolerance 384 flags 0     <- the top = current waypoint
   length 1                        OrderList::log_data: the list length (no BEGIN of its own)
   type 3                          per order: get_type()
   metric 0                        the node's metric byte
   BEGIN EXPLORETOORDER            the order's own log_data (EXPLORETOORDER/FLEETOORDER wrap MOVEORDER)
    BEGIN MOVEORDER
     BEGIN UNITORDER
      flags 1
     x 45048
     y 19704
     angle 1913782272
     dest 0 … tolerance 0 pause 0 retry 0 attempts 0 timer 0 facing 0
     dest_x 45048 dest_y 19704 last_x -1 last_y -1 coll_x 0 coll_y 0 orig_x 45024 orig_y 19680 off_x 504 off_y 504
   length 2                        <- PtrArray<Guy>
```

**The orders are listed newest first — the last block printed is the order
being executed.** (`OrderList::log_data@00730070` positions the cursor on the
tail and steps `next` before printing each; `walk_data` goes the other way.)
Six reader traps. **No block is ever closed by an `END` line** — `Log::end`
emits only for a non-empty name and every order passes the empty string, so
the nesting is *indentation only* (R7 L5). **The array templates disagree on
key order**: `Stack<PathData>` writes `size, length, increment`, while
`PtrArray<Guy>` and `SimpleArray<Coord>` write `length, size, increment`
(R7 L7). **Six kinds emit `UNITORDER` twice** — the multiple-inheritance
orders call both bases' `log_data` and both chains end at the shared virtual
base (R7 L13). And, as before: an empty stack writes only
`BEGIN STACK<TYPE>` and the next `length` line is the order list's; `STACK<TYPE>` is a template name shared
with every `Stack<T>` (the guys' `size/length/increment` follow the orders);
and each `log_data` closes its block before the parent writes its own fields,
which the text shows only by indent — `ox/whom/uid` sit one level shallower
than `flags`, so **a field shallower than the open block's own indent closes
it**; `docs/DATALAYER.md`'s "a field belongs to the innermost open block
regardless of indent" mis-files them under `UNITORDER`.

The blocks, by class (names upper-cased except `GroupMoveOrder`,
`GroupAttackOrder`, `GroupPatrolOrder`, which the binary keeps mixed).

**A seventh reader trap, and this one had teeth.** The mixed-case three
are the only order blocks whose names do not end in `ORDER` in caps, so a
case-sensitive `ends_with("ORDER")` walk drops them — and because the
`type`/`metric` pair before each body is matched to it **by position**,
dropping one body slides every later `type` onto the wrong order. Nothing
caught it for five months because no dump had a `GroupMoveOrder` in it;
run31 does. `crate::gamelog`'s walk upper-cases the name now, and
`a_group_move_order_carries_both_bases_and_names_its_leader` fails on the
old form.

| block | fields (after its bases' blocks) |
|---|---|
| `UNITORDER` | `flags` |
| `TARGETORDER` | `UNITORDER`, then `ox whom uid` |
| `THINKORDER`, `BUILDORDER`, `REPAIRORDER`, `BOARDORDER`, `AWAITBOARDORDER`, `FOLLOWORDER`, `ATTACKTOORDER`, `EXPLORETOORDER`, `FLEETOORDER`, `GROUPATTACKTOORDER`, `AIRPATROLORDER` | label only, then the bases |
| `MOVEORDER` | `UNITORDER`, then `x y angle dest tolerance pause retry attempts timer facing dest_x dest_y last_x last_y coll_x coll_y orig_x orig_y off_x off_y` (read back from the PE at the cited addresses; the dump's order) |
| `ATTACKORDER` | `TARGETORDER`, then `mandatory defensive in_range ever_in_range new_ord def_x def_y` (~~no dump has one yet~~ **181 of them in run29**, 2026-08-26, and the row is exactly as read back from the PE — the parser reads all seven and `a_units_3_record_is_read_whole_orders_included` pins them) |
| `GATHERORDER` | `TARGETORDER`, then `tx ty build_type wait goto_build non_flat_gather dist_mod been_there` |
| `GARRISONORDER` | `TARGETORDER`, then `search` |
| `GUARDORDER` | `TARGETORDER`, then `dx dy guard_x guard_y idle retry` |
| `ATTACKGROUNDORDER` | `UNITORDER`, then `att_x att_y accuracy attack_unit` |
| `GROUPORDER` | `UNITORDER`, then `oxx whose group_angle id form_id` (~~spelling open~~ **confirmed by run31**, 2026-08-26: the row is exactly this. `oxx` is the **leader's object**, `id` is shared by every member's order, and `form_id` is the member's index into the group's own parallel arrays — `docs/GROUPS.md` §12.1) |
| `GroupMoveOrder` | `MOVEORDER`, `GROUPORDER`, then `in_group`. **Observed in run31** — the order §6.6 step 6 adds, which no earlier dump held. Its `MOVEORDER` base carries the member's **slot destination** in `x`/`y` and the **click** in `orig_x`/`orig_y`, which is what makes `docs/GROUPS.md` §6.4's `to`/`off` asymmetry readable |
| `PATROLORDER` | `UNITORDER`, then `x_pos`/`y_pos` as **flat key lines with no `BEGIN`** (`length, size, increment, flags`, then one `list[scan]` line per entry), then `waypoint`. `SimpleArray<Coord>::log_data` writes **nothing at all** for an empty array, so an unstarted patrol looks like one with no arrays (R7 L11) |
| `FORMORDER` | `MOVEORDER`, then `newform delay` |
| `CASTORDER` | `x y paid spell`; `TRADEORDER` `oxx whose started loaded uid2`; `AIRORDER` `oxx whose cruising_alt sharp_turn old returning`; `SPECIALANIMORDER` `type started frames data1..data4 ox whom`; `STRAFEORDER` `xx yy` |

A worked block (the AI's builder, frame 2): `length 2 / type 6 / metric 0 /
BUILDORDER { TARGETORDER { UNITORDER { flags 4 } ox 2006 whom 1 uid 12 } } /
type 3 / metric 0 / EXPLORETOORDER { MOVEORDER { UNITORDER { flags 1 } x 41784
y 15816 … } }` — the explore-to is current, the build order behind it.

### 11.2 The building side

`BuildData::log_data@0062e810` writes, **by detail tier**: 4 `city` and
`GATHER_DOWN` (the chain head); 6 up to `orig_type`; and **7**
`MiningList::log_data` (`gather_from` — the wood/ore tile list), the
`GATHERPOINT` list and `BUILDQUEUE`. One parser trap: the `GatherPoint`
list's `type` key is a **hardcoded literal 0**, not the element's
`get_type()`, so a reader that keys on `type` will read it as `ORDER_NONE`
(R7 L14, L15).

**A `BUILDS=7` dump has now been captured** (`gamelog-run6-ancient-nubian-
builds7.txt`, `docs/ORACLE.md`) and it corrects the shape above. `BUILDQUEUE`
*is* a `BEGIN` block; **`MiningList` and the `GatherPoint` list are not** —
like `PATROLORDER`'s `SimpleArray<Coord>` (§11.1) they write no `BEGIN` of
their own, so what follows `mtn`/`cliff` on `BUILDDATA`'s own field indent is

```text
  length 82        <- the mining list's header, then size/increment/flags
  size 160
  increment -1
  flags 0
  tx 20            <- one tx/ty pair per tile, `length` of them
  ty 146
  …
  length 0         <- the GatherPoint list, empty here
```

Nothing else at that indent writes `tx`, so the pairs are unambiguous;
`rondata::gamelog::BuildDump::gather_from` reads them that way. The tile list
is what §6.4's machine needs and could not have, and with it a woodcutter's
citizen walks: `gamelog-run6`'s `0/1` matches the original's position **and
its whole order list for all 432 logged frames**.

`UnitData::log_data` writes the unit's `GATHER_DOWN` link.
To diff "how many are gathering here" follow `BUILDDATA.GATHER_DOWN →
UNITDATA.GATHER_DOWN` and read each unit's `GATHERORDER.BEEN_THERE`.
`LeaderData::gatherers` and the per-region arrays, and `CityData::gatherers`,
are the **AI's** tallies, not the economy's count — the economy has no stored
count; it walks the chain.

### 11.3 The command stream

`GLOG_COMMANDMANAGER` at detail 1 (`COMMANDMANAGER=1` in `gamelog.ini`):
`process_move_to` writes `to_x to_y queued set_angle angle orders disembark`
and `form width`; `process_attack` writes `ox whom ignore queued frame`. With
it on, a played game's every click is in the text log — the order stream the
harness needs without a recorded-game parser.

---

## 12. Constants

| name | used as | where |
|---|---|---|
| `ACCEL_CONSTRUCT` | the per-builder contribution, 100 | §5.2, `docs/CITIES.md` §3.3 |
| `UNIT_GATHER_RESPOND_RANGE`, `UNIT_BUILD_RESPOND_RANGE` | tiles, `× 0xc0` at use (`× 0x180` for stance 1/2 in the spot searches) | §5.5, §5.9, §6.6 |
| `UNIT_DISEMBARK_DISTANCE` | `come_out`'s ring outer radius | §5.8 |
| `BOAT_GARRISON_MAX_DISTANCE` | the sea `adjacent_to` | §10 |
| `KOREAN_BUILD_UNDER_FIRE`, `KOREAN_REPAIR` | §5.2, §5.6 | |
| `unit_defensive_respond_range`, `unit_guard_respond_range`, `unit_respond_range`, `city_capture_radius` | §7.2, §7.5, §8.5 | |
| `americans_marine_entrench`, `aircraft_heal_rate`, `memnon_regen_rate`, `decoy_time` | `Unit::process`'s upkeep | §2.2 |
| `starting_goods[6]`, `starting_resources.list[].lo/hi`, `ctw_starting_res_x` | §9.4 | |
| `AMERICANS_STARTING_FARMS`, `KOREAN_CITIZENS`, `SPANISH_EXTRA_SCOUT`, `GREEK_START_SCHOLARS`, `GREEK_UNIVERSITY_EARLY`, `KOREAN_TEMPLE_UPGRADES`, `EGYPTIAN_GRANARY_EARLY`, `FRENCH_LUMBERMILL_EARLY`, `IROQUOIS_SENATE` | §9.2–§9.3 | |
| `LeaderOptions +0x8` `peasants_wait` | the idle-citizen delay option (1–5 → 7, 12, 17, 32, 62; default 2) | §5.9 |
| `LeaderOptions +0x4` `peasants`, `+0xc` `buildings`, `+0x1c` the mask | the stance options, all 0 at `init` bar the mask's bits 1 and 3 | §5.10 |
| `MTN_TINY_SIZE` | the miner's `dist_mod` cap | §6.4 |

Literals, all position units unless said: the 48-unit snap (`0x30`, centre
`+0x18`); `0x60` = half a tile (adjacency, the tile waypoint tolerance);
`0xc0` = a tile; `0x180` = the world-cell waypoint tolerance; `0x300` = a
world cell; the planning thresholds `0x600/0xf00/0x1800` (2/5/8 cells);
`find_path`'s 4-cell rule; `0x140` = the working-the-tile radius; `0x120` =
the oil-well stance offset; `0x600` = 8 tiles (the non-flat return; the guard
and follow standoffs); the non-flat timers `300 + rnd%100`, `100 + rnd%50`,
`400 + rnd%200`, 32, 20, 1,000,000; the farm's `0.005f` per grow; the idle
cadences 16/32/64/128/256; the swarm ring `min(xs, ys) × 0x60 + 0x30`, halved
for a farm, nudged `+0x30`; `repaths` limits `500/r²`, `300/r²`, the 16
threshold; the guard's `retry = rnd%3 + 6`, `timer = 0x1e × tiles`; the follow
standoff `clamp(los × 0x180 − k, 0x180, 0x600)`; the group follower's `v/3`
capped 9, the 60° cone `0x55555555`, the `0x900` attack radius.

---

## 13. What `crates/sim` models, and how

`crates/sim/src/orders.rs` replaces three stubs — `Unit::job`, `movement.dest`
as an order, `combat.target` as the attack order — with the thing the
original has (what is listed as modelled is implemented and under test;
what is listed as an input is stated as such in the code):

- **`Order`** — one enum variant per kind this session implements: `Move`
  (with a `MoveKind` of `MoveTo`/`ExploreTo`/`AttackTo`/`FleeTo` and the
  `MoveOrder` fields that are live on open ground: `x, y, angle, dest,
  dest_x, dest_y, last, pause, timer`), `Build`, `Repair`, `Garrison{search}`
  (each a `Target {o, uid}`), `Gather` (`Target` plus `tx, ty, wait, goto_build,
  dist_mod, been_there`), `Attack` (`Target` plus `mandatory, defensive, def,
  in_range, ever_in_range, new_ord`), `Think`; each with `flags` (`PATHED`,
  `ACTION`, `POST`, `RETARGET`). `OrderKind::index()` gives the `OrderIndex`
  value for the log. Guard, follow, patrol, attack-ground, the group orders,
  board/await-board, cast, trade, strafe, air and special-anim are documented
  above and not implemented — each is stated in §14.
- **`do_explore_to`'s tail** (2026-08-31, item 106) — §4.1 has `do_job`
  sending `EXPLORE_TO` to `do_explore_to`, and that function is `do_move`
  **plus a mechanic**: one frame in fifteen, phased by `o`, a captain
  (`o_up < 0`) whose order list still heads with *this* order runs
  `Unit::find_goody_box@005f2540` and may re-aim the whole walk at a goody
  box it has seen. No draw, and `docs/GOODY.md` §7 is the whole of it.
- **`OrderList`** — a `VecDeque<Order>` whose **front is the current order**
  (the ring's `head->prev`); `add(order, QueuePos)`: `Last` → `push_back`;
  `First` → `push_front`; `New` → `close_orders` then `push_back`.
  `kill_current` does the per-kind teardown (the gather chain removal, the
  move's facing carry-over) and pops the front; `close_orders` kills from the
  front until empty; `update_action` walks the plain moves and writes
  `orders_x/y`, `dest_angle`, returning the action's position in the deque.
- **The path stack** — `Vec<PathData>` on the unit; `find_path` is the
  straight-line verifier of §4.6, with the march as the settled third
  reading has it and `invalid_loc` implemented over the tile masks
  (`crates/sim/src/path.rs`). ~~`find_wpath` is the **stub**~~ **The three
  grid planners are real as of 2026-08-23** (`docs/PATHFINDER.md`): a far
  move is planned into the chain of cell centres at order time, a near one
  still leaves `[goal]`, and the RNG-thresholded re-plan branch only draws
  when `find_path` refuses a line — so on open ground no move draws, which
  is the original's behaviour, not a divergence.
- **`do_move`** — §4.4's planning half on open ground (the fresh-move push,
  the waypoint take with both arrival tests, the 48-snap of the destination
  and `angle = find_angle` at order time in `order_move`), the step through
  `movement::move_step`, the arrival facing (`length == 1` or the action is
  a gather), the kill, **the waypoint take's own collision test** (§4.4:
  the probe, the kill under a `GATHER`/`ATTACK`/`BUILD_AT` action, the
  parked collider's `big_radius × 3` tolerance), and
  **`go_around_building` whole** (§4.6.1, 2026-08-26:
  the edge walk, the pick, the one-or-three pushes with their `off % 0xc0 / 2`
  skew, the give-up, and `find_path`'s acceptance with the recursive verify
  and the shore's `flags & 4`), and **the grid the re-plan runs on**
  (`field_0x88`, 2026-09-03, item 204): a unit that has not been colliding
  drops its loose near waypoints and plans on the tile grid, one that has
  keeps them and plans on the 48 grid, and the length a positive return is
  compared against is read *after* the pops, per arm.
  What is not modelled: the waypoint take's
  region check (the turn-in-place before a leg ending in another terrain
  region) and its `TRADE_ROUTE` arms, suspended searches, `resolve_block`,
  the entrench wait.
- **`do_build`/`do_repair`/`do_garrison`** — §5.2, §5.6, §5.7 on the existing
  `do_construct`/`repair_*`/`garrison` seams, with adjacency = `attack_dist <
  96` (replacing the tile-based stand-in), the swarm ring (`ExploreTo` to the
  §10 spot, re-queued in front with the same action bit), `check_build_order`,
  `build_done` with **its own AI arm** and the stance rules (§5.2's note)
  and **`find_build_spot` whole** (§5.5, 2026-09-03: the doubled range on
  worker stance 1/2, `find_builds`' circle over cells with the `0x200`
  region gate, `FILTER_CONSTRUCT`, the caller's own-and-not-under-attack
  pair, the builder tally and the strict-`<` minimum) — `find_repair_spot`
  is the seam left inside it — `come_out`'s citizen/scholar rally rules.
- **`do_gather`** — the chain on the building (`gatherers: Vec<usize>`,
  push-front, the prune), `num_gatherers(arrived, skip_decoys)` — the count
  the economy should read; **`economy::Site::gatherers` is still an input**,
  because a `Building` does not yet name the resource it gathers, so the
  wiring from the chain into `calc_gather` is the next step — §6.3's arrival
  for farms (walk to the centre, `covers_tile`) and the flat/other types (the
  §10 ring), `been_there` and the dirty flag, the oil-well stance, the
  university/platform `go_inside`, the §6.4 wood/ore machine with its three
  draws **given the building's `gather_from` list** (an input — the original
  fills it from terrain the simulation does not model, and a `BUILDS=7` dump
  carries it, so a woodcutter's citizen walks the original's walk for 432
  frames, §11.2), the §6.5 farm stand on the full clock (`farms.rs`: the
  farmer re-targets on the log's frame 102 as the original does; the tile is
  two sync-stream draws, right only on the traced stream),
  and `find_gather_spot` **whole** (§6.6, 2026-08-29): the `tregion` gate,
  the city-crossing rule, `is_gathering_at` as the room exemption, the
  strict maximum from zero, and the cap-headroom numerator off
  `ledgers[who]`. The Dutch `0x640` is arithmetically there and untested —
  nothing pays that bonus yet.
- **`do_attack`** — the order wraps `combat::State`'s target; `fight` is
  called once a frame with `mandatory`; **a recharging unit returns at once
  unless `new_ord`** (the reload gate — previously the stub chased while
  recharging); the chase is an inserted `MoveTo` (`QUEUE_FIRST`, no action
  bit) to the target's position (the straight-line stand-in for
  `find_attack_pos`, as before) and `do_move` kills it when the target comes
  in range; `new_ord`, `in_range`, `ever_in_range` are written; the captain
  hand-over, the `waiting` throttle and the DEFENSIVE post move are not
  (stated). A target the unit finds itself (`think`, `target_opportunity`)
  becomes an attack order in front of what it was doing.
- **Idle** — `idle`'s counter, `think`'s auto-attack on `idle == 1`/32 through
  the existing `find_melee_target`, `think_peasant`'s gate and
  `find_gather_spot`. §5.9's **build arm** — `not a scholar and
  (unit_masks & 0x400 or worker_stance ∈ {1,2}) and find_build_spot()`,
  ahead of the gather search — **landed 2026-09-03**, with §5.10. It had
  been written and taken back out twice: it is `worker_stance` that decides
  who asks, and while this crate wrote a flat 1 every citizen asked, which
  put run69's `1/6` off the original's point on **103**. `Unit::init` and
  `Build::train` between them make an AI's *trained* citizen 0, so the arm
  now asks who the original asks and neither map's word moved.
  `find_repair_spot` needs `FILTER_DAMAGED` and the difficulty gate and is
  still absent.
- **The group orders** — **landed 2026-08-25** as `crates/sim/src/group.rs`
  (`docs/GROUPS.md`), which reads the layer §8 only entered: the record and
  the pool, membership, and `action_move_to`/`action_move_near`,
  `action_siege_attack_to`, `action_attack`, `action_stance`,
  `action_halt`, `Groups::push_group`. §8.4's verdict stands unchanged — a
  multi-unit order is N independent ones — and `Form::compute`'s slot table
  is the declared seam, so every member takes the group's own destination.
  §8.2 was **incomplete**: `action_move_near` has an AI branch keyed on
  `!human && group.army >= 0` and then on the army's `hurry`, read in
  `docs/GROUPS.md` §6.5.
- **Not implemented** (documented above, stated here): `ATTACK_TO` as an
  order kind of its own, `GUARD`, `FOLLOW`, `PATROL`, `ATTACK_GROUND`,
  `GroupMoveOrder` (§8.3, §8.4), board/await-board, cast, trade, strafe,
  air, special-anim; `check_target_path`'s 16-frame re-path;
  `find_nearby_spot`'s **general** collision path — `FILTER_ALL` and a
  squad placement, whose `find_unit_with_radius` circle is
  `docs/COLLISION.md` §9's last entry (~~the collision half of
  `find_nearby_spot`~~ is otherwise **done**, item 66: the pairwise pair
  runs as the sweep's last test, and `Sim::find_nearby_spot_coll` names
  which half a call site wants);
  ~~`resolve_unit_collision`~~ (done, items 46 and 64 —
  `docs/COLLISION.md` §6), suspended searches, the
  entrench wait; `come_out`'s rally orders (its placement is now the
  original's two arms, `docs/CITIES.md` §11); `Wall::process`'s AI
  recruiter; the gamelog emission of the
  list (the harness reads the dump's, it does not yet write its own).
- **The start of a game** — **done 2026-08-21** (`docs/DATALAYER.md` §3):
  `rondata::diff::build_sim` adds the pre-placed buildings `2001..`
  (complete and active, typed by the §9.2 production order when the dump
  carries no type) and gives each starting citizen its `GATHER` on the
  building it stands beside by the §9.3 rule (`ordered = 2` on `2001`, the
  rest on successive farms); `check_start_orders` compares every derived
  target against the `GATHERORDER` the original logged, and **all ten
  citizens agree**. `BuildData::log_data@0062e810:269` puts `gather_from`,
  the `GATHERPOINT` list and `BUILDQUEUE` behind `set_detail(7)` — tier 4
  is `city`/`gather_down`, tier 6 stops at `orig_type` — so **`BUILDS=7`**
  is the dump that carries a camp's tile list. The farmers re-target on the
  original's own frames, and the whole farm record is diffed (§6.5).
- **The log** — **done 2026-08-21** (`docs/DATALAYER.md` §3.1):
  `rondata::diff::compare_orders` walks both lists front first — the log's
  reversed, because §11.1 writes it newest first — and the path stacks
  bottom first, and reports the kind, the action bit, the `flags` byte, the
  target's `whom`/`ox`, the stack's depth and each segment's goal. `flags`
  is reported without scoring (`0x8`/`0x10` have no reader); the path stack
  scores (2026-08-23, `docs/PATHFINDER.md` §10).

`Sim::tick` is unchanged in shape: income, buildings, then per unit attrition
→ **the order step** (`work`: the liveness check, `idle`, `do_job`) →
movement. The one-order-per-frame rule and the next-frame rule for inserted
moves fall out of dispatching once on the front at the top of `work`.

---

## 14. What is not established, and the checks

- **§5.10's list paths, and the chain's order inside a cell.** The build
  search always takes the circle at a citizen's ranges and the unit search
  need not, so `find_units`' list path — all units, `vector_dist <= range`,
  and the `div_3_table[pos >> 6]` region compare that indexes the cell grid
  with tile coordinates — is modelled as a plain distance-and-region scan
  rather than reproduced. It feeds a **count that breaks ties** between
  build sites, nothing else. And the circle path's within-cell order is the
  object chain's, which is push-front by `Object::add_to_world`: this crate
  threads only units into `chain_heads` (`docs/QUEUE.md` 48), so
  `find_construct_sites` takes descending slot order there instead. Neither
  has a capture behind it; two of a player's sites in one four-tile cell,
  with equal builder counts, is what would tell them apart.
- **`find_repair_spot` is still absent**, on all four of its arms
  (§5.5, §5.6, §5.9). It needs `FILTER_DAMAGED` and the difficulty gate,
  and it only ever runs when the build search comes back empty.

**Not established — the order system**

- `UnitOrder::flags` bits `0x8/0x10` beyond their writers (§1.3). `0x20`'s
  two readers and `0x80`'s setter are now named (R1); a **reader** for `0x80`
  is still missing, and `0x02` is settled as dead.
- The `0x28b` cast block in `Unit::work` step 5 — not credible as decompiled
  (an unconditional cast on every frame of every action-bit solo move); the
  listing around `0x60d6e0–0x60d710` settles it. Only an action-bit move whose
  target is an object reaches it — not the harness's state.
- `metric` — written 0, logged, never read. If a writer turns up the FIFO
  claim needs the priority re-checked.
- **From the second reading, still open** (`docs/audit/2026-08-21-orders.md`;
  the third pass of 2026-08-23 settled four of the six `FABLE:` rows — see
  that audit's closing section): ~~the order vtable's `+0x50 ↔ +0xcc`
  const-twin spacing (R1)~~ — **closed**, the PDB's own `vftable offset`s,
  §1.1; ~~whether `find_upath` kills the current order on failure (R2 O2)~~ —
  **closed, confirmed as §4.6 states it**: `is_move` (`+0x14`) and the
  `MoveOrder`'s `retry` (`+0x1c`) guard the `kill_current_order`;
  ~~`LeaderData::flags & 4` (R3 F3)~~ — **closed**, `is_human`, §5.5;
  ~~`process_all`'s network padding (R6 48)~~ — **closed, confirmed**:
  `Random::get(0,2)` padding and the seed-derived XOR only under the network
  semaphore bit, from a local `Random`, not the sim stream.
  ~~`BuildTypeData::find_gather_tcoords@0063bdc0` (R4, matters only when the
  harness builds a camp itself)~~ — **closed for the timber branch**, read
  and implemented 2026-09-01: the tile list is re-derived from the map and
  checked against run39's own (`docs/ECONOMY.md`, "The gather list, and its
  shuffle"). Still open: `do_move`'s attack-retarget block (R2 O1, nothing
  in `crates/sim` depends on it), and R4's **metal** branch, which walks a
  mountain range rather than the circle — both still marked `FABLE:` in the
  adjudication with the check that would settle them.
- ~~`OrderIndex` 0 and 5's names; `ATTACK_TO 2` vs `FLEE_TO 4`~~ —
  **closed** (`docs/audit/2026-08-21-orders.md` R1): the enum is in the PDB,
  `--type-index=0x1E22`. `NONE = 0`, `PATROL = 5`, `NUM_UNIT_ORDERS = 28`,
  and every value the table carries is confirmed. No dump needed.
- `UnitData::order_type` on an empty list (a lost jump table; callers guard).
- ~~The `leaders & 4` flag that switches `find_wpath`/`invalid_loc` onto
  `was_seen`~~ — **closed** (R7): `leader_flags & 4` is
  `LeaderData::is_human@006ec170`, so the fog-respecting path is the **human**
  one, as the first reading guessed.

**Not established — the move**

- What `astar_path` pushes on open ground beyond two cells (the chain and its
  tolerances) — the next mechanic; §4.6 says what the stub gets wrong.
- ~~What fills `coll_x/coll_y`~~ — **closed** (R2):
  `detect_unit_collision@00617060` writes them, not `astar_path`. What sets a
  search to `saving` is still inside `astar_path`.
- `MoveOrder::tolerance` (+0x14) has no writer; `MoveOrder::timer` no writer
  for a plain move.
- The group's packed top byte in `angle` on arrival for a single unit.
- The heading-vs-facing question for `docs/MOVEMENT.md` (§4.5).
- The command stream's delay from click to applied frame.

**Not established — build, gather, the start**

- `do_repair`'s lost `is(…)` literal (UNIVERSITY by analogy); `check_build_
  order`'s move-skip rule; `BuildType +0xfc/+0x100` (its stop rule); the
  worker-stance names; `unit_masks & 1` (decoy) and `& 0x400` ("was a
  builder") by bit name; which AI call issues the opening `flags 4`.
- The `UnitAnim` codes `'%' 0x19 0x1d` (they gate `wait` countdowns);
  `Farms::process`; what reads the four
  `unit_masks 0x78000000` bits; `BuildTypeData::max_gatherers`/`calc_gather`'s
  slot count (`docs/ECONOMY.md`'s open item, unchanged).
- `Leader::produce_building`'s scoring line by line (read in shape; the
  harness takes the sites from the dump); `init_teams`/`init_handicaps`
  (skimmed); the lobby labels for `starting_resources` rows 7 and 8;
  ~~whether `Build::activate(0, 1, 0)`'s `counted = 0` skips
  `gather_slots`~~ — **partly closed by the second reading**
  (`docs/audit/2026-08-21-orders.md`, C6): `counted = 0` skips
  `cities_built++` and the whole first-of-kind `do_bonus` block, which is
  *also* gated on `frame != 0`, so no starting building yields a founding
  bonus; `gather_slots` itself was not re-read and stays open. `uber_size`
  for the scout (assumed 1) — data, answered by `rondata`'s loaded `SCOUT`
  row (`UnitType +0x308`) or a `UNITS=3` start dump.
- Whether `OrderList::log_data` reaches the file at `UNITS=3` — confirmed by
  the dumps cited here (it does).
- **§6.6's two unexercised gates.** `is_neutralized` is not modelled at all
  (nothing in `crates/sim` infiltrates), and the Dutch `0x640` is in the
  arithmetic but no run pays that bonus, so neither has ever been executed.
  Both would fall out of a capture with a spy, or with `has_tribe_bonus(0x16)`
  in the lobby.

**Not established — combat and group**

- ~~`Form::compute`/`categorize`~~ — read in shape and a **declared seam**
  (`docs/GROUPS.md` §6.4, §13); ~~`action_move_near`'s leader-path copy in
  detail~~ — read (`docs/GROUPS.md` §6.7: the offset, the area-id guard and
  the follower's `0x600` rule); `do_form_change`; `find_attack_pos` (two
  overloads, called, not read); ~~`GroupData::find_leader`~~ — read
  (`docs/GROUPS.md` §4.4), less `FormData::type_cat`, which is what it
  sorts by; `is_attacking_near`; `pause = 15`'s effect.
- ~~The spelling of the `ATTACKORDER`/`GROUPORDER`/`GUARDORDER` log keys~~ —
  **closed** (R7): read as UTF-16 out of the PE, exactly as §11.1 lists them.

**Behavioural checks, each a logged run (`UNITS=3 BUILDS=6`, and
`COMMANDMANAGER=1` where a click is involved; the recipe in
`docs/ORACLE.md`).** `BUILDS=6` carries everything the eight checks below
need; **`gather_from`, the `GATHERPOINT` list and `BUILDQUEUE` need
`BUILDS=7`** (R7 L14), and the order list needs `UNITS=3`.

1. **The rotation and the cadence.** Select a citizen, shift-click two moves,
   ctrl-click a third: `length 3`, the `type` sequence newest-first shows
   `QUEUE_FIRST`'s rotation; the frame `length` drops shows one order per
   frame, the next the frame after; `idle`'s 0, 1, 2, +1/16 on a unit left
   alone.
2. **Orders at frame 0.** `[Start Game] UNITS=3`: units `1..5` each with one
   `GATHER` on `2001, 2001, 2002, 2003, 2004`, the scout with none; and the
   `resource` console echo at frame 0 against §9.4 for the seed; `ai off`
   before frame 0 pins the scout still.
3. **The far-move chain and the RNG draw.** Order a citizen 10+ cells across
   open ground: the `PATHDATA` entries on the first moving frame (the
   tolerance-384 chain, whether the first step points at the next cell
   centre); two seeds for a move between 2 and 8 cells should switch between
   cell-centre and tile-centre waypoints.
4. **Arrival facing.** One move, no others: `angle` on arrival equals the
   order's `angle`; with a second order queued, it does not.
5. **The stream lag.** `cheat select` + right-click at a known frame; the
   first frame with `dest 1` is the applied frame and the step is on it.
6. **The swarm ring.** A citizen to a site from behind an obstacle: `orders_x/
   y` and the `EXPLORETOORDER` destination against `min(xs, ys) × 96 + 48
   (+48)`; two shift-queued sites, finish the first → `check_build_order`'s
   re-sort and the `flags 4` it writes; a fresh citizen beside an unfinished
   own site with no order picks up a `BUILDORDER` at `idle == 2`; the
   `MOVEORDER` in front of a `GARRISONORDER`, and `search` after a "full"
   redirect.
7. **The gather count and the timers.** Two citizens to one farm: `BUILDDATA.
   GATHER_DOWN` lists both from the issue frame and the food rate steps only
   on the frame after the first `BEEN_THERE` flips; one citizen on a camp:
   `WAIT` 400–599 on the walk-out frame, 32 on the walk-back, the `GOTO_BUILD`
   flips, and the draw order against a seed-pinned combat stream; a farmer's
   frames between `CHAR_SOW` and the first re-target (200 or 201 grows); a
   miner's `WAIT` reads 1000000.
8. **Group and attack.** Two land units right-clicked: one `GroupMoveOrder`
   block per unit with the same `id`, `form_id` 0 and 1, `orders 1` in the
   command line; an attack-move pins `ATTACK_TO` and `ATTACKTOORDER`; a
   right-click attack shows the `ATTACKORDER` keys, `mandatory 1`, `new_ord
   1` then `0` after the first strike; a guard's `retry` in `6..8` after each
   reposition.
