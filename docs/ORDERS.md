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

**How this was established.** Symbol names, struct layouts and field offsets
from `game/sbl/rise.pdb`; behaviour from the decompile export under
`~/ghidra-projects/decomp` (`tools/ghidra/`). Seven readers took one sub-area
each — the list and the frame; the move order and the path seam; build/
repair/garrison and `come_out`; gather and `work`; the start of a game; the
combat and group orders and the command stream; the spatial queries — and
each read its functions in full: `Unit::process`, `work`, `do_job`, `do_idle`,
`check_idle`, `think`, `update_order`, `update_action`, `kill_current_order`,
`close_orders`, `clear_orders`, `add_think_order`, `do_think_order`,
`OrderList`/`LinkListBase` and `OrdersMemManager::*`; `Unit::add_move_order`,
`add_move_facing_order`, `do_move`, `find_path`, `repath`, `move_step`'s
arrival half, `go_around_building`, `resolve_unit_collision` (summarised), the
three `PathFinder::find_*path` wrappers up to their `astar_path` call;
`Unit::add_build_order`, `add_repair_order`, `add_garrison_order`, `do_build`,
`check_build_order`, `build_done`, `do_repair`, `do_garrison`,
`kill_garrison_order`, `go_inside`, `come_out`, `Group::action_swarm_around`,
`Unit::find_build_spot`, `find_repair_spot`, `think_peasant`, `Wall::process`'s
recruiter; `Unit::add_gather_order`, `do_gather`, `do_non_flat_gather`,
`find_gather_spot`, `Build::add_gatherer`/`remove_gatherer`/`check_gatherers`,
`BuildData::num_gatherers`/`is_gathered_by`/`calc_gather`'s count,
`UnitData::is_gathering_at`, `Build::find_gather_tiles`, `WorldData::
has_gather_access`, `GatherPoint*`; `Setup::build_game`/`build_empire`/
`build_cities`/`build_units`/`place_unit`/`small_city_buildings`/
`large_city_buildings`/`build_civ_specific`/`get_starting_citizens`, `Game::run`,
`init_rules_and_teams`, `init_starting_resources`, `do_frame`, `Objects::clear`/
`find_free`/`init_unit`, `Leader::produce_building` (in shape), `GameLog::
check_accept`; `Unit::add_attack_order`, `do_attack`, `fight`'s
order-management half, `do_attack_to`, `do_attack_ground`, `do_guard`,
`do_follow`, `do_patrol`, `add_*` of each, `do_group_move`,
`ungroup_move_order`, `kill_group_move`, `modify_group_order`, `Group::
refresh_group_order`, `distribute_attack`, `action_attack`, `action_move_near`
(the per-member choice in full, the leader-path copy in outline),
`CommandPackage::process_*`; `UnitType::find_nearby_spot`, `UnitData::
invalid_loc`, `Object::adjacent_to`, `WallData::covers_tile`,
`Unit::detect_unit_collision`'s interface. Every `*Order` constructor and
`clear`. Four claims were settled in the PE bytes or the listing
(`llvm-objdump`): `get_starting_citizens`' jump table, `add_move_order`'s
register-passed `find_angle` arguments, the `QueuePos` values in
`add_move_facing_order` (the decompile prints the `QUEUE_NEW` clear as
unconditional; it is not), and `MoveOrder::log_data`'s twenty keys (which also
settle the `this[-1]` offset shift, §1.1). Two real gamelogs at `UNITS=3`
confirmed the list orientation, the log format, the citizen's first moving
frame and a 327-frame build trace. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Status (2026-08-21).** First reading, implementation, blind second reading
and **all seven adjudications** are landed —
`docs/audit/2026-08-21-orders.md`. What is left of the queue item is the
harness work §13's last two bullets name (the start-of-game in `build_sim`
and reading the `UNITS=3` order blocks back), not the reading.

