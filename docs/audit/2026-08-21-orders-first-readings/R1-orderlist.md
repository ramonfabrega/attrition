# R1 — The order list and the unit's frame

A first reading of the order *plumbing* of Rise of Nations: the per-unit order
list, the `UnitOrder` base, the pool the orders are recycled through, how
orders are enqueued and killed, where in a unit's frame the current order is
stepped, and what an idle unit does instead. Read from the Ghidra export under
`~/ghidra-projects/decomp/` (`funcs/`, `types.txt`, `vtables.txt`), with two
claims settled in the PE bytes (`riseofnations.exe` at base `0x400000`) where
the decompiler's control flow could not be right. Every offset below is named
from `types.txt`; every slot from `vtables.txt`.

The sibling readers cover the individual orders (move, attack, gather, build,
garrison, …). This note is what they all sit on.

---

## 1. What it is

### 1.1 `UnitOrder` — the base (size 8)

`types.txt:19204`: `+0x0` vptr, `+0x4 char flags`. Nothing else. Every
concrete order derives from it; the move family derives *virtually* (the
`vtordisp{…}` decorations on every `MoveOrder` method, and the
`this + *(this->vbptr + 4)` adjustments in `get_new_order`), which is why
`MoveOrder` (`0x5c`) keeps its own fields from `+0x4` and carries the
`UnitOrder` sub-object at its tail.

**The vtable** (`UnitOrder::vftable @ 00b474f0`, `vtables.txt:32355`, 74
slots, `0x0`–`0x124`). The named slots, with what names them:

| slot | name | evidence |
|---|---|---|
| `+0x0` | deleting destructor | |
| `+0x4` | `clear` | `UnitOrder::clear@0047fff0` zeroes `flags`; every derived `clear` resets its own fields then this |
| `+0x8` | `walk_data(DataWalk*)` | save/load serialisation; base writes the one byte `flags` |
| `+0xc` | `log_data(Log*)` | §5 |
| `+0x10` | **`get_type() → OrderIndex`** — pure in the base | `ThinkOrder::get_type@004845c0` returns `THINK`; `OrderList::get_data_type@0046ed70` calls this slot; `OrdersMemManager::give_obj` indexes the pool by it |
| `+0x14` | `is_move` | `MoveOrder::is_move` (vtordisp variant `@00485e8e`) sits here; the base's `Window::get_button` fold is `return 0` |
| `+0x18` | `is_attack` | `AttackGroundOrder::is_attack` sits here in the attack-family vtables |
| `+0x1c` | `is_move_attack` | `UnitOrder::is_move_attack@0047ff00` = `is_move() \|\| is_attack()` — i.e. `+0x14 \|\| +0x18`, which is what names those two |
| `+0x20` | `is_targeted` | `TargetOrder::is_targeted` variants; `Unit::work` calls it before reading a `TargetOrder` (§4) |
| `+0x24` | `is_pathed` | `MoveOrder::is_pathed@00488980` = `flags & 1` |
| `+0x28` | `is_fleeing` | `MoveOrder::is_fleeing@004889a0` = `flags & 2` |
| `+0x2c` | `is_group` | `return 1` (folded as `StrafeOrder::is_air`) on exactly `GroupOrder` and `GroupMoveOrder` |
| `+0x30` | `is_air` | `return 1` on `AirOrder`, `AirPatrolOrder`, `AirAttackGroundOrder` |
| `+0x34` | `is_patrol` | `return 1` on `PatrolOrder`, `GroupPatrolOrder`, `AirPatrolOrder` |
| `+0x38` | `print_details(x, y)` | debug overlay; the base prints one string when `flags & 4` |
| `+0x3c` | `get_target_order` | `TargetOrder::get_target_order`; `Unit::work` reads `ox`/`whom` through it |
| `+0x40` | `get_move_order` | `MoveOrder::get_move_order`; `Unit::reset_move_orders` writes `orig_x/y` through it |
| `+0x44` | `get_attack_to_order` | `Unit::add_move_facing_order@005e55c0` takes the `ATTACK_TO` pool object through `+0x44`, `EXPLORE_TO` through `+0x48`, `FLEE_TO` through `+0x4c`, `MOVE_TO` through `+0x40` — the four are the move family's down-casts |
| `+0x48` | `get_explore_to_order` | as above |
| `+0x4c` | `get_flee_to_order` | as above |
| `+0x50`… | the rest of the down-casts | `+0x54 get_air_attack_ground_order`, `+0x5c get_patrol_order`/`get_air_patrol_order`/`get_form_order`, `+0x80 get_trade_order` (read in `Unit::work`), `+0x84 get_air_order`, `+0x88 get_strafe_order` (written in `Unit::work`), `+0x94 get_group_order` (`Unit::modify_group_order`), `+0xa8 get_special_anim_order` (`kill_current_order`), `+0xb0 get_think_order` (`add_think_order`), `+0xb8 get_move_order` (second base offset; `kill_current_order` reads `facing` through it), `+0xcc get_attack_order` (`Unit::think`), `+0xe4 get_gather_order` (`kill_current_order`), `+0xf4 get_cast_order` (`kill_current_order`), `+0x10c get_group_order`/`get_air_order`, `+0x110 get_group_move_order` |

The decompiler prints every one of the `return 0` / `return this` folds as
`Window::get_button` (the README trap); the table above names them from the
derived vtables that *do* override, and from what the caller does with the
result. There is **no per-frame virtual**: the step is not a vtable slot. It is
`Unit::do_job@00617a10`, a `switch` on `get_type()` (§4.3).

**`flags`** — one byte, meanings from the writers and readers:

| bit | meaning | evidence |
|---|---|---|
| `0x01` | *pathed* — a path has been computed for this move order | `MoveOrder::is_pathed`; `add_move_facing_order` sets/clears it from its 5th argument; `kill_group_order@005e3310` clears it and `kill_current_path` on every pathed move order; `kill_current_order` calls `kill_current_path` only if `is_move && is_pathed` |
| `0x02` | *fleeing* | `MoveOrder::is_fleeing` |
| `0x04` | *automatic* — engine-inserted rather than player-issued | set unconditionally by `add_think_order@005e3df0` and `add_spec_anim_order@005e4160`; `add_move_facing_order` sets it from its 7th argument (every `add_move_order` call in `Unit::think_*`/`do_*` passes `0` or `1` there); `UnitData::get_action@00608450` and `Unit::update_action@0060a870` skip a move order **only if it lacks this bit**; `UnitOrder::print_details` prints its one label only when set; `Unit::target_opportunity` tests it; `Unit::land_plane` clears it |
| `0x08` | attack-order bookkeeping — set by `Unit::find_new_target@005ff6a0` and twice in `Unit::fight@005fd4d0` | R-attack's area; not read here |
| `0x10` | attack-order bookkeeping — set and cleared in `Unit::fight` (`fight@005fd4d0:1182–1189`), tested at `:95` and `:404` | R-attack's area |
| `0x20` | set/cleared by `add_move_facing_order` from its 11th argument (`add_move_order` always passes `0`) | reader not found in this sub-area |

