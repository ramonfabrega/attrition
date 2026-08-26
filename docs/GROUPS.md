# Groups — the order layer between an army and its units

*First reading, 2026-08-25, in the main thread on Opus 5, from the Ghidra
export (`~/ghidra-projects/decomp`), the PDB type records (`types.txt`,
`vtables.txt`) and the four traced runs of `docs/ARMY.md` §16. Written to
close the one seam `docs/ARMY.md` §17 names: the AI's armies decide but
never move a unit, because the simulation had no `Group::action_*`.*

***Second reading run, adjudicated and applied, 2026-08-26** —
`docs/audit/2026-08-25-groups.md`, 124 verdict rows. Two blind readers and
an adjudicator, all on Opus 5. Every correction it names has landed in this
document and in `crates/sim/src/group.rs`; §14 is the record of what was
wrong and where each fix went. The first reading gave this document **low**
standing and was right to: it never opened `rise_z.map`, and §13 guessed at
five vtable slots the PE names outright.*

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
only and is a declared seam** (§6.4): it is what decides *where within the
formation* each member stands, and it is 1,200 lines across four functions.
That is the whole of the reason now — the second reading killed the other
two the first gave (no float barrier, and the output *is* captured; §6.4).
Everything the army's behaviour turns on — which unit gets which *kind* of
order, and when — is below it and is read.

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
  +0x30  priority    int     read by normalize's prune. A one-bit "this slot is a
                             live control group": 1 from
                             HotKeyGroups::clear@00715230,
                             HotKeyGroups::find_group@00714d00 and
                             Object::replace_hotunit@00643b70; 0 from
                             Group::clear@00713e80 and Groups::clear@00713f20 —
                             and Group::kill's `num == 0 -> clear(-1)` therefore
                             *loses* the bit, so an emptied hotkey slot reads
                             0 until it is allocated again (run29's slot 28)
  +0x34  role        int     OR of the members' type roles (find_role)
  +0x38  think_frame int
  +0x3c  new_speed   int     \ the group's march speed, reset every frame by
  +0x40  speed       int     / Groups::process to the leader's own
  +0x44  form_num    int     the member count Form::compute_dests laid out
  +0x48  facing      uchar   the formation's mirror flag (§6.3). Four writers:
                             Group::clear (0), compute_form's toggle/restore
                             pair (§6.3), and two outside this family that no
                             blind brief reached — Unit::kill_current_order
                             @005e2cb0, which writes the dying move order's own
                             reverse flag onto the group when the unit is the
                             group's leader, and Unit::set_angle@00605400,
                             which *toggles* it whenever the leader turns by
                             90° or more (the third pass; the ordinary source
                             of a live group's `facing 1`)
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
first **46** are the recycling pool (the loop bound is `base + 0x2e`).
`last_group[8]` is initialised to `{p × 0x40}` by `Groups::clear@00713f20`
— run29 prints `last_group 0 / 64 / 128 / 192 / 256 / 320 / 384 / 448` —
and that one slot per player is the one `get_open_slot` never returns.

**What the other 18 slots are for is open.** They are *not* "the hotkey and
selection groups": `HotKeyGroups` is a **separate array**
(`HotKeyGroups::clear@00715230` writes `hotkey_groups._16_4_`, not
`groups._16_4_`), and it is dumped separately — run29's `HOTKEYGROUPS` block
holds a `HOTKEYGROUPDATA` wrapper per hotkey group with the `GROUPDATA`
record nested inside it, while the pool is 512 flat `GROUPDATA` records
(8 leaders × 64) directly under `FULL DUMP`. Slots `who*64+46 … +63` of the
pool are allocated by nothing and searched by nothing; only
`Groups::process` still cycles them. `HotKeyGroups` and `SelectGroups` do
share `Group`'s `add`/`kill`/`action_begin` vtable slots, which is what the
first reading was seeing.

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
- otherwise a slot whose `stamp <= local_18` and whose **`get_num_cap() == 1`**
  — vslot `+0x8`, so **exactly one captain**, which is not the same number
  as one member — and which is not `last_group[who]`, becomes the running
  best and `local_18 = its stamp`. So the fallback is the *oldest group with
  a single captain*, and because `local_18` is lowered on each hit while the
  comparison stays `<=`, among slots of **equal** stamp the **last** one
  wins.

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
  `buildings`, and the same ids in the same order) the existing slot is
  reused; else `get_open_slot`, `copy_group` into it, and
  `last_group[who] = it`. `equals_group` normalises **only a side whose
  `id != −1`** — so a group built on the **stack** is never normalised,
  which is exactly every `Army::add_unit` and `find_target` probe call.
  `Groups::copy_group@006fa690` copies nine things and leaves `id`, `army`,
  `form`, `disband`, `order_num`, `priority`, `role`, `think_frame`,
  `new_speed`, `form_num`, `facing` and `march` behind — a reused slot keeps
  the previous occupant's.
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
`param_4`; `param_4` chooses `get_num_const` over the virtual `get_num`.
Both recursions **forward `param_4` unchanged**; the only site that
originates `param_4 = 1` is `Group::sort@00708090`'s fix-up
`add(o, who, 0, 1)`.

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
`priority == 0` and `id >= 0` — satisfies
`(!is_unit() || unit->group != id) && !is_build()`. Two halves matter and
the first reading had neither: an object that **is not a unit at all** is
culled outright unless it is a `is_build`, and the exemption is `is_build`
(vslot `+0x20`), which is **0 for a Wall** — so a *wall* member is subject
to the back-pointer cull and a plain building is not. Then `find_role` (the
OR of the members' `type.role`) and `speed = new_speed = the leader's
speed`.

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

**Implemented 2026-08-26** (`Sim::group_find_leader`), which is what the
key had been waiting for: `sim::form::type_cat` was built for
`Form::compute` and this is its second reader. The listing's winner test
is `local_8 < 0 || cat < best`, so the *first* qualifying member takes the
lead unconditionally and only a strictly lower category displaces it. The
simulation took the first on-map captain outright until then — right for a
group of one category, wrong for any other, and no dump held another until
run31.

**The capture is taken, and it is better than the one this entry asked
for.** ~~*Capture:* a group of **two** categories whose members have
**different headings** … `GROUPDATA`'s `curr` is then the only field that
says whose heading `update_positions` rotated by.~~ `curr` is not needed:
**`GroupOrder::oxx` names the leader's object outright** (§12.1), on every
member's order, and the slot table's anchor is that object's slot. run31
is twelve squads selected slingers-first, so `list[0]` is
`FORM_CAT_FOOT_RANGED` while the group's lowest category is
`FORM_CAT_FOOT`; the record's leader is a hoplite, which the
first-on-map-captain rule cannot produce. The check is
`run31_s_group_order_names_a_leader_the_first_member_rule_would_miss`.

**What it does *not* settle**, and it is now the most concrete open
question this document has: run31's leader is object **9** on frame 204
and object **6** on frames 328 onward — both hoplite captains, both
`FORM_CAT_FOOT`, and object 6 comes first in `list`. A tie going to the
first would name 6 every time. So either the walk is not the `list` order
this document assumes, or one of the two failed a test on that frame.
Nothing in the window says which; a reproduction of the whole 36-member
table (§12.1) is what would.

`GroupData::is_on_map@0070c450` is true for a building group with members,
and for a unit group with at least one active on-map captain.

`GroupData::get_form@0070b9f0` returns the `unit +0xaa` shared by every
active on-map member, and **−1 as soon as two differ** — a mixed group has
no formation. ~~`get_form_mod_option` is its width twin at `+0xab`.~~
**It is not a twin**: `get_form_mod_option@0070bd00` is the **mean** of
the members' `+0xab` bytes over those that are not −1, falling back to
`0x32` when none qualifies, which is what `crates/sim/src/form.rs`
implemented from the function itself. Run29 separates the two outright:
army 0's seven members carry `form_mod` `[−1, 50, 50, −1, 50, 50, 50]` and
the option is **50**, where a `get_form`-shaped rule would give −1
(`run29_s_halted_group_kept_its_members_formation_bytes`).

`GroupData::get_stance_type@0070d370` seeds from **`find_leader`** for a
unit group and from **`list[0]`** for a buildings group, and reads its
`vslot 0x108` (`get_stance_type`); if that is `STANCE_NONE` or
`STANCE_CASTER` it walks the members and takes the first non-`NONE` that is
not `CASTER`, falling back to `CASTER` if one was seen. `docs/ARMY.md` §6's
`set_stance` acts only on `STANCE_COMBAT` groups.

A unit's own `get_stance_type` is the **type's**,
`UnitTypeData::get_stance_type@0061d350` (the third pass; no earlier pass
had read it), and its order of tests is the fact — with the PDB's
`StanceTypes`, `COMBAT = 0`, `WORKER = 1`, `CASTER = 2`, `PACKER = 3`,
`NONE = −1`:

```
if (role & MILITARY)         -> (unit_flags2 & 4) ? PACKER : COMBAT
if (TypeIndex in 0x32..0x35) -> WORKER
if ((unit_flags2 & 6) == 2)  -> CASTER        // a caster that does not pack
else                         -> NONE
```

So a **military caster is combat**, a **civilian packer has no stance**,
and having an attack decides nothing — the `MILITARY` role bit does.
`UnitTypeData::has_stance_type@0061d3b0` is `get_stance_type() == s`, so
`get_stance_option`'s histogram filter (vslot `+0x104`) and
`action_stance`'s write filter (`+0x108`) are the same predicate.

`GroupData::get_stance_option@0070bab0` is the one §8's cycle actually
steps from, and the first reading never read it. It is the **modal**
option: a `count`-entry histogram (6 / 4 / 2 by stance type) over the
members that are valid and answer `has_stance_type(type)` (vslot `+0x104`),
counting each member's `unit +0xb1` — or `Build +0x7e` for a buildings
group — and returning the **argmax on a strict `<`**, so a tie goes to the
lowest index and an empty histogram gives 0. Its out-parameter is
`(distinct options seen < 2)`, i.e. "the group agrees".

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
split is re-done with a different rule — a **sea transport** and a *carried*
unit go to the first group, everything else to the second — and the same
recursion applies under the same `army < 0` guard. The test at `704cf4` is
`(ptype->+0x2b4 & 0x10) == 0`, `unit_flags` bit `e`, which this project has
named `uflags::TRANSPORT` since the AI mechanic
(`crates/sim/src/ai_load.rs`); it is **not** `is_special`, which is
`unit_flags2 +0x2b8 & 0x10`.

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
  caller's angle stands, and `reverse = |find_angle(dx, dy) − angle| ≥ 90°`.
  The idiom is `0x3fffffff < d && d < 0xc0000001` at `707e09`–`707e1a`, so
  **both bounds are inclusive**;
- if the leader's facing minus the given angle is `≥ 90°` by the same
  idiom, `group.facing` is **toggled** around the `Form::compute` call and
  toggled back afterwards — and the two toggles are **exactly symmetric**.
  The listing settles it: both are guarded by the same compare against
  `%esi`, computed at `707ea8` and never rewritten (`Form::compute@0072e8e0`
  pushes `ebx`/`esi`/`edi` on entry). So in an ordinary game
  `compute_form` leaves `group.facing` **as it found it**;
- **unless `game->semaphore.ptr[1] & 8` is set**, in which case `facing` is
  forced 0 and `reverse` false *before* the call, and the restore — still
  reading the pre-override `%esi` — flips `facing` to **1** on the way out.
  That bit is **bit 11 of `GameData +0x814 BitMask<256> semaphore`**, and it
  is not a network flag: `ConsoleWin::run_cmd@007d6a70` **sets** it right
  after `ScenarioEditor::init` and **resets** it right after
  `ScenarioEditor::close`, and `Options::do_formation@007215b0` calls
  `Group::dbg_jump_to_action` under it. It means **the scenario editor is
  open**. So the first reading's "a networked game never mirrors" is wrong
  twice: wrong about the flag, and wrong that the flag pins `facing` at 0;

- `Form::compute(form, group, x, y, angle, width, group.facing)`;
- and when `reverse` holds, every member's `off_x/off_y` in both the `Form`
  and the group are **negated**.

`Form::compute@0072e8e0` is `compute_rows_and_columns` then `compute_dests`,
and finally `group.o_angle = find_angle(…)`, `group.o_dist =
vector_dist(…)`.

### 6.4 The slot table

**Implemented and diffed** — `crates/sim/src/form.rs`, and
`run29_s_navy_slot_table_is_reproduced_from_the_install_s_own_spacing` in
`rondata::diff` reproduces run29's own row from the install's own columns.
This section is the table itself; what it still does not reproduce is at
the end, and every one of those is the original's doing rather than the
port's.

`Form::compute_rows_and_columns@0072d910` and
`Form::compute_dests@0072cba0` fill, for each member `i`, a destination
`to_x[i]/to_y[i]` (Form `+0x514`/`+0x714`), an offset `off_x[i]/off_y[i]`
(`+0x914`/`+0xb14`) and a facing byte `group.angles[i]`.

#### The record `Form` writes into

`FormData` is named field for field in the PDB, and the ten `Form` objects
are **both** `Forms::init`'s formation definitions **and** the scratch
buffer: `compute_form` takes `&forms.list[form]` (each `0xe98` bytes) and
`memset`s it from `+0x30` for `0xe60`, so only `form` (`+0x28`) and
`density` (`+0x2c`) survive between calls. `Form::init@0072dda0` sets
`density` to **2** for formations 0–4, **0** for Sparse, **1** for 6–9.

The type columns the table reads, named from the engine's own dumps rather
than inferred: `+0x1e8 attack` (`ObjectType::backup` assigns it to a field
called `attack`), `+0x224 guy_spacing`, `+0x228 x_spacing`,
`+0x22c y_spacing` (`ObjectType::log_data`'s own UTF-16 labels at
`0xadb0bc`/`0xadb0f4`) and `+0x308 uber_size` (`UnitType::log_data`, string
at `0xada018`). `UnitType::init@0061ab50:646`–`654` stores `x_spacing` and
`y_spacing` as the `X_SPACING`/`Y_SPACING` columns **times**
`UNIT_FORMATION_SPACING`, and `guy_spacing` as its column times
`UNIT_GUY_SPACING`. `unit_formation_spacing` is **12**, printed beside
every dumped group.

#### `Form::categorize@0072e250`

Two passes over the member list, each opening on active (`+0x8`), on the
map (`+0xbc`) and **a captain**:

1. everything whose `FormData::type_cat` is not `FORM_CAT_COMMAND`;
2. the commanders, each folded into the first non-empty category **above**
   the biggest of pass 1 — `biggest + 1` when there is none, and
   `FORM_CAT_COMMAND` outright when the formation is Square. A group of
   nothing but commanders leaves `biggest` at 8 and folds every one into
   `FORM_CAT_COMMAND_RANGED`, which `type_cat` itself can never return.
   `biggest` moves on a strict `<`, so ties go to whichever category
   reached the count first.

Each member gets `category[i]` and `cat_id[i]` (its index within the
category), and its category's `num_category`, `x_spacing` and `y_spacing`
are widened: for `uber_size == 1` by the type's own two, otherwise by
`min(uber_size, 3)` (**2** in a Column) times the width across and
`(⌈uber_size / figures⌉) × y_spacing` back, plus `0x30` more for modern
infantry — and *only in pass 1*, because the commander fold does not ask.
`action_guard`'s flag (`+0xd1c`) injects a **phantom** member into
`FORM_CAT_ARTILLERY` with spacings `0xc0`/`0x180` before either pass.

The leader is the member with the **lowest** category index, first one
wins; `+0x34 idx` is its slot and `+0x30 o` its object.

`FormCatIndex`'s eighteen values are the PDB's own `LF_ENUM` record, and
`FormData::type_cat@0072dfc0` returns only eight of them —
`MECH`, `MOUNTED`, `FOOT`, `FOOT_RANGED`, `MOUNTED_RANGED`, `ARTILLERY`,
`COMMAND`, `CIVILIAN`. In this project's own names for the same words:

```
armed = attack != 0 && !(unit_flags2 & SUPPLY_OR_HERO) && !is_caravan
        && type ∉ {MERCHANT, MERCHANTDUTCH, FURTRAPPER}
if armed:
    obj_masks & CIVILIAN                 → CIVILIAN
    obj_masks & FOOT                     → FOOT + (max_range != 0)
    unit_flags2 & PACKS
      or obj_masks & ANTI_AIR            → ARTILLERY
    obj_masks & MOUNTED                  → (max_range != 0 && human)
                                             ? MOUNTED_RANGED : MOUNTED
    obj_masks & VEHICLE                  → MECH
    otherwise                            → ARTILLERY
otherwise:
    unit_flags2 & 0x60 (SPECIAL_FORCES)  → ARTILLERY
    obj_masks & CIVILIAN                 → CIVILIAN
    otherwise                            → COMMAND
```

Two consequences worth stating. **A warship falls off the end of the tree
into `FORM_CAT_ARTILLERY`** — its `OBJ_MASK` is `N`/`NRL`, none of which
the tree tests — which is why run29's navy is in category 6, the one
category that also staggers its odd rows. And the `human` gate is a real
per-player divergence: **the same horse-archer army forms up as
`MOUNTED_RANGED` for a person and plain `MOUNTED` for the AI.**

#### `Form::compute_rows_and_columns@0072d910`

`game/Data/rules.xml` lines 1446–1487 give **ten** formations in document
order — 0 Line, 1 Refused, 2 Envelop, 3 Echelon Right, 4 Echelon Left,
5 Sparse, **6 Square**, **7 Wedge**, **8 Column**, 9 Mob — and
`Forms::init@0072e9a0` errors unless the count is 10. The general arm is

```
span   = max over c of (count[c] < 6 ? count[c]·width[c]
                                     : count[c]·width[c]/2)
cols[c] = clamp(1, count[c], ((span · form_mod)/50)/width[c])
rows[c] = ceil(count[c]/cols[c])
```

with **Column** short-circuiting to `rows[c] = ⌈count[c]/3⌉` and leaving
`cols[]` uninitialised — safe only because Column's own placement arm
never reads it. `form_mod` is the call's `width`;
`GroupData::get_form_mod_option@0070bd00` supplies it when the caller
passes −1, which every army call does, and it is **50**: the mean of the
members' `unit +0xab` bytes over the non-plane on-map ones, with `0x32` as
the fallback for a building group, an empty group, and — the live case —
a group whose bytes are all still −1. Because §6.6 step 1 then writes that
same 50 into every member, it stays 50 for ever. run29's `UNITDATA` prints
`form_mod 50` on each member of the navy, so this is a **record**, not a
reading.

**Wedge** picks a priority category `+0xd14` by a fixed preference —
foot, foot ranged, mounted, mounted ranged, mech, mech ranged, artillery,
artillery ranged, then the first non-empty — gives every *other* category
`cols[c] = max(count[c]/2, 10)`, and counts its own rows 1, 2, 3, … until
the members run out.

#### `Form::compute_dests@0072cba0`, the captain arm

With `cat = category[i]`, `slot = cat_id[i]`, `w = x_spacing[cat]`,
`d = y_spacing[cat]`, `cols = cols[cat]`, and `k = 1` always — `+0x2c` is
overwritten by 2 for any member with a category, which is every member
(`0072cba0:166`–`169`):

```
q    = slot % cols ;  row = slot / cols
X    = (q even ?  ((q+1)>>1)·w  :  −((q+1)>>1)·w)
if cols is even and not guarding:                    X += w/2
if (cat < 3 or cat == 6 or cat > 11) and row is odd: X += w/2
Y0   = −k · row · d
   1 Refused       : Y = Y0 − |X|
   2 Envelop       : Y = Y0 + |X|
   4 Echelon Left  : Y = reverse ? Y0 − X : Y0 + X
   3 Echelon Right : Y = reverse ? Y0 + X : Y0 − X
   otherwise       : Y = Y0
X    = reverse ? −X : X                       # the group's own `facing`
angles[i] from the signs of (X, Y − Y0); with X == 0 exactly,
   formation 4 → 0xe0, formation 3 → 0x20, else 0
```

then the rank stack, which puts each category behind the ones ahead of it:

```
if wedge >= 0:  Y −= x_spacing[wedge] · rows[wedge] · k
prev = −1
for c in 0 ..= cat:
    if c != wedge and count[c] != 0:
        if prev >= 0:  Y −= (k · d[c]) / 2
        else:          prev = c
        if c < cat:    Y −= trunc((rows[c] − 0.5f) · d[c] · k)
```

**Column** is `X = ((slot+1) % 3 − 1)·w`, `Y = −(slot/3)·d`, and **Mob**
puts slot 0 on the anchor and the rest on concentric rings of 5, 10, 15, …
Both then take the rank stack above. **Square has no placement arm at
all** — see the four limits below.

`to[i]` is the destination rotated by the formation angle:
`to_x = dest_x + cos·X + sin·Y`, `to_y = dest_y + sin·X − cos·Y`, the same
matrix `update_positions` uses (§6.6), with the calls inlined so the
listing shows `sin_table` rather than `sinx`/`cosx`.

#### The anchor, and the one place `to` and `off` disagree

The last loop slides the whole block so that one member sits at the
origin, and quantises: `group.off[i] = div_3_table[form.off[i] >> 4]`.
Two things about it were wrong or missing in every earlier pass.

**The anchor is the first member of the *lowest-indexed* non-empty
category, not the last.** The rank-stack walk's `prev` is written **only
while it is still negative** — `72cfe2 movl %ecx, %esi` sits on the
`prev < 0` arm, and the `prev >= 0` arm reloads the stack slot it never
wrote (`72d837`, and the same shape at `72d81a`/`72d878` in the guarding
copy). So `prev` sticks at the first non-empty category it meets and never
advances, and `prev == cat` — the test that arms the anchor at
`:389`–`:394` — can only hold for a member of the lowest non-empty
category. Reader A's `prev = c` (A.28) is wrong and this document repeated
it; the listing settles it.

**The destinations keep the anchor's x that the offsets lose.**
`off[i]` is slid by the whole anchor, but `to[i]` is slid by
`cosx(angle, 0) + sinx(angle, −anchor_y)` — and `%edx` is **explicitly
zeroed** at `72d737` before the `cosx` call, which returns 0 for a zero
distance (`0092d0c7`). So a group whose anchor is off-centre marches to
destinations displaced from where its own offsets say it will stand, by
exactly the rotated `anchor_x`; and an **even** column count is precisely
what puts the anchor off-centre, so this fires on the commonest case
there is. It is invisible in `GROUPDATA`, which logs `off` and `curr` and
not `to`. *Capture:* a `UNITS=3` dump of a group move — the members' order
destinations against their `off_x`.

run29's `UNITS=3` half was opened on 2026-08-26 and **is not that
capture**. Every order block in its four states was read: the one group
with non-zero offsets — the navy, `id 66` — holds **no orders at all**,
and the one group whose members hold move orders — army 0's, `id 69` —
carries `form −1`, `form_num 0` and every offset zero. The window has no
group that both stands in a formation and is walking to one.

~~What is wanted is a **human** group move.~~ **run31 is that capture,
and the asymmetry is observed** (`docs/ORACLE.md`, run31; the check is
`run31_s_anchor_marches_to_a_point_its_own_offset_says_is_the_click`).
Twelve squads right-clicked to a far point: the one member whose slid
offset is exactly `(0, 0)` — the anchor — holds a `GroupMoveOrder` whose
destination is **not** the click its own `orig_x`/`orig_y` records, on all
three moves and all forty frames they are readable over. Were `to` slid by
the whole anchor rather than by its `y` alone, that member's order would
point at the click itself. Frame 204's is the click `(9123, 5841)` against
a destination of `(9096, 5640)`.

What the capture does **not** yet do is reproduce the displacement's
size. That needs the whole 36-member table, which needs the follower arm
(§12.1) and an account of the leader (§4.4); it is the next step and the
record is on disk for it.

The quantisation is a **floor**: `init_coord_lookup_array@00681db0` builds
`div_3_table` as `j / 3` for `j ≥ 0` and `(j − 2) / 3` for `j < 0`, which
is `floor(j / 3)` on both sides, with the pointer aimed into the middle of
`orig_div_3_table` so a negative index is legal, and `>> 4` is an
arithmetic shift.

#### run29, end to end

Everything above closes on one row of one record, with no free parameter:

| step | value |
| --- | --- |
| `X_SPACING 55 × UNIT_FORMATION_SPACING 12` | `w = 660` |
| four warships, all `FORM_CAT_ARTILLERY` | `count[6] = 4` |
| `span = 4 × 660` (four is under six) | `2640` |
| `cols = ((2640 × 50)/50)/660`, `form_mod 50` | `4`, so `rows = 1` |
| slots `0, −w, +w, −2w` | `0, −660, 660, −1320` |
| `cols` even, not guarding: `+ w/2` | `330, −330, 990, −990` |
| anchor is slot 0 of category 6, slid out | `0, −660, 660, −1320` |
| `floor(· / 48)` | **`0, −14, 13, −28`** |

which is `GROUPDATA` `id 66`'s own `off_x`, with `off_y` and `angles` all
zero and `form_num 4`. `curr = [(0,0), (473,480), (−440,−446),
(946,960)]` follows exactly from §6.6's `update_positions` under the
leader's logged `angle` of `−1_605_566_464`. A **truncation** instead of
the floor gives `[0, −13, 13, −27]`, which is why the rounding had to be
settled first.

#### The four things this does not reproduce

Each is the original's, not the port's:

1. **Formation 6, Square, is dead code in the shipped executable.**
   `compute_rows_and_columns`' whole `== 6` arm fills
   `FormData::space[18][4]`, `across` (`+0xe8c`) and `per[7]` (`+0xd3c`) —
   and **nothing in the 48k-function export reads any of the three**
   (`grep field_0xd68 field_0xe8c field_0xd3c` finds the constructor and
   that arm, and nothing else). `compute_dests` has no Square branch at
   all — `if (form != 6) { … }` with no `else` — and the same arm sets
   `wedge = −1`, so the wedge arm cannot stand in for it either. Every
   member's `to` and `off` stay at the `memset` zero, except member 0's
   `to`, which `compute_form` seeds with the order's own point.
   *Capture:* a `GROUPDATA` frame for a player's group set to Square —
   every `off` should read 0.
2. **A wedge's own row count is seeded from uninitialised stack.**
   `Form::compute` declares `int rows[18]` and never initialises it, and
   `compute_rows_and_columns`' wedge arm **reads `rows[wedge]` before
   writing it** (`72dc90`, `movl (%ebx,%esi), %eax`). The placement inside
   the wedge does not depend on it — `compute_dests` re-accumulates from
   `FormData::total`, which *is* zeroed — but the rank stacking of every
   *other* category subtracts `x_spacing[wedge] · rows[wedge]`, so **a
   wedge with a second category is not reproducible by anyone**, us
   included. The simulation seeds it 0.
3. **Mob past its first member.** Slot 0 on the anchor is exact; the rings
   need `cosx`/`sinx` arguments the decompiler drops and no pass has
   recovered from the listing (`72ce96`–`72cec5`, magic-number divides and
   a wrapping counter that grows its period by 5).
4. **`categorize`'s two type substitutions.** A loaded sea transport is
   sized by its **cargo's** type; a land unit ordered onto **water** — the
   test is `compute_form`'s own `(terrain & 0x30) == 0x20` at the
   destination — is sized as the leader's current Transport Barge, or
   Merchant Fleet if it is a caravan. The simulation keeps no cargo list,
   so each member is sized by its own type.

And two smaller facts the listing gives for free: `compute_dests` takes an
**eighth stack argument that is pushed uninitialised** (`pushl %ecx` at
`72e914`, `%ecx` clobbered by the `compute_rows_and_columns` call) and
never read; and `compute_form`'s reverse negation at the tail applies to
`off_x`/`off_y` on **both** the `Form` and the group and **not** to `to`,
and only when `param_10 == 0` — so a guarding call never mirrors.

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

**The three loops do not gate on the same thing, and the gap has a
consequence — but not the one the second reading drew.** The `QUEUE_NEW`
clear at `70524f` exempts a shooting siege unit only when `local_44 == 0`
— i.e. only when the army is **not** hurrying — while the order loop at
`7054c7` takes the "leave it alone" arm when `local_44 == 0 **or**
local_48 < 0`, i.e. also when the army *is* hurrying but `find_city`
returned −1. The second reading concluded that a hurrying army with no
city "clears its shooting siege unit's orders and then issues it nothing".
**The third pass overturned that**: the order loop re-evaluates
`UnitData::order_type@00616e80` *after* the clear loop has run
`Unit::clear_orders` → `close_orders@005e37f0`, which kills every order,
and `order_type` returns `NONE` on an empty list — so the
`is_siege && order_type == ATTACK` test at `7054c7` no longer matches and
the unit **takes the move like every other member**. The whole truth of
the shooting siege unit, by queue position: not hurrying → left alone;
hurrying with a city → into the city; hurrying without one → under
`QUEUE_NEW` cleared and marched, under `QUEUE_LAST`/`QUEUE_FIRST` (no
clear loop) still shooting and skipped. `crates/sim/src/group.rs` does
exactly that
(`a_hurrying_army_with_no_city_clears_and_marches_its_shooting_siege`).

### 6.6 The ordinary path, per member

1. `unit +0xaa = form` unless the type's `TypeIndex` is one of 0x32..0x35 —
   `PEASANTS`, `PEASANTSKOREAN`, `SCHOLARS`, `SCHOLARSKOREAN`, the PDB's own
   `TypeIndex` enum — and `unit +0xab = width`. Those are exactly the four
   ids `UnitType::determine_roles@0061c320` opens by giving `role = 0x200`,
   which is this project's `role::CITIZEN`, so "a citizen or scholar keeps
   whatever formation it had" is right and is now an identification **by
   type record**.
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
   the destination as a `UCoord` (`/0x30`), the angle as
   **`(signed char)group.angles[i] × 0x1000000 + angle`** — a *signed* byte,
   and an **addition**, not `<< 24 |`; `compute_form` subtracts the same
   product — `pathed = 1`, the action bit from `param_8`, the formation's
   reverse flag, and the original click for the order's `orig`.

   `role & 0x10` is settled by the type record:
   `UnitType::determine_roles@0061c320` sets it in exactly two places —
   `is(0x45)` for a **land** type and `is(0x143)` for a **sea** one — and
   the PDB's `TypeIndex` names those `SCOUT` and `BARK`. (Vslot `+0x60` is
   `ObjectTypeData::is@0065f7d0`, the lineage test;
   `UnitType::init_final_flags@0061dc70` calls it with the constants that
   reproduce five of `ai_load::uflags2`'s names, which this project derived
   independently.) So the gate reads **"a scout on land or a bark at sea,
   unless `unit_masks & 0x40000`, marches alone rather than in the
   formation"** — and `docs/ORDERS.md` §8.2's gloss "workers, caravans" is
   wrong.
7. `unit_masks &= ~0x400`.

Then `update_positions(leader)` turns every slot offset into
`curr_x/curr_y`. The listing at `7139e8`–`713a41` is sharper than
"rotates … by the leader's heading": the sine-table row is reached by
`leal (%ecx,%ecx,2)` + `shll $4`, i.e. **× 48**; `curr_y = sin·x − cos·x`
is a **subtraction**, so the matrix is a rotation composed with a **y-flip**
(determinant −1, and a naive port mirrors); and the loop bound at `713a38`
is `cmpl 0x44(%eax)` — **`form_num`, not `num`**. The third pass read the
whole function (`00713810`): θ is the calling unit's own heading
(`unit +0x50`, loaded at `713844`), replaced by the angle from the unit to
its current order's `(+0x2c, +0x30)` point when that order's `+0x10` is
set; then, for each of `form_num` members,

```
curr_x = off_x·48·sin(θ + 90°) + off_y·48·sin(θ)
curr_y = off_x·48·sin(θ)       − off_y·48·sin(θ + 90°)
```

Its only caller is `Unit::do_group_move@005e79a0`. `UNITDATA` logs the
heading as `angle`, so run29's group `id 66` is reproduced **to the bit**
from its leader's logged `angle` through the sim's own
`sin_component`/`cos_component` (§12's check 9). ~~The y-flip alone stays
unpinned, because every `off_y` in the window is zero.~~ **run31 pins it**
(`run31_s_curr_is_the_leader_s_heading_through_the_y_flip`): its group
stands in three ranks, `off_y` of −6, −3 and 0 on every one of the forty
frames its three moves are readable over, so the flipped column is
multiplied by something at last. The same matrix lands on the record; the
**unflipped** one misses it by hundreds, at any heading within a
twentieth of a degree of the leader's. This is what `do_group_move` adds
to the leader's position each frame (`docs/ORDERS.md` §8.3).

Two things run31 settles that nobody had asked. **Which heading**: not the
group's `o_angle`, not the bearing from the leader to its own slot and not
the order's angle, but the leader's `UnitData::angle` — and the leader is
the object `GroupOrder::oxx` names, which is also the anchor of the slot
table. And **`curr` is a mid-frame quantity**: `do_group_move` computes it
inside the frame and the unit turns afterwards, so the `angle` the
end-frame dump prints is a hair *past* the heading the rotation used. On
the nine frames of forty where the leader was not turning the dumped angle
reproduces all seventy-two numbers exactly; on the rest a heading within
0.05° of it does, and nothing else in the record moves. That is worth
carrying into any later `curr` diff: the dumped heading is an
approximation of the one that was used, and only sometimes the same.

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
`GroupData.form = −1`. That write is on the **group**, unconditional, and
the *first* statement inside the `buildings == 0` arm, before any member is
examined (`0070d0c0:29`). No member's `+0xaa` is touched by a halt, which
matters because `get_form` (§4.4) reads those bytes: the group forgets its
formation index and the members do not.

**Observed 2026-08-26**, and it took the `UNITS=3` half of a dump to see
it: army 0's group in run29 carries `form −1` with `form_num 0` at every
frame of the window while **all seven of its members carry `form 0`**, so
`get_form` gives back the 0 the group's own field has lost. The navy's
group, which has moved and not halted, carries `form 0` and so do its
four. `run29_s_halted_group_kept_its_members_formation_bytes` is the
check.

Then, per active on-map member that is not a **plane** and is not
`is_entering_or_exiting`. `UnitData::is_plane@0046ce40` is
`domain == 2 && !(unit_flags & 0x20)` — there is **no altitude test**, so a
halt skips every plane whether it is flying or parked, and bit `f` is
`unitrules.xml`'s "flies like a helicopter", so a **helicopter is not a
plane and is halted normally**:

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
to `n − 1`. **`cur` is `GroupData::get_stance_option(this, NULL)`** — the
**modal** option over the members (§4.4), not the leader's own stance. Then
a `COUNT_PEASANTS` count is taken and, if non-zero, a `COUNT_MILITARY`
count — both discarded, a leftover.

Per member, twice over:

- if the object is **valid** and `is_build` (vslot `+0x20` — 1 for a Build,
  **0 for a Wall**), its **`Build +0x7e`**, reached through `+0xac`, takes
  `s` — but only when the object's own stance type matches the group's, and
  a mismatch `goto`s past the unit branch too, ending that member's turn.
  So a **wall never receives a stance**;
- if the object is a **unit** (vslot `+0x18`) whose stance type matches and
  which is **not a plane** (vslot `+0xc0` is `is_plane`, devirtualised by
  name in this same family at `action_halt@0070d0c0:41` and
  `action_move_near@00704990:330`), then `unit +0xb1 = s`,
  `unit +0x8 |= 0x10`, and for a **combat** stance:

| `s` | what happens |
| --- | --- |
| 0, 3, 4 | `Unit::clear_mandatory` — the unit's current order stops being the player's |
| 1, 2, 5 | for a **human** leader only, and only when the current order lacks the action bit and there is no action order carrying it: `Unit::repath` then `kill_current_order(0)` |
| else | `Unit::clear_orders` |

That last row is **reachable**, and the first reading called it dead: there
is no clamp on a non-negative `s`, and
`CommandPackage::process_stance@00949ed0` forwards `StanceCommand::stance`
from the wire with **no validation** (three `ScenarioFuncSet` entries do the
same). A stance index of 6 or more on a combat group clears every matching
member's orders.

With `docs/COMBAT.md`'s stance order (0 aggressive, 1 defensive, 2 stand
ground, 3 raid, 4 raze, 5 hold fire), `docs/ARMY.md` §6's three calls read:
a **mustering or defending** army goes **defensive**, a **marching** one
**aggressive**, a **navy** and the pre-age-4 no-siege army **raid**. Every
one of those is an AI leader, so the middle row is dead for an army and
`set_stance` never kills an order — it only clears `mandatory` on 0 and 3.
Note that `kill_current_order` is also the third writer of
`GroupData::facing` (§1), so a *human*'s stance change can rewrite the
group's mirror flag.

## 9. `Group::action_siege_attack_to(x, y, ·, ·, angle)@0070d830`

A building group returns; then the scenario purge; then `num < 1` returns.
A **sub-group** is built on the stack, carrying the parent's `id` and
`army` — which is what keeps its members alive through `normalize`'s
back-pointer cull (§4.3) — holding every member that is
`is_valid_unit() && **is_on_map()** && ptype->is_siege()`. For an AI leader
(`!(leader_flags & 4)`) that sub-group is widened and then narrowed:

- empty → the **first supply wagon** of the parent joins it;
- still empty → the **first hero**;
- then, over the sub-group's members, each is scored by the **sum over
  every member of the parent group that is `is_valid_unit() && is_on_map()`**
  of `(|dx| + |dy|) >> 10` — its total Manhattan distance to the group in
  1024-unit steps — and the **smallest** wins (strict `<`, so a tie goes to
  the first). That unit is the **anchor**. The scoring loop runs *after* a
  `find_leader` pick and **overwrites** it, so the `find_leader` copy is
  dead for an AI.

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
  and a mismatch falls back to the same whole-group attack-move. The two
  sides are not read the same way: the **source** resolves through
  `flags & 0x100` to `region` or `region2`, going `>> 6` then `>> 2`, while
  the **destination** is always read at `+0x4` and reaches its cell through
  `div_3_table[v >> 8]` directly;
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
  `action_halt` does (`4` siege, `2` special, `1` spy). A **plane** has its
  `STRAFE` order re-pointed if it has mana and then `goto`s the next member
  **unconditionally** — it never receives an attack or a move here. A packer
  packs or unpacks first. A unit whose **`get_action`** is a `ATTACK` on
  this target (`ox`, `whom`), **mandatory**, with fewer than three orders
  queued and out of range, is skipped — but **only** on one of two
  sub-arms: `orderlist.count == 1`, **or** its current *order* `is_move()`
  (vslot `+0x14`) and the target is in range **of that move order's
  destination**. Otherwise it falls through and is re-ordered. And the main
  path is
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
reachability. ~~A `UNITS=3` dump of any early frame carries its output.~~
Better than that: **`GROUPDATA` carries its output directly**, per member,
every frame the category is on, and run29's window already has it (§6.4).
No `UNITS=3` order list is needed for the slot table.

run29's own window is the second capture this document leans on. Its three
frames (15100–15102) hold **5,392 `GROUPDATA` records, 44 of them live** —
512 pool slots a frame, 8 leaders × 64, plus the `HOTKEYGROUPS` block's
nested records. `rondata::diff` reads the whole record (§12).

**run31 is the third, and it is the one a human made.** Three right-clicks
on a twelve-squad selection, on the human's own island, under
`[End Frame] UNITS=9 GROUPS=9` — no `DUMP_ALL`, because
`GameLog::full_dump` gates `dump_groups` on the ini's own `GROUPS` key and
the pool is therefore an ordinary **per-frame record** — 684 KB and
about 0.6 sim-frames a second on run31, against `DUMP_ALL`'s 80 MB a frame
(`docs/ORACLE.md`, "The group pool is a per-frame record"). What it holds
that nothing before it did:

- a group of **36** members rather than 12 — a player's selection group
  keeps every figure, so `compute_dests`' **follower arm** is executed for
  the first time (§12.1);
- **two categories**, `FORM_CAT_FOOT` and `FORM_CAT_FOOT_RANGED`, selected
  ranged-first so the lowest category is not `list[0]` (§4.4);
- **`off_y` that is not zero** — three ranks — which is what pins
  `update_positions`' y-flip (§6.6);
- a group that both stands in a formation and holds orders to one, which
  is §6.4's `to`/`off` asymmetry;
- and **`GroupMoveOrder`**, the order §6.6 step 6 adds and no dump had
  ever shown (§12.1).

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
- **`Form::compute`'s slot table** (§6.4) whole, in `crates/sim/src/form.rs`
  — `type_cat`, `categorize`, `compute_rows_and_columns`, `compute_dests`,
  `get_form_mod_option` and `update_positions` — so each member now takes
  **its own slot destination** and the group carries `form_num`, `off`,
  `curr` and `angles`. Its four remaining limits are §6.4's last part, and
  all four are the original's;
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
| ~~`Form::compute`'s slot table (§6.4)~~ | — | **Closed 2026-08-26**: `crates/sim/src/form.rs`, diffed against run29's `GROUPDATA` from the install's own columns. What is left is the four limits at the end of §6.4 — Square, the wedge's uninitialised seed, Mob's rings, and `categorize`'s two type substitutions — and every one of them is the original's |
| the group pool (§3) | 64 slots a leader, `get_open_slot`'s recycling | one group per army, never recycled; `push_group`'s `force == 0` rule and `equals_group` are modelled, the slot allocation is not |
| `GroupMoveOrder`, and the order's angle | §6.6's per-frame formation, and step 6's `angle + (angles[i] << 24)` | every member gets a plain `MoveOrder` whose angle is `add_move_order`'s own bearing to the slot rather than the formation's — `docs/ORDERS.md` §8.4's verdict, unchanged. The angle **byte** is now computed and carried on the group (§6.4), so what is left is the order adder, not the table. **The record now exists** (§12.1), so this seam has a diff waiting for it |
| ~~the follower arm of `compute_dests` (§6.4)~~ | — | **Closed 2026-08-26**: `Sim::form_follower_slot`, read from `0072d3a0`–`0072d4f0` and checked by `a_follower_hangs_off_the_last_captain_alternating_sides`. The simulation still has no group that *contains* a follower — `Group::add`'s `keep_captain` (§4.1) — so the arm is implemented and unreached from the sim's own side; run31's record is what it was written against |
| `action_guard` (§9) | the escort half of a siege attack | with siege *and* a matching area the non-siege members keep their orders instead of guarding; no traced army has siege |
| `find_nearby_spot`'s collision (§6.6 step 4) | re-slotting an invalid slot | the sim has no unit collision, so no slot is ever invalid |
| `invalid_loc` on a slot, the `tregion` re-slot | §6.6 step 4 | same |
| `QUEUE_FIRST`'s insert dance (§6.2, §10) | `set_up_insert` / `action_halt` / recurse / `finish_insert` | `charge`'s `QUEUE_FIRST` is a plain push-to-front on each member |
| the scenario filter (§5) | `ignore_orders` | never set outside a scenario |
| `unit_masks` `0x100` / `0x400` / `0x4000000` | three bits `action_halt` and §6.6 clear | unmodelled bits; no reader in the sim |
| `is_entering_or_exiting` (§7), `OBJECT_NEW_THINK` (§8) | a halt skips a unit in a doorway; a stance write flags the unit for a re-think | two writes the third pass recorded and the sim does not model; no reader in the sim for either. `unit +0xab = width` **has left this row**: it is written now, and `get_form_mod_option` reads it (§6.4) |

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

   Then the second reading's seven, each written to fail first and each of
   which did (`docs/audit/2026-08-25-groups.md`, items 26–34):
   `halt_clears_the_group_s_form_and_leaves_every_member_s_own` (§7),
   `stance_cycles_from_the_modal_option_not_the_leader_s` (§8),
   `stance_skips_a_plane_but_not_a_helicopter` and
   `a_helicopter_takes_a_group_move_where_a_plane_does_not` (§7, §8, §6.6),
   `attack_keeps_a_lone_shot_and_a_march_into_range_and_re_orders_the_rest`
   (§10's two sub-arms),
   `the_siege_anchor_scores_only_members_that_are_on_the_map` (§9), and
   §6.5's clear/order asymmetry — which the third pass rewrote as
   `a_hurrying_army_with_no_city_clears_and_marches_its_shooting_siege`
   after finding the second reading's consequence inverted, and which,
   with `the_stance_type_is_decided_by_the_military_role_first` (§4.4's
   type predicate, seven cases), makes the two tests the third pass ran red
   first (`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts").
8. In `army.rs`: `do_forming_walks_the_army_to_the_projected_muster_origin`
   (the origin is §8's projection, and the destination the 48-snap),
   `a_forming_army_actually_moves_its_units_over_the_ticks` (the end-to-end
   one: the order the group issued is stepped by `Unit::work` and the unit
   is nearer the muster spot 64 frames on),
   `engagement_points_the_whole_army_at_the_first_fighter_s_target`,
   `engagement_ignores_a_building_target`, `close_halts_the_units_it_held`,
   `send_here_moves_the_point_the_muster_cell_and_the_units`, and
   `charge_drags_the_army_onto_the_attacker`.

9. **In `rondata::diff`, the `GROUPDATA` record itself** — the widening the
   audit puts first, and the project's own rule applied: when the original
   dumps a record, diff the whole record. `crate::gamelog::groups` parses
   all twenty scalars and the six parallel per-member arrays; four tests
   read them over run29's three windowed frames, and three of the four were
   made to fail on purpose before landing.

   - `run29_s_group_pool_is_five_hundred_and_twelve_whole_groupdata_records`
     — the pool is 512 records a frame in `id` order, `buildings`,
     `disband` and `priority` all 0, the six arrays exactly `num` long, a
     live slot's `who` equal to `id / 64`, and `last_group[8]` inside each
     leader's allocatable 46 with player 1's at slot **70**. It also asserts
     the writer's own **field order** and that **`march` never appears**.
   - `run29_s_engagement_bumps_order_num_by_one_and_writes_no_form` — the
     record's `order_num 0 → 1 → 1` and `form −1` across the
     `Army::engagement` frame, and the harness reproducing both deltas.
   - `run29_s_navy_group_is_a_line_of_four_rotated_at_forty_eight_units_a_step`
     — §6.4's fixture: `off_x = [0, −14, 13, −28]` with `off_y` and
     `angles` zero, `form 0`, `form_num == num`, and the `curr` pair that
     pins **× 48** and the shared rotation.
   - `run29_s_navy_group_s_curr_is_the_leader_s_heading_applied_to_the_slot_table`
     — the third pass's widening: every `curr` in the window reproduced
     **exactly** from the leader's logged `UNITDATA` `angle` through
     `sin_component`/`cos_component` (§6.6), with a quarter-turn control;
     `stamp` and `think_frame` never past the frame, `speed == new_speed`,
     `role == 0` on an empty slot, the navy's `role ⊇ {SEA, SEA_MILITARY,
     MILITARY}` and army 0's whole word `LAND | MILITARY | RANGED | MOUNTED
     | MELEE` — the OR over every member the group has ever had, because
     `Group::kill` clears no bit.
   - `run29_s_priority_bit_says_allocated_rather_than_hotkey` — see §1.
     This one **failed on its first run and was right to**: the audit
     predicted `priority 1` on every `HOTKEYGROUPDATA`, and hotkey slot 28
     carries 0 with a `stamp` of 13125. It held a group and lost its last
     member; `Group::kill`'s `num == 0 → clear(−1)` runs `Group::clear`,
     which writes 0 over the bit, and only
     `HotKeyGroups::find_group@00714d00` puts it back. The bit means
     **"this slot is a live control group"**, not "this slot is in the
     hotkey array".
10. In `rondata::diff`:
    `run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_point`
    — not this document's mechanic, but the one that makes it reachable:
    the two blocks either side of run28's frame, and the release that
    leaves no `FORMING` bit so `Army::engagement` can call §10
    (`docs/ARMY.md` §16.6, §17 item 6).

11. **The slot table** (§6.4), eight in `form.rs` and one in
    `rondata::diff`. The one that matters is the last:
    `run29_s_navy_slot_table_is_reproduced_from_the_install_s_own_spacing`
    goes `unitrules.xml` → `x_spacing 660` → `FORM_CAT_ARTILLERY` →
    `form_mod 50` → `cols 4` → the slot arithmetic → the floor divide, and
    compares the result with `GROUPDATA` `id 66`'s own `off_x`, `off_y`,
    `angles`, `form_num` and `curr` across all three windowed frames. It
    also asserts that **fifteen** of the install's types carry
    `x_spacing 660` and that every one of them lands in
    `FORM_CAT_ARTILLERY`, so a change to the loader or to `type_cat`
    breaks it rather than passing quietly.

    In `form.rs`, seven more, and **eleven deliberate breakages were run
    red before any of them landed** — a truncating quantiser, a missing
    even-column shift, a missing anchor slide, either half of the rank
    stack removed, a placing Square, a `form_mod` off by one, the `human`
    gate ignored, a Column two to a rank, and the left-right alternation
    dropped. Ten of the eleven turned a test red on the first try; the
    eleventh, **flipping the sign of `update_positions`' `cos θ · y`
    term, did not** — because every `off_y` in run29's window is zero, so
    no capture on disk could tell a rotation from a rotation-with-a-flip.
    ~~`run29_s_navy_curr_is_the_slot_table_under_the_leader_s_logged_heading`
    now says so in place and pins the flip from the listing instead.~~
    **run31 pins it from a record** — §12.1's fourth check, whose
    unflipped control is exactly that breakage and which fails on it.

~~What no test pins is the **positions** of an army's units in a traced
game … which is also the capture §6.4's `to`/`off` asymmetry needs.~~ The
asymmetry has its capture; what no test pins is still the *positions* of
an army's units in a traced game, because `scene_at` does not read a
block's `UNITDATA` order lists.

## 12.1 What run31 added

`gamelog-run31-humangroup.txt`, four checks in `rondata::diff`, each made
to fail on purpose before it landed. The fixture is `run31_moves()`: every
frame that carries a `GroupMoveOrder` **and** a live `who 0` group — forty
of them, over three right-clicks.

- `run31_s_human_group_move_is_thirty_six_figures_of_two_categories` —
  the shape. `num 36`, `form 0`, `form_num 36`, `army −1`; the membership
  is the selection walk's, twelve slinger figures then twenty-four hoplite
  ones; a captain heads each triple (`o_up < 0`, and `SubObjectData`'s
  flag `0x10` says the same); every member carries `form 0` and
  `form_mod 50` from §6.6 step 1; exactly one member sits on the origin
  and it is **not** `list[0]`; and `off_y` takes three values.
- `run31_s_group_order_names_a_leader_the_first_member_rule_would_miss` —
  §4.4. One `oxx` across every order in the frame, equal to the anchor's
  object, whose `type_cat` is `FORM_CAT_FOOT` where `list[0]`'s is
  `FORM_CAT_FOOT_RANGED`.
- `run31_s_anchor_marches_to_a_point_its_own_offset_says_is_the_click` —
  §6.4. One click across every order (and the group's own `ox`/`oy` is
  that click); the anchor's `GroupMoveOrder` destination is not it, by
  more than a `UCoord`, on every frame the anchor holds one.
- `run31_s_curr_is_the_leader_s_heading_through_the_y_flip` — §6.6, with
  the unflipped matrix as its control.

And one in `sim::form`:
`a_follower_hangs_off_the_last_captain_alternating_sides`.

**`GroupMoveOrder`, the record.** `GroupMoveOrder::log_data@00485910`
opens a block named — alone in the family — in **mixed case**, and writes
*two* bases plus one field of its own:

| block | fields |
| --- | --- |
| `MOVEORDER` | the whole §11.1 row: this member's **slot destination** in `x`/`y`, the formation angle, and the **click** in `orig_x`/`orig_y` |
| `GROUPORDER` | `oxx` — the **leader's object** — `whose`, `group_angle`, `id` (shared by every member's order) and `form_id`, this member's index into the group's own arrays |
| `GroupMoveOrder` | `in_group` |

`crate::gamelog` reads all of it. The mixed case was a real trap: the
order walk's `ends_with("ORDER")` dropped the block *and* slid every later
`type`/`metric` onto the wrong body, because the pairing is positional.
`a_group_move_order_carries_both_bases_and_names_its_leader` is the parser
test and it fails on the case-sensitive form.

**What is left to do with the capture**, in order: reproduce the whole
36-member table — which needs `Group::add`'s `keep_captain` on the sim
side, the leader question of §4.4, and then §6.4's displacement measured
rather than only observed; and diff `GroupMoveOrder`'s angle against
§6.6 step 6's `angle + (angles[i] << 24)`, which §12's third seam is
still standing in for.

## 13. What is not established

Still open:

- ~~**`Form::compute`'s slot table** (§6.4), the largest gap.~~
  **Closed 2026-08-26**, and the four things left in it are the
  original's, not ours: Square is dead code, a wedge's row count is seeded
  from uninitialised stack, Mob's rings are unrecovered, and
  `categorize`'s two type substitutions have no cargo list to read. Each
  is stated with its own falsifying capture at the end of §6.4. On the way
  the three sub-questions this entry named were all answered:
  `get_form_mod_option` is **50** for an army — and run29's `UNITDATA`
  prints `form_mod 50`, so it is a record — the type widths are
  `x_spacing`/`y_spacing`, the `X_SPACING`/`Y_SPACING` columns times 12 by
  the engine's own dump labels, and a warship lands in
  `FORM_CAT_ARTILLERY`.
- ~~**`FormData::type_cat`** (§4.4) … what is not established is *which*
  category each shipped type falls in.~~ **Read and implemented**
  (`sim::form::type_cat`, §6.4): the tree is eight reachable values off
  `attack`, five `obj_masks` bits, two `unit_flags2` bits, `max_range` and
  the three trader ids, and `rondata::diff` asserts the fifteen light
  warships all land in `FORM_CAT_ARTILLERY`. ~~What is **still** open is
  the consequence for `find_leader`: the simulation takes the group's
  first on-map captain rather than the lowest-category one.~~ **Fixed
  2026-08-26** — `Sim::group_find_leader` keys on `type_cat` (§4.4). The
  *capture* is still owed, and it is not the one this entry named: an
  army holding a wagon and a hoplite would settle the key only if the
  record could say which member led, and the member at slot `(0, 0)` is
  the anchor of the **lowest-indexed non-empty category** (§6.4), which
  is the same thing `find_leader` computes — so it cannot disagree.
  ~~*Capture:* a two-category group in a formation with non-zero `off`
  whose members' **headings differ**, so that `curr` names the heading
  `update_positions` used.~~ **Taken by run31, and by a better field**:
  `GroupOrder::oxx` names the leader outright (§4.4, §12.1). What the
  capture opened instead is **which** of two equal-category captains
  leads — object 9 on one frame and object 6 on the next, where a
  first-wins tie says 6 both times. That is §4.4's new open question and
  it is the most concrete one here.
- ~~**§6.4's `to`/`off` asymmetry still has no capture.**~~ **run31 is
  the capture** and the asymmetry is observed (§6.4, §12.1): the anchor's
  own order does not point at the click. **Still open:** its *size*. The
  displacement should be the rotated `anchor_x` and nothing in the record
  measures that without the whole 36-member table, which needs the
  follower arm on a sim-side group (§12) and the leader settled.
- **The 18 pool slots above `who*64 + 45`** (§1). Nothing allocates them
  and nothing searches them; `Groups::process` still cycles them. What they
  are for is open.
- **`pathfinder +0x70`** (§6.7) is set to 1 around an army group's
  `find_wpath` and cleared after; what reads it is inside `astar_path`,
  which `docs/PATHFINDER.md` did not reach.
- **`action_guard`** (§9) is 300 lines and unread; `GUARD` is not
  implemented in the simulation either (`docs/ORDERS.md` §14).
- ~~**The formal name of object vslot `+0x1c`.**~~ **`is_wallbuild`**, by
  the PDB's own method records (`SubObjectData` introduces `is_unit` at
  vftable offset 24, `is_wallbuild` at 28, `is_build` at 32, `is_seen` at
  72); the map lacks the `BuildData` symbol only because the slot is the
  COMDAT-folded `return 1` at `0041e0e0`. The third pass closed it, and
  named every other slot this document had inferred from uses the same
  way (`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts").
- **The blind list this document adds:** `Groups::get_open_slot`'s
  "UH OH, NEED MORE GROUPS" path, `Group::sort`, `refresh_group_order`,
  `distribute_attack`, `kill_group_move` — none of which any traced game
  has executed. `tools/trace/report.py … blind docs/` will list them.
  `distribute_attack` and `kill_group_move` were read blind (§14) and the
  reading is still the only evidence for both.
- **Why nine members of thirty-six hold a `GroupMoveOrder` and the rest a
  plain `MoveOrder`** on run31's first move — eighteen on its second and
  all thirty-six on its third. §6.6 step 6's gate does not obviously
  separate them: they are one type, one group, one formation, and the
  nine are `form_id 15..23`, a contiguous run starting at the leader's own
  slot. Either the walk re-enters, or one of the gate's five tests moves
  with the frame. *Capture:* already on disk — it is a reading of the
  same three moves against §6.6 step 6, member by member.
- **`compute_dests`' modern-infantry scatter** (`72d456`): three
  `guy_spacing`-sized jitters on a follower, keyed on the destination,
  the **object number** and the member index. `Sim::form_follower_slot`
  does not implement it, and the object number is not a quantity the
  simulation reproduces. No traced game has put modern infantry in a
  formation.

Closed by the second reading, struck here and answered where they belong:

- ~~**Which form index an AI group carries.**~~ run29 answers it directly:
  an army group in formation **0** (`id 66`, `army 2`, `form_num 4`),
  single-unit groups at **9** (`id 65`, `67`), and a never-moved army group
  at **−1** (`id 69`). §6.4.
- ~~**The four `TypeIndex` values 0x32..0x35** … an identification by
  index, not by a type record.~~ `PEASANTS`, `PEASANTSKOREAN`, `SCHOLARS`,
  `SCHOLARSKOREAN`, from the PDB's own `TypeIndex` enum. §6.6 step 1.
- ~~**`role & 0x10` in §6.6 step 6** … one of the two readings is wrong.~~
  `is(SCOUT)` on land, `is(BARK)` at sea, from `determine_roles`' two
  writers and the type record. `docs/ORDERS.md` §8.2's gloss is the wrong
  one. §6.6 step 6.
- ~~**`ObjectData` vslots `+0x1c`, `+0x20`, `+0x48`, `+0x108`, `+0x10c`**
  are read from their uses, not from a type record.~~ All five are in the
  PE's own vtable arrays and four are in `vtables.txt`; three of the
  guesses were **wrong**. `+0x1c` any building, walls included; `+0x20`
  **`is_build`**, 1 for a Build and **0 for a Wall**; `+0x48` **`is_seen`**;
  `+0x108` `get_stance_type`; and the **type** vtable's `+0x10c` `is_siege`,
  settled outright by `ObjectData::is_siege@0046ef90` rather than by the
  weaker `COUNT_SIEGE` argument. §4.3, §8, §10.
- ~~**`Group::priority`** … has no writer in the export.~~ Five writers,
  and it is a one-bit "I am a control group". §1.
- ~~**No second reading yet.**~~ Run, adjudicated and applied: §14.

## 14. Second reading — landed, and applied

Two blind readers on Opus 5 (A over §6–§8, B over §3, §4, §9, §10), and a
third adjudicator on Opus 5 against the decompile, the listing,
`rise_z.map` and the PE. `docs/audit/2026-08-25-groups.md` is the record —
124 verdict rows, five `FABLE:` markers, eight named assertions. **All 25
document corrections and all 9 Rust-changing verdicts are applied**
(2026-08-26); the sections above are the corrected text and this section is
the ledger of what was wrong.

The four largest errors, and where each answer now lives:

- **§6.4 and §13's account of the slot-table seam was void on both legs.**
  There is no float barrier — zero float instructions across the `Group`
  family, fourteen in `Form::compute_dests` alone, all integer-exact as
  `((2·rows − 1)·depth·k)/2`, the `0.5f` read from the PE at `0xb694c0`.
  And "no capture pins its output" was **false**:
  `GroupData::log_data@0045e1d0` dumps `off_x`, `off_y`, `curr_x`,
  `curr_y`, `angles`, `form`, `form_num`, `o_dist` and `o_angle` per
  member, and run29 already carried a four-member group in formation 0.
  §13 asked for a capture that was on disk. → **§6.4**, and the diff in
  §12's check 9.
- **§13's five guessed `ObjectData` vtable slots: three were wrong.**
  `+0x48` is `is_seen`; `+0x20` is `is_build`, and **0 for a Wall**;
  `+0x10c` is settled by `ObjectData::is_siege@0046ef90`. §8 and §10 had
  built rules on the guesses. → **§4.3, §8, §10, §13**.
- **§13's "`Group::priority` has no writer in the export" was wrong** —
  there are five. → **§1**.
- **Two live bugs in `crates/sim/src/group.rs`**: `group_action_halt` wrote
  each *unit's* `form` where `0070d0c0:29` writes the *group's* — and
  `group_get_form` reads the unit bytes, so it was not cosmetic — and
  `group_action_attack`'s "already attacking" skip was unconditional where
  the original's has two sub-arms. → both fixed, with tests written to fail
  first (§12).

**What survived:** §9's Manhattan anchor, which reader B missed entirely
and which the adjudicator confirms per-term `>> 10` and strict `<`; the
sim's `siege_anchor` was right. The full list of what is doubly confirmed
is in the audit.

**Four of the audit's five `FABLE:` markers were settled while applying it**
(2026-08-26, on Opus 5 — recorded in the audit's own "Markers settled"
section, and still owed a Fable ratification per `docs/DECISIONS.md`
entry 22):

1. `unit_flags` bit `f` is `unitrules.xml`'s **"flies like a helicopter"**;
   exactly three types carry it and all three are `<DOMAIN>Air`. Not
   vacuous: a helicopter is not a plane and is ordered normally. → §7.
2. `game->semaphore.ptr[1] & 8` is **bit 11 = "the scenario editor is
   open"**, set and reset by `ConsoleWin::run_cmd` around
   `ScenarioEditor::init`/`close`. Not a network flag. → §6.3.
3. A.23's asymmetric `facing` restore is **confirmed by the listing**, and
   its consequence is narrower than A thought: the toggle and the restore
   read the same unrewritten `%esi`, so `facing` is *invariant* across
   `compute_form` unless the editor bit is set. → §6.3.
4. `role & 0x10` is `is(SCOUT)` on land and `is(BARK)` at sea, by the type
   record. → §6.6 step 6.

**And one finding that came out of settling marker 3, which no reading
had:** run29 shows live groups with `facing = 1`, which `compute_form`
alone cannot produce in a game with the editor closed. The third writer is
**`Unit::kill_current_order@005e2cb0`** — outside the `Group` family
entirely, which is why the brief never reached it. → §1, §8.

**The third pass — the Fable ratification, run in the main thread on
2026-08-26** (`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts").
Eight of the nine Rust-changing verdicts and all four Opus-settled markers
hold from their citations; the fifth marker is closed (`is_wallbuild`,
above). **One verdict was overturned in its consequence**: §6.5's
clear/order asymmetry is real, but a hurrying army with no city *marches*
its cleared siege unit rather than stranding it, because the order loop
re-reads an `order_type` the clear loop has already emptied. Outside the
floor it found the predicate behind every stance decision
(`UnitTypeData::get_stance_type`, §4.4 — the sim's test order was wrong
on three cases), the rounding of the slot table (a floor, §6.4), the
exact form of `update_positions` and a bit-for-bit reproduction of run29's
`curr` from the leader's logged heading (§6.6, §12), a fourth writer of
`facing` (`Unit::set_angle`, §1), and the loaders' two overwrites
(`Groups::clear` zeroes `stamp` and `priority` after `Group::clear`;
`Group::clear` zeroes `who`). Two Rust changes, both run red first; one
new widening.

## 15. The slot table's implementation, and what it corrected

Written 2026-08-26 on Opus 5, from the decompile and the listing, against
the two run29 fixtures the third pass had already landed. It is the last
of this mechanic's seams that cost work rather than a grep, and closing it
turned up **five things no pass had**, three of which correct text the
audit had accepted.

**Corrections to what was written:**

1. **The block's anchor is the first member of the *lowest-indexed*
   non-empty category, not the last.** §6.4 said "last" and reader A's
   A.28 wrote the loop as `prev = c`; the machine code writes `prev`
   **only on the `prev < 0` arm** and the other arm reloads a slot it
   never wrote (`72cfe2`/`72d837`, and the same shape at
   `72d81a`/`72d878`). Ghidra had it right and the reading did not. The
   difference is visible: it decides which member ends at `(0, 0)` in
   every mixed-category group. → §6.4.
2. **`compute_dests`' last loop drops the anchor's x from the
   destinations.** `off` is slid by the whole anchor; `to` is slid by
   `cosx(angle, 0) + sinx(angle, −anchor_y)`, with `%edx` explicitly
   zeroed at `72d737`. So a group with an off-centre anchor — which an
   **even** column count always produces — marches to points displaced
   from where its own offsets put it. No pass had looked at that loop's
   arguments, because the decompiler drops them. → §6.4, and a capture in
   §13.
3. **Formation 6, Square, is dead code in the shipped executable.**
   A.28 said "I did not derive Square" and stopped there. It cannot be
   derived: `compute_rows_and_columns`' `== 6` arm writes `space[18][4]`,
   `across` and `per[7]`, and **no function in the export reads any of the
   three**; `compute_dests` has no Square branch, and the same arm sets
   `wedge = −1` so the wedge branch cannot cover for it. A Square group's
   slots stay at the `memset` zero. → §6.4.

**And two the readings could not have had, because they needed the
implementation or the record:**

4. **A wedge's own row count is read before it is written** — `int
   rows[18]` in `Form::compute` is never initialised and `72dc90` reads
   `rows[wedge]` into the accumulator. A pure wedge is deterministic; a
   wedge with a second category is not reproducible by anyone. → §6.4.
5. **`get_form_mod_option` is 50, and the record says so.** The reading
   gives 50 as the fallback and shows the mean can never move off it once
   §6.6 step 1 has written 50 to every member. run29's `UNITDATA` prints
   `form_mod 50` on each member of the navy — the open question in §13
   asked for a reading and the dump had the answer. → §6.4.

The type field names the table turns on came from the engine's own
`log_data` strings rather than from use: `attack`, `guy_spacing`,
`x_spacing`, `y_spacing`, `uber_size`. `FormCatIndex`'s eighteen values
are the PDB's `LF_ENUM` record, checked here rather than taken from A.16.