- **The blind side, done.** Seven readers on Opus 5, split the same way as
  the first reading, each given only the entry points and the traps: 355
  numbered claims across R1 the order system (38), R2 the move order (42),
  R3 build/repair/garrison (57), R4 gather (48), R5 the start of a game
  (56), R6 the combat and group orders (64), R7 the spatial queries and the
  log (50). The reports are **not in the repo** (entry 7) — they are at
  `~/ghidra-projects/reading/orders-2026-08-21/`, with a README there
  giving the state and how to finish.
- **The adjudication, all seven**, one per sub-area, each taking every
  disagreement back to the decompiled function, the listing or the PE:
  R1 (A 11 · B 11 · both 15 · neither 2), R2 (A 4 · B 8 · both 3 ·
  neither 1 · open 2), R3 (**A 0 · B 21** · both 39 · neither 4 · open 2),
  R4 (A 5 · B 13 · both 29 · neither 1), R5 (A 0 · B 9 · both 26 ·
  neither 2), R6 (A 6 · B 14 · both 24 · neither 8 · open 1),
  R7 (A 6 · B 13 · both 29 · neither 0 · open 2). Ten corrections landed in
  `crates/sim`, every one re-verified in the listing before it was applied;
  the document corrections are marked inline throughout, each naming the
  sub-area that found it.
- **The three first-reader disagreements are all closed, none needing a
  behavioural check.** `UnitOrder::flags & 4` is the action bit, settled by
  `Group::set_up_insert@0070e520:25` — a reader neither reading had cited
  (R1). The `OrderIndex` values are in the PDB (R1). `do_gather`'s approach
  radius is `min(x_size, y_size) × 0x60 + 0x30`, settled in the listing at
  `0x5ef756` **twice over**, by R4 and R7 independently.
- **What the second reading cost the first.** Two claims the blind side
  raised against this document held: `add_repair_order` really has no
  `QUEUE_FIRST` branch (R3), so §3.1's "one shape, 21 functions" has
  exceptions. One did not: `do_attack` owning no reload logic is **not** a
  disagreement — §7.2 already put the gate in `fight`, and the
  implementation was right where the prose was wrong (R6).