`UnitOrder::clear` zeroes the byte; `get_new_order` zeroes it on construction.

### 1.2 `OrderIndex` — the type enumeration (28 values)

No `enums/OrderIndex.txt` was exported, so the table is rebuilt from three
independent places that agree: the `switch` in `get_new_order@00730550` (case
number → constructor and `malloc` size), every class's `get_type` body (the
name it returns), and the names Ghidra prints for the `OrderIndex`-typed
comparisons in `Unit::work` and `Unit::do_job`. `OrdersMemManager` has
**`SafeRecycler<UnitOrder>[28] order_lists`** (`types.txt:65950`), one per
value, which pins the count.

| value | name | class | size | notes |
|---|---|---|---|---|
| 0 | `NONE` | — | — | never constructed; `do_job` maps it to `do_idle`; `close_orders` stops on it |
| 1 | `MOVE_TO` | `MoveOrder` | `0x5c` | |
| 2 | `ATTACK_TO` | `AttackToOrder` | `0x5c` | move family |
| 3 | `EXPLORE_TO` | `ExploreToOrder` | `0x5c` | move family |
| 4 | `FLEE_TO` | `FleeToOrder` | `0x5c` | move family |
| 5 | (`PATROL`) | `PatrolOrder` (`0x4c`, exists) | — | **`get_new_order` has no case 5** (falls to `return 0`), `do_job` has no case, and `add_patrol_order@005e4560` asks the pool for `GROUP_PATROL`; dead value |
| 6 | `BUILD_AT` | `BuildOrder` | `0x20` | |
| 7 | `GATHER` | `GatherOrder` | `0x34` | |
| 8 | `BOARD_SHIP` | `BoardOrder` | `0x20` | |
| 9 | `AWAIT_BOARD` | `AwaitBoardOrder` | `0x20` | |
| 10 | `ATTACK` | `AttackOrder` | `0x30` | |
| 11 | `FOLLOW` | `FollowOrder` | `0x2c` | |
| 12 | `GUARD` | `GuardOrder` | `0x38` | |
| 13 | `REPAIR` | `RepairOrder` | `0x20` | |
| 14 | `CAST_SPELL` | `CastOrder` | `0x30` | |
| 15 | `TRADE_ROUTE` | `TradeOrder` | `0x34` | |
| 16 | `STRAFE` | `StrafeOrder` | `0x54` | |
| 17 | `AIR_PATROL` | `AirPatrolOrder` | `0x68` | |
| 18 | `CHANGE_FORM` | `FormOrder` | `0x64` | move family |
| 19 | `GROUP_MOVE` | `GroupMoveOrder` | `0x78` | move family |
| 20 | `GROUP_ATTACK` | `GroupAttackOrder` | `0x54` | |
| 21 | `GROUP_ATTACK_TO` | `GroupAttackToOrder` | `0x78` | move family |
| 22 | `GROUP_PATROL` | `GroupPatrolOrder` | `0x64` | what `add_patrol_order` builds |
| 23 | `ATTACK_GROUND` | `AttackGroundOrder` | `0x20` | |
| 24 | `AIR_ATTACK_GROUND` | `AirAttackGroundOrder` | `0x48` | |
| 25 | `SPECIAL_ANIM` | `SpecialAnimOrder` | `0x2c` | |
| 26 | `GARRISON` | `GarrisonOrder` | `0x24` | |
| 27 | `THINK` | `ThinkOrder` | `0x8` | the base and nothing else — `get_new_order` mallocs 8 and sets the vtable |

"Move family" is the set `Unit::kill_current_order` and `Unit::work` treat
together: `{1,2,3,4,0x12,0x13,0x15}` = MOVE_TO, ATTACK_TO, EXPLORE_TO,
FLEE_TO, CHANGE_FORM, GROUP_MOVE, GROUP_ATTACK_TO — the classes with a
`MoveOrder` base.

### 1.3 `OrderList` — the per-unit queue (size `0x1c`, at `UnitData+0xc8`)

`OrderList : LinkListBase<UnitOrder*, unsigned char, RecycledOrderNode>`
(`types.txt:19036`, base at `:43471`). `OrderList` adds a vptr (two slots:
`get_data_type` = `order->get_type()`, `get_new_data(i)` = `get_new_order(i)`,
both used only by `walk_data` on load), so inside the unit the layout is:

| `UnitData` offset | field | meaning |
|---|---|---|
| `+0xc8` | vptr | `OrderList::vftable @ 00b41af4` |
| `+0xcc` | `current_data : UnitOrder*` | the cursor's order |
| `+0xd0` | `current_metric : uchar` | the cursor node's `metric` |
| `+0xd4` | `current_node : RecycledOrderNode*` | the cursor |
| `+0xd8` | `length : int` | |
| `+0xdc` | `head_node : RecycledOrderNode*` | **the newest** node (see below) |
| `+0xe0` | `ordered : int` | zeroed by ctor and `clear`; no writer found otherwise |

Every `field_0xcc/0xd0/0xd4/0xd8/0xdc` the decompiler prints on a `Unit*` is
one of these — `Unit` has no fields of its own (`Unit` and `UnitData` are both
`0x158`; `Unit : UnitData : SubObjectData : ObjectData`).

`RecycledOrderNode` (`types.txt:43479`, `0x10`): `next`, `prev`,
`data : UnitOrder*`, `metric : uchar`. Nodes come from a global
`Recycler<RecycledOrderNode>` pool (`pop()` on add, pushed back on remove).
`metric` is set to 0 on `add` and never written afterwards in this sub-area;
it is serialised and logged but carries nothing.

**The list is circular and doubly linked, and it is a FIFO whose head is the
newest element.** `LinkListBase<…>::add@0046d5a0`: the new node is spliced in
before `head_node` (`new->next = head; new->prev = head->prev`) and then
**`head_node = new`**. So `head` is the most recently added order and
`head->prev` — the tail — is the oldest. The cursor is left on the new node.

**`Unit::update_order@006179d0`** is the "current order" accessor: if
`head_node == 0` return `NULL`; else set the cursor to **`head_node->prev`**
(the tail, the oldest) and return its `data`. The same four-line reposition is
inlined at the top of almost every order function (`kill_current_order`,
`close_orders`, `work`, `update_action`, `UnitData::get_order@0060b030`,
`UnitData::order_type@00616e80`, `UnitData::get_action`). So:

