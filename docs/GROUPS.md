# Groups — the order layer between an army and its units

*First reading, 2026-08-25, in the main thread on Opus 5, from the Ghidra
export (`~/ghidra-projects/decomp`), the PDB type records (`types.txt`,
`vtables.txt`) and the four traced runs of `docs/ARMY.md` §16. Written to
close the one seam `docs/ARMY.md` §17 names: the AI's armies decide but
never move a unit, because the simulation had no `Group::action_*`. Second
reading: not yet run (§13).*

**What this is.** Nothing in Rise of Nations gives an order to a unit
directly. A player's click, an AI army's tick and a unit's own `go_to` all
build a **`Group`** — a list of up to 128 unit ids belonging to one leader
— and call a `Group::action_*` on it, and that function walks the members
and calls `Unit::add_*_order` on each. `docs/ORDERS.md` §8 read the entry
from the command stream; this document reads the layer itself: the record,
the pool of 64 groups a leader owns, membership, and the five actions
`docs/ARMY.md` cites — `action_move_to`, `action_siege_attack_to`,
`action_attack`, `action_stance`, `action_halt` — plus `Groups::push_group`,
which is how a group made on the stack becomes one the units point at.

**Confidence.** The record and its field names are the PDB's. The pool
(§3), membership (§4), `action_halt` (§7), `action_stance` (§8),
`action_siege_attack_to` (§9) and `action_move_near`'s structure (§6) are
read line by line: high. `action_attack` (§10) is `docs/ORDERS.md` §8.5's
reading, re-walked here for the two branches the AI takes and otherwise
cited rather than repeated. **`Form::compute`'s slot table is read in shape
only and is a declared seam** (§6.4, §13): it is what decides *where within
the formation* each member stands, it is 1,200 lines across four functions,
and it is the one piece of this mechanic whose output no capture has yet
pinned. Everything the army's behaviour turns on — which unit gets which
*kind* of order, and when — is below it and is read.

**Naming.** Offsets are the PDB's: `struct /rise.pdb/GroupData` (0x9cc
bytes; a `Group` is 0x9d4 with its vptr and virtual base), `GroupsData`,
`Form` (0xe98, no named fields — its offsets are cited by value).
Coordinates as everywhere: a tile is `0xc0` units, a cell `0x300`. A
`UCoord` is a **48-unit** step — `div_3_table[v >> 4]` is `v / 0x30` — and
that is the unit `Unit::add_move_facing_order` takes, which is why the
simulation's move order snaps its destination to 48 (`docs/ORDERS.md`
§4.3). `QueuePos` and `OrderIndex` are `docs/ORDERS.md` §1.5 and §1.2.

## 1. Shape of the mechanic

```
GroupData (types.txt, 0x9cc — every field below is the PDB's own name)
  +0x4   id          int     the pool slot this group occupies, −1 for one on the stack
  +0x8   army        int     the owning ArmyData slot, −1 for a player's selection
  +0xc   num         int     members
  +0x10  form        int     the formation index the last move applied, −1 none
  +0x14  stamp       int     the frame membership last changed — get_open_slot's age
  +0x18  ox, +0x1c oy  Coord where the last move was ordered from
  +0x20  o_dist      int     Form::compute's leftover distance
  +0x24  o_angle     int     the formation angle of the last move
  +0x28  disband     int     cleared by action_begin at the top of every action
  +0x2c  order_num   int     bumped by every action that issues orders; part of a
                             GroupMoveOrder's id
  +0x30  priority    int     read by normalize's prune; never written in the export
  +0x34  role        int     OR of the members' type roles (find_role)
  +0x38  think_frame int
  +0x3c  new_speed   int     \ the group's march speed, reset every frame by
  +0x40  speed       int     / Groups::process to the leader's own
  +0x44  form_num    int     the member count Form::compute_dests laid out
  +0x48  facing      uchar   the formation's mirror flag (§6.3)
  +0x49  buildings   uchar   this is a group of buildings, not units
  +0x4a  who         uchar
  +0x4b  march       uchar
  +0x4c  off_x[128]  int     \ the slot offsets Form::compute_dests wrote,
  +0x24c off_y[128]  int     / rotated into
  +0x44c curr_x[128] Coord   \ the offsets rotated by the leader's heading —
  +0x64c curr_y[128] Coord   / what do_group_move adds to the leader's position
  +0x84c angles[128] char    the per-slot facing byte, packed into a move order's
                             top byte
  +0x8cc list[128]   short   the member unit ids
```

`GroupsData` (0x44) is the pool: `Array<Group> list`, `int last_group[8]`,
`int *const_last_group`, `int proc_group`. **Sixty-four groups a leader**
(`Groups::get_open_slot@006fa460` indexes `who * 0x40`), of which the
first **46** are the recycling pool (the loop bound is `base + 0x2e`); the
rest belong to the hotkey and selection groups (`HotKeyGroups`,
`SelectGroups`, whose own vtables share `Group::add`/`kill`/`action_begin`).

`Group`'s vtable is six slots (`vtables.txt`, `HotKeyGroup::vftable` and
its siblings): `+0x0 get_button`, `+0x4 get_num`, `+0x8 get_num_cap`,
`+0xc add`, `+0x10 kill`, `+0x14 action_begin`. Every `(*(code**)(*this +
0x10))(o, who, …)` in the family below is `Group::kill` — remove one unit
— and every `(*(code**)(*this + 0x14))()` is `action_begin`.