What the implementation leaves as inputs is §13.

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
`go_around_building`'s geometry, `think`'s cadence table (several bodies
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
4. **Workers** (`ObjectData::is_worker`): **`think_peasant(0)`** — §5.9.
5. Caravans → `think_caravan`; rare-goods merchants (AI only) on `idle == 1`
   or every 32 frames → `do_gather(search)`; AI specials → `think_spellcaster`,
   `think_scout`; everyone on `idle == 1`/32: fishermen → `think_fish`;
   merchants on `idle == 1`/128 → `think_merchant`; carriers → `think_carry`;
   scouts → `think_scout`; `add_to_army`.

So an idle soldier does its target search on its first idle frame and every
32 frames after, phased by `o`; an idle citizen runs `think_peasant(0)` every
frame, with its own gate on `idle` (§5.9). For a unit nobody orders and
nothing approaches, the observable per-frame writes are `idle` and `collide =
0`.

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
| `coll_x, coll_y` | the blocker's position, **written by `detect_unit_collision@00617060`** — not by `astar_path`, as the first reading assumed (R2). `do_move` probes `detect_unit_collision(coll)` every other frame while a search is pending. |
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
path; `add_move_order@00616ed0` is a thin wrapper that snaps and computes the
angle. It fills `x, y, angle, dest = 0, dest_x/y = x/y, last = −1, off_x/y,
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
keeps its heading. (A flag for `docs/MOVEMENT.md`: `move_step` writes
`UnitData::angle` with the *heading* every step, yet that document logged a
facing 1–2° off the heading while walking — `Guy::set_angle`/`do_turn` may
write it back; the citizen case cannot tell, since it turns instantly.)

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
rewrite. `go_around_building@005fc350` is a
tile-walk along the blocking tile's edge that pushes up to three `PathData`s
(turn-in point, a `flags 8` midpoint, the target tile centre), each placed
`off % 0xc0 / 2 − 0x30` from the tile centre; on failure `mo->dest = 0; masks
&= ~8; path_recursion = 10`. Its trigger — an invalid tile on the straight
line — open ground never has.

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

### 4.7 Collision, in one paragraph

`detect_unit_collision(x, y, a, b, c, d, e)@00617060` returns non-zero when the
unit's collision circles at `(x, y)` overlap another unit's (or, with some
flags, a building's); `do_move` probes the waypoint with it before taking it
and `move_step` probes every step (§10). `resolve_unit_collision(x, y)@
005f9d30` is what a failed step falls into: it may kill the order (arrived at
a wall it was building; a transport to board; in range of the target),
attack the blocker (`add_attack_order QUEUE_FIRST`), wait (`pause = rand % 9 +
1` — a sync-stream draw), side-step (push `{tile corner, tol 0, flags 2}` and
point `dest_x/y` at it), or, with a path and `repaths[who] < 16`, pop the
stack down to a real waypoint, snap the unit to its 48-cell centre and
`find_upath(…, anti = action is ATTACK)`. It only ever *pushes* waypoints and
writes `dest/dest_x/y/pause`; the next `do_move` handles the rest. A map with
a handful of well-spaced units never reaches it.

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
and 1 gather** (1 is the normal citizen). In the logged run the builder stood
adjacent at frame 176, its move was gone at 177, `do_construct` first ran at
178 (`flags 1 → 3`, `job_counter 100`), and completion at 327 left the AI
builder with a `GATHERORDER` on the farm it had just built.

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
**`EXPLORETOORDER`** (type 3); for `pos == QUEUE_LAST` on a member whose
action is `GATHER` with `action != 0`, the pos becomes `QUEUE_NEW` (the gather
is pre-empted); then `add_build_order(o, who, QUEUE_LAST, action)` (or
`add_repair_order`), and for `BUILD_AT` the site's `build_masks &= ~0x2000`.

Four additions from the second reading (R3 S1, S6, S9, S10): a `QUEUE_FIRST`
swarm is a **re-entry that halts the group first**; a barge or a carried unit
has its footprint substituted; the scratch group closes with an
`action_move_to`; and the members' spots deconflict **through
`find_nearby_spot`'s own occupancy test**, not through any geometric
spreading — which is why two builders never need a formation.

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

**`find_build_spot@00603e20`**: range `UNIT_BUILD_RESPOND_RANGE × 192` (`× 384`
for stance 1 or 2); `Objects::find_builds(SEARCH_FRIENDLY, range,
FILTER_CONSTRUCT)`; keep own sites whose `build_masks & 0x20` is clear; if
any: count builders per site (friendly units whose action is `BUILD_AT` on
it), pick the fewest (ties → first), `swarm_around(site, QUEUE_LAST, BUILD_AT,
0)`; return 1. **`find_repair_spot@00604320`**: a human
(`leader_flags & 4` set — `LeaderData::is_human@006ec170` is literally
`return leader_flags & 4`, which also settles §14's flag question) always
searches, at `UNIT_BUILD_RESPOND_RANGE × 192`, doubled to `× 384` on worker
stance 1 or 2; an AI searches at `× 192` flat and **only when its effective
difficulty ≥ 2** (per-leader in network mode, the lobby's otherwise), else
return 0 — the gate and the branch shape per audit R3 F3, third pass
2026-08-23. Then `find_any_building(SEARCH_FRIENDLY, range, FILTER_DAMAGED,
FILTER_NOT_UNDER_ATTACK)` whose tile's territory owner is nobody, me or a
mutual ally; `swarm_around(b, QUEUE_LAST, REPAIR, 0)`. `find_gather_spot` is
§6.6.

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
AI peasant: think_civilian_transport(1) first
not a scholar and (unit_masks & 0x400 or stance ∈ {1,2}) and find_build_spot(): return 1
stance ∈ {0,1} (or no stance type and 0): find_gather_spot(AI ? −1 : UNIT_GATHER_RESPOND_RANGE × 192) → deselect; human: group = −1; return 1
not a scholar: human with (0x400 or stance ∈ {1,2}) and find_repair_spot(): return 1
unit_masks &= ~0x400
AI: region/scout logic, then find_repair_spot()
```

With the default option the gate passes at `idle == 2` — the third idle frame
— and then at every fifth increment of the counter (every 80 frames, phased
by `o`). `think_peasant(1)` from a `THINK` order has no gate. A citizen with
`unit_masks & 0x40000` (AI-controlled) uses `T = 1` and an unlimited gather
range.

`Wall::process@00640450`, for completeness: every 32 frames phased by `o` a
site that is not active and whose owner is not human wants `max(4, helpers)`
builders (more for a wonder) and recruits with `find_unit(SEARCH_FRIENDLY,
0xf00, FILTER_TYPE PEASANTS, FILTER_NOT_BUSY)` → `add_build_order(site,
QUEUE_NEW, 0)`. The same function resets `helpers` and `build_masks & 0x800`
every frame (`docs/CITIES.md` §3.3).

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

**Second reading (R4 G16), and it is load-bearing.** `is_gathering_at@
00608880` matches on **`get_action()`** — §3.3's walk — not on the front of
the list. `do_gather` and `do_non_flat_gather` insert their walks as
`QUEUE_FIRST` moves *without* the action bit, so during every walk-out and
walk-back leg the front order is a `MOVE` and only `get_action` still finds
the `GATHER`. Under the "first order" shorthand a woodcutter would stop
counting the moment it set off — and be **pruned out of its own chain by
`check_gatherers`**. The shorthand contradicted §3.3, which was right; the
implementation carried the bug until the audit. Two further details: the
inside-the-building match is **scholar-only** (`ptype[4] ∈ {0x34, 0x35}`;
G17), and `num_gatherers`' second argument is both the decoy filter **and** a
`count_inside` mode selector, `COUNT_TYPE + 2` (G13). This is `Site::gatherers`' real
source in `docs/ECONOMY.md`; the count is **`|{u in chain : u.first_order is
GATHER(this) and u.been_there and not a decoy}| + inside(PEASANTS on a
platform / SCHOLARS in a university)`**. Every UI/AI caller uses `num_gatherers
(0, 0)` — chain members regardless of arrival.

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

`Build::find_gather_tiles@00623350` (wood/ore, at building creation): fills
`gather_from` (`BuildData+0x98`, a `MiningList`) from `BuildTypeData::
find_gather_tcoords`, sets the world tile bit `0x1000` ("gathered at") on each,
and **if the list grew, appends `count × 4` entries by picking `Random::get
(game_random, 0, 0xffff) % count` and moving that entry to the back** — a
shuffled, fourfold list, drawn from the sync stream. `do_non_flat_gather`
ranks tiles by `i >> 2`.

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
    a = guy0.cur_anim; a == 0x1d → return
    a == 0x19: wait--; if wait != 0 → return;  wait = all_gathering() ? −1 : 300 + rnd % 100; return
    T = (tx*0xc0 + 0x60, ty*0xc0 + 0x60)                           # the tile centre
    if vector_dist(u, T) < 0x140:                                  # working the tile
        wait--; if wait == 0: wait = all_gathering() ? −1 : 100 + rnd % 50
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

In plain terms: a woodcutter alternates between chopping at a tile for 400–599
frames (re-checking every 100–149 until every peer is out) and walking back
to the camp for a 32-frame dump animation; a miner walks out once and stays.
`all_gathering` staggers the return trips. **Three sync-stream draws** (`% 100
+ 300`, `% 50 + 100`, `% 200 + 400`), so a simulation that wants the combat
stream to line up has to take them in order. `WorldData::has_gather_access@
006b4e50(tx, ty, who, 1, 0)`: the tile has `mask & 0x8000`, its `wdata` owner
is −1/`who`/an ally, and at least one orthogonal neighbour is neither terrain
class `(mask & 0x30) == 0x20` nor itself a resource tile (`& 0x4000`).

For the dump's woodcutter citizen (§4.8): frame 1 — `goto_build 1, wait 0`,
adjacent (it was placed beside the camp), `wait → −1`, `been_there = 1`; frame
2 — choose a tile (`wait = 400 + rnd % 200`, `goto_build = 0`); frame 3 — out
of `0x140` of the tile centre, `find_nearby_spot(T, 0xc0, 0x100)`, `move(spot)`
queued; frame 4 — the move steps. That is the citizen's `(4008, 28296)`.

### 6.5 The farm (`do_gather`, FARM, arrived)

```
ft = FarmsData::get_farm_type(farm)
ANIMAL_FARM: set_anim(CHAR_SOW); every 256 frames phased by (o*7 + frame + who):
    move((cx + 1 + GameAccess::rnd(n)) * 0xc0 + 0x60, (cy + 1 + GameAccess::rnd(n)) * 0xc0 + 0x60); return
dx, dy = u.tile − b.tile_corner(); out of the footprint → dx = dy = 1
state = farms.farm_data[farm].state[dy][dx]
0: guy0.cur_anim != '$' → as 1;  else → new tile
1: set_anim(CHAR_SOW); Farms::grow(farm, dx, dy); return
2: guy0.cur_anim != '#' → set_anim(CHAR_REAP); Farms::snip(farm, dx, dy); return;  else → new tile
3: set_anim(CHAR_REAP); return
new tile: move((cx + GameAccess::rnd(n)) * 0xc0 + 0x60, (cy + GameAccess::rnd(n)) * 0xc0 + 0x60)
```

**`GameAccess::rnd@0043cca0(n)` is `Random::get(game_random, 0, 0xffff) % n`** —
the sync stream (`n ≤ 1 → 0` without a draw). `docs/COMBAT.md` §10 calls it
a second, unsynchronised stream; it is not. The modulus `n` is passed in ECX
and the decompiler lost it; ~~geometry says the farm's tile extent (3)~~
**run13 measured 4** (`docs/SYNC.md` §4.1, 2026-08-24): the six farmers'
twelve re-target draws on sim-frame 101 give the dump's new `MoveOrder`
goals under `% 4` — `(corner + r)·0xc0 + 0x60`, snapped to the 48-cell
centre by `add_move_order` (§4) — and under no other modulus, and `r = 3`
occurs twice among them. `Farms::
grow@008d91c0` sets `state = 1` and adds **`0.005f` to a `float[4][4]
percent`** until it reaches `1.0f`, then `state = 2`; `snip` turns 2 into 3;
what regrows 3 → 0 (`Farms::process`, presumably) was not read. So a float
accumulator sits upstream of the sync RNG — a count-to-N for the simulation,
with N (200 or 201) a behavioural check (§14). ~~A farmer therefore stands on
its first tile for some two hundred frames before its first re-target.~~

**The check has been run, and N is not 200.** In `gamelog-run6` — a fresh
Ancient-Age start at `UNITS=3 BUILDS=7` — **all six farm citizens, on both
players, take an inserted move in front of their gather order on frame 102**,
and their positions part company on 103 (`docs/DATALAYER.md` §3.1). That is
one shared, deterministic tick for every farmer, which is what a count-to-N
started at frame 0 looks like — so the count is about **101**, not 200. ~~Two
readings fit and the decompile has not been re-read to choose between them:
`grow` is called twice a frame, or the increment is `0.01f` and the `0.005f`
the decompiler printed is half of it. Until one is settled the simulation
keeps `FARM_GROWS`, and the constant is wrong by a factor of two; the
harness's order diff is now the instrument that says so.~~ **Settled
2026-08-24 (`docs/SYNC.md` §3.3): neither.** `Farms::inc_time@008d8600`,
run every frame from `Objects::inc_time`, adds a *second* `0.005f` to every
growing cell (and takes `0.01f` from every cut one, and rolls the farm's
sync-stream draw), so a farmed cell gets two adds a frame — and `0.005f`
summed in single precision first reaches `1.0f` on the **201st** add, not
the 200th. `grow`'s add on frame 100 is the 201st: state 2, still under a
sowing farmer, and the next frame is the "new tile" above with its two
draws — the log's frame 102. Two corrections to the pseudo-code above fell
out of the same reading: `'#'` is the **sow** animation (index 35, the one
every farmer shows) and `'$'` the reap, so case 2's "not `'#'`" is *not
sowing* and the farmer that just ripened its own cell goes straight to a
new tile; and the regrowth from 3 is `inc_time`'s `−0.01f` a frame, 101
frames to empty. `FARM_GROWS` is retired; the clock is `farms.rs`.

### 6.6 `Unit::find_gather_spot(range)@005f5170`

Walk the owner's buildings: keep those that exist, are active, `is_gather_
type`, not neutralised, university-iff-scholar, with `num_gatherers(0,0) <
gather_max` or already gathered by this unit, in the unit's `tregion`; a
citizen standing in a city skips a building of *another* city when its own
city's population is `< 2`, **or** when `mypop <= otherpop + 2` — i.e. it
crosses to the other city's building only when its own city is more than two
more crowded than that one. (The first reading had **both clauses
inverted**; R4 G43, `find_gather_spot@005f5170:108`. `CityData +0x5a/+0x5c`
is AI bookkeeping and is an input to `crates/sim`.) within `range` if `> 0` — **a scholar ignores `range` entirely**, the
caller's value being kept only `if (!bVar2)` where `bVar2` is the
`SCHOLARS`/`SCHOLARSKOREAN` test (R4 G42) — **score = `(Σ_goods rate_g) × 500 /
(dist / 0xc0 + 2)`**, `rate_g` the leader's per-good rate for the building's
`best_gather_type`; pick the max; if not already gathering there →
`add_gather_order(best, QUEUE_LAST, 0)`. Integer division throughout.

### 6.7 How a gather order ends

`kill_current_order` on a `GATHER` (§3.2) → `Build::remove_gatherer`. The chain
also self-prunes: `check_gatherers` on every `add_gatherer` and on arrival;
`num_gatherers` ignores a member whose first order is no longer a gather at
this building, so a citizen re-tasked without a kill stops counting at once.
Building destroyed: `do_gather` kills on `!exists || !is_active`; `work` step
8 kills on a `uid` mismatch once the slot is reused. Full: `add_gatherer`
refuses; `do_gather` kills and queues `THINK` → `think_peasant(1)` →
`find_gather_spot`.

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
  `(o, who)` use `Objects::find_collision` (land: `CollCheck::collide_here`
  with `new_block_radius` in quarter-tiles) **and** `find_ordered_collision`
  (a 9-cell walk against other units' `orders_x/y` — a spot another own unit
  is walking to is taken); any other filter uses `find_unit_with_radius`
  (reject iff `vector_dist ≤ other.big_radius + r_coll`) and the own-player
  ordered variant. A building's `new_block_radius` is 0, so only `0x4000`
  tiles and the footprint test keep units off it.
- **The base bearing** at every build/repair/gather/garrison call site is
  **`find_angle(me − target)`** (asm-confirmed at the swarm, garrison and
  gather sites): the sweep starts on the unit's own side of the target.
  `come_out` uses south (`0x80000000`), `find_path`'s teleport and `go_to`
  `0x55555555` (a third of a turn — arbitrary, not a sentinel), `init_unit`
  the unit's own angle.

| caller | centre | min | max | step | `bo` |
|---|---|---|---|---|---|
| `action_swarm_around` (build/repair/gather approach) | the site | `min(x_size, y_size) × 0x60 + 0x30` (halved for a FARM under `BUILD_AT`) | 0 → default | −1 → `/8` | the site (footprint refused) |
| the `+0x30` nudge | the nudged point | 0 | 0 | 0 | the site |
| `do_garrison` | the target | `size × 0x60 + 0x30` (a wider `+0x1b0..+0x330` ring for one type — the `is(…)` literal is lost) | −1 | 0 | — (then retried with `nocoll 1`) |
| `do_gather` | the building | `size × 0x60 + 0x30` | −1 | 0 | — |
| `do_non_flat_gather`: the camp; the tile | the camp; the tile centre | `d`; `0xc0` | −1; `0x100` | 0; `2` | — |
| `come_out` | own position | `block_radius` | `block_radius + UNIT_DISEMBARK_DISTANCE` | 0 | — (then `nocoll 1`) |

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
`GroupAttackOrder`, `GroupPatrolOrder`, which the binary keeps mixed):

| block | fields (after its bases' blocks) |
|---|---|
| `UNITORDER` | `flags` |
| `TARGETORDER` | `UNITORDER`, then `ox whom uid` |
| `THINKORDER`, `BUILDORDER`, `REPAIRORDER`, `BOARDORDER`, `AWAITBOARDORDER`, `FOLLOWORDER`, `ATTACKTOORDER`, `EXPLORETOORDER`, `FLEETOORDER`, `GROUPATTACKTOORDER`, `AIRPATROLORDER` | label only, then the bases |
| `MOVEORDER` | `UNITORDER`, then `x y angle dest tolerance pause retry attempts timer facing dest_x dest_y last_x last_y coll_x coll_y orig_x orig_y off_x off_y` (read back from the PE at the cited addresses; the dump's order) |
| `ATTACKORDER` | `TARGETORDER`, then `mandatory defensive in_range ever_in_range new_ord def_x def_y` (no dump has one yet) |
| `GATHERORDER` | `TARGETORDER`, then `tx ty build_type wait goto_build non_flat_gather dist_mod been_there` |
| `GARRISONORDER` | `TARGETORDER`, then `search` |
| `GUARDORDER` | `TARGETORDER`, then `dx dy guard_x guard_y idle retry` |
| `ATTACKGROUNDORDER` | `UNITORDER`, then `att_x att_y accuracy attack_unit` |
| `GROUPORDER` | `UNITORDER`, then `oxx whose group_angle id form_id` (spelling open) |
| `GroupMoveOrder` | `MOVEORDER`, `GROUPORDER`, then `in_group` |
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
| `LeaderOptions +0x8` | the idle-citizen delay option (1–5 → 7, 12, 17, 32, 62; default 2) | §5.9 |
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
  a gather), the kill. Collision, suspended searches, `resolve_block`,
  `go_around_building`, the entrench wait are not modelled (no terrain, no
  crowds; stated).
- **`do_build`/`do_repair`/`do_garrison`** — §5.2, §5.6, §5.7 on the existing
  `do_construct`/`repair_*`/`garrison` seams, with adjacency = `attack_dist <
  96` (replacing the tile-based stand-in), the swarm ring (`ExploreTo` to the
  §10 spot, re-queued in front with the same action bit), `check_build_order`,
  `build_done` and the stance rules, `come_out`'s citizen/scholar rally rules.
- **`do_gather`** — the chain on the building (`gatherers: Vec<usize>`,
  push-front, the prune), `num_gatherers(arrived, skip_decoys)` — the count
  the economy should read; **`economy::Site::gatherers` is still an input**,
  because a `Building` does not yet name the resource it gathers, so the
  wiring from the chain into `calc_gather` is the next step — §6.3's arrival
  for farms (walk to the centre, `covers_tile`) and the flat/other types (the
  §10 ring), `been_there` and the dirty flag, the oil-well stance, the
  university/platform `go_inside`, the §6.4 wood/ore machine with its three
  draws **given the building's `gather_from` list** (an input — the original
  fills it from terrain the simulation does not model — but no longer an
  *absent* one: ~~the harness has none at level 0, so a woodcutter's citizen
  stays at the camp~~ a `BUILDS=7` dump carries it and the harness reads it
  in, and with it a woodcutter's citizen walks the original's walk for 432
  frames, §11.2), ~~the §6.5 farm stand as a count-to-N (`FARM_GROWS = 200`,
  an assumption **now contradicted** — the original re-targets at frame 102,
  §6.5; the two `GameAccess::rnd` re-target draws and the animation-gated
  "new tile" branch are still not taken, so the farmer stands)~~ the §6.5
  farm stand on the full clock (`farms.rs`, 2026-08-24: the farmer
  re-targets on the log's frame 102 as the original does; the tile is two
  sync-stream draws, right only on the traced stream),
  `find_gather_spot` by distance (the per-good rate term is an input, taken
  as 1).
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
  `find_gather_spot` (`find_build_spot`/`find_repair_spot` need the object
  searches: not yet).
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
  air, special-anim; `check_target_path`'s 16-frame re-path; the collision half of
  `find_nearby_spot` (§10 — the first candidate is free on open ground);
  `go_around_building`, `resolve_unit_collision`, suspended searches, the
  entrench wait; `come_out`'s rally orders (the existing `come_out` keeps its
  placement); `Wall::process`'s AI recruiter; the gamelog emission of the
  list (the harness reads the dump's, it does not yet write its own).
- **The start of a game** — **done 2026-08-21** (`docs/DATALAYER.md` §3):
  `rondata::diff::build_sim` adds
  the pre-placed buildings `2001..` (complete and active, typed by the §9.2
  production order when the dump carries no type) and give each starting
  citizen its `GATHER` on the building it stands beside by the §9.3 rule
  (`ordered = 2` on `2001`, the rest on successive farms), and
  `check_start_orders` compares every derived target against the
  `GATHERORDER` the original logged — **all ten citizens agree**, on
  `gamelog-run4` and on `gamelog-run6`. ~~The score does *not* move~~ **it
  moves now**: `BuildData::log_data@0062e810:269` puts `gather_from`, the
  `GATHERPOINT` list and `BUILDQUEUE` behind `set_detail(7)` (tier 4 is
  `city`/`gather_down`, tier 6 stops at `orig_type`; the first reading said
  `BUILDS=6`, which would have wasted a run — R7 L14), that dump has been
  captured, and with the tile list in hand **a woodcutter's citizen walks
  the original's walk for all 432 frames**. What still cannot move: the AI's
  units, which need the order stream — `COMMANDMANAGER=1`, or the `UNITS=3`
  order blocks replayed as they appear — and ~~the farmers, which part at
  frame 102 on `FARM_GROWS` (§6.5)~~ the farmers, which now re-target on
  102 as the original does and part on the *tile* — two sync-stream draws
  on the sim's own stream past the four frames run12 traced
  (`docs/SYNC.md` §5).
- **The log** — ~~the harness should read the `UNITS=3` order blocks (§11.1)
  and diff `type/ox/whom/uid/flags` and the path stack per frame; not yet
  written.~~ **Done 2026-08-21** (`docs/DATALAYER.md` §3.1):
  `rondata::diff::compare_orders` walks both lists front first — the log's
  reversed, because §11.1 writes it newest first — and the path stacks bottom
  first, and reports the kind, the action bit, the `flags` byte, the target's
  `whom`/`ox`, the stack's depth and each segment's goal. `flags` is
  reported without scoring (`0x8`/`0x10` have no reader); ~~the path stack
  too~~ **the path stack scores since the pathfinder landed** (2026-08-23,
  `docs/PATHFINDER.md` §10).
  It found the harness's farms outside any city on its first run — the
  citizens held `THINK` where the original held `GATHER`, invisibly, from
  frame 1.

`Sim::tick` is unchanged in shape: income, buildings, then per unit attrition
→ **the order step** (`work`: the liveness check, `idle`, `do_job`) →
movement. The one-order-per-frame rule and the next-frame rule for inserted
moves fall out of dispatching once on the front at the top of `work`.

---

## 14. What is not established, and the checks

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
  semaphore bit, from a local `Random`, not the sim stream. Still open:
  `do_move`'s attack-retarget block (R2 O1, nothing in `crates/sim` depends
  on it) and `BuildTypeData::find_gather_tcoords@0063bdc0` (R4, matters only
  when the harness builds a camp itself) — each still marked `FABLE:` in the
  adjudication with the check that would settle it.
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
- The `UnitAnim` codes `'%' '$' '#' 0x19 0x1d` (they gate `wait` countdowns);
  the farm re-target moduli (ECX, lost — `llvm-objdump` at `0x5efd9x`);
  `Farms::process` and the 200-vs-201 grow count; what reads the four
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