- the **current** order is the oldest one;
- `add` appends to the back of the queue (the front of the ring);
- `QUEUE_FIRST` (§2.2) is `add` followed by **`head_node = head_node->next`**,
  which rotates the ring one step so that the just-added node becomes
  `head->prev` — the new current — with the old current behind it.

`remove_current@0046d620` unlinks the cursor's node (if it was the only node,
empties the list; if it was `head`, `head = head->next`) and moves the cursor
to `node->next`. `clear@0046f600` returns every node to the recycler and zeroes
`length`, `head_node`, `current_node`, `ordered`. `prev`/`tail` move the cursor
(`tail` = to `head->prev`). `Unit::work` uses `tail` then `prev` to look at the
*second*-oldest order (the caravan branch).

### 1.4 `OrdersMemManager` — the pool

One global, `ordmgr`, `SafeRecycler<UnitOrder>[28]`, each a `clean_pool` and a
`dirty_pool` (`Stack<UnitOrder*>`: `list`, `size`, `length`, `increment`).

- `get_obj(OrderIndex)@00730ac0`: pop the type's `clean_pool` if non-empty and
  call the order's `clear()` (vslot `+0x4`) on it; otherwise
  `get_new_order(idx)` (malloc + constructor; `Error::report("Order does not
  exist")` if that returns `NULL`). Every `Unit::add_*_order` starts here.
- `give_obj(UnitOrder*)@00730bb0`: `idx = order->get_type()`, `order->clear()`,
  push on `order_lists[idx].dirty_pool` ("Misuse of order memory!" on `NULL`).
  `Unit::kill_current_order` ends here.
- `cycle@00730e20`: move every dirty entry to its clean pool. Called from
  **`Game::do_frame@00591ef0`** — once a frame — and from `init`/`clear`.
  So an order freed this frame is reusable next frame, never this one.
- `init@00730cd0`: pre-allocates 30 of each type (the `0x1e` loop) and cycles.

The pool is an allocation detail with one observable: an order object's fields
are whatever `clear()` leaves, so every derived `clear` is the constructor's
twin and both must be read (the brief's "read the loader" rule; R2–R6 own
those).

### 1.5 `ThinkOrder`

Size 8, no fields, `get_type() = THINK`, `log_data` writes `THINKORDER { UNITORDER
{ flags } }`. It is a *placeholder that makes the unit re-evaluate next frame*.
Created in exactly one place: `Unit::do_gather@005ef2a0` (three sites, lines
113–114, 146–147, 170–171) — when a gather order cannot proceed it does
`kill_current_order(0); add_think_order(…)`.

`Unit::add_think_order@005e3df0` (its `QueuePos` argument is **ignored**):
`get_obj(THINK)`, down-cast through `+0xb0`, `flags |= 4`, `add`,
`clear_partial_path`, **`head_node = head_node->next`** (so it becomes the
current order — the `QUEUE_FIRST` rotation, unconditionally), `update_action`.

Stepped by `do_job` → `Unit::do_think_order@005e5bf0`: `kill_current_order(0)`
(the THINK is consumed); if the list is now non-empty and its current order's
type is not `NONE`, return — that order runs next frame; else, if the unit's
type is `PEASANTS`/`PEASANTSKOREAN`/`SCHOLARS`/`SCHOLARSKOREAN` (`TypeIndex`
`0x32`–`0x35`, read from `ptype+0x4`), **`think_peasant(this, 1)`** — the
forced auto-work search. So a failed gather becomes "find me another job" on
the next frame, without waiting for the idle timer (§4.5).

### 1.6 `QueuePos` — the three enqueue modes

Settled in the listing of `Unit::add_move_facing_order@005e55c0` (`5e55e6:
cmp eax,2 / jne` → the clearing path; `5e56b7: cmp eax,1` → the
`QUEUE_LAST && mode==1` special) against the decompile's names:

| value | name | UI (`Options::picked_spot@00721c40:174–186`) | effect |
|---|---|---|---|
| 0 | `QUEUE_FIRST` | ctrl-click (`local_34`) | no clearing; add, then rotate so the new order is current; the old current resumes after it |
| 1 | `QUEUE_LAST` | shift-click (`GetKeyState(0x10)` high byte) | no clearing; add at the back |
| 2 | `QUEUE_NEW` | plain click | `close_orders(0)` first — kills every order, oldest first — then add |