The flow, in the order the army reaches it: `Army::add_unit` builds a
one-unit `Group` on the stack and `Groups::push_group`s it into the pool
(§3.2) → every unit it holds carries the slot in `UnitData +0x80` → each
army tick calls `Army::do_forming` / `march_to_target` / `engagement`,
which call `Group::action_*` on that slot (§6, §9, §10) → each action walks
the members and calls `Unit::add_*_order`, and the per-unit order system of
`docs/ORDERS.md` takes it from there.

## 2. Who calls what, and with which arguments

The army's calls, from `docs/ARMY.md` §8, §9, §11, §14, with every
argument named:

| caller | call |
| --- | --- |
| `do_forming`, siege branch | `action_siege_attack_to(g, x', y', ·, ·, muster_angle)` — arguments 3 and 4 are **never written** (`sub esp, 8` at `6f45df`); the callee reads only 1, 2 and 5 |
| `do_forming`, move branch | `action_move_to(g, x', y', QUEUE_NEW, set_angle 1, muster_angle, ATTACK_TO, action 1, form −1, width −1, disembark 0)` |
| `march_to_target` | the same two, at `(x', y')` with `angle`, and for a nearly-whole building target `action_move_to(g, building.pos, …, ATTACK_TO, …)` |
| `engagement` | `action_attack(g, ox, whom, mandatory 0, QUEUE_NEW, ignore 0)` |
| `send_here` | `action_move_to(g, p, QUEUE_NEW, 1, muster_angle, order, 1, −1, −1, 0)` |
| `charge` | `action_move_to(g, target.pos, QUEUE_FIRST, 0, 0, ATTACK_TO, 1, −1, −1, 0)` |
| `Army::close`, `Army::stop` | `action_halt(g, 0)` |
| `set_stance(s)` | `action_stance(g, s)` on every group whose `get_stance_type` is `STANCE_COMBAT` |
| `find_target`'s probe | `push_group`, then `action_stance(5)`, `action_attack(farm, L, 1, QUEUE_NEW, 0)`, `action_move_to(back, MOVE_TO, QUEUE_LAST)` |

`Group::action_move_to@0070fba0` is one line: `action_move_near(this, x, y,
tolerance 0, queue, set_angle, angle, orders, action, form, width,
disembark)`.

The player's calls are `docs/ORDERS.md` §8.1's table; `Unit::go_to` builds
a one-unit group and calls `action_move_to` with the action bit **0**.

## 3. The pool

### 3.1 `Groups::get_open_slot(who)@006fa460`

Walk slots `who × 64 + 0 .. +45`, `local_18 = frame`, best `−1`:

- a slot that is **empty** (`get_num() == 0`) **or a building group**, and
  is not `last_group[who]` → taken at once, the walk ends;
- otherwise a slot whose `stamp <= local_18` and whose `get_num() == 1` — a
  **singleton** — and which is not `last_group[who]`, becomes the running
  best and `local_18 = its stamp`. So the fallback is the *oldest
  single-unit group*.

None found: the engine logs "UH OH, NEED MORE GROUPS! No non-singular
groups found!" at `GLOG_MISC` detail 1 — a line worth grepping a dump for
— and then takes the last slot in the range whose `stamp <= frame` and
which is not `last_group[who]`, singleton or not.

Then, unless the chosen slot is a building group, **every unit of `who`
whose `+0x80` names that slot has it cleared to −1**: the old members are
evicted before the new group moves in.

### 3.2 `Groups::push_group(who, g, force)@0070f9e0`

- **`force == 0` and `g.num < 2`**: no slot is taken. Every active member
  has `UnitData +0x80 = −1` — the unit is left group-less — and the
  function returns **−1**. A single-unit selection is not worth a slot.
- Otherwise: if `g` **equals** the leader's `last_group` slot
  (`Group::equals_group@00708000`: same `who`, same `get_num()`, same
  `buildings`, and the same ids in the same order, both sides normalised
  first) the existing slot is reused; else `get_open_slot`, `copy_group`
  into it, and `last_group[who] = it`.
- Then every active member: if its current `+0x80` is a **different** live
  slot, that group's `kill(unit, who, 0, 0)` removes it there first; then
  `+0x80 = the new slot`.
- Returns the slot.

`Army::add_unit` passes **`force = 1`**, so an army's first, one-unit group
does take a slot (`docs/ARMY.md` §3.2). `find_target`'s two-unit probe
passes 1 as well.

### 3.3 `Groups::process@006fa210`

Once a frame, for **one** slot of each in-use leader — `proc_group`,
cycling 0..63 — the same prune `Group::normalize` runs (§4.3) plus
`find_role`, and then `speed = new_speed = UnitData::speed(leader)`, or 0
when there is no leader. That is the only thing that keeps a group's march
speed from drifting; `docs/ORDERS.md` §8.3 reads the consumer.

## 4. Membership

### 4.1 `Group::add(o, who, keep_captain, const)@00714350`

The argument named `keep_captain` above is `param_3` and `const` is
`param_4`; `param_4` chooses `get_num_const` over the virtual `get_num`,
and is 1 only on the recursive call that pulls in a squad's subordinate.

- An empty group takes `who` as its own.
- Refused unless the group is empty or `who` matches.
- If the object is a unit: with `keep_captain == 0`, a **non-captain** is
  not added — its **captain** is added instead (`add(unit +0x8e, who, 0,
  const)`, a tail call) — and with `keep_captain != 0` a captain first has
  itself `kill`ed out of the group. So a group holds captains, and asking
  for a figure gets you its squad.
- `disband = 0`. `uVar5 = object vslot 0x1c` is the object's "is a
  building" answer; a group is refused a member whose answer differs from
  `buildings` once it has one — **units and buildings never mix**, which is
  the error `action_attack` reports if they do.