The engine's own internal orders use `QUEUE_FIRST` almost everywhere
(`think_*`, `do_*`, `target_opportunity`: 22 of 25 `add_move_order` call sites
grepped) and `QUEUE_NEW` where it means to replace the player's intent
(`fight`, `do_attack`'s chase). `QUEUE_LAST` is the player's shift-click only.

---

## 2. The per-frame step

### 2.1 Enqueue — `Unit::add_*_order`

All 21 `add_*_order` functions follow one shape, read in full for
`add_move_facing_order@005e55c0` (137 lines) and confirmed by grep on the rest
(`close_orders|clear_orders|kill_current_order` appears exactly once in each,
except `add_await_board_order`, `add_move_order` (a wrapper), `add_spec_anim_order`
and `add_think_order`, which never clear):

1. If `queued == QUEUE_NEW`: clear `unit_masks & 0x4000000`, `path.length = 0`
   (`+0xc0`, the `Stack<PathData>` at `+0xb8`), **`close_orders(this, 0)`**,
   `clear_partial_path`, `update_action`. (The listing shows this path is taken
   only for value 2; the decompile prints it as unconditional and cannot be
   right — `5e55e9 jne 5e56b7` skips it.)
2. `get_obj(<type>)`, down-cast, fill the fields (the move order: `x`,`y` =
   tile centre `tile*0x30+0x18`; `angle`; `dest = 0`; `dest_x/y = x/y`;
   `last_x/y = -1`; `facing`; `orig_x/y` from the two `Coord` args; `pause`,
   `retry`, `timer` = 0; `off_x/y = x mod 0x300`; `flags` bits `0x1`, `0x4`,
   `0x20` from three int args), `LinkListBase::add`.
3. If `queued == QUEUE_FIRST`: `clear_partial_path`, `head_node = head_node->next`.
4. `update_action`.

Two special cases inside the move one, both visible only from the listing
(the decompile's `bVar2`): with `QUEUE_NEW` and `mode == 2` (attack-move) while
the current order is `ATTACK` with a live target, the attack is re-added with
`QUEUE_FIRST` *after* the move, so an attack-move issued mid-fight keeps the
fight; and with `QUEUE_LAST && mode == 1` on a type with `unit_flags2 & 0x10`
(`ptype+0x2c8`), the mode becomes 3 (`EXPLORE_TO`) and `unit_masks |=
0x4000000` — the mask `Unit::work` later uses to convert the order back (§4.2).

**Adding never touches the current order's state** except through
`clear_partial_path` and `update_action` — a `QUEUE_FIRST` insert suspends the
current move with its path intact (`flags & 1` stays), and it resumes when the
inserted order is killed.

### 2.2 Kill — `Unit::kill_current_order(silent)@005e2cb0`

Tears down the **current** (oldest) order. In order:

1. `unit_masks &= ~0x20000`.
2. `type = current->get_type()` (0 if the list is empty).
3. Per-kind teardown (only if `ptype != 0`):
   - **move family** (`1,2,3,4,0x12,0x13,0x15`): `facing =
     move_order->facing` (`MoveOrder+0x28`, through the `+0xb8` down-cast); if
     `facing >= 0 && !silent && game.playing && !game.loading`: if
     `reversing()` then `facing = (facing == 0)`; set or clear `unit_masks &
     0x2` by it; and if the unit is in a group (`group` `+0x80` ≥ 0) and is the
     group's leader (`GroupData::find_leader`, or `list[0]` for a building
     group), write `facing` into the group record's `+0x48` byte. This is the
     "face the way you were told at the end of the move" carry-over; `0x2` in
     `unit_masks` is the *reversed* bit `docs/MOVEMENT.md` discusses.
   - **`GATHER` (7)**: leader flags `|= 0x2000000` (the AI's "re-plan
     gatherers" hint); take `ox`/`whose` from the gather order (`+0xe4`
     down-cast, fields `+0x8`/`+0xc`); if playing: the gatherer is this unit,
     or — for a type with `unit_flags & 0x10` (`ptype+0x2b4`) that is inside
     something (`down` `+0x28` ≥ 0) — the carrier (`down`,
     `inside_down_who`); if the target object exists, is a building (its
     vslot `+0x10`) and belongs to the same player,
     **`Build::remove_gatherer(build, o)`**; then `unit_masks &= 0x87ffffff`
     (bits 27–30 — the gather-state bits) on that unit; and if it is a
     peasant/scholar (`0x32`–`0x35`) with a hold-doober (`+0x86` ≥ 0),
     `Doober::remove_hold_doobers` and `+0x86 = -1`. This is the seam into
     `docs/ECONOMY.md`'s gatherer counts: **killing a gather order is what
     decrements the building's gatherers.**
   - **`TRADE_ROUTE` (15)**: `end_trade_route(this)`.
   - **`CAST_SPELL` (14)**: if `paid` (`+0x1c`) `&& !silent && playing`:
     `SpellType::unpay_cast_costs(spell, o, who)` — a killed cast refunds; and
     if the unit is a merchant that is packing, remember to `update_gpiece`
     and `scene->recalc_builds = 1` at the end.
   - **`SPECIAL_ANIM` (25)**: if its `type == 1` and playing and the target
     object is alive with a `launching` array (`ObjectData+0x44`),
     `SimpleArray<int>::remove(launching, o)` — the unit leaves the launch
     list.
4. Reposition to the tail, `remove_current`; if the removed order `is_move &&
   is_pathed` → `kill_current_path`; `OrdersMemManager::give_obj(order)`;
   reposition again.
5. `clear_partial_path`, `update_action`, the merchant fix-up if flagged.

`silent` (`param_1`) suppresses the three "playing" side effects (facing
carry-over, cast refund — not the gatherer removal, which keys on `playing`
alone). `close_orders` passes its own argument through; every other caller in
this sub-area passes 0.

### 2.3 The wholesale operations

- **`close_orders(silent)@005e37f0`**: `while (head_node && current->get_type()
  != NONE) kill_current_order(silent)`. Since no live order has type 0 it
  empties the list, oldest first — so the teardowns (gatherer removal, cast
  refund, facing) fire for every order in queue order.
- **`clear_orders@005e3860`**: `unit_masks &= ~0x4000000`, `path.length = 0`,
  `close_orders(0)`, `clear_partial_path`, `update_action`. 29 callers (death,
  capture, garrison, …).
- **`reset_move_orders@005fd080`**: for every order in the list (walking from
  the tail by `prev`), if `is_move`: `orig_x/y = x/y` (`MoveOrder+0x44/0x48 =
  +0x4/+0x8`) — restart point for a re-path.
- **`modify_group_order(id, oxx, whose, …)@005e3530`**: for every order with a
  `GroupOrder` part (`+0x94`) whose `id` (`GroupOrder+0xc`) matches, write
  `oxx`/`whose` (`+0x4`/`+0x8`).
- **`kill_group_order(id)@005e3310`**: for every order: if `is_move &&
  is_pathed` → `flags &= ~1`, `kill_current_path`; if it has a `GroupOrder`
  part with that `id` → `kill_current_order(0)`. (The loop condition is
  `length != 0 && current->next != head`.)
- **`UnitData::get_order`** = `update_order` without the `Unit` wrapper;
  **`UnitData::order_type`** = `get_order()->get_type()` (does not guard an
  empty list — the decompile shows the `if (head == 0)` with an empty body; a
  jump-table it could not recover follows, so treat a call on an empty list
  as undefined rather than `NONE`).

### 2.4 `update_action` / `get_action` — "what am I doing" vs "where am I going"

`Unit::update_action@0060a870` first writes `orders_x/y` (`+0x70/+0x74`) =
the unit's position and `dest_angle` (`+0x58`) = `angle` (`+0x50`). Then it
walks from the current order forward (`current_node->prev`, i.e. oldest →
newer) **over every plain move order** — one with `is_move()` and
`!(flags & 4)` — and over every `CHANGE_FORM`, copying each move order's
`x`, `y`, `angle` (`+0x4`, `+0x8`, `+0xc`) into `orders_x`, `orders_y`,
`dest_angle` as it goes, and stops at the first order that is neither. It
returns that order, or `NULL` if it ran out. `UnitData::get_action@00608450`
is the same walk without the writes. So after `update_action`:

- `orders_x/y` is the *final* destination of the leading run of moves
  (what the UI draws the flag at, what `docs/MOVEMENT.md` calls the order
  point), and
- the *action* is the first non-move order — an ATTACK behind a move, a BUILD
  behind a move — or `NULL` when the queue is moves only.

A move with `flags & 4` (automatic) counts as an action, which is how an
auto-inserted move (a think-order walk, a flee) is what `work` looks at rather
than skipped over.

### 2.5 Serialisation — `OrderList::walk_data@00730270`

On save: `length`, then for each order starting at the tail (the oldest) and
stepping `current_node->prev` each time — which, in a ring whose `next`
direction runs newest → oldest, is oldest → second-oldest → … → newest:
`type` (from `get_data_type` = `get_type()`), the node's `metric` byte, the
order's own `walk_data`. On load: `clear`, then `length` times pop a node,
link it in as the new head, `get_new_order(type)` into `data`, read `metric`,
the order's `walk_data`. Reading oldest first and making each read node the
new head rebuilds the same ring (the last read, the newest, ends up as
head), so the queue survives a round trip in order. This is also the one place the decompile shows
`OrderList::get_new_data` used: it is `get_new_order(OrderIndex)`, an
allocation outside the pool.

---

## 3. The seams

Where this plumbing hands off to the mechanics already documented, with the
call and the argument:

| from | to | what is passed |
|---|---|---|
| `Unit::do_job@00617a10` (§4.3) | `do_move(this, order)` → `move_step(this, move_order, …)` (`do_move@005f7b30:750`) | the current `MoveOrder*`; `docs/MOVEMENT.md` "Where movement sits in the frame" owns everything from there |
| `do_job` | `do_build(this, order)`, `do_repair`, `do_garrison` | `docs/CITIES.md` §3 (the clock), §15 (garrisons); `crates/sim/src/{build,garrison}.rs` |
| `do_job` | `do_gather(this, order)` | `docs/ECONOMY.md`; and *back*: `do_gather` is the only creator of `ThinkOrder` (§1.5) |
| `do_job` | `do_attack(this, order)` → `fight` | `docs/COMBAT.md`; `fight` also reads/writes `UnitOrder.flags` bits `0x8`/`0x10` |
| `do_job` with `NONE` | `Unit::do_idle` (vslot `0x184`) → `check_idle`, `think` | §4.5 |
| `kill_current_order` on `GATHER` | `Build::remove_gatherer(build, o)` | the gatherer count `docs/ECONOMY.md` reads |
| `kill_current_order` on `CAST_SPELL` | `SpellType::unpay_cast_costs` | refund |
| `kill_current_order` on the move family | `unit_masks & 0x2`, the group's `+0x48` | the facing/reversed state `docs/MOVEMENT.md` uses |
| `Unit::process` | `process_healing`, `process_cloak`, `process_attrition`, `process_supply`, `suffer_attrition` | `docs/ATTRITION.md`, `docs/SUPPLY.md` — unchanged by this reading, see §4.1 |
| `Unit::update_action` | `orders_x/y`, `dest_angle` | the "order point" and desired facing `docs/MOVEMENT.md` reads from `UnitData` |

What the sim has today (`crates/sim/src/lib.rs:82–175`): `Unit` carries
`job: Option<Job>` with `Job::{Build, Repair, Garrison}(usize)` and a
`Movement { facing, des_angle, dest: Option<Pos>, speed, … }`. That is one
slot, not a list; there is no `MOVE_TO` as an order, no queue, no `QueuePos`,
no idle timer, no `flags`. The shape this reading implies is a `VecDeque`-like
`orders` per unit with front = current (oldest), `push_back` for
`QUEUE_LAST`/`QUEUE_NEW`-after-clear and `push_front` for `QUEUE_FIRST`, an
`OrderKind` enum with the 27 live values and their ids (for the gamelog `type`
field), and a `flags: u8` on each entry whose bits `0x1`/`0x2`/`0x4` are what
§1.1 says.

---

## 4. Where it sits in the frame

### 4.1 The unit's frame — `Unit::process@00610bc0` (593 lines)

Called per alive unit from `Objects::process_all@0065dce0` (vslot `+0x9c`),
which itself walks players in the order `(frame + i) % 10` for `i = 0..9`
(so **the player processed first rotates with the frame**), and within a
player the unit array in index order; dead objects only have `hold_frames`
(`+0x32`) decremented. Buildings (objects ≥ 2000) and the rest (≥ 3000)
follow, per player in fixed order.

Inside `process`, in order, with the gates:

1. **Countdowns.** `recharging` (`+0xae`), `full` (`+0xac`), `waiting`
   (`+0xad`), `healing` (`+0x38`) each decrement if non-zero.
2. **Decoy / caster / aircraft upkeep** — `unit_masks & 1` (a decoy) takes the
   `else`: `mana_burn++` and `close()` when it reaches
   `decoy_time * (general_upgrade + 2) / 2`; then, if `is_on_map` and the tile's
   territory byte has a bit outside the owner's ally mask,
   **`suffer_attrition(frame % 7 == 0)`** — the decoy bleed `docs/ATTRITION.md`
   (line 621–626) already records. Otherwise: a type without
   `unit_flags2 & 2` that is air (`ptype+0x218 == 2`) runs the fuel/`mana_burn`
   and the aircraft heal (`repair_damage`, vslot `0x168`, every
   `aircraft_heal_rate[heal_level]` frames phased by `o`, `×2/3` for a hero);
   a caster type runs `Caster::process_spells`, the 32-frame shockwave check
   and the mana regen (`memnon_regen_rate` scaled `>>8`, halved on odd/even
   frames, tribe bonus 10 and `SPIES_GENERALS_RECOVER_CRAFT` doubling).
3. **The carrier's own training queue** (`num_queued` `+0xa0` ≠ 0 and leader
   flag `&2`): `queue_time += accel_train × ai_speed`; when ≥ `train_time`
   the pop-cap check, `Objects::init_unit`, `go_inside`, `num_queued--`,
   `queue_time = 0`, the per-leader counters. (A unit that trains units — not a
   building; `docs/CITIES.md`'s clock is the building's.)