- Not already a member (`GroupData::member`), and `num < 128`: the four
  offset arrays and the angle byte are zeroed at the new index, `list[num]
  = o`, `num += 1`, `buildings = uVar5`, `stamp = frame`.
- `role |= type.role` (`UnitType +0x2c8`), and if the unit has a
  subordinate (`+0x90 >= 0`) that is active, it is `add`ed too with
  `keep_captain = 1`.
- `id >= 0` → `compute_speed`.

### 4.2 `Group::kill(o, who, keep_captain, const)@00714110`

The mirror: a non-captain kills its captain instead; a captain's
subordinate is killed first. Then the id is found in `list`, the unit's
`+0x80` is cleared **only if it names this group**, `disband = 0`, the tail
of all five arrays shifts down over the hole, `num -= 1`. `num == 0` →
`clear(−1)`; else `stamp = frame`. Finally `speed = new_speed = the
leader's speed`, or 0.

### 4.3 `Group::normalize@00711540` and `get_num_cap@007145c0`

`normalize` walks the members **from the last to the first** and drops a
member that is `list[i] < 0`, or whose object is not valid, or — when
`priority == 0` and `id >= 0` — is a unit whose `+0x80` no longer names
this group and which is not a building. Then `find_role` (the OR of the
members' `type.role`) and `speed = new_speed = the leader's speed`.

`get_num_cap` is the same backwards walk, dropping only invalid objects,
and counts the members for which `is_captain` holds — `docs/ARMY.md` §3.3's
`num_captains`. It calls `normalize` first only when `num < 4 && id >= 0`.

`GroupData::num_valid@0070d7e0` counts members whose object flag byte has
`OBJECT_VALID`; `engagement` (`docs/ARMY.md` §11) issues its attack only to
groups with `num_valid() != 0`.

### 4.4 `GroupData::find_leader@0070ccb0`

Two passes. In each, over the members in order: the object must be active
and a **captain**, and (first pass only) **on the map**; its
`FormData::type_cat(type)` is the key, and the member with the **strictly
lowest** category wins — so a tie goes to the **first**. The second pass
drops the on-map test; after it, −1. Writes the winner's *slot index*
through the out-parameter and returns its *unit id*.

`GroupData::is_on_map@0070c450` is true for a building group with members,
and for a unit group with at least one active on-map captain.

`GroupData::get_form@0070b9f0` returns the `unit +0xaa` shared by every
active on-map member, and **−1 as soon as two differ** — a mixed group has
no formation. `get_form_mod_option` is its width twin at `+0xab`.

`GroupData::get_stance_type@0070d370` is the leader's `vslot 0x108`; if
that is `STANCE_NONE` or `STANCE_CASTER` it walks the members and takes the
first non-`NONE` that is not `CASTER`, falling back to `CASTER` if one was
seen. `docs/ARMY.md` §6's `set_stance` acts only on `STANCE_COMBAT` groups.

## 5. `action_begin` and the scenario filter

Every action starts with two things. `Group::action_begin@00714100` is
`disband = 0`, one line. And the **ignore-orders filter**: when a scenario
has set `ScenarioData::ignore_orders` and `who < 8`, every object in
`objects_ignoring_orders[who]` is `kill`ed out of the group before anything
else. Neither the simulation nor any skirmish reaches the filter
(`ignore_orders` is a scenario field); it is named here so a reader of the
decompile is not surprised by the loop at the top of all five actions.

## 6. The move — `Group::action_move_near@00704990`

The longest function of the family (1,420 lines decompiled) and the one
every other one falls back to. `action_move_to` is its `tolerance = 0`
front. In order:

### 6.1 The domain split

`is_on_map()` and `num > 0` required; then `action_begin`, and the
destination is clamped into the world.

Two stack groups are built: **land** and **sea/air**. A unit whose type's
`domain == 1` (sea) goes to the second; otherwise the unit's
`get_inside` is asked, and a unit **riding inside something** is
represented in the second group by **its carrier**. If both halves are
non-empty **and `group.army < 0`** — a player's selection, not an army's
group — each is `push_group`ed with `force = 1`, has `facing` copied from
the parent, and `action_move_near` recurses on it; then the parent is
cleared and the call returns. **An army's group never splits**: the test is
`*(int *)&this->field_0x8 < 0` at `70497e`/`704c5e`.

If the destination cell is **not water** (`terrain & 0x30 != 0x20`) the
split is re-done with a different rule — an `is_special` unit and a
*carried* unit go to the first group, everything else to the second — and
the same recursion applies under the same `army < 0` guard.

### 6.2 `QUEUE_FIRST`

`set_up_insert` copies the leader's action-bit orders aside;
`action_halt(this, 0)`; the whole call re-runs as `QUEUE_NEW`;
`finish_insert` puts the saved orders back after the new ones. The leader's
`unit_masks & 0x100` is preserved across it. This is `docs/ORDERS.md`
§1.5's rotation, done at the group level.

### 6.3 The formation, and the reverse flip

A **plane leader** of domain 2 that is not `+0x2b4 & 0x20` returns here —
air groups do not move this way.

`form = param_9`, or, when that is `9` or `−1`, `GroupData::get_form()`
clamped up to 0; `group.form = form`. `width = param_10`, or
`get_form_mod_option()` when −1. The group's own location comes from
`get_loc_to` for `QUEUE_LAST` and `get_loc` otherwise
(`GroupData::get_loc@0070e030`: the leader's position, replaced by
`(ox, oy)` when the leader is within `0x181` of it, or by the leader's
position translated by its current move's origin when that origin is within
`0x180` of `(ox, oy)` — "where the group *will* be").

**`Group::compute_form@00707c80`** then:

- `find_leader` — no leader is the error box "No valid group for formation"
  and a null return;
- `dx, dy` = the destination minus the group's location;
- with `set_angle == 0`: a zero delta takes the leader's own facing minus
  its packed slot byte when the group has moved since (`(ox, oy) !=
  (group.ox, group.oy)`), else `group.o_angle`; a non-zero delta takes
  `find_angle(dx, dy)`. With `set_angle != 0` — **every army call** — the
  caller's angle stands, and `reverse = |find_angle(dx, dy) − angle| > 90°`;
- if the leader's facing minus the given angle is past 90°, `group.facing`
  is **toggled** around the `Form::compute` call and toggled back; under the
  network semaphore bit (`semaphore[1] & 8`) `facing` is forced 0 and
  `reverse` false, so a networked game never mirrors;
- `Form::compute(form, group, x, y, angle, width, group.facing)`;
- and when `reverse` holds, every member's `off_x/off_y` in both the `Form`
  and the group are **negated**.

`Form::compute@0072e8e0` is `compute_rows_and_columns` then `compute_dests`,
and finally `group.o_angle = find_angle(…)`, `group.o_dist =
vector_dist(…)`.

### 6.4 The slot table — a seam

`Form::compute_rows_and_columns@0072d910` and
`Form::compute_dests@0072cba0` fill, for each member `i`, a destination
`to_x[i]/to_y[i]` (Form `+0x514`/`+0x714`), an offset `off_x[i]/off_y[i]`
(`+0x914`/`+0xb14`) and a facing byte `group.angles[i]`. The shape, from
one reading:

- `Form::categorize` sorts the members into **18 `FormCatIndex`
  categories** by type, filling per-category counts (`+0x3c..0x84`),
  spacings (`+0x84`, `+0xcc`) and a per-member category (`+0x314`) and
  index-within-category (`+0x114`);
- `compute_rows_and_columns` turns the counts into rows and columns per
  category, with three special forms — 6 (a square: `ceil(sqrt(n))`), 7
  (a single line), 8 (three columns) — and otherwise a width driven by the
  `width` argument (`(max_area × width / 50) / spacing`, at least 1);
- `compute_dests` walks the members and lays each category's block out
  around the destination, rotating every offset by the formation angle
  through `sin_table`; a **non-captain** is placed relative to its captain
  rather than the block; `is_modern_infantry` scatters by a
  position-derived `% 3`; and the whole block is finally shifted so the
  leader's category sits on the destination.

**It is not implemented.** Two reasons, both stated so the next session can
weigh them: the arithmetic involves the export's only floats in this family
(`(float)cols[i] − 0.5f` at `0072cd8f`, exactly `(2·cols[i] − 1)/2`, so
representable — but the whole 1,200 lines would have to be read to know
where else), and **no capture pins its output**. The check that would is
named in §13.

### 6.5 The AI branch

This is the part `docs/ARMY.md` needs and the part `docs/ORDERS.md` §8.2
did not read. At `704b6a`:

```
ai_group = !(leaders[who].flags & 4)      # not a human
           && group.army >= 0             # an army's group, not a selection
hurry_city = ai_group && armies[who][group.army].hurry != 0
             ? ObjectsData::find_city(dest, SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)
             : none
```

and then, three times over — in the `QUEUE_NEW` clear, in the order loop
and in the path loop — each member is classified:

- **not an AI group** → the ordinary path;
- **an AI group with no hurry city** → a member that is **siege
  (`UnitType` vslot `+0x10c`) and whose `order_type()` is `ATTACK`** is
  **left entirely alone**: its orders are not cleared, it is given no move,
  and it gets no path. Everything else takes the ordinary path;
- **an AI group with a hurry city** → a member that is a **supply wagon,
  siege, or hero** is sent **into that city**: `can_garrison` →
  `add_garrison_order(city.o, who, search 0, QUEUE_NEW, 0)`; otherwise
  `find_nearby_spot` around the city between `0x300` and `0x600` at the
  angle from the group, falling back to the city's own point, and
  `add_move_order(there, QUEUE_NEW)`. Everything else takes the ordinary
  path.

So: **a hurrying army stables its siege, wagons and generals in the nearest
friendly city and marches the rest**, and a non-hurrying one never
interrupts a siege unit that is already shooting. `hurry` is
`find_muster_spot`'s "there is an enemy at the target" flag
(`docs/ARMY.md` §13), so both branches are reachable from the army tick.

### 6.6 The ordinary path, per member

1. `unit +0xaa = form` unless the type's `TypeIndex` is one of 0x32..0x35
   (four types the decompile does not name), `unit +0xab = width`.
2. `QUEUE_LAST` inverts the unit's path stack.
3. The slot destination is clamped into the world.
4. For `i > 0`, the slot's `tregion` is compared with slot 0's.
   - **Different region**: the unit is taken out of the group (`+0x80 =
     −1`), and a `find_nearby_spot` around **slot 0's** destination
     re-places it — using the `TRANSPORTBARGE` type's footprint when the
     unit is land, slot 0 is an invalid location for it and the leader can
     transport, or the carried unit's type when the member is a loaded
     boat; failing that, slot 0's own point. Then `+0x80 = group.id` again.
   - **Same region**, and the slot is an invalid location for the unit
     (`invalid_loc`) while slot 0's is not: a **captain** is re-placed by
     `find_nearby_spot` around slot 0 between `0xc0` and `0x480`; a
     **non-captain** around the **last captain seen** in the walk, at the
     given angle turned ±90° depending on whether it is that captain's own
     figure.
5. A `QUEUE_NEW` on a **packing packer** with orders queued rotates the
   tail order to the front instead of clearing, and the member's queue mode
   becomes `QUEUE_FIRST`.
6. The order itself. With `orders ∈ {MOVE_TO, ATTACK_TO}` and the unit not
   `is_modern_infantry`, a **`GroupMoveOrder`** is added — unless the unit
   is a `role & 0x10` type that is not `unit_masks & 0x40000`, or
   `group.num < 2`, or `unit_masks & 4`, or the type is sea, or `form ==
   9`. Otherwise a plain **`add_move_facing_order`**. Both take
   the destination as a `UCoord` (`/0x30`), the angle as `group.angles[i]
   << 24 | angle`, `pathed = 1`, the action bit from `param_8`, the
   formation's reverse flag, and the original click for the order's `orig`.
7. `unit_masks &= ~0x400`.

Then `update_positions(leader)` rotates every slot offset into
`curr_x/curr_y` by the leader's heading — what `do_group_move` adds to the
leader's position each frame (`docs/ORDERS.md` §8.3).

### 6.7 The path

One path is planned and offset, exactly as `docs/ORDERS.md` §8.2 outlines:
a single-entry `grouppath` at the **leader's slot destination** with the
call's tolerance and `FINAL`; within `0x900` nothing more is planned;
otherwise `PathFinder::find_wpath` from the group's location, with
`pathfinder +0x70 = 1` set around the call for an army's group (an AI hint
the pathfinder reads).

If that produced nothing, **each member plans its own** single-entry path
and `find_wpath`s it, and the stack is inverted.

Otherwise each waypoint is popped from the top and, for each member,
translated by `slot[i] − slot[leader]` and clamped. A terrain guard follows:
the **area id** of the member's waypoint cell (`world +0x134`, one 0x1c-byte
record per cell, `+0x4` land and `+0x6` water, the latter chosen when the
cell is `HALFLAND 0x100` and the tile's terrain is water) is compared with
the leader's; if they differ **and** the member's waypoint is an invalid
location, the waypoint is snapped back into the leader's cell. Then it is
pushed — but a **follower** only receives a **non-final** waypoint when it
is *not* in a group move (`orders != MOVE_TO`, or modern infantry, or sea,
or `form == 9`) **and** is within `0x600` of the leader. Group-move
followers get the final waypoint only; the intermediate legs are the
leader's, and `do_group_move` re-derives theirs each frame.

Finally every member's stack is inverted, `group.order_num += 1`, and the
`Form`'s tables are zeroed.

## 7. `Group::action_halt(mask)@0070d0c0`

The scenario filter, `action_begin`, and then — **only for a unit group** —
`group.form = −1` and, per active on-map member that is not an airborne
plane and is not `is_entering_or_exiting`:

```
if (mask & 4) and the type is_siege:    skip
if (mask & 2) and is_special:           skip
if (mask & 1) and is(SPY):              skip
unit_masks &= ~0x4000000
unit.path_length = 0
close_orders(0); clear_partial_path(); update_action()
unit_masks &= ~0x100
```

Every army caller passes `mask = 0`, so the three exemptions are dead for
the AI; the mask is `action_attack`'s `ignore` argument passed through
(§10), and the player's halt command sets it.

`Army::close` calls this on each of its groups before freeing the slot, and
`Army::stop` — `Armies::leader_defeated`'s only body — is exactly this loop
(`docs/ARMY.md` §14).

## 8. `Group::action_stance(s)@0070d440`

`is_on_map()` required, then `action_begin`. `type = get_stance_type()`
picks the number of options: `STANCE_COMBAT` **6**, `STANCE_WORKER` **4**,
`STANCE_CASTER` and `STANCE_PACKER` **2**; anything else returns.

`s < 0` cycles: `−1` steps forward `(cur + 1) % n`, `−2` back with a wrap
to `n − 1`. Then a `COUNT_PEASANTS` count is taken and, if non-zero, a
`COUNT_MILITARY` count — both discarded, a leftover.

Per member, twice over:

- if the object is **valid** and can attack (`vslot 0x20`), its
  **`ObjectData +0x7e`** takes `s` — but only when the object's own stance
  type matches the group's, and a mismatch **ends that member's turn**;
- if the object is a **unit** (`vslot 0x18`) whose stance type matches and
  which is not carrying something (`vslot 0xc0` is 0), then `unit +0xb1 =
  s`, `unit +0x8 |= 0x10`, and for a **combat** stance:

| `s` | what happens |
| --- | --- |
| 0, 3, 4 | `Unit::clear_mandatory` — the unit's current order stops being the player's |
| 1, 2, 5 | for a **human** leader only, and only when the current order lacks the action bit and there is no action order carrying it: `Unit::repath` then `kill_current_order(0)` |
| else | `Unit::clear_orders` — unreachable for a combat stance, which has six options |

With `docs/COMBAT.md`'s stance order (0 aggressive, 1 defensive, 2 stand
ground, 3 raid, 4 raze, 5 hold fire), `docs/ARMY.md` §6's three calls read:
a **mustering or defending** army goes **defensive**, a **marching** one
**aggressive**, a **navy** and the pre-age-4 no-siege army **raid**. Every
one of those is an AI leader, so the middle row is dead for an army and
`set_stance` never kills an order — it only clears `mandatory` on 0 and 3.

## 9. `Group::action_siege_attack_to(x, y, ·, ·, angle)@0070d830`

A building group returns. Then a **sub-group** is built on the stack,
carrying the parent's `id` and `army`: every member whose type
`is_siege`. For an AI leader (`!(leader_flags & 4)`) that sub-group is
widened and then narrowed:

- empty → the **first supply wagon** of the parent joins it;
- still empty → the **first hero**;
- then, over the sub-group's members, each is scored by the **sum over
  every member of the parent group** of `(|dx| + |dy|) >> 10` — its total
  Manhattan distance to the group in 1024-unit steps — and the **smallest**
  wins (strict `<`, so a tie goes to the first). That unit is the
  **anchor**.

A human's group skips the widening and the scoring and takes the
sub-group's own `find_leader` as the anchor (`0070dabd`), so a player's
siege-attack command escorts only when the selection already holds a siege
unit.

Then:

- **no siege, no wagon, no hero** (or no anchor) → `action_move_to(this, x,
  y, QUEUE_NEW, 1, angle, ATTACK_TO, 1, −1, −1, 0)` — the whole group
  attack-moves, and that is all. **This is the branch every traced army has
  taken**: runs 21–27 have no siege in any army;
- for an AI, the anchor's **area id** (§6.7's `world +0x134` record, with
  the halfland rule) is compared with the destination cell's **land** area,
  and a mismatch falls back to the same whole-group attack-move;
- otherwise the **sub-group** attack-moves to `(x, y)`, its slot offsets and
  angle bytes are copied back onto the matching members of the parent
  (`Unit::replace_form_id` re-indexes each), and the parent gets
  **`action_guard(anchor, who, QUEUE_NEW, 1)`** — everyone escorts the
  anchor while the siege walks in.

## 10. `Group::action_attack(o, whom, mandatory, queue, ignore)@00712490`

`docs/ORDERS.md` §8.5 reads this function; it is not repeated. What matters
for `engagement` (`docs/ARMY.md` §11), which calls it with `mandatory = 0`,
`QUEUE_NEW`, `ignore = 0`:

- `is_on_map()` and `o >= 0`; a **building group** writes the three `Build`
  fields per member instead and returns;
- `QUEUE_FIRST` is `set_up_insert` / `action_halt(ignore)` / recurse
  `QUEUE_NEW` / `finish_insert`;
- a **capturable** target (`BuildData::check_capture_eligible`) becomes
  `Build::check_capture` plus `action_move_to(target.pos, queue, …, MOVE_TO,
  1, −1, −1, 0)` and the function returns — the group walks in to capture
  rather than attacking;
- otherwise **three passes over the members, one per domain** (`type
  +0x218 == 0, 1, 2`), skipping the `ignore` mask's kinds exactly as
  `action_halt` does (`4` siege, `2` special, `1` spy). A plane retargets
  its strafe; a unit already carrying a **mandatory** `ATTACK` on this
  target, with fewer than three orders queued and out of range, keeps it; a
  packer packs or unpacks first; and the main path is
  `add_attack_order(unit, o', whom', queue, mandatory, action 1)` where,
  **when `mandatory == 0`**, `o'` is `Unit::find_melee_target` within
  `min(dist + 0xc0, unit_respond_range × 0x240)` of the unit and the given
  target is only the fallback;
- `group.order_num += 1`.

So `engagement` does not point ten units at one soldier: each takes
whatever is nearest to it within the respond range, and the army's target
is the seed.

## 11. The captures

**Every function this document cites by address has executed in a traced
game.** `tools/trace/report.py <log> blind docs <every log>` over the
eighteen traces on disk lists 96 never-entered addresses across `docs/`,
and **none of them is one of this document's twenty-five**. The reading
below is therefore never the only evidence that a path exists — only, in
places, the only evidence of what it *computes*.

The first frames, from `report.py … functions` over run28
(`docs/ARMY.md` §16.6 — run24's islands game with six hoplites dropped on
the AI army's own point at 15020–15030):

| function | first entered |
| --- | --- |
| `Groups::Groups`, `Group::clear`, `Group::add`, `Group::compute_speed`, `GroupData::member`, `Form::Form`, `Form::init` | setup, before frame 0 |
| `Groups::process`, `get_open_slot`, `copy_group`, `push_group`, `equals_group`, `Group::sort`, `find_role`, `get_num_cap`, `action_begin`, `update_positions`, `action_move_to`, `action_move_near`, `compute_form`, `GroupData::find_leader` / `is_on_map` / `get_form` / `get_form_mod_option` / `get_loc` / `get_num_const` / `count`, **`Form::compute`, `categorize`, `compute_rows_and_columns`, `compute_dests`** | frame 0 |
| `Group::normalize`, `action_swarm_around`, `action_queue_up` | frame 1 |
| `Group::kill` | 238 |
| `Group::action_halt`, `set_up_insert`, `finish_insert` | 825 |
| `Group::action_stance`, `action_siege_attack_to`, `GroupData::get_stance_type` | 10232 |
| `Group::action_attack`, `is_attacking_to`, `GroupData::get_loc_to` | 13062 |
| `Group::target_opportunity` | 15020 |
| `GroupData::num_valid` | 15100 |

Two of those are worth naming. **`action_siege_attack_to` at 10232** is an
AI army with no siege, no wagon and no hero taking §9's whole-group
attack-move — the branch every traced army has taken, and the one the
simulation implements. **`Group::action_attack` at 15100**, inside
`Army::engagement`, is followed in the same frame by `Unit::find_melee_
target` and `Unit::add_attack_order`: §10's `mandatory == 0` retarget,
executed.

`Form::compute_dests` runs on frame 0 — the starting units' own group
orders — so the seam of §6.4 is a seam of *arithmetic*, not of
reachability. A `UNITS=3` dump of any early frame carries its output.

## 12. What the simulation carries, and what checks it

`crates/sim/src/group.rs` models the group as a **value** — the members and
the fields an action reads and writes — because the simulation collapses an
army to a single group (`docs/ARMY.md` §3.2) and has no player selection:

- the record's live fields (§1) on [`army::Army`] as `army::GroupState`,
  and `Group` as the value an action is applied to, built from an army
  (`Sim::army_group`) or on the stack (§9's sub-group, §2's probe);
- `find_leader` (§4.4) over the sim's units, `is_on_map`, `get_stance_type`,
  `num_valid`, `normalize`'s prune, `find_role`;
- **`action_halt`** (§7) whole, mask and all;
- **`action_stance`** (§8): the option count by stance type, the negative
  cycle, the per-member write and the combat table's three arms;
- **`action_move_near`** (§6): the domain split's `army < 0` guard, the
  `QUEUE_FIRST` rotation, `get_form`/`get_loc`, `compute_form`'s angle and
  reverse rules, **the AI branch of §6.5 whole**, the ordinary path's order
  choice, and the leader-path-plus-offset of §6.7;
- **`action_siege_attack_to`** (§9): the sub-group, the wagon and hero
  fallbacks, the anchor's Manhattan score, and the whole-group attack-move;
- **`action_attack`** (§10): the three domain passes, the `ignore` mask, the
  `mandatory == 0` melee retarget, `order_num`;
- and `army.rs`'s five callers become behaviour: `do_forming`,
  `march_to_target`, `engagement`, `send_here`, `charge`, `Army::close`'s
  halt and `set_stance`.

**Seams**, each with what it costs:

| seam | stands in for | what it costs |
| --- | --- | --- |
| `Form::compute`'s slot table (§6.4) | where in the formation each member stands | every member is given the **same** destination; the group arrives as a heap, not a line |
| the group pool (§3) | 64 slots a leader, `get_open_slot`'s recycling | one group per army, never recycled; `push_group`'s `force == 0` rule and `equals_group` are modelled, the slot allocation is not |
| `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `MoveOrder` — `docs/ORDERS.md` §8.4's verdict, unchanged |
| `action_guard` (§9) | the escort half of a siege attack | with siege *and* a matching area the non-siege members keep their orders instead of guarding; no traced army has siege |
| `find_nearby_spot`'s collision (§6.6 step 4) | re-slotting an invalid slot | the sim has no unit collision, so no slot is ever invalid |
| `invalid_loc` on a slot, the `tregion` re-slot | §6.6 step 4 | same |
| `QUEUE_FIRST`'s insert dance (§6.2, §10) | `set_up_insert` / `action_halt` / recurse / `finish_insert` | `charge`'s `QUEUE_FIRST` is a plain push-to-front on each member |
| the scenario filter (§5) | `ignore_orders` | never set outside a scenario |
| `unit_masks` `0x100` / `0x400` / `0x4000000` | three bits `action_halt` and §6.6 clear | unmodelled bits; no reader in the sim |

**The checks**, cheapest first, in `group.rs` and `army.rs`'s test modules:

1. `push_group_refuses_a_singleton_unless_forced` — §3.2's one live rule.
2. `action_move_to_gives_every_member_the_destination` — the ordinary path
   of §6.6, and the action bit.
3. **`a_hurrying_army_stables_its_siege_and_marches_the_rest`** and
   **`a_shooting_siege_unit_is_left_alone_by_a_move_that_is_not_hurrying`**
   — §6.5's two AI predicates, the part of this mechanic
   `docs/ORDERS.md` §8.2 had not read. Both were made to fail first, by
   `&&`-ing `false` into each predicate: the first then marches the siege
   into the fight, the second re-orders a unit mid-shot.
4. `action_halt_clears_the_orders_and_the_mask_exempts_siege` — §7's mask.
5. `action_stance_writes_the_combat_stance_and_cycles_on_a_negative` — §8's
   option count, the `−1`/`−2` cycle and `clear_mandatory` on 0/3/4.
6. `siege_attack_to_is_an_attack_move_when_the_group_has_no_siege` and
   `siege_attack_to_sends_the_siege_and_leaves_the_escort_behind` — §9's
   two arms; the second pins the `action_guard` seam where it stands.
7. `action_attack_without_mandatory_lets_each_member_take_what_is_nearest`
   — §10's melee retarget, against the same call with `mandatory = 1`.
8. In `army.rs`: `do_forming_walks_the_army_to_the_projected_muster_origin`
   (the origin is §8's projection, and the destination the 48-snap),
   `a_forming_army_actually_moves_its_units_over_the_ticks` (the end-to-end
   one: the order the group issued is stepped by `Unit::work` and the unit
   is nearer the muster spot 64 frames on),
   `engagement_points_the_whole_army_at_the_first_fighter_s_target`,
   `engagement_ignores_a_building_target`, `close_halts_the_units_it_held`,
   `send_here_moves_the_point_the_muster_cell_and_the_units`, and
   `charge_drags_the_army_onto_the_attacker`.

9. In `rondata::diff`:
   `run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_point`
   — not this document's mechanic, but the one that makes it reachable:
   the two blocks either side of run28's frame, and the release that
   leaves no `FORMING` bit so `Army::engagement` can call §10
   (`docs/ARMY.md` §16.6, §17 item 6).

What no test pins is the arithmetic of §6.4, because the simulation does
not carry it, and the *positions* of an army's units in a traced game,
because `scene_at` does not yet read a block's `UNITDATA` order lists
(§13).

## 13. What is not established

- **`Form::compute`'s slot table** (§6.4), the largest gap. *Capture:* a
  `UNITS=3 COMMANDMANAGER=1` dump of a `do_forming` frame — one of run21's
  army-0 ticks at `252 + 256k` — reading the `MOVEORDER` destinations of
  every member of one army against the muster point. Each member's `dest`
  is the slot table's output directly, and eight units in one group would
  pin the row-and-column arithmetic for one form index. The same run gives
  `group.angles[i]` in the order's packed top byte.
- **Which form index an AI group carries.** `action_move_to` passes −1, so
  `get_form()` decides, and that is the members' `unit +0xaa` — written by
  the previous `action_move_near` and by nothing else the export shows for
  an AI unit. The first move of a group therefore runs on whatever
  `Setup::build_units` or `come_out` left, which is not read. *Same
  capture:* `UNITS=3` prints the unit record.
- **`FormData::type_cat`** (§4.4) decides which member leads a mixed group
  and is unread; the simulation takes the group's first on-map captain,
  which is right for a one-type army and wrong for a mixed one. *Capture:*
  the same dump, with a wagon and a hoplite in one army — the leader is the
  unit whose `MOVEORDER` destination is the group's own point.
- **The four `TypeIndex` values 0x32..0x35** exempted from the `+0xaa`
  write in §6.6 step 1 have no names in the export. They are the four ids
  `determine_roles` calls `citizen_id` (`docs/DATALAYER.md`'s
  `role::CITIZEN`), so the exemption reads as "a citizen or scholar keeps
  whatever formation it had" — but that is an identification by index, not
  by a type record, and it is not settled.
- **`role & 0x10` in §6.6 step 6.** `docs/ORDERS.md` §8.2 glosses it
  "workers, caravans"; `UnitType::determine_roles` gives `0x10` as
  `is(SCOUT)` on land and `is(BARK)` at sea. One of the two readings is
  wrong. It matters only for the `GroupMoveOrder`-vs-plain choice, which
  is a seam either way (§12, the seam table). *Check:* the same `UNITS=3` capture — a
  scout in a group of two shows which order kind it took.
- **`ObjectData` vslots `+0x1c`, `+0x20`, `+0x48`, `+0x108`, `+0x10c`** are
  read from their uses, not from a type record: "is a building", "can
  attack", the one `action_attack` reads to decide a member must walk
  rather than shoot, "stance type" and "is siege". `+0x10c` is confirmed by
  `docs/ARMY.md` §3.3's `COUNT_SIEGE`, which reads the same slot; the other
  four are inferred. *Check:* `vtables.txt` for the concrete classes.
- **`pathfinder +0x70`** (§6.7) is set to 1 around an army group's
  `find_wpath` and cleared after; what reads it is inside `astar_path`,
  which `docs/PATHFINDER.md` did not reach.
- **`Group::priority`** gates `normalize`'s prune and has no writer in the
  export — the same shape as `docs/ARMY.md` §18's `rally_dist`.
- **`action_guard`** (§9) is 300 lines and unread; `GUARD` is not
  implemented in the simulation either (`docs/ORDERS.md` §14).
- **The blind list this document adds:** `Groups::get_open_slot`'s
  "UH OH, NEED MORE GROUPS" path, `Group::sort`, `refresh_group_order`,
  `distribute_attack`, `kill_group_move` — none of which any traced game
  has executed. `tools/trace/report.py … blind docs/` will list them.
- ~~**No second reading yet.**~~ Run and adjudicated the same day: §14.

## 14. Second reading — landed, **corrections not yet applied**

Two blind readers on Opus 5 (A over §6–§8, B over §3, §4, §9, §10), and a
third adjudicator on Opus 5 against the decompile, the listing,
`rise_z.map` and the PE. `docs/audit/2026-08-25-groups.md` is the record —
124 verdict rows, five `FABLE:` markers, eight named assertions.

**Until those corrections land, this document is the one that is wrong**
(`docs/audit/README.md`). What is already known to be wrong here, so that
nobody implements from it in the meantime:

- **§6.4 and §13's account of the slot-table seam is void on both legs.**
  There is no float barrier — zero float instructions across the `Group`
  family, fourteen in `Form::compute_dests` alone, all integer-exact as
  `((2·rows − 1)·depth·k)/2`, the `0.5f` read from the PE at `0xb694c0`.
  And "no capture pins its output" is **false**:
  `GroupData::log_data@0045e1d0` dumps `off_x`, `off_y`, `curr_x`,
  `curr_y`, `angles`, `form`, `form_num`, `o_dist` and `o_angle` per
  member, and run29 already carries a four-member group in formation 0.
  §13 asked for a capture that was on disk.
- **§13's five guessed `ObjectData` vtable slots: three are wrong.**
  `+0x48` is `is_seen`; `+0x20` is `is_build`, and **0 for a Wall**;
  `+0x10c` is settled by `ObjectData::is_siege@0046ef90`. §8 and §10 build
  rules on the guesses.
- **§13's "`Group::priority` has no writer in the export" is wrong** —
  there are five. The field is a one-bit "I am a control group".
- **Two live bugs in `crates/sim/src/group.rs`**: `group_action_halt`
  writes each *unit's* `form` where `0070d0c0:29` writes the *group's* —
  and `group_get_form` reads the unit bytes, so it is not cosmetic — and
  `group_action_attack`'s "already attacking" skip is unconditional where
  the original's has two sub-arms.

What survived: **§9's Manhattan anchor**, which reader B missed entirely
and which the adjudicator confirms per-term `>> 10` and strict `<`; the
sim's `siege_anchor` is right. The full list of what is doubly confirmed
is in the audit.