4. **`process_healing(this)`** — unconditionally.
5. **If not inside anything** (`inside_up` `+0x82` < 0):
   1. clear `unit_masks2 & 0x10`;
   2. every 16 frames (`(frame + o) & 15 == 0`): `targeted >>= 2` (`+0x3d`,
      arithmetic), **`process_cloak`**; and every 32 frames
      (`(frame + o) & 31 == 0`): clear `unit_masks2 & 0x40000`,
      **`process_attrition`**, then normalise `unit_masks2` bits 0/1, the
      jammed check, the merchant's rare-goods announcement;
   3. **if `attrition` (`+0x9e`) ≠ 0 and `(frame + o) % attrition == 0`: if
      `!process_supply()` → `suffer_attrition(1)`, else `unit_masks2 |=
      0x40000`** — exactly `docs/ATTRITION.md`'s "cadence" block;
   4. **`work()`** (vslot `0x188`) — **the order step, §4.2**;
   5. `Guy::process` for guys `[0, guy_mark)` and `[ptype+0x304, guys.length)`
      — the bodies, after the unit.
   **Else** (inside): clear the moving/fighting masks (`unit_masks2 &
   ~0x8060`, `unit_masks & ~0x11000`, and `~0x1800` unless `0x2000`); every 32
   frames phased by **`who`** (not `o`): if `unit_masks & 0x40000` and the
   container is a city whose flags lack `0x42` → `come_out(0)`; and for every
   guy, last-position = position. **No order is stepped while inside.**

So the ordering `docs/MOVEMENT.md` states — healing, cloak, attrition, the
supply check and the bleed, *then* `work` (→ `do_job` → `do_move` →
`move_step`), then `Guy::process` — is what the function does; and
`docs/ATTRITION.md`'s claim that `Unit::process` is the sole caller of both
halves and that the decoy is the second entry into `suffer_attrition` both
hold. Nothing in this reading moves any of it. What this reading adds is that
**every order, not only movement, is stepped at that one point** — a build
order's clock, an attack's recharge, a gather's walk all run after the
unit has bled for the frame — and that a garrisoned or boarded unit's orders
are frozen, not killed.

### 4.2 `Unit::work@0060d180` (453 lines) — the order step

1. `order = update_order()` (the oldest), `type = order ? order->get_type() :
   NONE`.
2. Every 32 frames (`(frame + o) & 31 == 0`): `visible = 0` unless
   `SubObjectData.flags & 0x80`; `unit_masks &= ~0x4`.
3. If `type != ATTACK`: `SubObjectData.flags &= 0x7f`; if `unit_masks & 0x100`
   and the order is automatic (`flags & 4`) and `type != EXPLORE_TO`: clear
   `0x100`.
4. **Caravan bookkeeping**: a caravan (`is_caravan`) whose `Caravan` record
   has flags `&6` and whose order type is not `CAST_SPELL`: if its *action*
   is not `TRADE_ROUTE` → `end_trade_route`; else `Caravan::build_road` if
   due, and if the current order is not the trade order, look at the
   second-oldest (`tail` then `prev`) and end the route if that is not
   `TRADE_ROUTE` either.
5. **Move-family pre-step** (`MOVE_TO`, or the `else` for `ATTACK_TO`,
   `EXPLORE_TO`, `FLEE_TO`, `CHANGE_FORM`, `GROUP_MOVE`, `GROUP_ATTACK_TO` —
   the decompile's `if (…) { if (type == MOVE_TO) goto … }` is a lost
   three-way branch; the bodies are clear):
   - (`MOVE_TO` only) if `unit_masks & 0x4000000`: `remove_current` the move,
     and re-add it through `add_move_facing_order(…, mode 3, QUEUE_FIRST, its
     own flag-4, its `timer`, …)` — the deferred `MOVE_TO → EXPLORE_TO`
     conversion §2.1 set up; then re-fetch `order`/`type`.
   - unless the type is a merchant-kind (`ptype+0x2b8 & 4`) that is not
     packed (`unit_masks & 0x80000`): **every 16 frames**, `action =
     update_action()`; if it `is_targeted` and is not `AWAIT_BOARD`: a `GUARD`
     repaths every 64 frames; any other — not `CHANGE_FORM`, and either not a
     group order or a group order whose `oxx/whose` is me —
     `check_target_path(this, action->get_target_order())`, and a non-zero
     result re-fetches the order (`LAB_0060d710`).
   - if the order is automatic (`flags & 4`) **and it is the only order**
     (`length == 1`): if the distance to the target object (read through
     `+0xb8`) is under `ptype+0x240`, reset the move's `x/y`, `dest_x/y`,
     `last_x/y` to the unit's position and `retry = 0`, and if pathed with a
     non-empty path, pop to the last flagged `PathData` and push one to here.
     Then **if the unit's tile equals the target's tile → `kill_current_order`**
     (arrived); else `add_cast_order(this, -1, -1, -1, -1, 0x28b, QUEUE_FIRST,
     0)`. *The decompiled flow around this cast is not credible as printed
     (an unconditional cast of type `0x28b` on every frame of every automatic
     solo move); the block concerns a specific unit kind whose target is
     reached by "casting" — an R-cast question. Flagged in §6.*
6. **Non-move orders** (`LAB_0060d733`): if `type != NONE` and
   `SubObjectData.flags & 8` (the "freshly ordered" bit `check_idle` sets):
   for `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER` or an `ObjectData::is(…)`
   kind, leader flags `|= 0x2000000`; clear the bit.
7. If a path search is open (`openlist` `+0x104`) and the order is not a
   move → `clear_partial_path`.
8. **Target liveness**: `action = update_action()`. If none or not targeted:
   `spell_time = 0` unless casting. Else: `(ox, whom)` from
   `get_target_order` (`+0x3c`; fields `+0x8`, `+0xc`); if `ATTACK`,
   `set_in_danger(0)` on self and on the target if it is a unit; **if the
   target's `uid` (`TargetOrder+0x10`) ≠ the object's `ObjectData::uid`
   (`+0x30`)** — it died or was replaced — then for `STRAFE` reset its
   target and `+0x3c = 1`; if the action `is_attack` and is the current order,
   clear its target; **`repath`, `kill_current_order(0)`**. For a `TRADE_ROUTE`
   action the same uid test against `TradeOrder+0x24` (the decompile drops the
   body; the comparison is there).
9. `do_launch` if carrying air and not inside; `safe` (`+0xb2`) countdown.
10. **If `type != NONE`: `idle = 0`** (`+0xb0`); and for a type with
    `unit_flags & 0x40000` but not `0x4000`, neither `unit_masks & 0x800` nor
    `unit_masks2 & 0x8000`: `unit_masks |= 0x1000`.
11. Collision follow-up: if `collide_frame` (`+0x48`) is within the last 4
    frames and the type is not the exempt kinds and the order is **not** in the
    move family: `detect_boat_collision(x, y, 0)`.
12. **`do_job(type, order)`** — §4.3.
13. Post-step: a type with `unit_flags & 0x20` whose `SubObjectData.flags & 1`,
    not inside, and with no order or one whose `+0x30` (`is_air`) is 0: find
    the nearest other unit of the **same type** within `0x180` and push the
    two apart by half their separation each (`set_new_location` on both,
    world-restricted) — the stacking separation rule.
14. `unit_masks &= ~0x10`.

### 4.3 `Unit::do_job(type, order)@00617a10` — the dispatch

A `switch` on `OrderIndex`, one call per case, nothing else: `NONE →
do_idle()` (vslot `0x184`); `MOVE_TO, FLEE_TO → do_move`; `ATTACK_TO →
do_attack_to`; `EXPLORE_TO → do_explore_to`; `BUILD_AT → do_build`; `GATHER →
do_gather`; `BOARD_SHIP → do_board`; `AWAIT_BOARD → do_await_board`; `ATTACK →
do_attack`; `FOLLOW → do_follow`; `GUARD → do_guard`; `REPAIR → do_repair`;
`CAST_SPELL → do_cast`; `TRADE_ROUTE → do_trade`; `STRAFE → do_strafe`;
`AIR_PATROL → do_air_patrol`; `CHANGE_FORM → do_form_change`; `GROUP_MOVE →
do_group_move`; `GROUP_ATTACK → do_group_attack`; `GROUP_ATTACK_TO →
do_group_attack_to`; `GROUP_PATROL → do_patrol`; `ATTACK_GROUND →
do_attack_ground`; `AIR_ATTACK_GROUND → do_air_attack_ground`; `SPECIAL_ANIM →
do_spec_anim`; `GARRISON → do_garrison`; `THINK → do_think_order`. (`PATROL`
= 5 has no case.) Each `do_*` takes the order pointer `work` fetched; the
`do_*` themselves call `kill_current_order` when done (e.g. `do_move` at
lines 139, 183, 319, 550, 720, 726) and the next frame's `work` picks up the
next oldest.

**One order is stepped per unit per frame.** When an order completes inside
its `do_*`, the next one is not started until the following frame's `work`.
(The exception is what `do_*` themselves do *inside* the step — `do_move`
finishing and the body still moving that frame is R-move's.)

### 4.4 The per-frame virtual: none

The brief asked which vtable slot of the order is the per-frame step. There
is none — `+0x10` is `get_type`, and the step is `do_job`'s switch. The order
object is data; the behaviour is on `Unit`.

### 4.5 Idle — `do_idle`, `check_idle`, `think`

`Unit::do_idle@0060dcd0` (`NONE` in `do_job`): `set_anim(CHAR_DEFAULT)` unless
a hero mid-attack-animation; `collide = 0` (`+0x88`); **`check_idle()`;
`think()`**. So `think` runs **every frame** the list is empty — its own
cadence is inside.

**The idle counter** — `UnitData.idle` (`+0xb0`, a byte) —
`Unit::check_idle@006032c0`: `0xff → 1`; if `idle > 1` and `(o + frame) & 15
!= 0` leave it; else `idle += 1`. `work` sets it to 0 whenever the type is not
`NONE`. So from the frame the list empties: 0 → 1 → 2 on two consecutive
frames, then +1 every 16 frames (phased by `o`), wrapping at 255 to 1. Two
side effects hang on it: at `idle == 4` with `unit_masks & 0x2000000`
(entrenched), `set_angle(trench_angle)`; and the entrench itself
(`unit_masks2 & 0x800` types) at `idle ≥ americans_marine_entrench`
(`GraphicEvents::add_entrench`, `unit_masks |= 0x2000000`, `trench_angle =
angle`); cloak particles at `idle == 1` / `idle ≥ 10`; and
`SubObjectData.flags |= 0x18` on the first idle frame, setting the leader's
`0x2000000` for fishermen/merchants/fur trappers.

**`Unit::think@005f6e40`** (343 lines; the decompile drops several `return`s —
the branch *conditions* are legible, several branch *bodies* are not). The
legible structure, in order, each `goto LAB_005f761a` being "done, clear
`SubObjectData.flags & 0x10`":

1. A non-captain whose captain's action is `ATTACK` with a valid target:
   `add_attack_order(target, QUEUE_NEW, …)` — squads follow the captain's
   fight.
2. `PEASANTS`/`PEASANTSKOREAN` (`0x32`/`0x33`): `unit_masks &= 0x87ffffff`,
   leader flags `|= 0x80000`.
3. If not `SubObjectData.flags & 0x10` and `idle > 2`: a 16-frame gate (body
   lost).
4. **Auto-attack** (unless `unit_masks & 0x100`): on the first idle frame
   (`idle == 1`) or every 32 frames, a captain with a remembered `near_o/
   near_who` (`+0x34/+0x36`) that is alive and `is_in_range` →
   `add_attack_order(near, QUEUE_NEW, 0, 0)`. Then, for fighting kinds:
   `think_merchant` (if packed) / `think_attack` (if not packing).
5. **Workers**: `ObjectData::is_worker` → **`think_peasant(this, 0)`** — the
   idle citizen's job search (`think_peasant@005f5760`: its own 5-frame and
   option-dependent gate on `idle`, then `think_civilian_transport`,
   `find_build_spot` for builders, the worker-stance branches; R-gather's).
6. Caravans → `think_caravan(0)`; rare collectors (merchants, AI players only)
   on `idle == 1` or every 32 frames → `do_gather(search)`,
   `unpack_merchant`, the "no rare found" message, **`idle += 1`**.
7. AI-player branches (`leaders.list[who].flags & 4`, or `ai_off`): specials
   → `think_spellcaster` (on `idle == 1` or every 32), the idle-scout message
   at `idle == 1`; `think_scout(0)`.
8. For everyone on `idle == 1` or every 32 frames: fishermen → `think_fish`;
   merchants → `think_merchant` (on `idle == 1` or **every 128 frames**).
9. Carriers → `think_carry`; units that are not supply/hero and either have
   type flag `+0x2c8 & 0x10` or are `0x3a` → `think_scout(0)` if in no army;
   `add_to_army`.

So **an idle soldier with an empty list does its target search on the first
idle frame and then every 32 frames, phased by `o`; an idle citizen runs
`think_peasant(0)` every frame with its own gate on `idle`; a THINK order
forces `think_peasant(1)` once, next frame.** For the harness's default state
— an idle unit nobody orders — the observable per-frame writes are
`idle` (visible in the gamelog at `UNITS=3`), `collide = 0`, and, for a
unit with no neighbour to fight and no work to find, nothing else.

---

## 5. What the gamelog writes

`UnitData::log_data@0060b070` opens detail level **3** (`set_detail(3)`, line
72) and, at its end (line 760), calls `Stack<PathData>::log_data(&path)`,
**`OrderList::log_data(&orderlist)`**, `PtrArray<Guy>::log_data(&guys)`, then
closes the unit block. Neither `OrderList::log_data` nor any order's
`log_data` calls `set_detail`, so **the whole order list appears at
`UNITS=3`** and nothing of it below. (`docs/ORACLE.md` "The detail level is the
knob" — consistent; this is part of the "forty more" fields it mentions.)

`OrderList::log_data@00730070` (labels resolved from the PE's `.rdata`:
`0xae2284 "length"`, `0xae22b0 "type"`, `0xb4212c "metric"`,
`0xb41cf0 "flags"` — wide strings, printed as narrow):

```
length = <N>
type = <OrderIndex>          ┐
metric = <node.metric>       │  repeated N times
<ORDER>                      │
  …the order's fields…       │
  UNITORDER                  │
    flags = <flags>          │
  (end)                      │
(end)                        ┘
```

**Order of listing: newest first.** The loop positions the cursor on the
tail, then at the top of *each* iteration steps `current_node = current_node
->next` before printing — tail→next is `head`, the newest. So the **last**
`type`/block printed is the current order, the one `work` will step. The
harness must read the list back to front to find "the order being executed".
(`walk_data` goes the other way, oldest first — §2.5.)

Each order's block is its class label, its own fields, then the nested
`UNITORDER { flags }` via `UnitOrder::log_data@0047f960` (`begin("UNITORDER")`,
`say("flags", flags)`, `end`). The labels and field names, from every
`*Order::log_data` in the export (the derived readers own the semantics):

| block | fields |
|---|---|
| `THINKORDER` | — |
| `MOVEORDER` | `x y angle dest tolerance pause retry attempts timer facing dest_x dest_y last_x last_y coll_x coll_y orig_x orig_y off_x off_y` |
| `ATTACKTOORDER`, `EXPLORETOORDER`, `FLEETOORDER`, `GROUPATTACKTOORDER`, `AWAITBOARDORDER`, `BOARDORDER`, `BUILDORDER`, `FOLLOWORDER`, `REPAIRORDER`, `AIRPATROLORDER` | label only, then their bases' blocks |
| `TARGETORDER` | `ox whom uid` |
| `ATTACKORDER` | `mandatory defensive in_range ever_in_range new_ord def_x def_y` |
| `ATTACKGROUNDORDER` | `att_x att_y accuracy attack_unit` |
| `AIRATTACKGROUNDORDER` | `total_time sx sy` |
| `AIRORDER` | `oxx whose cruising_alt sharp_turn old returning` |
| `CASTORDER` | `x y paid spell` |
| `FORMORDER` | `newform delay` |
| `GARRISONORDER` | `search` |
| `GATHERORDER` | `tx ty build_type wait goto_build non_flat_gather dist_mod been_there` |
| `GROUPORDER` | `oxx whose group_angle id form_id` |
| `GroupAttackOrder` (mixed case in the binary) | `temporary oxxx whosoever` |
| `GroupMoveOrder` | `in_group` |
| `GroupPatrolOrder` | — |
| `GUARDORDER` | `dx dy guard_x guard_y idle retry` |
| `PATROLORDER` | `waypoint` |
| `SPECIALANIMORDER` | `type started frames data1 data2 data3 data4 ox whom` |
| `STRAFEORDER` | `xx yy` |
| `TRADEORDER` | `oxx whose started loaded uid2` |

Which base blocks nest inside which derived block (e.g. whether
`ATTACKORDER` wraps `TARGETORDER` wraps `UNITORDER`) follows the class
hierarchy; each reader should confirm its own from its `log_data`.

Also logged at `UNITS=3`, and relevant here: `idle` (`+0xb0`), `num_queued`,
`orders_x`/`orders_y` (the `update_action` outputs), `unit_masks`/`unit_masks2`
(whose bits `0x2`, `0x4000000`, `0x20000`, `0x40000` this note names) —
`docs/ORACLE.md` lists them in the level-3 field set.

---

## 6. Confidence, and what is not established

**High** (read in full, arithmetic-free, cross-checked by a second source):
the `OrderList` layout and the ring's orientation (`add`, `update_order`,
`remove_current`); `QueuePos` values and their effects (listing + UI);
`get_new_order`'s type→class→size table and the 28-entry pool; the
`UnitOrder` vtable's named slots `+0x4..+0x4c`; `kill_current_order`'s per-kind
teardown; `close_orders`/`clear_orders`/`reset_move_orders`/`modify_group_order`
/`kill_group_order`; `do_job`'s dispatch; `do_idle`, `check_idle`'s counter;
`add_think_order`/`do_think_order`; the labels and listing order of
`OrderList::log_data`; `Unit::process`'s placement of `work` after
healing/cloak/attrition/supply and before `Guy::process`.

**Medium**: `Unit::work`'s steps 5 and 8 — the bodies are read but the
decompiler lost control flow in three places (the move-family three-way
branch, the `0x28b` cast, the `TRADE_ROUTE` uid test), so the *conditions*
are stated and the unreadable consequences flagged; `Unit::think`'s cadence
table — the gates (`idle == 1`, `& 31`, `& 127`) are literal, several bodies
are dropped; the names of the down-cast slots above `+0x50` (each named from
one caller's use; none contradicts another).

**Low / not established:**

- **`OrderIndex` 0 and 5.** `NONE` is Ghidra's name for 0 (printed in
  `work`/`do_job`); 5 is inferred as `PATROL` from `PatrolOrder`'s existence
  and the gap. Neither is constructed; nothing in this sub-area depends on the
  name.
- **`UnitOrder.flags` bits `0x8`, `0x10`, `0x20`** — writers found, semantics
  belong to R-attack (`fight`, `find_new_target`) and to whoever passes
  `add_move_facing_order`'s 11th argument non-zero (no caller found via
  `add_move_order`; `Unit::work`'s re-add passes the old `flags & 4` only).
- **`metric`** — written 0, logged, never read in this sub-area. If a sibling
  finds a writer, the "FIFO" claim needs the priority re-checked.
- **The `0x28b` cast in `work`** (step 5, last bullet) — cannot be as printed;
  needs the listing around `0x60d6e0–0x60d710`. Only matters for an automatic
  solo move whose target is an object, i.e. not the harness's default state.
- **What `think_peasant(0)` does each frame for an idle citizen with nothing
  to do** — R-gather's, but it is the harness's default state for the five
  starting citizens, so it needs to be pinned: the gate at its top
  (`idle < threshold` by the leader's worker option, then `(idle-2) % 5`) says
  it is *not* every frame, and the rest is the job search.
- **`UnitData::order_type` on an empty list** — the decompile's lost jump
  table; callers in this sub-area guard it (`work` checks `head_node`), so it
  is a robustness note, not a behaviour.
- **Behavioural check worth running** (20 minutes, the recipe in
  `docs/ORACLE.md`): `UNITS=3`, select a citizen, shift-click two moves and
  ctrl-click a third; the gamelog's `length = 3` and the `type` sequence
  (newest first) will show `QUEUE_FIRST`'s rotation directly, and the frame on
  which `length` drops shows "one order stepped per frame, next one the frame
  after". The same log pins `idle`'s 0,1,2,+1/16 cadence on a unit left
  alone.
