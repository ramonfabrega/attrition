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
  +0x3c  new_speed   int     \ the group's march speed, reset by Groups::process
  +0x40  speed       int     / to the leader's own once every 64 frames — one
                             slot per player a frame, not every slot (§19)
  +0x44  form_num    int     the member count Form::compute_dests laid out
  +0x48  facing      uchar   the formation's mirror flag (§6.3), and a running
                             one rather than a setting. Four writers, and three
                             of them can fire in one frame:
                             Group::clear (0); compute_form's toggle/restore
                             pair, which nets to nothing (§6.3);
                             Unit::set_angle@00605400, which *toggles* it
                             whenever the group's leader is turned by 90° or
                             more — the ordinary source of a live group's
                             `facing 1`; and Unit::kill_current_order
                             @005e2cb0, which on a dying move *assigns*
                             `order.facing XOR reversing(leader.angle −
                             order.angle)`, again only from the leader. The
                             last one runs **before** the next layout
                             (`70524f` precedes `7053ec`), which is why the
                             flag a dump prints is never the mirror a layout
                             used — §6.3's table
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

- **It opens with `get_num`** (vslot `+0x4`), or `get_num_const` when
  `const` is set — and `Group::get_num@00714700` is `normalize` whole
  for a group of fewer than four with an `id`. So on a seated group every
  step of the walk below can prune the step before it (§23, item 557).
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

**Implemented 2026-08-26** — `Sim::group_add_keeping`, with `Unit::o_up`
and `Unit::o_down` for the two shorts. The consequence is the whole reason
run31's group is 36 and not 12: adding a captain drags its figures in
behind it, in `o_down` order, and adding a figure adds its captain instead.
`Group::sort@00708090` is the fix-up that keeps that ordering true — it
walks the list and, on a follower whose captain is not the last one seen,
`kill`s it and re-`add`s it with `keep_captain = 0`, which (a follower
never being added in its own right) simply drops it. It is `categorize`'s
own **first statement**, so every formation the engine has ever laid out
ran it; it is not implemented, because the simulation only builds lists
that are already in order.

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

~~**What it does *not* settle**, and it is now the most concrete open
question this document has: run31's leader is object **9** on frame 204
and object **6** on frames 328 onward — both hoplite captains, both
`FORM_CAT_FOOT`, and object 6 comes first in `list`. A tie going to the
first would name 6 every time. So either the walk is not the `list` order
this document assumes, or one of the two failed a test on that frame.~~
**Answered 2026-08-26 by the 36-member table** (§6.8, §12.2), and neither
horn was right: **`find_leader` names object 6 both times.** `oxx` is not
`find_leader`'s output but the *current origin of the block*, and
`Group::refresh_group_order` moves it whenever the unit it names can no
longer serve. Frame 204's record is `compute_dests`' table re-origined onto
member 15 — object 9 — and frames 328 onward are the table itself, with
object 6 on the origin exactly as this section's rule says. Frame 205, one
frame later, has it re-origined again onto member 24. The layout never
changed; only which member the block was hung from.

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
`(ox, oy)` within `0x180`, else `leader − order + (ox, oy)` when the move
**order's destination** is within `0x180` of `(ox, oy)` — "where the group
*will* be". Both gates' operands: §12.5).

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
- if the leader's facing, **minus its slot's packed angle byte**, minus the
  given angle is `≥ 90°` by the same idiom, `group.facing` is **toggled**
  around the `Form::compute` call and toggled back afterwards — and the two
  toggles are **exactly symmetric**.
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
  and the group are **negated** — `to` is not, so a flipped formation
  marches to its own slots and stands on the *opposite* side of its
  leader. Implemented 2026-09-07, item 267 (§12).

`Form::compute@0072e8e0` is `compute_rows_and_columns` then `compute_dests`,
and finally `group.o_angle = find_angle(…)`, `group.o_dist =
vector_dist(…)`.

**The mirror the layout uses is not the `facing` the record keeps.** The
toggle is symmetric, so `GROUPDATA` prints the *pre-call* value while
`Form::compute` sees `facing XOR (leader ≥ 90° off the bearing)`: a dumped
`facing 1` says nothing about whether the block is mirrored. run31's three
moves show both halves — two carry `facing 1`, one of them unmirrored and
the other mirrored; the third carries `facing 0`, unmirrored — and each is
reproduced exactly, all 36 slots, by one of the two mirrors
(`run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns`).

#### The predicate, and the two writers that made it look wrong

**Settled 2026-08-26, out of run31's own dump** — no new run: the earlier
pass had asked the record the wrong frame and had missed two of `facing`'s
four writers (§4.1). The predicate, from the listing at
`707e7e`–`707ec1`:

```
S, L     = find_leader(&S)            ; S the member slot, L the object
q        = units[who][L].angle − ((signed char)group.angles[S] << 24)
toggle   = reversing(q − angle)       ; 0092cf20, both bounds inclusive
mirror   = group.facing XOR toggle
```

`local_20` is `find_leader`'s **out-parameter** and `local_14` its return —
the slot indexes `angles`, the object indexes the player's unit array. The
same quantity is what the zero-delta branch assigns to the angle outright
(`707e3d`–`707e6d`), which is how "the leader's own facing minus its packed
slot byte" is read there.

That leaves `group.facing` at the moment of the call, and it is **not** the
value the last frame's `GROUPDATA` printed. Two writers move it:

- **`Unit::set_angle@00605400`** toggles it whenever the group's **leader**
  is set to a heading 90° or more from the one it has — the same
  `reversing` window. Every marching leader does this the moment it turns
  onto a new bearing, so a live group's `facing 1` is ordinarily just "the
  leader has turned round since the last layout". A follower's turn does
  nothing: it calls `find_leader` and compares.
- **`Unit::kill_current_order@005e2cb0`**, on a dying order of the move
  family (`{1, 2, 3, 4, 0x12, 0x13, 0x15}`), reads the order's own
  `MoveOrder +0x28 facing` — the mirror **that** order was laid out with —
  and, when the unit is the group's leader, **assigns** it back:
  `group.facing = order.facing XOR reversing(leader.angle − order.angle)`
  (`5e3018`–`5e30eb`). It is an assignment, not a toggle, so everything the
  march did to the flag is discarded.

The ordering is load-bearing: `action_move_near`'s `QUEUE_NEW` clear loop
at `70524f` runs **before** `compute_form`, its `Unit::clear_orders` at
`70538d` ahead of the call at `7053ec`. So a group's second right-click
hands the flag back *first* and lays out *second*, and `Form::compute`
never sees the march's flag at all — it sees the last
layout's own answer, XOR'd by the toggle.

run31's three clicks are the proof, every number already in the file
(`run31_s_three_mirrors_come_out_of_facing_s_three_writers`):

| click | frame | leader's heading at *f−1* | the move's angle | `facing` at the call | mirror | `facing` the frame prints |
|---|---|---|---|---|---|---|
| 1 | 204 | 120.000° (a unit that has never moved) | 77.756° | 0, from `Group::clear` | **0** | 1 — the leader turned −118.4° |
| 2 | 328 | 88.347° | 157.247° | 0, handed back by order 1 | **0** | 0 — the leader turned only +76.0° |
| 3 | 356 | 163.542° | −15.595° | 0, handed back by order 2 | **1** | 1 — the leader turned +154.7° |

The mirrors run 0, 0, 1 and the dumped flags 1, 0, 1, and the middle click
is where the two models part: `facing 1` at frame 327 would mirror a block
that the record lays out square. The earlier pass took frame **204's own**
heading (1.577°, 76° off) rather than frame 203's, and 204 is the one frame
where the reading is hopeless — the group is one frame old and its leader
has already snapped 118° into the march.

`MoveOrder::facing` is the last piece and the record carries it: run31's
three orders print `facing` 0, 0 and 1, which is exactly the mirror each
layout used, on all 945 order blocks.

**Both writers were already written down** — §4.1's field table has had
them since the third pass, and `docs/ORDERS.md` §3.2 describes the
hand-back in full. What no document had was the *ordering*, and without it
the hand-back reads as a carry-over into some later frame rather than as
the input to the very next layout. The finding cost two greps of the
listing; the day before it, this was booked as a run.

`o_angle` is **written twice**, and the second write is the one that lands:
`Form::compute`'s tail puts the leader-slot bearing in it, and then
`action_move_near@00704990:464` overwrites it with the **formation angle**
for a `QUEUE_NEW`/`QUEUE_LAST` move, along with `(ox, oy) = the click`. It
does not touch `o_dist`. run31 shows both: every one of its 36 members'
`GroupMoveOrder` carries an `angle` equal to the group's `o_angle` to the
bit, on all three moves — and its `o_dist` is 215 or 217 where the
leader-slot offset is 216 (§6.4).

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

~~What the capture does **not** yet do is reproduce the displacement's
size.~~ **The size is measured, 2026-08-26**, and the record hands it over
in a field nobody had read: **`o_dist` is the displacement.**
`Form::compute`'s tail measures from the order's point to the *leader's*
slot, and the leader is the anchor — so `o_dist = |anchor_x|`, and
`action_move_near` overwrites `o_angle` afterwards but leaves this
(§6.3). For run31's group the anchor is a column count's half-step off
centre, `x_spacing/2 = 432/2 = **216**`, and the record prints **215** on
one move and **217** on the other two — `vector_dist`'s octagonal
approximation of the same 216 at three different bearings. The whole
36-member table is now reproduced from the install's own columns
(§12.2), so the displacement is derived rather than fitted.

**The anchor rule is confirmed on the listing**, not only the decompile:
`72d17c`–`72d198` is `cmpl %eax, -0x1c(%ebp)` (the rank-stack's `prev`
against this member's category), `cmpl $0x0, -0x20(%ebp)` (its `cat_id`),
then `movl %esi, -0x40(%ebp)` / `movl %ebx, -0x38(%ebp)`. So the anchor is
`cat_id 0` of the lowest non-empty category, full stop — which is the same
member `find_leader` names. A record whose block sits somewhere else has
been **re-origined** since, and that is §6.8.

**Where the slot destination ends up in the order.** §6.6 step 6 passes it
as a `UCoord` (`/0x30`), and the round trip is lossy in a way that is
exactly `Unit::add_move_order`'s own snap: the `MOVEORDER`'s `x`/`y` is
`floor(to / 48) × 48 + 24`. run31 reproduces all 36 members' destinations
that way on every frame the un-refreshed table is readable — and note it is
`x`/`y` that carries it, **not** `dest_x`/`dest_y`, which is the path
stack's current waypoint (`docs/ORDERS.md` §11.1) and lags a whole click
behind on most members.

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

   Both write the angle to `MoveOrder +0xc` and the reverse flag to
   `MoveOrder +0x28 facing`; `add_group_move_order` writes the angle a
   second time, to the `GroupOrder`'s own `group_angle`, which is why the
   two are equal on every record run31 holds.
   **`Unit::add_move_order@00616ed0` is the same call** with
   `find_angle(dx, dy)` for the angle and **−1** for the facing — and that
   −1 is the gate that keeps §6.3's hand-back off an order that never
   belonged to a formation. Implemented 2026-08-26:
   `Sim::add_move_facing_order`, which the group path passes both to.

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

**And "heading" is the right word, confirmed 2026-08-27** (`docs/MOVEMENT.md`,
"Two angles", queue item 34). `UnitData::angle` is not the direction the unit
faces — that is `GuyData::angle`, which turns at the turn rate — but the
bearing to where it is going, which `move_step` writes outright at the top of
every step. So the mid-frame gap this section had to allow for has a name and
a size: it is the frame's turn, and it is at most the turn rate. `crates/sim`
reads `Movement::heading` at all three of the `update_positions` sites and at
both `GroupData::facing` writers.

### 6.7 The path

**Implemented 2026-08-26** — `crates/sim/src/grouppath.rs`, and §16 has
what building it corrected in the text below.

One path is planned and offset, exactly as `docs/ORDERS.md` §8.2 outlines:
a single-entry `grouppath` at the **leader's slot destination** with the
call's tolerance and `FINAL`; within `0x900` nothing more is planned;
otherwise `PathFinder::find_wpath` from the group's location, with
`pathfinder +0x70 = 1` set around the call for an army's group.

Six things about that paragraph are sharper than it reads, and all six
were settled from the listing while it was being built:

- **The goal is the leader's `form.to[idx]` raw.** `7060df`–`706137` loads
  `0x514(%edx,%eax,4)` / `0x714(%edx,%eax,4)` at `form +0x34` and stores
  them into the `PathData` with no snap. §6.6 step 3 clamps the *order's*
  destination and `add_move_facing_order` snaps it to `u × 0x30 + 0x18`;
  the path entry is neither clamped nor snapped, so a member's stack
  bottom and its order's `dest` are **different numbers** — run20's `1/0`
  by `0x18` on both axes.
- **`grouppath` and `cols` are function-local statics of
  `action_move_near`**, not fields of anything (`0xee1538` and `0xee155c`,
  with `_Init_thread_header` guards at `705295`). The stack is drained to
  zero by every arm before the function returns, which is what makes
  "`grouppath.length == 0` after the search" mean *the leader was
  unusable*, and not "left over from last time".
- **The `0x900` is measured from the search's start to the leader's
  slot**, not from the group to the order's point:
  `vector_dist(|start_x − slot_x|, |start_y − slot_y|)` at
  `70614e`–`706172`, where `(start_x, start_y)` is the pair pushed to
  `find_wpath` four instructions later.
- **The start is the leader's top-of-stack when it has one**, and
  `get_loc`'s answer otherwise (`706921`–`70692e`). After a `QUEUE_NEW`
  clear it never has one; after a `QUEUE_LAST` it does, which is what
  makes step 2's pre-invert (§6.6) load-bearing.
- **`pathfinder +0x70` is the `army` mode**, not a private hint:
  `PathFinder::find_wpath@00688fc0` sets that same word at `:242` for an
  AI's armed non-worker off water, and clears it on the way out at `:258`.
  Setting it from outside is therefore the only way a **human**'s army
  ever gets AI army costing, since a human jumps the whole mode block
  (`leaders & 4` at `0068973d`). The gate is `GroupData::army >= 0`
  (`70617e: cmpl $0x0, 0x8(%eax)`), so a group on the stack — a scout's,
  a `find_target` probe's — never sets it. The engine names this pair:
  `PathFinder::find_wpath_army@00683730` is four instructions,
  `+0x70 = 1; find_wpath(&grouppath, …); +0x70 = 0`, and it hard-codes
  `grouppath` rather than using its own stack argument. It has **zero
  callers** because it is inlined here, which is what
  `docs/PATHFINDER.md` §2's `army` row already said.
- **The tail invert asks none of the move's questions.** `706909`–`7069a8`
  tests active, on the map and not a plane, and nothing else: a member the
  move stabled in a city or left shooting has its own untouched stack
  turned over with everyone else's.

If the search produced nothing, **each member plans its own** single-entry
path and `find_wpath`s it — without the `+0x70` hint — and the stack is
inverted. A member whose own search also fails gets the goal pushed by
itself, off the copy the caller kept.

Otherwise each waypoint is popped from the top and, for each member,
translated by `slot[i] − slot[leader]` and clamped (`706520`–`7065be`; the
bounds test is `tiles × 0xc0` and the clamp `cells × 0x300`, which are the
same number). A terrain guard follows: the **area id** of the member's
waypoint cell (`world +0x134`, one 0x1c-byte record per cell, `+0x4` land
and `+0x6` water, the latter chosen when the cell is `HALFLAND 0x100` and
the tile's terrain is water — `WorldData::get_tregion`'s coastal
refinement, and a **16-bit** compare at `7066ce`) is compared with the
leader's; if they differ **and** the member's waypoint is an invalid
location, the waypoint is snapped back into the leader's cell — or, on a
short move, replaced by the leader's own point outright.

That snap is `p + (w/0x300 − p/0x300) × 0x300` on **both** axes.
Ghidra renders the `x` half with a `0xc0` stride, which would make it an
asymmetric original bug worth reproducing; it is not one. `70672d`–`706764`
is `leal (%eax,%eax,2)` then `shll $0x8` — × 3 × 256 — and the
`imull $0x2aaaaaab` / `sarl $0x7` pairs either side are signed divides by
`0x300`. Reading the decompiler here would have shipped a real defect.

Then the waypoint is pushed — but a **follower** only receives a
**non-final** waypoint when it is *not* in a group move
(`orders != MOVE_TO`, or modern infantry, or sea, or `form == 9`) **and**
is within `0x600` of the leader. Group-move followers get the final
waypoint only; the intermediate legs are the leader's, and
`do_group_move` re-derives theirs each frame. Two other members are
dropped before that: an **AI**'s sea member on a non-final waypoint that
is not the leader (`706473`, opening on `leaders & 4`, so a human's navy
is exempt), and — for **Column** on a non-final waypoint of a planned move
— nobody, but the slot index becomes `cols[i]` rather than `i`
(`7064db`–`706507`), and `cols` is the static above, sized to `num` on
every call and **written by nothing**. That arm reads uninitialised heap.

Finally every member's stack is inverted, `group.order_num += 1`, and the
`Form`'s tables are zeroed.

### 6.8 `Group::refresh_group_order@00713a50` — the other writer of `off`

Found 2026-08-26, and only because run31's record refused to match a table
that had every other number in it right. **`compute_dests` is not the last
thing that writes the group's offsets.** Once the move is out, the block
gets handed from member to member, and each hand-over slides it.

The trigger is in `Unit::do_group_move@005e79a0`. Every member executing a
group move first looks up the unit its order's `GroupOrder::oxx` names —
the block's current origin — and checks that it is still usable: active, on
the map, in **this** group, holding an order that is a group order (or is a
`CHANGE_FORM`), and whose `GroupOrder::id` is this one's. If any of that
fails, and the mover is still more than `0x5ff` from its own point, the
mover calls

```
refresh_group_order(group, order_id, this->o, this->who)
```

on itself. Otherwise the fallback is `ungroup_move_order` — the member
leaves the formation and walks alone.

What the call does, in order:

1. `normalize` (§4.3);
2. reads the taking unit's own current order, its `get_group_order()`
   (vslot `0x94`) and that order's **`form_id`** (`+0x10`) — its index into
   the group's arrays;
3. **subtracts `off[form_id]` from every entry**, over `form_num` of them,
   so that the taking member sits at `(0, 0)`. The subtraction is on the
   **quantised** offsets, the record's own small numbers, so it is exact;
4. `Unit::modify_group_order` on every valid on-map member — this is what
   re-points `oxx` and rewrites the destinations;
5. `update_positions`, now off the **taking** unit's heading.

So `GroupOrder::oxx` is *the block's current origin*, which starts as
`find_leader`'s pick and moves. That is the whole of §4.4's old question
about run31's leader, and it is why the anchor of a dumped `GROUPDATA` is
not always the member `find_leader` would name.

run31 is one frame of forty past the layout on its first move and dead on it
for the other two: its frame 204 is `compute_dests`' table re-origined onto
member 15, its frame 205 (no orders left, so outside the fixture) is that
re-origined again onto member 24, and frames 328–366 are the table itself.
Nothing about the *shape* differs between them —
`run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns`
reproduces all three from one computation.

The simulation implements step 3 and step 5 (`GroupState::reorigin`,
`Sim::group_refresh_order`). Steps 1, 2 and 4 need the group-order layer
§12's third seam stands in for, and the **trigger** with them: no simulated
group has ever lost its origin, because every member gets a plain
`MoveOrder` that names nobody.

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
  taken** — runs 21–27 have no siege in any army — ~~every~~ until run133's
  wagon army took the wagon branch on 1277 (item 567, `docs/ORDERS.md`
  §24);
- for an AI, the anchor's **area id** (§6.7's `world +0x134` record, with
  the halfland rule) is compared with the destination cell's **land** area,
  and a mismatch falls back to the same whole-group attack-move. The two
  sides are not read the same way: the **source** resolves through
  `flags & 0x100` to `region` or `region2`, going `>> 6` then `>> 2`, while
  the **destination** is always read at `+0x4` and reaches its cell through
  `div_3_table[v >> 8]` directly;
- otherwise the **sub-group** attack-moves to `(x, y)` — on **its own
  record**, `Group::clear`'s, so its `facing` is 0 whatever the parent's
  is (§26, item 736) — its slot offsets and
  angle bytes are copied back onto the matching members of the parent
  (`Unit::replace_form_id` re-indexes each), and the parent gets
  **`action_guard(anchor, who, QUEUE_NEW, 1)`** — everyone escorts the
  anchor while the siege walks in. **Built by item 567**
  (`docs/ORDERS.md` §24): golden chapter four's army, with a Supply
  Wagon and no siege, takes this branch on 1277, and the escort's
  `GUARDORDER` is compared block for block.

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
  **when `mandatory == 0`**, `o'` is `Unit::find_melee_target` (its word 1
  for a unit target and 2 for a building, `docs/COMBAT.md` §66.2) within
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
- `find_leader` (§4.4) over the sim's units — with the slot index the
  original's out-parameter carries — `is_on_map`, `get_stance_type`,
  `num_valid`, `normalize`'s prune, `find_role`;
- **`add`'s two recursions** (§4.1): `Sim::group_add_keeping` over
  `Unit::o_up`/`o_down`, so a group can hold a squad's figures and not just
  its captains. Nothing in the simulation *builds* such a group yet — an
  army holds captains — but the harness does, and it is what run31's
  fixture stands on;
- **`action_halt`** (§7) whole, mask and all;
- **`action_stance`** (§8): the option count by stance type, the negative
  cycle, the per-member write and the combat table's three arms;
- **`action_move_near`** (§6): the domain split's `army < 0` guard, the
  `QUEUE_FIRST` rotation, `get_form`/`get_loc`, `compute_form`'s angle and
  reverse rules, **the AI branch of §6.5 whole**, and the ordinary path's
  order choice. ~~**Not §6.7**~~ — **§6.7 landed 2026-08-26**, in
  `crates/sim/src/grouppath.rs`: the one search off the leader's raw slot,
  the `0x900` short-circuit, the `pathfinder +0x70` hint, the per-member
  translation with its clamp and its area guard, the follower cutoff, the
  no-leader arm and the ungated tail invert. §6.6 step 2's `QUEUE_LAST`
  pre-invert lands with it, because it means nothing without a stack to
  invert, and the orders the member loop issues are now born `PATHED`
  (`add_move_facing_order`'s `param_5`). §12.4 has what it moved;
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
- **`refresh_group_order`'s re-origin** (§6.8): `GroupState::reorigin` and
  `Sim::group_refresh_order`, the second writer of `off`. Its trigger is a
  seam;
- **the mirror flag's whole state machine** (§6.3): `sim::group::reversing`,
  the `QUEUE_NEW` clear hoisted ahead of the layout, `Sim::unit_set_angle`
  on `Unit::move_step`'s own call, and `Sim::hand_back_facing` inside
  `kill_current_order`. `MoveOrder::facing` carries the mirror out with the
  order, so the flag a later layout reads is the last one's answer — **and
  the tail negation beside it**: `Form::flipped`, from `set_angle &&
  reversing(find_angle(dest − group_loc) − angle)`, negating `Form::off`
  and, quantised first because the divide is a floor, the group's own;
- and `army.rs`'s five callers become behaviour: `do_forming`,
  `march_to_target`, `engagement`, `send_here`, `charge`, `Army::close`'s
  halt and `set_stance`.

## 12.0 The seams, each with what it costs

Split out of §12 on 2026-09-18 (item 350) because the section reached the
paperwork guard's ceiling, and this half is the one that grows: every
landing either closes a row or narrows one. Nothing cites it by number, and
§12 still means the list above.


| seam | stands in for | what it costs |
| --- | --- | --- |
| ~~`Form::compute`'s slot table (§6.4)~~ | — | **Closed 2026-08-26**: `crates/sim/src/form.rs`, diffed against run29's `GROUPDATA` from the install's own columns. What is left is the four limits at the end of §6.4 — Square, the wedge's uninitialised seed, Mob's rings, and `categorize`'s two type substitutions — and every one of them is the original's |
| the group pool (§3) | 64 slots a leader, `get_open_slot`'s recycling | one group per army, never recycled; `push_group`'s `force == 0` rule, `equals_group` and **its kill out of the old group** (item 350, `docs/ARMY.md` §3.4) are modelled, the slot allocation is not — so the only group a push can empty is an army's |
| `GroupMoveOrder` | §6.6's per-frame formation — the follower that tracks the leader's *current* position plus a rotated offset | every member gets a `MoveOrder` and marches to its own slot independently; `docs/ORDERS.md` §8.4's verdict, unchanged. ~~And the order's angle: step 6's `angle + (angles[i] << 24)`.~~ **The angle has left this row 2026-08-26**: `Sim::add_move_facing_order` carries the formation's own bearing plus the slot's packed byte, and `MoveOrder::facing` carries the mirror, which is what §6.3's hand-back reads |
| **Column's `cols[i]`** (§6.7) | the slot index a Column formation translates a non-final waypoint by | the member's own slot. `cols` is a function-local static of `action_move_near` (`0xee155c`) that nothing writes, so the original reads uninitialised heap there; every simulated group is form 0, and a `debug_assert!` refuses to pretend otherwise |
| ~~the follower arm of `compute_dests` (§6.4)~~ | — | **Closed 2026-08-26**: `Sim::form_follower_slot`, read from `0072d3a0`–`0072d4f0`. ~~The simulation still has no group that *contains* a follower — `Group::add`'s `keep_captain` (§4.1).~~ It does now: `Unit::o_up`/`o_down` and `Sim::group_add_keeping` carry §4.1's two recursions whole, so a group built from captains holds every figure, and the arm is reached from the sim's own side by run31's 36-member fixture |
| `refresh_group_order`'s **trigger** (§6.8) | `do_group_move`'s "is `oxx` still usable, and am I still `0x5ff` out" | the re-origin and the re-rotation are implemented (`GroupState::reorigin`, `Sim::group_refresh_order`); nothing in the simulation ever *fires* them, because a plain `MoveOrder` names no origin to lose. `modify_group_order`'s order rewrite is unmodelled with the rest of the group-order layer, one row up |
| `action_guard` (§9) | the escort half of a siege attack | with siege *and* a matching area the non-siege members keep their orders instead of guarding; no traced army has siege |
| `find_nearby_spot`'s collision (§6.6 step 4) | re-slotting an invalid slot | it never asks the occupancy index (`docs/COLLISION.md`) |
| `invalid_loc` on a slot, the `tregion` re-slot | §6.6 step 4 | same |
| `QUEUE_FIRST`'s insert dance (§6.2, §10) | `set_up_insert` / `action_halt` / recurse / `finish_insert` | `charge`'s `QUEUE_FIRST` is a plain push-to-front on each member |
| the scenario filter (§5) | `ignore_orders` | never set outside a scenario |
| `unit_masks` `0x100` / `0x400` / `0x4000000` | three bits `action_halt` and §6.6 clear | unmodelled bits; no reader in the sim |
| `is_entering_or_exiting` (§7), `OBJECT_NEW_THINK` (§8) | a halt skips a unit in a doorway; a stance write flags the unit for a re-think | two writes the third pass recorded and the sim does not model; no reader in the sim for either. `unit +0xab = width` **has left this row**: it is written now, and `get_form_mod_option` reads it (§6.4) |

**The checks**, cheapest first, in `group.rs` and `army.rs`'s test modules:

0. **`compute_form_negates_the_offsets_when_the_bearing_opposes_the_angle`**
   — §6.3's tail: one destination and angle, two opposite approaches, the
   destinations agreeing and the offsets negating. Red twice.
1. `push_group_refuses_a_singleton_unless_forced` and
   **`a_pushed_group_takes_its_members_out_of_the_army`** — §3.2's two
   live rules.
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
7b. `reversing_takes_both_bounds` and
   **`a_group_s_mirror_is_the_last_layout_s_and_not_the_march_s`** — §6.3's
   state machine driven by the simulation rather than replayed out of a
   dump: an unmirrored layout, a leader that overshoots and flips the flag,
   and a second click at a different bearing where reading `facing`
   straight would mirror a block that must not be. Its record-side twin is
   §12.3.

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

12. **The 36-member table** (§12.2), one in `rondata::diff` and two in
    `group.rs` —
    `adding_a_captain_takes_its_figures_and_adding_a_figure_takes_its_captain`
    for §4.1's two recursions and
    `a_refresh_re_origins_the_table_onto_the_member_that_took_it_over`
    for §6.8's slide. Five more deliberate breakages, all red first try.

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

~~**What is left to do with the capture**, in order: reproduce the whole
36-member table … and then §6.4's displacement measured rather than only
observed~~ — **done 2026-08-26, §12.2.** What is left of the list is the
last item: diff `GroupMoveOrder`'s angle against §6.6 step 6's
`angle + (angles[i] << 24)`, which §12's third seam is still standing in
for. run31 **cannot** settle it: its group is a Line, so every
`group.angles[i]` is 0 and every order's `angle` is the formation's own,
to the bit, on all 36 members of all 40 frames. The adder needs a
formation that leans.

## 12.2 The 36-member table, reproduced

`run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns`,
in `rondata::diff`. It stands run31's group up in the harness and runs
`sim::form` over it — nothing fitted, no fixture written from the answer —
and then compares against all forty records.

The chain, from `unitrules.xml` to the record:

| step | value |
| --- | --- |
| Hoplites and Slingers: `X_SPACING`/`Y_SPACING`/`GUY_SPACING 12`, `UBER_SIZE 3` | `× UNIT_FORMATION_SPACING 12` → `144` each |
| `categorize` widens a multi-figure rank by `min(uber_size, 3)` | `x_spacing = 432`, `y_spacing = ⌈3/3⌉ × 144 = 144` |
| `type_cat`: hoplites `FOOT`, slingers (ranged) `FOOT_RANGED` | `count = 8` and `4` captains |
| `span = max(8 × 432 / 2, 4 × 432)`, `form_mod 50` | `1728`, so `cols = 4` for both, `rows = 2` and `1` |
| `Group::add`'s subordinate recursion (§4.1) | **36** members, the record's own `list` object for object |
| `compute_dests` + the rank stack + the floor divide by 48 | the record's `off`, all 36, both coordinates, all 40 frames |
| §6.6 step 6's `UCoord` round trip on each `to` | the record's `MOVEORDER` `x`/`y`, **900** of them |

What it settles that nothing before it could:

- **§6.4's `to`/`off` displacement, measured**: `o_dist` is it, and it is
  `x_spacing/2 = 216` (§6.4).
- **§4.4's leader question, answered**: `find_leader` names object 6 both
  times, and frame 204's record is one `refresh_group_order` past the
  layout (§6.8).
- **`Group::refresh_group_order` itself** — a second writer of `off` that
  no earlier pass had looked for, found because the table would not match
  on one frame in forty when every other number in it was right.
- **The mirror is not `facing`** (§6.3), which §12.3 then closed out of the
  same dump.

Five deliberate breakages were run red before it landed, and each did on
the first try: dropping `Group::add`'s subordinate recursion (12 members
instead of 36), sizing a multi-figure type by one figure instead of three
(`x_spacing 144`), making the re-origin a no-op (frame 204 alone fails),
allowing only the unmirrored layout (the third click alone fails), and
comparing the raw `to` instead of the `UCoord` round trip.

## 12.3 The mirror's predicate, out of the same dump

`run31_s_three_mirrors_come_out_of_facing_s_three_writers`, in
`rondata::diff`. §12.2 left "which mirror, and why" as this document's
sharpest open question and named the run that would settle it. **No run
was needed.** The question was a reading error twice over — the wrong
frame's heading, and two of `facing`'s four writers unlooked-for — and
run31's own file holds every number the corrected machine needs. §6.3 has
the machine and the table; this section is what checks it.

Six predictions from three clicks: each move's mirror, taken from the
`facing` byte its own `MOVEORDER`s carry, and each click frame's dumped
`GROUPDATA::facing`. All six land. The control in the same test is the
model this document held until today — read `facing` off the previous
frame's record and XOR the toggle — and it gets **one of the three**
wrong, which is the whole of the disagreement.

Five deliberate breakages, and the fifth is the useful one:

| broken | result |
| --- | --- |
| the hand-back dropped, so the flag is what the march left | red at frame 328 — the mirror is 1 where the record needs 0 |
| `set_angle`'s toggle dropped | red at frame 204 — the dumped flag is 0 where the record prints 1 |
| `compute_form`'s toggle dropped | red at frame 356 — the mirror is 0 where the record needs 1 |
| the click frame's own heading instead of the frame before | red at frame 204 |
| the hand-back's `reversing` **inversion** dropped | **green** |

The last row is the honest limit. Both of run31's kills catch the leader
10.6° and 6.3° off the dying order's angle, so the inversion never fires
and the listing at `5e3062`–`5e307b` is its only evidence. §13 carries it
with the capture that would reach it.

## 12.4 What §6.7 moved, and what it did not

Landed 2026-08-26 against `gamelog-run20-islands-dumpall.txt`, whose unit
`1/0` is the cheapest possible fixture for this section: a **one-member**
group on auto-explore (`Sim::scout_issue` → `group_action_move_to` →
`push_group(force = 1)`), so the slot translation is the identity and what
is left is exactly *when* the path is planned and *what goal* it is planned
to.

**Two of the three disagreements §13 named are closed.**

| | before | after | the original |
|---|---|---|---|
| the order's flags at frame 1 | `0` | `1` (`PATHED`) | `1` |
| the stack at frame 1 | empty | 7 entries | 9 |
| the stack's bottom | `(41976, 36600)`, the order's snapped `dest` | `(41952, 36576, 0, 1)` | `(41952, 36576, 0, 1)` |
| whole entries shared | 0 | **4** | — |
| `rondata --diff`, order disagreements | 1 (`Flags`) | **0** | — |

The position at the plan frame is the same on both sides —
`(38040, 40344)` — so the two searches are now genuinely comparable, and
the test pins that too.

**The third, the route, is not this mechanic's.** Both sides now plan from
the same point, to the same goal, with the same `toff`, and the chains
still part in the middle: the original's runs along cell row 50 where the
simulation's runs along row 51, and it carries one more node at each end.
That is `astar_path`'s, and it belongs to `docs/PATHFINDER.md`. With one
member the *translation* half of the route disagreement cannot be measured
at all; no capture on disk holds a multi-member group's path stacks.

**The path-stack count went up, and that is the count's fault rather than
the port's.** `rondata --diff` scores run20 at **21** path-stack
disagreements where the old code scored 17, because at frame 1 there is now
a seven-entry stack to disagree with instead of an empty one, and the
differ compares slot for slot from the bottom while the two chains agree
one slot apart. §13's old entry said the same thing about the frame-2
comparison; the instrument with teeth is
`run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot`, which
compares **whole entries** — position, tolerance and flag — and does not
care where in the stack they sit.

**The checks**, five, and every one made to fail on purpose first:

- `diff::tests::run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot`
  — the table above. Red first by handing `add_move_facing_order`
  `pathed = false`, and again by pushing the order's snapped `dest` in
  place of `form.to[idx]`.
- `grouppath::tests::a_group_move_plans_one_path_and_the_follower_takes_only_the_goal`
  — two members, form 0, `MOVE_TO`: the leader carries the chain and the
  follower carries its own raw slot alone. Red first by pushing every
  waypoint to every member.
- `grouppath::tests::a_short_group_move_plans_no_route_at_all` — the
  `0x900` gate, at a distance chosen to clear `find_wpath`'s **own** near
  test (`vector_dist 2250`, cell-Manhattan 4), or it would prove nothing.
  Red first by removing the gate.
- `grouppath::tests::a_follower_out_of_formation_takes_the_legs_only_while_it_is_near`
  — an `EXPLORE_TO` group, one follower inside `0x600` and one outside.
  Red first by dropping the distance test.
- `grouppath::tests::the_tail_invert_turns_over_a_stack_the_move_never_wrote_to`
  — §6.5's shooting siege under a non-hurrying AI move: the move skips it
  and the invert does not. Red first by gating the invert on the same
  `Member::Move` the rest of the section uses.

## 12.5 `get_loc`'s two substitutions, and the cell they move

**Closed 2026-09-17, item 314.** §12's second seam said the `(ox, oy)`
substitutions were unmodelled and that `Sim::group_loc` returned the
leader's position flat. It does not any more, and the thing that closed it
was **one field of one record**.

**Why it mattered.** `group_move` feeds `get_loc`'s answer to two places:
the formation angle (§6.3) **and** `group_plan_path`'s start, which is the
cell `find_wpath_from` searches from. A substitution here therefore moves
the route, not just the bearing — and item 312's five-link chain on Great
Lakes began with the leader `1/37` planning **one head waypoint more** than
the original from what looked like the same start.

**Both gates, from the listing.** The decompiler loses both `vector_dist`
operand pairs to `unaff_EDI`/`unaff_ESI`, so no reading of
`funcs/GroupData/get_loc@0070e030.c` can settle them.
`llvm-objdump -d --start-address=0x70e030` does, in a minute:

| | at | operands | on success |
| --- | --- | --- | --- |
| arm 1 | `70e109`–`70e12d` | `vector_dist(leader − (ox, oy))` | return `(ox, oy)` |
| arm 2 | `70e190`–`70e1bc` | `vector_dist(order − (ox, oy))` | return `leader − order + (ox, oy)` |

Both compares are `cmpl $0x180` followed by `jg`, so **both bounds are
inclusive** and the decompiler's `< 0x181` on the first names the same set.
Before either, `ox < 0 || oy < 0` returns the leader's position untouched
(`70e0f8`, `70e103`).

**`order` is the order's `x`/`y`, and only the type record says so.** Arm 2
calls the order's vtable `+0xb8` — `MoveOrder::get_move_order` — and reads
`[+4]` and `[+8]` off what comes back. `MoveOrder +0x4`/`+0x8` are named
**`x`/`y`**, the order's *destination*; `orig_x`/`orig_y` are at `+0x44`
and `+0x48`. The surrounding code reads identically either way, and on the
block that settled this the two hold the **same pair**, so neither the
decompile nor the dump can separate them — the layout can. (§6.3's prose
said "origin" until this entry.)

The two gates either side of arm 2 are the object's `+0x18` (it is a unit)
and `UnitData::is_moving` (`+0xd8` — the head order exists and its
`get_type` is non-zero); with `get_move_order` after them, all three
collapse in this simulation to "the current order is a move".

**The values, from run92.** No Great Lakes dump carried a `GROUPDATA` past
**5591**, so a `GROUPS` window over `[7668, 7690)` was taken for this one
field (`docs/RUNS.md`, "run92"). On block 7674 — the state the plan of
sim-frame 7674 reads:

- group `64` (`who 1`, `army 1`, `num 12`, `form_num 9`) carries
  `(ox, oy) = (36303, 23348)`;
- the leader `1/37` stands at `(43174, 24583)`, **6981** from it — arm 1
  refused;
- its current order's destination is `(36600, 23400)`, **301** from
  `(ox, oy)` — inside `0x180`, so **arm 2 fires**;
- the answer is `(42877, 24531)`, whose `0x300` cell is **`(55, 31)`**
  against the leader's own **`(56, 32)`**.

`(55, 31)` is exactly the cell of the head waypoint this crate used to emit
and the original does not: the original's search begins *inside* that cell
and never steps into it. One diagonal cell, and it was the first link of
the chain that cost the headline 251 frames.

`run92_says_great_lakes_7674_takes_get_loc_s_second_arm` is the assertion —
the original's record and the arithmetic over it, made to fail once on the
arm-2 gate.

**What this does not establish.** `get_loc`'s off-map arm — `is_on_map`
(`+0xbc`) failing, then `ObjectData::get_inside` re-aiming both the player
and the object index at the container — is still not modelled; the
simulation reads the leader's own position and no traced group has had a
garrisoned leader. Nor is the `buildings` branch (`list[0]` instead of
`find_leader`), which no simulated group takes. Both are in §13.

## 13. What is not established

Still open:

- ~~**`Group::target_opportunity` is how a squad engages, and the simulation
  does not have it.**~~ **It is not** (item 395, 2026-09-19,
  `docs/COMBAT.md` §21). `Unit::think_attack@005f5a80`'s tail does call
  `Group::target_opportunity(group, o, who, …)` on a unit find — but that
  function's loop only enters a member which is **itself a captain**
  (`o_up < 0`), and a squad's members are not, so it never reaches them.
  What hands a member its captain's target is `Unit::think@005f6e40`'s
  **first statement**, above every gate in the function: a non-captain
  reads its captain's action, and if that action is an ATTACK order on a
  target it can validly attack it takes
  `add_attack_order(target, QUEUE_NEW, captain's mandatory, 0)` and the
  think ends. That is what the golden record's witness is really saying —
  `ObjectData::near_o`/`near_who` are written by `find_nearby_target`
  alone, and over chapter one's 901 frames only the two **captains** ever
  carry them. Implemented as `Sim::captain_mirror`; the golden word moved
  **619 → 624** with it.

  What `Group::target_opportunity` does on that frame is hand the target
  back to the **asker**, which is how a grouped captain gets its own order
  — see the listing below. It stays unimplemented, and so does
  `Unit::find_melee_target@005ff9c0`'s own copy of the mirror
  (`docs/COMBAT.md` §21.3), whose only reachable caller here is §10's
  per-member `find_melee_target`.

  **And what the tail actually is, from the listing** (`005f5da6`
  onwards), because the entry above understated it: `think_attack` calls
  `find_melee_target(this, max_dist, &who, 0, 1, 0)` and then, **only if
  the find is a unit**, `if (this->group >= 0 && GroupData::member(group,
  o, who, 1)) Group::target_opportunity(group, found_o, found_who, o, 1)`
  — and returns 1. There is **no `add_attack_order` in this function at
  all**. An *ungrouped* unit that finds a unit target here therefore takes
  no order from `think_attack`; a building find falls through to the city
  block below instead. This crate calls `add_attack_order` directly, which
  is a second divergence in the same arm.

  `Group::target_opportunity@007107d0` itself: a **15-frame cooldown** on
  the group (`+0x38`, `frame − last > 0xe`), then over the member list
  (`+0x8cc`, count `+0xc`) every member that is alive, on the map, a
  **captain** (`o_up < 0`) and combat-role (`type +0x2c8 & 0x10000`) takes
  `Unit::target_opportunity(member, o, who, 1)` — except that a member
  whose action order is absent or not "moving" (`vtable +0x10`) and whose
  `order_type` is `NONE`, `ATTACK_TO` or `GROUP_ATTACK_TO` runs its **own**
  `find_melee_target` at `min(dist + 0xc0, unit_respond_range × 0x240)`
  instead. `param_4 == 0` and the member being the asker's own captain is
  a third arm, `Unit::target_opportunity` without the action test.

  ~~**And the group now moves a unit the original leaves seated**~~
  **(item 386) — it was not the group** (item 391, `docs/COMBAT.md` §18.2).
  This crate's `1/6` did drop its target at the end of 617 and take a
  `GROUP_ATTACK_TO` where the original holds `(1368, 7992)` for the whole
  record; the order came from `Armies::emergency` → `Army::process` →
  `Group::action_siege_attack_to`, and what reached the emergency was a hit
  this crate had `0/7` deliver a frame early. `Unit::target_opportunity`
  answers on the victim's **captain**, not on the figure that was hit, so
  the order belonged to `0/6` and `0/7` had none until the end of 617.
  ~~With that corrected, nothing in chapter one's 901 frames reaches the
  emergency again~~ — **it is reached again, on 618** (item 395,
  `docs/COMBAT.md` §21.4): once `0/7` retaliates on the frame the dump has
  it retaliating, the hit reaches `Armies::emergency(1)` and this crate's
  bypass tick still walks all three of who=1's hoplites, toward
  `(38664, 13320)`. The **gate is not the defect** — `Object::do_damage`'s
  test at `0064bbfd` wants the victim's leader to read `leader_flags & 4 ==
  0`, which is `LeaderData::is_human@006ec170` (`return leader_flags & 4`),
  so the emergency is the *computer* leader's and fires in the original too
  (`docs/ARMY.md` §15's "non-human" reading is right, and this names the
  bit's own function). What diverges is inside `Army::process`, and it is
  what stands at the golden word's 624. The golden word moved 617 → 618 on
  391 regardless. Neither
  `Group::target_opportunity` nor a "the group should defer" rule was
  involved — the group branch of `Unit::target_opportunity` is guarded by
  `type +0x2c8 & 0x10000 == 0`, and every hoplite is combat-role, so a
  hoplite never takes it at all.

- **`get_loc`'s other two arms** (§12.5). The substitutions landed; two
  branches around them did not. `is_on_map` (`+0xbc`) failing sends the
  original through `ObjectData::get_inside`, which re-aims **both** the
  player index and the object index at the container before either
  substitution is considered — a garrisoned leader's group therefore
  measures from the building, and `Sim::group_loc` reads the leader's own
  position. And `buildings != 0` takes `list[0]` where `find_leader` is
  taken otherwise. Neither is reached: no traced group has had a leader
  inside anything, and no simulated group is a building group.
  *Capture:* a `GROUPS` window over an army whose leader boards a
  transport or garrisons, read for the group's `(ox, oy)` and the
  member's own position on the same block.

- ~~**§6.7 is not implemented at all**, and run20 measures the cost.~~
  **Implemented 2026-08-26**, `crates/sim/src/grouppath.rs`; §12.4 has the
  table and §16 has what the listing corrected on the way. Of the three
  disagreements this entry named, the **goal** and the **timing** are
  closed and the **route** is not:
  - ~~the goal~~ — the bottom entry is `(41952, 36576, 0, 1)` on both
    sides now.
  - ~~the timing~~ — frame 1 is `PATHED` with a seven-entry stack.
  - the **route** is open, and it has changed owner. Both sides plan from
    the same position to the same goal with the same `toff`, and
    `astar_path` still parts a cell row through the middle and drops a
    node at each end — `docs/PATHFINDER.md`, not this document.
  What this document still owes is the **translation** half, which run20
  cannot reach: its group has one member, so `slot[i] − slot[leader]` is
  zero on every waypoint. *Capture:* a `UNITS=3` window over an army of
  three or more given one move, read for the members' path stacks rather
  than their positions — the same window item 23 already owes, widened.
  It would also settle the follower cutoff and the AI sea guard, neither
  of which any dump on disk exercises.

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
  `GroupOrder::oxx` names the leader outright (§4.4, §12.1). ~~What the
  capture opened instead is **which** of two equal-category captains
  leads — object 9 on one frame and object 6 on the next, where a
  first-wins tie says 6 both times.~~ **Closed 2026-08-26** (§12.2):
  `find_leader` names object 6 both times, and `oxx` is the block's
  *current origin*, which `refresh_group_order` moves (§6.8).
- ~~**§6.4's `to`/`off` asymmetry still has no capture.**~~ **run31 is
  the capture**, the asymmetry is observed, ~~**Still open:** its
  *size*~~ **and measured** (§6.4, §12.2): `o_dist` is the displacement,
  and it is `x_spacing/2 = 216`, printed as 215 or 217 by `vector_dist`'s
  approximation at three bearings.
- ~~**The mirror's predicate** (§6.3) — this document's sharpest open
  question, and it was opened by the same capture that closed two.~~
  **Closed 2026-08-26 out of run31's own dump, with no new run** (§6.3,
  §12.3). Both horns of the old dilemma were half right and the entry
  named neither writer: `facing` *does* move between the clicks, and what
  moves it is `Unit::set_angle` during the march and
  `Unit::kill_current_order` at the next click, the second of which runs
  **before** the layout and assigns the last order's own mirror back. The
  heading to read is the frame **before** the click, not the click frame:
  the old entry's "76° off" is frame 204's own end-of-frame angle, and 204
  is the one frame in the run where that reading cannot work.
  ~~*Capture:* a `UNITS=3` + `GROUPS=1` window across a right-click…~~
  **Not owed.** What is still owed is narrower, and it is the one
  breakage that stayed green:
- **The hand-back's inversion** (§6.3, §12.3). `kill_current_order` writes
  `order.facing XOR reversing(leader.angle − order.angle)`, and run31's
  two kills catch the leader 10.6° and 6.3° off the dying order's angle,
  so the `XOR` term never fires. The listing at `5e3062`–`5e307b` is its
  only evidence.
  *Capture:* a `UNITS=3` + `GROUPS=1` window over a group ordered one way,
  turned **right around** while marching — a second click behind it, or a
  path that doubles back — and then re-ordered, with the leader's `angle`
  and the standing order's `angle` read on the frame before that third
  click. The same window would settle the packed byte's sign in
  `angle + (angles[slot] << 24)` if the formation is set to Refused or an
  Echelon so that `angles` is not all zero — the listing gives it as an
  addition at `705f42`–`705f4d` against `compute_form`'s subtraction at
  `707e95`–`707ea6`, and no run has separated them.
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
  reading is still the only evidence for both. **Two came off the list on
  2026-08-26 without a trace run**: `refresh_group_order` is read and
  implemented (§6.8), and run31's record *is* evidence it ran — one of its
  forty frames is unexplainable without it; and `Group::sort` is read
  (`00708090`) — it is `categorize`'s own first statement, so every
  formation any run has laid out has executed it. The trace has not seen
  either because no traced game has held a **player's** selection group,
  which is a gap in the runs rather than in the reading.
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

## 16. The path's implementation, and what the listing corrected

Built 2026-08-26 from §6.7's own text plus the listing at `706128`–`7069a8`.
The project's default applies here as it did to §15 — *where a reading's
product is arithmetic, the implementation is a pass of the audit* — and it
earned its keep again. Four things the decompiler said, or did not say,
would have shipped as defects:

1. **The area-guard snap's `x` stride is `0x300`, not `0xc0`.** Ghidra
   renders it as `local_14 + (param_11 / 0x300 - (int)local_14 / 0x300) *
   0xc0` against the `y` half's `* 0x300`, which reads as a plausible
   original bug: a tile stride where a cell stride belongs, asymmetric,
   exactly the kind of thing this project reproduces on purpose. The
   listing at `70672d`–`706764` is `leal (%eax,%eax,2)` then `shll $0x8`
   — × 3 × 256 — on **both** axes. Transcribing the decompile would have
   introduced a defect the original does not have, and no test on disk
   would have caught it, because no capture crosses a coastline.
2. **`cols` is a function-local static that nothing writes.** The
   decompile shows `cols._padding_` a dozen times with every field folded
   onto one name, which reads like an ordinary array being filled. It is
   `0xee155c`, allocated to `num` ints on every call by
   `action_move_near`'s own prologue and written by no instruction in the
   export. Column's non-final waypoints therefore index uninitialised
   heap. That is a **seam by necessity**, not by choice, and §12's table
   says so.
3. **`pathfinder +0x70` is not a private hint.** §6.7's old text called it
   "an AI hint the pathfinder reads", which is true and unhelpful. It is
   the *same word* `find_wpath` sets for an AI's own armed non-worker
   (`00688fc0:242`) — the `army` mode of `docs/PATHFINDER.md` §3 — and the
   only reason to set it from outside is that a **human** jumps that whole
   block. So an army's group move gets AI army costing whoever owns it,
   and a stack group never does.
4. **The `0x900` is measured to the leader's slot, from the search's own
   start.** Not from the group to the order's point, which is what "within
   `0x900` nothing more is planned" invites. `70614e`–`706172` subtracts
   the slot pair that was pushed four instructions earlier from the exact
   pair handed to `find_wpath` at `706187`.

And one the *test* corrected rather than the listing: a first draft of
`a_short_group_move_plans_no_route_at_all` put its goal two cells away,
which is inside `find_wpath`'s **own** near test (`docs/PATHFINDER.md` §3),
so removing the `0x900` gate left the assertion green. A guard that cannot
fail has not been tested; the distance was moved to `(1500, 1500)` —
`vector_dist 2250`, cell-Manhattan 4 — where only the `0x900` gate can
produce a one-entry stack.

## 17. `QUEUE_FIRST`'s insert, and the walk it drops

*2026-08-31, item 106. Amends §6.2, which the guard's size pin will not let
grow; §15 and §16 are the same shape.*

§6.2's closing sentence — "this is `docs/ORDERS.md` §1.5's rotation, done at
the group level" — is **wrong**, and the rest of §6.2 is what says so.
`Group::set_up_insert@0070e520` copies only the leader's orders whose
`flags & 4` is set, and that is the **action** bit. A plain transit move
does not carry it, so it is not copied; `action_halt(this, 0)` then kills
it, and `finish_insert` has nothing to put back. A group's `QUEUE_FIRST`
therefore **drops** the walk a member was on, where the *unit's* own
`QUEUE_FIRST` (`docs/ORDERS.md` §1.5) stacks the new order in front of it
and keeps the old.

The difference is diff-backed, and the caller that shows it is
`Unit::get_goody_box@005f7690` (`docs/GOODY.md` §7.3): on frame 825 run39's
AI scout re-aims at a goody box, and its `FRAME 826` record holds **one**
order — the box's walk alone, with the `think_scout` target it was given on
796 gone. A unit-level `QUEUE_FIRST` leaves two, which is what this crate
did until this item, having passed the queue position straight down to
`add_move_facing_order`.

`finish_insert@0070e620` re-issues each copy as a **group** action at
`QUEUE_LAST`, switching on the order's own index over twenty-one cases.
`crates/sim/src/group.rs` carries the move and the attack arms and stands
in for the rest; ~~no capture on disk reaches a group `QUEUE_FIRST` whose
leader holds one of the others~~ run157 reaches the `BUILD_AT` case, which
§24 builds; `army_charge` is the only other caller.

## 18. The speed cap — what `GroupData::speed` is, and who writes it (item 515, 2026-09-22)

`UnitData::get_speed@00608720`'s last arm is four lines, and until this
item they were a declared seam in three places
(`crates/sim/src/orders.rs`, `docs/MOVEMENT.md`'s layer 3, and
`do_group_move`'s own comment). They are the reason a mixed formation
marches at its slowest member's pace, and they cost Great Lakes' word 583
blocks of drift:

```c
if (param_3 == 0 && -1 < this->group && local_8 == 0 &&
    (g = groups[this->group].speed, g != 0 && g < speed))
   speed = g;
if (speed < 3) speed = 3;
```

Four gates, and each is load-bearing.

- **`param_3 == 0`** is the flag, and the cap is the *only* thing it
  selects. `do_move` (`005f7b30:556`), `Unit::find_path`,
  `Object::poor_target@0064a270:26` and `:31`, and `do_group_move`'s
  **cross**-formation arm (`5e83f6`) pass 0. `GuyData::get_speed`, the
  animal's air step, and `do_group_move`'s **in**-formation arm (`5e8355`)
  pass 1. So the body is never capped, and neither is the third a
  follower is given to close up with — which is what keeps the cap from
  freezing a straggling block.
- **`local_8 == 0`** is `get_action()` answering null, re-used from the
  order scale twenty lines above. No `OrderIndex` is 0, so the test is
  exactly "this unit has no action order". A unit *attacking* inside a
  group is therefore uncapped **and** takes the `× 9/8` of the same
  `local_8`; a unit merely walking is capped and scaled by nothing.
- **`g != 0`** — a group whose speed has never been computed caps nothing.
- **`g < speed`** — the cap only ever lowers.

### 18.1 A one-pass-lagged minimum, whose two writers are inlined

`Group::compute_speed@00707f80` sets `speed` and `new_speed` to the
**leader's** `UnitData::speed` — `find_leader`'s leader, so the lowest
`FormData::type_cat` among the group's captains, not the lowest speed —
and it runs from `Group::add@00714350` (only when the group's `id` is not
−1, which excludes every stack-local group), `Group::kill@00714110`,
`Group::normalize@00711540` and `Group::clear@00713e80` (to zero).

That is not where a marching group's number comes from. Two more
functions write the pair, and **both have zero callers in the
48k-function export because both are inlined at their one call site** —
grep says dead code and the listing says otherwise, which is
`docs/audit/README.md`'s recurring lesson in its purest form:

| written as | inlined at | what it does |
| --- | --- | --- |
| `Group::leader_report_speed@007137f0` | `Unit::do_group_move@005e79a0:95`–`97` | `speed = new_speed; new_speed = get_speed(x, y, 1); march = 0` |
| `Group::report_speed@00713bb0` | `Unit::do_group_move@005e79a0:336`–`338` | `if (UnitData::speed(this) < speed) speed = new_speed = it` |

The leader's runs **after** its own `do_move` returns non-zero, so a
frame the leader spends turning in place publishes nothing. The
follower's runs in the in-formation arm only, after the `dest == here`
early return and under `march == 0 || has_general(0x8000, -1) >= 0`, and
it reports `UnitData::speed` — **layer two**, not `get_speed` — so
neither the ground the follower stands on nor the order it carries enters
the number the rest of the group walks at.

The consequence is a two-phase accumulator, and the lag is observable:
`new_speed` collects this pass's minimum, and the leader's next step
promotes it to `speed` and restarts the accumulator at the leader's own
uncapped speed. A group whose slow members stop reporting therefore
**drifts back up to the leader's speed over two frames**, and drops back
the frame one of them reports again. A model that recomputed
`compute_speed` every frame would be a frame and a half wrong in both
directions.

Two more properties fall out and are worth stating because they read as
bugs:

- **A group whose `speed` is 0 cannot be lowered.** `report_speed`'s test
  is `<`, so until a leader has published twice the cap is nothing
  whatever the followers say. This is why `Group::normalize`'s tail
  matters: it is what a freshly pushed group is primed by.
- **`copy_group@006fa690` copies `speed` and not `new_speed`**, so the
  two can disagree across a `Groups::push_group`. Nothing reads
  `new_speed` but the accumulator itself and `GroupData::log_data`.

### 18.2 The measurement — Great Lakes' raid, six units and one control

Great Lakes' long word stood at **10817** with a block of eight rows
reading as a collision: `1/28` colliding with the gaia animal `8/0` where
the original collides with nothing. It was a consequence. On 10818 this
crate's `1/28` stood at `(17208, 26952)` and the original's at
`(16838, 27194)` — 370 units east, 242 north, about fifteen frames of its
own walk — while the animal sits at `(17278, 26821)` on **both** sides
and never moves in the window. The word's own block could not say so
because `1/28 pos` first parted 583 blocks earlier, on 10235, and a
`firsts` map records only a key's first parting. That row was parked
**477**; it and the word were one cause.

**The step, and why the trig is exonerated in one line.** Both sides
leave `(4776, 30168)` on sim-frame 10234 with the same facing
(`1252851712`), the same `last_speed` 25 and the same `avg_speed` ramp
`6, 10, 13, 16, 18, 19, 20, 21, 22, 22, 22`. This crate steps `(25, 7)`
and the original `(24, 7)`. Run that angle through the original's own
sine table (`docs/MOVEMENT.md`, "The sine table"): a distance of **26**
gives `(25, 7)` and a distance of **25** gives `(24, 7)`, both exactly.
So `sin_component`/`cos_component` agree to the bit and the whole
question is a scalar — and `1/28`'s `myspeed` is 26.

**Solving that scalar frame by frame** over `[10230, 10830]`, from the
dump's own `guy.angle` and position deltas, for all six members of group
65 (`tools/gamelog/track.py`, then the sine table in reverse):

| unit | `myspeed` | solved step | blocks |
| --- | --- | --- | --- |
| `1/27` | 26 | **25** | 491 |
| `1/28` | 26 | **25**, and 26 on exactly five | 487 / 5 |
| `1/29` | 26 | **29** | 456 |
| `1/40` | 25 | 25 | 473 |
| `1/41` | 25 | 25 | 481 |
| `1/42` | 25 | 25 | 470 |

Group 65 is two raider squads, `myspeed` 26 and 25. Two of the fast three
walk at 25, which no other arm of `get_speed` can produce from 26 — the
rest are halvings, a doubling and two eighths. **`1/29` is the control**:
its order stack carries an `ATTACK` throughout (dumped `type 10` from
10241), so `get_action` answers non-null, the cap is skipped, and the same
`local_8` sends it down the `× 9/8` — `26 × 9 / 8 = 29`, to the unit.
One predicate explains why one member of a group ignores the cap **and**
why it walks faster than its own quoted speed.

**The five frames are the lag, and they are the strongest evidence
here.** `1/28` is the group order's own `oxx`, so it is the leader. Its
own report is `get_speed(x, y, 1)` — uncapped, 26 — and the slow squad
stands still through this stretch and reports nothing. The cap therefore
runs 25, 25 (the pass the previous march left), then 26 for five frames
as the leader's own speed works through the accumulator, then back to 25
on sim-frame 10241. With the two reporters in, `crates/sim` reproduces
sim-frames 10234–10240 **exactly** — position, heading and guy clock —
where a `compute_speed`-shaped model walks all seven at 25.

### 18.3 What it moved, and the residue

| | before | after |
| --- | --- | --- |
| Great Lakes long word, sequence / count | 10817 | **10834** |
| block 10818, keys parted | 8 | **0** |
| block 10235, keys parted (parked 477) | 3 | **0** |
| `1/28`'s position error at the word | (370, −242) | **(1, 0)** at its first parting |
| run100 widening over the word's window | 324 | **285** (over 1,509 blocks against 1,492) |
| Great Lakes endpoint `off` / `unlinked` / `build_diverged` | 53 / 3 / 9 | **42** / **5** / **10** |
| East Indies endpoint `off` / `build_unlinked` / `build_diverged` | 64 / 2 / 27 | **63** / **1** / **30** |
| East Indies ladder B `off` / `extra` | 48 / 14 | **49** / **9** |
| East Indies ladder C `extra` | 14 | **13** |

Eleven of Great Lakes' roster are back on the original's point at frame
24,001, 13,167 frames past the word — the largest single fall that
counter has taken. East Indies moves too and its own word does not (9711
either side): every map marches formations, so a member that now walks at
its group's pace arrives elsewhere everywhere. `docs/DECISIONS.md` 36
asks for the number rather than a trade.

~~**The residue is one frame of the cap and it is not the cap's.** `1/28`'s
position now first parts on block **10242**, one world unit, on the frame
the original's cap returns to 25 and this crate's stays at the 26 its own
leader reported. In the original a member with a 25 reports on that
frame; here none can, because `1/40` holds an `ATTACK` at the head of its
order stack where the original holds the `GROUP_MOVE` (dumped `type 19`
through 10245), so it never enters `do_group_move` and never reports at
all. That is an order-stack divergence of the raid, not of §18, and it is
what the next item on this frame should name.~~ **The frame was right and
the reading was not** (item 518, §19): the original's `1/40` holds its
`ATTACK` at the head too — the log writes the list newest first — and no
member reports on either side. The cap is §18's after all, and its third
writer is `Groups::process@006fa210`, which this section did not count.

### 18.4 Coverage

**Diff-backed**: every row of §18.3, from
`run100_s_word_block_is_every_record_the_dump_carries` over
`[9340, 10847]` (1,509 blocks, 4,160,848 record rows),
`run53_s_24000_frames_put_the_ceiling_where_run33_did`,
`great_lakes_endpoint_is_pinned`, `east_indies_endpoint_is_pinned` and
`the_east_indies_ladder_is_pinned`. All were red before the change and
are the numbers above after it.

**Dump-backed**: §18.2's table whole — the six solved step distances, the
`myspeed` column, `1/29`'s 29 and `1/28`'s five frames — read out of
run100 with `tools/gamelog/track.py` and the simulation's own sine table
run backwards. This is the first evidence for the group cap that is not a
reading, and it is also the first for the *lag*: the five frames are what
separates the accumulator from a per-frame recompute, and nothing but a
per-frame solve could have shown them.

**Listing-backed**: §18.1's two inlined writers. Neither
`Group::report_speed` nor `Group::leader_report_speed` has a caller
anywhere in the export; both were found by reading `do_group_move`'s own
body against the standalone functions' addresses. `copy_group`'s
asymmetry and `Group::add`'s `id != -1` gate are read the same way.

**Reading-only, and owed a check**: `march` (`+0x4b`). The leader's
report clears it and the arm below it —
`LeaderData & 0x8000 && has_general(0x8000, -1) >= 0` — sets it; no
capture has a general, so this crate writes only the clear and the
follower's gate on it is always open. `GroupData::log_data` does not
print it, so no dump can settle it either: it needs a capture with a
forced march.

**Not established**: which of `Group::add`, `Group::kill` or
`Group::normalize` primes a given group in the original, and when.
(Item 518: whichever does, `Groups::process` re-primes every group once
every 64 frames on its slot's own frame — §19 — so an unprimed group runs
uncapped for at most that long.)
`crates/sim` models the `normalize` tail alone — at
`Sim::group_normalize` and at the two sites that open with it,
`Group::kill_group_move` and `Group::refresh_group_order` — and the
`Form::categorize → Group::sort → kill/normalize/add` chain is not
modelled at all. On Great Lakes the probe's own closing normalize
(`Army::find_target`) is what primes group 65, and the first seven frames
of its march are exact; on a group primed by some other chain the first
two frames could run uncapped.

## 19. The pool resets one slot a frame — `Groups::process`, the cap's third writer (item 518, 2026-09-22)

§18 counted two writers of `GroupData::speed`: the membership sites that
recompute it from the leader (`compute_speed` and the `normalize` tail),
and the two reporters inlined in `do_group_move`. There is a third, and it
is the one that runs whatever the group is doing: `Groups::process@006fa210`,
the last act of `GameDaemon::process_all@00732700` — so after
`calc_markets` and before `Armies::process_all` and the unit loop.

Once a frame, for each in-use player, it takes **one** pool slot —
`player·64 + proc_group` — runs `normalize`'s prune and `find_role`, and
then writes `speed = new_speed = UnitData::speed(find_leader)`, or 0 for a
slot with no leader (§3.3 had this; §18 did not use it). `proc_group`
starts at 0 (`Groups::Groups`, `Groups::clear@00713f20`) and steps once a
call, wrapping at 64, and the function runs unconditionally every frame,
so on frame `f` the slot is `f mod 64`. **Every group's cap goes back to
its leader's own speed once every 64 frames**, on a frame fixed by its
slot number and by nothing it does.

### 19.1 What was booked, and what killed it

515 left `1/28`'s position one world unit out on block **10242** and booked
it as an order-stack item: `1/40` "holds an `ATTACK` at the head where the
original holds the `GROUP_MOVE` it is dumped with". **The dump says
otherwise, and the widening had already said so.** `OrderList::log_data`
writes newest first and the last block is the current order
(`docs/ORDERS.md` §11.1); `1/40`, `1/41` and `1/42` print `type 19` then
`type 10` on every block 10236..10249 — an `ATTACK` at the head in the
original too — and `compare_orders` parts nothing on them. Both sides were
printed once on 10242 before the quiet row was trusted.

So no member of group 65 can report on either side after the ungroup:
`1/27`'s blocked step ungroups the fast squad on frame 10240 in both
simulations (`Unit::move_step+0x823`, the blocked stand, is the trace's own
draw on that frame), and the slow squad is fighting. Yet the original's
`1/28` steps **26** on blocks 10237..10241 and **25** from 10242 — solved
from the dump's own position deltas — while this crate's cap read
`(26, 26)` on every block 10236..10246. Something that is not
`do_group_move` writes the cap between `1/28`'s step on frame 10240 and its
step on frame 10241.

Every writer of `+0x40` in the export was a reading, and five were named
before the run, each with what would kill it (`tools/gamelog/captures.txt`,
run120): R1 a push by player 1 (`push_group` → `equals_group` normalizes the
player's last slot), R2 a member's target search (`find_nearby_target:499`),
R3 `repath` → `kill_group_move`, R4 a membership change, R5 none of these.
`near_o` could not decide R2: it is written only on a nearer candidate, so
its silence dates nothing (`docs/COMBAT.md` §44.2.1).

**run120** is run100's game again with per-frame function coverage over
frames 10236..10244 and nothing raised: 11 blocks, 0 differing from run100,
10,251 frames of draw stream identical to run53. On frames 10240 and 10241
**none** of `Group::normalize`, `Groups::push_group`, `Group::equals_group`,
`Group::kill`, `Group::add` or `Group::kill_group_move` is entered —
R1, R3 and R4 dead — and `Groups::process`, `Group::find_role` and
`GroupData::count` are entered on **every** frame of the window.
`GroupData::find_leader` is entered on 10238–10241 and 10243: the frames
whose slot holds a live group. 10241 mod 64 is 1, and player 1's slot 1 is
`group 65`.

It also accounts for the rest of §18.2's schedule: the raid's first two
frames at 25 and then five at 26 are the leader's reports walking the cap
up from the value the probe's normalize left, and the drop on 10241 is the
slot's reset, with no member able to report it back down or up.

### 19.2 The numbering, which the cursor makes load-bearing

A reset on the wrong frame is as wrong as none, so the slot a group sits in
now matters, and until this item nothing here numbered one. `crates/sim`
now carries it as `GroupState::pool`, for an army's group and a pushed one
alike, and allocates it as `Groups::push_group@0070f9e0` and
`Groups::get_open_slot@006fa460` do (§3.1, §3.2):

- a group **equal** to the seat `last_group[who]` names — same members,
  same order — reuses that slot;
- otherwise the lowest of `0..46` whose seat is empty and is not
  `last_group[who]`;
- `last_group` starts at each player's slot 0 (`Groups::clear` writes
  `who·64`), so a player's first push lands in slot 1;
- an army takes a slot the way `Army::add_unit@006f9f40` does, by pushing
  its first squad when it has no live group.

"Empty" is what `get_num` answers after the `normalize` it opens with: a
unit that has joined an army since it was pushed points elsewhere —
`Unit::set_group@00605220` writes `+0x80` and leaves the old list alone —
and the prune drops it.

**The dump checks it.** `UNITDATA` prints `group` on every unit of every
block and nothing compared it; it is a row of
`run100_s_word_block_is_every_record_the_dump_carries` now, and over all
1,509 blocks it parts on **no** unit: army 64, raid 65, scout 67, everyone
else −1. The run up to the window agrees in shape too: run97 has the raid
entering 65 on block 8187 and the scout moving 66 → 67 on 8529, which is
what the allocator gives from this crate's own push sequence.

### 19.3 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word | 10834 | **11185** |
| run100 widening over `[9340, 10847]`, keys parted | 285 | **245** |
| blocks 10242, 10243, 10835 | 3, 9 and 6 rows | **empty**, pinned |
| run100 standing position residue | six units | **four** — `1/27`, `1/28` leave |
| Great Lakes endpoint `off` / `unlinked` | 42 / 5 | **57** / **4** |
| East Indies endpoint `off` | 63 | **62** |
| East Indies ladder C `extra`; B `off` / `extra` | 13; 49 / 9 | **15**; **47** / **15** |

The widening's fall is like for like: the same window, 4,462,984 rows
against 4,369,314 — the new `group` row is the difference — and 40 keys
fewer. The endpoint rows are 12,816 frames past the new word and every
army on both maps now has its cap reset on its slot's frame, so they are
evidence about the run-up and not about the reset; `docs/DECISIONS.md` 36
asks for the numbers.

**The new word, 11185, is a market frame**: ours nine draws against eight,
parting at index 2, `Leader::use_market+0x1ed` against
`Leader::make_stuff+0x221`. No dump on disk reaches it — run100 ends on
block 10899 — so its widening is owed.

### 19.4 Coverage

**Trace-backed**: that `Groups::process` runs on every frame and that no
other writer of the cap runs on 10240 or 10241 (run120's per-frame sets).
**Diff-backed**: the numbering (the `group` row, 1,509 blocks), and the
reset's frame, through `1/28`'s and `1/27`'s positions closing on 10242
and 10243. **Guards**, each made to fail on purpose:
`group::groups_process_resets_one_pool_slot_a_frame_on_the_frame_mod_64`
(cursor one frame late) and
`group::a_push_takes_the_lowest_empty_slot_that_is_not_the_last` (no
`last_group` exclusion; no equality reuse).

**Reading-only**: `equals_group`'s own `normalize` of the last slot on
every push (§3.2) — R1's mechanism, real in the listing, not modelled, and
reached on no frame of run120's window; the prune's `priority` arm, which
only a hotkey group sets.

**Not established**:

- The early-game numbering. run69 and run71 have the scout moving
  65 → 64 → 66 → 65 on pushes that are equal to the last slot by this
  reading, so something else moves `last_group` in between; and the
  second squad's `go_to` on 6994 is 66 in run79 and 65 here. A one-unit
  or one-type group's reset cannot change its cap, and both are back in
  the army's 64 a block later, so nothing measured depends on either;
  the window's own numbering is exact.
- `get_open_slot`'s two fallbacks (46 live groups at once; no capture
  comes close).
- ~~An army's group is pruned here by `Army::normalize`, not by the
  pass.~~ Both prune it, by the back-pointer, since item 557 (§23.4).

**SEAM**: `find_role` is not recomputed, because nothing here reads
`GroupData::role`.

## 20. Great Lakes 11531 — a bowman arrives two frames early, on a lag its group march set (item 533, 2026-09-22)

Item 327 moved Great Lakes' long word to **11531**, 72 blocks past
run123's last. It was booked by its frame and its draw delta only. Ours
spends four draws and the original three, parting at index 1: ours
`Guy::set_anim+0x97a < Unit::do_idle+0x7d` against the original's
`< Guy::inc_time+0x271`. The draw is an idle roll, but the cause is
here, in a group's march. **No mechanism is fixed by this section.** It
is the capture and the widening, and what they say.

### 20.1 This crate's side, before the capture

`RON_DEBUG_SITES=11526-11534` and `RON_DEBUG_UNIT` on the run53 test.
The two idle rolls on 11531 belong to army 1's bowmen **`1/34`** and
**`1/36`** (type 127). Each is at the end of the squad `1/31..1/39`'s
march to a `GROUP_ATTACK_TO` point. `1/36` stops at (39912, 20232) on
11529 and rolls on 11530 and 11531. `1/34` stops at (39768, 20328) on
11530 and rolls on 11531 and 11532. The original rolls once on 11530
(agreeing), once on 11531, and once each on 11533 and 11534. The stanza
wrote four readings before the run (`tools/gamelog/captures.txt`,
run125): the arrival, the order, the clock, and another unit.

### 20.2 run125, and the widening

run125 is run123's game at run123's detail over `[11440, 11600)`
(`docs/RUNS.md`). `run125_s_word_frame_is_widened_whole` walks run123
from **11250** and run125 from 11440 to 11599. That is 350 blocks, every
record `widen_block` reads on every unit, and the leader record whole for
both players.

**The word's two blocks part on one unit, `1/34`**, and nothing else:

| block | row | ours | theirs |
| --- | --- | --- | --- |
| 11531 | `orders.len` | 0 | 1 |
| 11532 | `idle` | 1 | 0 |
| 11532 | `g.cur_time[0]` / `last_time` | 1 / 0 | 7 / 6 |

**R1, the arrival, is the frame.** The original's `1/34` reaches
(39768, 20328) on block **11533**, and this crate's on **11531**. `1/36`
arrives on 11530 on both sides. So this crate's `1/34` starts its idle
two frames early, and its first roll is the word. R2 is dead: both hold
the one `ATTACKTOORDER` until arrival. R3 is dead: the clocks agree until
the arrival. R4 is dead: the only row is `1/34`'s.

### 20.3 Backwards: the lag is the march's

`RON_SQUAD_WALK=34` prints `1/34`'s gap, theirs minus ours, wherever it
changes.

- **11250–11363: zero.** The squad agrees on every record until its group
  order (11259). The two sides number the order's `group.id` differently
  (11258117 against 11264430). The first row that is not an id is `1/31`'s
  `half_step` on **11305**. This crate's probe of sim-frame 11304 hits
  (911, 465) in `1/32`'s block and goes soft. The original takes the same
  step with no flag (`sweep_verdict` on this side; no probe capture exists
  for the other side).
- **11364–11417: parts and closes.** `1/34`'s gap opens to 10–20 units and
  **closes to zero on 11385–11411 and on 11417**.
- **11446–11462: the lag that holds.** Both sides push a formation hop
  onto the path stack and turn in place on it, **a frame apart**. Ours
  pushes (41318, 20333) on 11457 and stands 11456→11457. The original
  pushes (41315, 20335) on 11458 and stands 11457→11458, with no
  collision recorded (`collide_frame` 7481, stale). From 11462 the gap is
  **22/35 on x, alternating** with the half steps, all the way to
  (37, 0) on 11528 and the arrival.

**A payoff probe on 11304 alone does not move the word.** Suppressing
this crate's one soft flag clears `1/31`'s rows. The squad then re-parts
on 11357 (`1/35`'s flag) and 11361–11364, and the word stays on 11531.
The 11304 flag is one of several soft-collision partings under the march,
and it is not what sets the lag the word arrives on.

**A second residue, which moves no position**: the squad's
`GROUP_ATTACK_TO` (21) becomes a plain `ATTACK_TO` (2) **a frame early**
here. The first trio switches on 11512 against the original's 11513, and
the second on 11524–11525 the same way. Every such unit's `flags` read 5
against the original's 4 from the next block.

### 20.4 What this has *not* established

- ~~**Why the formation hop comes a frame early here on 11457.** Candidates
  are the group's speed cap (§18, §19: slot `f mod 64`), the hop's
  distance test, and the leader's own position, which the gap walk has
  not read for `1/31`. The widening names the frame and not the writer.~~
  **§21: none of the three.** The hop is the leader's waypoint turn a
  frame early, and the lag under it is a soft-collision probe's stride
  (`docs/COLLISION.md` §4.2).
- ~~**Which of the march's soft-collision partings matter.** 11304 does
  not, measured. 11357 and 11361–11364 are unprobed.~~ **§21: all of
  them, as one cause.** 11304 and 11357 are the two clean ones, and the
  rest is their cascade.
- **The original's sweep on 11304.** It needs a `RON_COLLIDE_PROBE`
  capture (`docs/COLLISION.md` §9), which run125 is not.
- ~~**The group order's early dissolution** (20.3's last paragraph). Its
  writer is unread.~~ **§21: a consequence.** It parts on no block once
  the squad walks the original's points.

### 20.5 Coverage

Diff-backed, in `run125_s_word_frame_is_widened_whole`:

- the arrival blocks of `1/34` and `1/36`, both sides;
- `1/34`'s gap on 11456, 11457, 11458 and 11528;
- the five rows that first part on the word's two blocks;
- the squad's first non-id parting, `1/31 half_step` on 11305. This was
  made to fail on purpose by suppressing the 11304 flag, and it fails
  while the word's pins hold;
- the first trio's `order:kind` on 11512.

`the_widening_behind_each_pinned_word_exists` names this test for
`LONG_WORD_GREAT_LAKES`. The coverage pin reads run125's 11529–11533, and
no key moved: run125 prints what run123 printed.

## 21. The hop was a probe's stride — Great Lakes 11531 → 11582 (item 539, 2026-09-22)

§20 left the word on `1/34`'s arrival two frames early, and it named
three candidates for the formation hop a frame early on 11457: the cap's
reset, the hop's distance test and the leader's position. **The frame
was right again, and none of the three was the mechanism.** The cause is
in `docs/COLLISION.md` §4.2, and this section is the chain that leads
there.

### 21.1 The hop, and what fires it

The hop is `do_group_move`'s follower arm when the slot lies more than
120° off the bearing to the goal: it pushes the leader's next waypoint
plus the member's offset (`005e82a7`–`005e8320`, §6.6). `RON_SQUAD_PATH`
and a per-member dump of `dest_x/dest_y` and `in_group`, both sides,
over 11440–11462, give the inputs:

- `1/34`'s slot is `1/37`'s position plus its rotated offset, and
  `1/37` is the group order's `oxx`, the leader.
- The original's `1/37` reaches its waypoint `(41160, 20520)` and turns
  toward `(39628, 20529)` on block **11457**. This crate's did the same
  on **11456**. The leader's heading swings about 9°, the slot swings
  behind the follower, and the follower's hop follows one frame later on
  each side.
- The leader was **22 units ahead** from block 11450. The original's
  `1/37` alternates 25- and 12-unit steps, and this crate's took 25 on
  blocks 11448 and 11450 where the original took 12. Its `unit_masks`
  carries `0x100000`, the soft-collision half step (`docs/MOVEMENT.md`),
  on blocks 11447 and 11449, and this crate's did not.

### 21.2 The soft flags, classified

A sweep watch on the leader on sim-frame 11446 comes back **clear**. The
original's leader goes soft there, and the only block within reach is
`1/33`'s: the original's `1/33` stands at x 41520, unit cell 865, whose
block covers 866, the leader's leading edge. This crate's stands at x
41517, unit cell 864. That is three units, and it comes from `1/33`'s
own soft flag the frame before. So it is a cascade, and a cascade is
answered by its **clean** partings: a soft flag that parts while every
squad member's unit cell agrees.

`RON_SOFT_AUDIT=1` on the run125 widening prints every block where a
squad member's flag or unit cell parts. Before the change it printed
**193** blocks over `[11250, 11599]`. The clean ones are two, 11305
(`1/31`) and 11357 (`1/35`), both with this crate soft and the original
not, and both with every position in the squad equal to the unit. From
11361 on, every flag parting sits beside a unit-cell parting.

### 21.3 The clause

`RON_SWEEP=<frame>:<who>/<o>` records this crate's sweep. Both clean
partings are the fast path's leading edge, stepping west, finding a
group-mate's block **two cells** along an edge whose first cell lies in
a world cell that holds no bit. The listing does not advance past an
empty slot, so the original's second cell is **one** cell along, and it
is empty (`docs/COLLISION.md` §4.2 has the loop). A payoff probe had
already shown 11304 alone could not move the word (§20.3): its fix
re-parted on 11357, which is the same clause.

### 21.4 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word | 11531 | **11582** |
| run125 widening `[11250, 11599]`, keys parted | 772 | **403** |
| squad blocks with a soft-flag or unit-cell parting | 193 | **0** |
| squad positions parting, any block of the window | `1/34` from 11364, and the rest | **none** |
| `1/34` arrives, theirs / ours | 11533 / 11531 | **11533 / 11533** |
| run100 standing position residue | four units | **three**: `1/35` leaves |
| run97 walk-slot band below the word | two rows, `1/35` | **none** |
| run97 order residue, set / rows | three units / 28,222 | **two** / **28,220**: `1/33` leaves |
| Great Lakes endpoint `off` / `unlinked` / `extra` / `build_diverged` | 50 / 1 / 0 / 9 | **46** / **0** / **3** / **10** |

The new word, **11582**, is a building placement. Ours spends 948 draws
against 9 on that frame, parting at index 5: ours
`Leader::produce_building+0xc99` against the original's
`Guy::set_anim+0x104b`. On block 11583 this crate holds a building the
original does not (`1/2022`) and sends citizen `1/9` on a two-order walk
to it. The original's `1/9` stands idle, its leader books a free
peasant (`free_peasants` 1 against 0), and `1/2016` has a citizen queued
(`queue:queued`, `num_queued[83]`). Two blocks earlier, on 11580, the
leader's `MAKE` slots 2 and 3 stand in the opposite order: `cat` 4 and 7
here against 7 and 4, `t` 560/66 against 66/560. The block is inside
run125, so its widening is on file.

### 21.5 Coverage

**Diff-backed**, in `run125_s_word_frame_is_widened_whole`:

- both bowmen's arrivals;
- no squad position parting on any of the 350 blocks;
- the old word's blocks 11531–11532 pinned empty;
- the 38 rows that first part on 11580–11583.

It was made to fail by restoring the fixed stride: the arrivals go back
to 11531. **Guard**:
`collide::tests::the_leading_edge_does_not_step_past_an_empty_world_cell`.
**Not established**: the probe-side region gate (`docs/COLLISION.md`
§4.2), which no Great Lakes frame can test because the map is one
region. The wider endpoint and East Indies effects are in the landing's
journal, measured by the gate.

## 22. Great Lakes 11757 — a detour around a unit the army left behind (item 554, 2026-09-22)

Item 545 moved Great Lakes' long word to **11757**, past run125's last
block. Ours spends **7** draws against the original's **8**, parting at
index **0**: the original's first draw is
`Guy::set_anim+0x97a < Unit::move_step+0x823`. Ours spends that draw
on **11758**. run130 is the capture (`docs/RUNS.md`), and
`run130_s_word_frame_is_widened_whole` is the widening. **The word did
not move.**

### 22.1 This crate's side, before the capture

`RON_DEBUG_SITES=11750-11760` on the run53 test names the unit: ours
spends the `move_step` draw on 11758 from **`1/62`**, a type-82 unit of
army 2 on an `AttackTo` to (36552, 23448). It stands on 11758 and
re-paths on 11759. run125's own widening, read backwards, already shows
army 2 parting inside that capture, and run130's stanza wrote four
readings, each with what would kill it, before the run. R1 is the same
unit a frame early. R2 is another unit. R3 is the march's lag. R4 is
the group.

### 22.2 The chain, from the word backwards

Every step below is a row of the widening, both sides printed:

1. **11758, the word's block.** The original's `1/62` stops against
   `1/23` (`collide_o 23`, `coll` (41378, 22078)) and goes to its stand
   (`cur_anim` 8 → 0, `t1/33`) at (41398, 22093). This crate's is still
   walking at (41458, 22138) and stops a frame later. **R1 holds and R2
   is dead**: `1/62` is the only player-1 figure that changes animation
   while moving on the block.
2. **11702.** `1/62`'s position first parts, 17/8 off, on the first
   frame it walks the stretch where its path differs.
3. **11689, frame 11688.** Both sides collide at (42272, 23027) and plan
   a local detour (path flag 2) to the same waypoint (41928, 22680).
   The routes differ in the middle. The original's goes (42264, 22872) →
   (42264, 22632) → (42216, 22584). This crate's goes (42264, 22872) →
   (42312, 22824) → (42312, 22632) → (42264, 22584), one cell east and
   one slot longer. **The only unit within 480 of `1/62` that stands
   elsewhere is `1/64`**, army 2's own: (42117, 22691) in the original and
   (42138, 22711) here, one unit cell over on y. `1/11` and `1/63` agree
   exactly. **R3 holds.**
4. **11514.** `1/64`'s lag opens, with a step of 12 here against 25 in
   the original (`last_speed`), 5/12 behind. It widens to 21–27 units
   and holds there until the detour.
5. **11513, frame 11512.** Army 2, mustering (status 17), re-issues its
   group attack-to: `1/60` takes the `GROUPATTACKTOORDER`, and every
   member's destination agrees on both sides. **Its membership does
   not.** The original's stance 1 reaches `1/60`, `1/61`, `1/64` and
   `1/65` and leaves `1/62` and `1/63` at 0, and its `1/64` is left in
   **no pool group** (`group -1`). This crate's reaches all six and keeps
   them in 66. **R4, as the stanza framed it, is dead**: `1/62`'s own
   pool group reads 66 on both sides on the word's blocks. The membership
   parting is upstream, and it is on `1/64`.
6. **11424.** The pool pointers had parted once before: the original's
   `1/62`–`1/64` arrive in pool group 69 (their `come_out` group,
   `docs/ARMY.md` §3.2), and this crate's in none. They agree again from
   11425.

### 22.3 The payoff probe, as an assertion

`run130_s_word_is_1_64_s_lag` changes one thing. Before frame 11688 it
seats this crate's `1/64` on the original's point from block 11688
(`Sim::probe_relocate`, through `set_new_location`), and nothing else.
`1/62`'s detour then matches the original's cell for cell. Its march
agrees to the word, and on 11758 it stands at (41398, 22093) with the
original's clock. `1/34`'s idle pick on the same block, 3 → 0 here
against 3 → 2, comes right with it, because it is the draw stream a
frame on. So **the word is `1/64`'s lag, and the detour search is
right**: given the original's grid, it returns the original's path.

### 22.4 What this has *not* established

- ~~**Why the original leaves `1/64` out of the pool on 11512, and why its
  stance misses `1/62` and `1/63`.**~~ **§23**: the stance walks group
  66's list, which on 11512 is `{1/60, 1/61, 1/64, 1/65}` — `Group::add`
  normalized the squad's head and middle out of it on 11424 — and the
  categorize of the group order then runs `Group::sort`, whose `kill`
  clears `1/64`'s pointer. The reader list above was right to name §3.1,
  §3.2 and §4.2; the frame that decided it was 11424's `Group::add`,
  which none of them is.
- ~~**Hypothesis, not a measurement**: a unit outside the pool walks at
  its own speed, 25, where a member walks under the group cap (§18).~~
  **§23: not the cap.** `1/64`'s pointer is −1, so `do_group_move`'s
  first line ungroups its order on 11513 and it walks a plain move. The
  step of 12 was this crate's in-formation follower.
- ~~**The cheapest next check**: a capture of this game with `cover=1`
  over 11420–11426 and 11508–11514.~~ **Taken as run134**, with
  `GROUPS=1` beside it; §23.

### 22.5 Coverage

**Diff-backed**, in `run130_s_word_frame_is_widened_whole`, over 400
blocks [11400, 11799]:

- the word's eight rows on 11758;
- `1/62`'s path first parting on 11689, and its position on 11702;
- `1/64`'s positions on 11689;
- `1/64`'s first step and position parting on 11514;
- army 2's pool pointer and stance on 11513, all six both sides;
- the first pool parting on 11424.

The counterfactual is `run130_s_word_is_1_64_s_lag`. The coverage pin
reads run130 around 11758, and no key moved. **Reading-only**: nothing
in this section. ~~**Open**: the whole of §22.4.~~ **Closed by §23**,
and every row above is now pinned agreeing, as that move's value diff.

## 23. The back-pointer is its own state — Great Lakes 11757 → 11806 (item 557, 2026-09-22)

§22 left the word on `1/64`'s lag behind army 2's group order of frame
11512. The original's `1/64` goes `group -1`, and its stance misses
`1/62` and `1/63`, and no mechanism was named. **The frame was right,
and the cause was two frames**: 11424, where the squad joins the army,
and 11512, where the army orders it. Neither alone does anything
visible.

### 23.1 The readings, and run134

The stanza (`tools/gamelog/captures.txt`, run134) wrote four readings
before the run, each with what would kill it. R1 was a one-unit push:
`push_group(force 0)` writes `+0x80 = -1` (§3.2). R2 was the pool's
per-frame reset (§19). R3 was an eviction or a `kill` (§3.1, §4.2). R4
was the army's own filter (`add_unit`, `set_group`).

**run134** is run125's game, with `GROUPS=1` and per-frame coverage over
11420–11514 (`docs/RUNS.md`). It is the first dump on this map above
run92's 7689 to print the pool's lists. `DEATHS` is off because of
run92's trap: `dump_deaths` leaves the log's type at `WORLD`, and the
pool would be dropped. It took eleven minutes, 230 MB of dump and 18.9
MB of trace, and all six checks passed. Its 80 blocks shared with
run125 differ on nothing outside the two changed records.

On sim-frame 11512, **no** `push_group`, `get_open_slot`, `set_group`
or `Army::add_unit` runs. R1, R3's eviction and R4 are dead. R2 is
dead by arithmetic: 11512 mod 64 is 56, and group 66 is slot 2. What
does run is `Army::set_stance` → `Group::action_stance`, then
`action_move_to` → `compute_form` → `Form::categorize` →
**`Group::sort`**, `Group::kill`, `normalize` and `add`. That is R3's
`kill`, reached from a function no reading had named.

### 23.2 What the pool says

Group 66's list, from `GROUPDATA id 66`, block by block:

| blocks | list |
| --- | --- |
| 11416–11424 | `60, 61` |
| 11425–11507 | `60, 61, 64` |
| 11508–11512 | `60, 61, 64, 65` |
| 11513– | `60, 61, 65, 62, 63, 64` |

`1/62`–`1/64` are one squad (`o_up`: 64 → 63 → 62). From 11425 all
three point at 66, but only the tail is listed. `1/62` and `1/63` are
**named and not listed**, so `Object::get_army` answers −1 for them.
`1/64` is listed and named until 11512, then listed and naming nothing.

### 23.3 The mechanism

**11424, `Army::add_unit` → `Group::add(64, who, 0, 0)`.** Every step
of `add` opens with `get_num` (vslot `+0x4`) unless `const` is set. For
a group of **fewer than four** with an `id`, `get_num` is `normalize`
whole, and `normalize` drops a member whose `+0x80` does not name the
group (§4.3). The squad arrives from its `come_out` group 69 (run134's
coverage on 11423: `Unit::come_out`, `push_group`, `get_open_slot`,
`copy_group`), so all three pointers name 69:

1. `add(64)`, then `add(63)`, then `add(62)`: the redirect to the
   captain. `62` is appended, and the list is `60, 61, 62`.
2. `add(63, keep 1)`: `get_num` sees three members and normalizes. `62`
   names 69, so it is dropped. `63` is appended.
3. `add(64, keep 1)`: the same, `63` is dropped and `64` is appended.

Then `Unit::set_group@00605220` climbs to the captain and writes 66
down the chain, on all three. §4.1's reading of `add` was right about
the walk and silent on the `get_num`; the `num < 4` gate is
`Group::get_num@00714700`'s first arm.

**11512, the group order.** `action_stance` walks the list, so
`60, 61, 64, 65` take stance 1. `62` and `63` are not listed and keep
0. Then `action_move_to`'s layout calls `Form::categorize`, whose first
statement is `Group::sort@00708090`. It walks the list keeping the last
captain seen. `64` is a follower whose top captain, `62`, is not `61`,
so it:

- `kill`s `64` with `keep 0`. That redirects to `62`, whose chain is
  killed with `keep 1`. `64` is found in the list and removed, and **its
  `+0x80` names 66, so it is cleared**. `63` and `62` are not listed, so
  nothing happens to them;
- `normalize`s the group;
- `add`s `64` with `const 1`. There is no `get_num` this time, so the
  whole squad is appended, and `add` writes no pointer.

On the same frame `1/64`'s own turn reaches `do_group_move`, whose
first line is `if (+0x80 == -1) ungroup_move_order`. So it walks a
plain move at its own speed. `1/65` keeps its group order and its cap.
§22.4's step of 12 was this crate's in-formation follower, not the
cap. `do_group_move`'s follower arm also clears `+0x80` for a member
its group's list does not hold (`5e7ed8`) before ungrouping.

### 23.4 This crate's side

`sim::Unit::group_ptr` carries `+0x80` as a pool slot, or `None`.
**Writers**: `Sim::set_group` (`Army::add_unit`'s tail);
`push_group`, both arms (installed: the slot; `force 0` with one
member: `None`, which no production caller takes); `get_open_slot`'s
eviction; `Sim::seat_kill`; and the follower arm's not-listed clear.
**Readers**: `Sim::seat_of`, and through it `group_of`,
`group_speed_of` (the cap) and `pool_group_of` (the dump's row);
`army_of`, which is `Object::get_army` (the named group must list the
unit); `pool_members`; `same_group_soft`; and the follower's "in my
group" test (`5e7c8c`, the two pointers equal).

A seated group's list follows the original's own operations:
`seat_add` (`get_num`'s normalize per step), `seat_kill`,
`seat_normalize` and `seat_sort`. `army_add_unit` goes through
`seat_add` for an army with a live group. An army with none still
builds a stack group, which is never normalized, and pushes it.
`Group::sort` runs where `Form::categorize` does, in
`group_action_move_to`'s layout, after the `QUEUE_NEW` clear and before
the order loop. So a member the sort brought in is decided by the
order loop. `army_normalize` and `groups_process` prune by the
pointer.

**SEAM**: five writers of `+0x80 = -1` are not modelled: `do_build`,
`build_done`, `do_attack` (which saves and restores it),
`do_non_flat_gather` and `think_peasant`. They reach builders and
gatherers, whose pointer names a pushed slot or nothing. A slot freed
by one of them sooner in the original could number the next push
differently (§19.4's early-game residue). Nor is `come_out`'s own push
(the 11424 residue below): a pointer naming **any** other group at
11424's add leaves the same list.

### 23.5 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word | 11757 | **11806** |
| run130 widening `[11400, 11799]`, keys parted | 655 | **284** |
| run130 widening, block 11758 | 8 rows | **empty**, pinned |
| `1/62`'s detour (11689) and position (11702) | parted | **agree** |
| `1/64`'s position and step from 11514 | parted | **agree** |
| army 2's pointer and stance on 11513 | 3 of 6 parted | **agree** |
| run134, group 66's list on 104 blocks | not compared | **agrees on all** |

The widening's fall is like for like: the same 400 blocks and
1,353,421 record rows either side. What stands is 11400's 236 keys
(standing residue at the floor), 11424's `come_out` push, 11507's
`1/65` (the same shape), (558)'s ids on 11513 and 11769, and blocks past
11576 that the chain never reached. The endpoint rows move too, 12,195
frames past the new word (the gate's numbers are in the journal):
every army's list now follows `add` and `sort` on every map.

**The new word, 11806**: ours 7 draws against the original's 6,
parting at index 1. Ours spends
`Guy::set_anim+0x97a < Unit::move_step+0x823`, and the original
spends `Unit::resolve_unit_collision+0xb52`. It is past run130's last
block (11799), so its widening is owed a capture.

### 23.6 What this has *not* established

- **`Groups::process`'s prune of `1/64`** on slot 2's frame, 11522,
  and `Army::normalize`'s on the army's next tick. Both are readings
  here; run134 ends on 11519.
- **The original's `come_out` push** on 11423 (group 69). It is
  modelled nowhere, and it is the one residue of run134's pool: three
  pointer rows on block 11424.
- **(558)**: the group order's `id` still carries the army slot (2)
  where the original's carries the pool index (66). It parts on 11513
  and 11769 (`order:group.id`). The pool slot is carried now, so the fix
  is a one-line change to `group_id`, but the change is not this item's.

### 23.7 Coverage

**Diff-backed**: §23.2's table whole and every pointer of army 2 on 104
blocks (`run134_s_pool_list_is_the_original_s`, which was made to fail
twice on purpose: without the per-step `get_num`, 11425's list is the
whole squad, and without the sort, 11513's is the old order). Also
§22's chain closed, pinned in `run130_s_word_frame_is_widened_whole`.
`run130_s_word_is_1_64_s_lag` now asserts that `1/64` is on the
original's point before the detour, unprobed. **Trace-backed**: §23.1's
function sets (run134). **Guards**:
`group::a_squad_joining_a_small_seated_group_is_listed_by_its_tail_alone`
and `group::the_sort_re_seats_a_stray_follower_s_squad_and_clears_its_pointer`,
each made to fail by its mutation. **Reading-only**: the five SEAM
writers, and `do_group_move`'s not-listed clear (`5e7ed8`), which no
frame of this chain reaches.

## 24. The build behind the goody walk — chapter seven-b's control, 1036 → 1148 (item 632, 2026-09-23)

run157 is chapter seven-b's control (`docs/GOLDEN.md` §11). Its word was
**1036**: who=1's citizen `1/1`, ours 8 draws against 7, parting at draw 2.
Ours took an idle roll where the original's walked. The widening on file,
`chapter_seven_b_s_word_frame_is_widened_whole`, walked the capture whole
and put the first parting on **990**, and that is where this section
starts.

### 24.1 What the dump holds on 990

`1/1` is walking an explore-to (37128, 23160) toward a site, with a
`BUILDORDER` on `1/2007` under it (`flags 4`, the action bit). On 990, a
multiple of fifteen, the goody look runs from `do_explore_to`
(`docs/GOODY.md` §7). `Unit::get_goody_box@005f7690` puts the unit in a
one-member group and asks for the box's walk at `QUEUE_FIRST`. The
original's record on 990 holds, current first:

1. the box's explore-to (36504, 22680);
2. an explore-to **(37080, 23160)**. This is a new approach, not the old
   one: `off_x` is 216 against 264, because the ring spot is measured from
   where the unit stands now;
3. the `BUILDORDER`.

It also has `orders_x` 37080, group 65 and `form_mod` 50. The approach is
re-issued once more on 1005, the next look, at (37032, 23160). The walk
ends on 1036, and on 1037 the unit walks on to its site. This crate held
the box's walk alone, so on 1036 its list was empty.

### 24.2 The case

§17 is the frame. `set_up_insert` copies the leader's orders that carry
the action bit, so the `BUILDORDER` is copied and its transit walk is
not. `finish_insert@0070e620`'s case 6 re-issues the copy as
`action_swarm_around(o, who, QUEUE_LAST, get_type(), flags & 4)`. Case
`0xd` does the same with `REPAIR`. `Group::action_swarm_around@0070fbe0`
at `QUEUE_LAST` does the following:

- It walks the members twice: `domain` 0 (land) first, then 1 (sea). `domain`
  is `+0x218`, and an air member (2) is never taken.
- It keeps each member that is not `is_busy`, subject to a gather filter.
- A **citizen** (`0x32`/`0x33`) that is not inside anything and cannot
  cast `0x293` gets the ring spot. That is the same `find_nearby_spot`
  and `+0x30` nudge as `QUEUE_FIRST`, with the angle
  `find_angle(site − spot)` (`orders.rs`'s `swarm_spot`). Then:
  - **`add_move_facing_order(…, kind, …, QUEUE_LAST)`**. The kind is
    `local_40 = ~(leader_flags >> 1) & 2 | 1` for `BUILD_AT`: `EXPLORE_TO`
    under a computer and `MOVE_TO` under a human. For `REPAIR` it is
    `MOVE_TO`.
  - The queue position becomes **`QUEUE_NEW`** when the member's action is
    a gather and the order carries the action bit (`00710487`).
  - Then `add_build_order` (or `add_repair_order`) at `QUEUE_LAST`.
  - If the ring finds no spot, **neither** is queued.
- Every other member goes into a scratch group, which is sent `MOVE_TO` the
  site at `QUEUE_NEW`.

`update_action` then puts `orders_x` on the approach, since the box's walk
and the approach are the leading transit run. That is run157's 37080, and
it is why the next look on 1005 re-issues: the "already aimed" test
compares the box's cell with `orders_x`'s.

### 24.3 The width twin

`action_move_near`'s per-member store at `00705749` guards `+0xaa`
(`form`) with the four citizen and scholar ids. The `+0xab` store (`form_mod`) comes after
the test, not inside it, as §6.6 step 1 already says. This crate had put
both inside. So a citizen in a group carries the group's width, 50, and
keeps its own `form`. That is run157's `1/1` on 990: `form 9` stands,
and `form_mod` goes from −1 to 50.

### 24.4 This crate

- `Sim::group_action_move_to`'s `QUEUE_FIRST` arm now sends `Body::Build`
  and `Body::Repair` copies to `group_action_swarm_around_last`
  (`group.rs`).
- Each member's half is `Sim::swarm_around_last` (`orders.rs`).
- The width store moved out of the citizen test.
- `a_group_s_queue_first_keeps_the_build_behind_the_walk`
  (`cities_tests.rs`) pins the list, the action bit, `orders_pos`, and the
  width against the form. It was made to fail once with the arm removed.

The remaining SEAMs are named where they stand. None is reached by a
capture on file:

- the scratch group's move for non-builders;
- the `count_inside` and `0x293` exemptions and the cast order;
- the gather filter's `local_30`;
- `is_busy`;
- `BUILD_AT`'s clear of the site's `+0x60 & 0x2000`;
- `finish_insert`'s eighteen other cases.

### 24.5 What it moved

**run157: 1036 → 1176.** On 990 the only row left standing is `1/1`'s
group id, 64 against 65. The id comes from the scout's standing `group`
row, where ours gave the scout 65 and the original gave it 64. Each side
then hands `1/1` the slot the other gave the scout. The id is a slot and
not an identity, and it spends no draw.

The value diff on the moved frame, from run157's own coordinates: on 1037
`1/1` stands at (36480, 22656) on both sides, `idle 0`, and holds explore-to
(37032, 23160) and the `BUILDORDER`. On this item's own tree the word
stopped at 1148, run156's word and shape. With item 629's merchant arm
(`docs/MERCHANT.md` §3.2) that frame agrees too, and the word is **1176**,
who=1's `Leader::produce_building`: 69 draws against 249 at draw 34, where
the original spends `Build::find_gather_tiles`. The scout `1/0`'s explore
path parts from 1077, value only.

**The width twin reaches the long captures.** East Indies' computer
citizens carried a standing `form_mod`, ours −1 against the original's 50,
from the first block of every window. Seven widenings lose those rows and
no other rows: run99, run139, run143, run149, run152, run155 and run159.
Each loses three, except run139, which loses two. Both long words hold,
East Indies 11747 and Great Lakes 12038.

### 24.6 What this has *not* established

- The member loop's arms that no capture reaches. They are listed in
  §24.4 and have been read once.
- Whether the long captures reach a group `QUEUE_FIRST` over a builder.
  The gate's long words are the check, and the journal names what they
  did.

### 24.7 Coverage

The following claims are **diff-backed** by run157's widening, which is
now run over (605, 1178), and by the word's own walk. The width claim is
also backed by the seven East Indies widenings:

- the three-order list on 990;
- `orders_x`;
- the 1005 re-issue;
- the width on a citizen;
- the walk resumed on 1037.

**Reading-only**: the `QUEUE_NEW` gather arm, the `REPAIR` twin, and the
two-pass domain order.

## 25. A unit's own mirror, `unit_masks & 2` — Great Lakes 15175 → 15383 (item 711, 2026-09-24)

*Established by the listing and by run196's dump. The arithmetic is
diff-backed on block 15095; the bit's history rests on one dumped
unit and the listing.*

### 25.1 The frame

On 15095 who=1's army 3 (group 67, list `[76, 77, 78, 79]`) takes the
hero arm of `action_siege_attack_to` (§9). The Despot `1/79` walks on,
and the three free Longbowmen get `action_guard` on it. Both sides lay
out the same escort: `1/76` at `dx 0`, `1/77` at `−144`, `1/78` at
`+144`, all at `dy 264`. A scratch probe of `group_action_guard` read
ours' local list, leader, `facing` and slot table on 15094, and all of
them are the original's. So the offsets agree, and the **posts** parted:
`1/77` stood on (42696, 22392) here and (42984, 22344) there, and `1/78`
the reverse. From that point the two archers walked to each other's
posts, and on 15175 ours spent one `Unit::do_guard` stand more.

### 25.2 The bit

`Unit::do_guard@005e5c70` negates `dx` when the **target**'s
`unit_masks & 2` is set (`5e5fed`–`5e5ff6`), before the heading turns
the offset (`docs/ORDERS.md` §24). The dump prints the Despot's
`unit_masks` as 0x4000A on 15094, so the bit is set.

The whole executable has two writers of the bit. `llvm-objdump` finds
them; the decompiler names neither as `unit_masks`:

- **`Unit::set_angle@00605400`** XORs it (`605424`) whenever the new
  angle is `reversing` from the old `+0x50`. This happens for every
  unit, **before** the leader test that flips `GroupData::facing`
  (§4.1). So the bit is the unit's own copy of the mirror its turns have
  accumulated.
- **`Unit::kill_current_order@005e2cb0`**'s move branch writes it
  (`5e3087` sets, `5e308d` clears). It takes the same value it hands the
  group, again before the leader test (§6.3).

The Despot's history in run192 matches the first writer. It is born
0x40000 on 14983. On 14985 its angle goes 0x55555555 → −756678656, a
`reversing` turn, and the mask becomes 0x4000E (bit 2 set, along with
`find_path`'s 8 and the army's 4).

**Built**: [`sim::Movement::mirror`] carries the bit. Every port of
`set_angle` flips it: `unit_set_angle`, `Movement::set_heading` and
`Movement::set_facing`. `hand_back_facing` writes it before its leader
test, and `do_guard` reads it off the target.

### 25.3 What it moved

**Great Lakes 15175 → 15383.** On 15383 ours spends 4 draws against 3,
parting at index 1. Ours spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d`
where the original spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
That is past run196, so run202 was taken. On its own blocks
(`run202_s_word_frame_is_widened_whole`), the word is the citizen `1/70`.
It holds one order in the original on 15383 and none here. The Despot's
move order also parts on its `facing` from 15351, a value row with no
draw.

**The value diff on 15095** (`run196_s_word_frame_is_widened_whole`):
the four `orders_x`/`orders_y` rows are gone, along with all 89 rows
run196's own blocks held up to the old word and the 221 above it.
Nothing first-parts on 15040..15232. East Indies' word holds at 15985
and chapter eleven's at 1139.

### 25.4 What this has *not* established

- ~~**Two more readers are not wired.** `Unit::set_new_location` tests
  the bit at `5f9290` and negates a pair of crew offsets under it.
  `Guy::set_anim` tests `+0x68 & 2` of its argument at `5dafad`. No
  capture has been compared on either.~~ **Both are unreachable on this
  install** (§26.5): the first needs `guy_mark != 1` and the second a
  `GROUP_IDLE2` packet. The bit itself is compared since item 736.
- `kill_current_order`'s three guards before the write (the order's
  `+0x8`, and the game's `+0x558`/`+0x55c`) are the call site's as it
  stood. This item did not re-read them.
- Units a scenario or loader places with an angle: ours starts the bit
  clear, which is `UnitData::UnitData`'s zero.

### 25.5 Coverage

**Diff-backed**:
- the post arithmetic under the mirror. The unit test
  `a_guard_s_offset_is_mirrored_by_its_target_s_own_flag` runs on
  run196's own numbers, and run196's widening compares the posts block
  for block.
- the first writer, on the Despot's own history.

**Reading-only**: the `kill_current_order` writer, and the two unwired
readers.

## 26. The anchor's sub-group has its own record — Great Lakes 15619 → 16460 (item 736, 2026-09-25)

*Established by run202 and run211's dumps, the decompile of
`action_siege_attack_to` and `Group::clear`, and the widening. The chain
from the sub-group's `facing` to the escort's posts is diff-backed on
every block of run211; the record's other fields are reading only.*

### 26.1 The frame, read whole first

On 15619 the original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`
first, a blocked step (`docs/ANIM.md` §4's table, `:281`), and ours does
not. run211's widening with `RON_ROW_WALK` on group 67 (`1/76`–`1/79`)
read the whole chain backwards:

- **15619**: `1/76`'s position parts; on 15617, its `half_step`.
- **15607**: `1/77` and `1/78` walk to each other's posts. In the
  original both stand on their posts with one `GUARD`; ours holds a leg
  over the `GUARD`, `1/77` to (41976, 21672) and `1/78` to (42216,
  21528), the original's posts traded. Item 711's shape: a mirrored `dx`.
- **The target's mirror.** `do_guard` negates `dx` when its target
  carries `unit_masks & 2` (§25.2). A new widening row (§26.4) shows the
  Despot `1/79` carrying it **here and not there from 15607**. The dump
  prints 0x40008 on every block of run211.
- **The bit's writer.** The Despot's `ATTACK_TO` died on 15606, and
  `kill_current_order` wrote the order's `facing` into the bit (§25.2).
  That order is 15350's, and its `facing` has parted since 15351, 1 here
  and 0 there (**parked 716**). So 716 is not a second parting; it is
  the head of this one.

### 26.2 The readings, and what killed each

Written before reading `action_siege_attack_to` and after the widening,
so not blind. Each killer tests the claim's own unit.

- **R1, `set_anim`'s mirror reader (717) turns `1/76`.** Killed if
  `1/76`'s bit agrees on 15617 or its heading agrees on 15618. **Killed
  by the widening**: `1/76` parts on nothing but `half_step` before
  15619, and the new `mirror` row agrees on it. And the reader cannot
  spend a draw (§26.5).
- **R2, `set_new_location`'s mirror reader (717) moves `1/76`'s post.**
  Killed if the post agrees on every block to 15619. **Killed**: no
  guard or order row of `1/76` parts before 15619.
- **R3, `1/76`'s speed or path parts with no mirror involved.** Killed if
  its next node and its speed agree on 15616. **Killed**: nothing of
  `1/76`'s parts on 15616. The half step is the collision's, and the
  neighbours are the ones that moved.
- **R4 (mine), the target's mirror swaps the posts, and the move's
  `facing` sets the mirror.** Killed if `1/79`'s bit agrees on 15607.
  **It holds**: ours 1, theirs 0.

**The disk answered, so run217 (the packet) was not taken.** Every term
was printed: the posts (`GUARDORDER`), the bit (`unit_masks`) and the
move's `facing` (`MOVEORDER +0x28`).

### 26.3 The cause

`Group::action_siege_attack_to@0070d830` builds its sub-group **on the
stack** (§9). It calls `Group::clear(local, −1)`, copies the parent's
`+0x4 id` and `+0x8 army`, and sets `+0x14 stamp = 0` (`:60–66`).
`Group::clear@00713e80` zeroes the `short` at `+0x48`, which is `facing`
and `buildings`, along with every other scalar field. So the anchor's
`ATTACK_TO` is laid out on **`facing` 0**, XOR `compute_form`'s toggle
(§6.3). The layout's writes go to that stack record and die with the
call.

This crate keys a group's record by its seat (`Sim::gstate`: the army
slot, or the pushed slot). The sub-group carried `army: g.army`, so it
read **army 3's record**: its `facing`, which the leader `1/76`'s turns
had left at 1, and its `form`, `o`, `o_angle` and slot angles. It also
wrote its layout onto that record. On 15095, the first issue, the army's
`facing` was 0 and the two agreed. On 15350 it was 1.

**Built** (`crates/sim/src/group.rs`, `group_action_siege_attack_to`
only, by the commander's leave): for the sub-group's `action_move_to`,
the seat's record is replaced by `GroupState::default()` (which is
`Group::clear`'s), with the parent's `pool` (the carried `id`) and
`stamp 0`. The parent's record is put back afterwards. The army reads
(`hurry`, the AI branch) still see the carried `army`.
`the_anchor_s_sub_group_lays_out_on_its_own_cleared_facing` fails
without it (`Some(true)`).

### 26.4 What it moved

**Great Lakes 15619 → 16460.** On 16460 ours spends 1 draw against 3,
parting at index 1. The original spends `Guy::set_anim+0x97a <
Unit::move_step+0x823`, a blocked step again, which ours does not. That
is past run211 (15859), so run218 was taken (`docs/RUNS.md`).
Its widening (`run218_s_word_frame_is_widened_whole`) finds nothing on
15860..16459. On block 16460, a frame before the word, `1/23` (three
figures, outside group 67) stands stopped in the original at (41632,
21466) and walks on here to (41632, 21440). On the word's block the
original's carries `collide_o 79`: **The Despot blocks it**. No escort
member parts, and no mechanism is named.

- **East Indies holds at 15985.**
- **Every closed golden chapter holds**, and all twenty word tests pass
  at their words.

**The value diff.**
- `run211_s_word_frame_is_widened_whole`'s floor goes 399/89/1014 →
  398/0/398. `1/79`'s move `facing` is gone from run202's walk.
  run211's own 89 up to the old word and the 526 above it are gone, and
  nothing arrived: **nothing parts on 15441..15859**.
- run202's own row (716) is gone, 398/1/399 → 398/0/398.

**The new row, `mirror`** (`unit_masks & 2` against
`Movement::mirror`) is in `widen_block`. It was carried since item 711
and compared nowhere. Outside group 67 it parts on two units, and does
so with or without this item's fix. **All of it is East Indies**:
- the scout `1/0` from 8242 (run99), standing on 9960 (run139, run143),
  beside its `order:move.facing` (parked 275's family);
- `1/31` from 10875 (run149, run152).

Each of those widenings gains that one key. On Great Lakes, the `mirror`
row parts nowhere under the word: the floor stays 398.

### 26.5 717's readers are unreachable on this install

- **`Unit::set_new_location` at `5f9290`** is inside the `guy_mark != 1`
  arm. That arm lays the squad's figures out on a grid, and the bit
  negates both axes of the grid. `guy_mark` is `UnitType::init`'s
  literal 1 (`anim::SQUAD_SIZE`), and every `UNITDATA` of run211 prints
  `guy_mark 1` (53,602 records). The arm is never entered.
- **`Guy::set_anim` at `5dafad`** is in the idle roll, **after** the
  draw. It is reached only when the packet has a `GROUP_IDLE2` animobj
  (`get_animobj(5)`), and a guy of the unit's first `guy_mark` other
  than this one stands on slot 4–6. No shipped unit packet names
  `CHAR_GROUP_IDLE2` (§3.2 of `docs/ANIM.md`), and `guy_mark` is 1. It
  would set the slot, never a draw.

So there is nothing to wire. **717 closes on this reading**, and
**716 closes on the diff**.

### 26.6 What this has *not* established

- **The sub-group's slot angles.** `Group::clear` does not touch `+0x84c
  angles`, so the original's stack sub-group reads whatever the stack
  held. Here it reads an empty table, byte 0. The anchor's `ATTACK_TO`
  `angle` agrees on 15351, so the toggle agreed there.
- **The copy-back.** `replace_form_id` copies the sub-group's slot
  offsets and angle bytes onto the parent's members. This crate writes
  nothing back, and before this item it wrote the whole layout onto the
  parent's record instead. The parent's `GROUPDATA` (its `o`, `o_angle`
  and `order_num`) is printed on every block but not compared.
- ~~**`Form::categorize`'s sort** runs on the stack sub-group in the
  original. This crate's `group_action_move_to` sorts the **seat's**
  list. A one-member sub-group's sort is a no-op, but a sub-group of two
  or more siege would sort the whole army here.~~ Worse than that: the
  seat's sort swapped the whole army in for a one-member sub-group
  whenever the army's list needed re-seating. Built by item 800 (§27).
- **An anchor that leads its parent.** If its dying move is handed back
  inside the sub-group's clear loop, this crate writes it onto the
  swapped-in stack record, and the original onto the parent (`+0x80`).
  No capture has such an anchor.

### 26.7 Coverage

**Diff-backed** (run202 and run211, every block): the anchor move's
`facing`; the Despot's `mirror`; the escort's posts, orders and
positions; nothing parting on 15441..15859.

**Reading only**: the stack record's other cleared fields (§26.6), and
§26.5's two unreachable readers (backed by the dump's `guy_mark` and the
install's packets, not by a run that reaches them).

## 27. A stack sub-group sorts its own list — East Indies 17403 → 17501 (item 800, 2026-09-26)

*Established by run233's dump and the widening, with a scratch print of
the call that issued the order. Diff-backed on every block of run233.*

### 27.1 The frame, read whole first

On 17403 ours spent 7 draws against 6, parting at index 0: ours
`Unit::do_move+0xe84`, the original `Guy::set_anim+0x97a <
Guy::inc_time+0x271`. run233's widening, with `RON_ROW_WALK` on `1/60`
and `1/67`..`1/69` and the dump's own records for 17340..17410:

- **`1/67`..`1/69` are one squad born on tick 17362** (absent to block
  17362, `o_up`/`o_down` 67 → 68 → 69). On their birth block, 17363, the
  original pushes them into pool slot 69 (`stamp 17362`, `form 0`) and
  ours leaves them group-less. On 17364 both sides add them to army 1,
  and from there to the window's end they part on nothing but the pool
  id (`group` 69 here, 71 there: 689). **Not on the chain.**
- **`1/60` parts on nothing but the pool id until block 17403.** It is
  army 1's other member: a unit of three figures that walks as the siege
  arm's anchor (§9), a supply wagon or a hero.
- **Block 17403**, the state after tick 17402: army 1 attack-moves. The
  first parting's field list is `1/60`'s order block and every field of
  its walk:

  | `1/60` on 17403 | theirs | ours (before) |
  |---|---|---|
  | order type | 2, `ATTACK_TO` | 21, `GROUP_ATTACK_TO`, flags 5 |
  | `x`, `y` (the slot) | (38136, 42024) | (37992, 41784) |
  | `orig_x/y` | (38122, 42017) | (38122, 42017) |
  | `angle` | 1745223680 | 1745223680 |
  | position | (34408, 41098), `last_speed 41` | (34368, 41088), stopped |
  | `unit_masks` | 0x840008 (no `& 4`) | not in danger |

  The squad's escort posts (`GUARDORDER`) part by 48 on the same block,
  because they are laid out on the anchor where it stands.

### 27.2 The readings, and what killed each

Written from the rows, before the call was printed. Each killer tests its
own reading's unit, the anchor's order:

- **R1, the in-danger gate** (§6.6 step 6, `unit_masks & 4`) gave the
  original a plain move. Killed if the original's `1/60` carries no
  `& 4` on 17403. **Killed**: 0x840008.
- **R2, the original issued a group move and ungrouped it on the same
  frame** (`ungroup_move_order`, ORDERS §8.3). Killed if the order
  carries `dest 1` or an `orig` other than its `x/y`, which the ungroup
  writes. **Killed**: `dest 1`, `orig` (38122, 42017) against `x/y`
  (38136, 42024).
- **R3, the anchor's sub-group is laid out with more than one member
  here.** Killed if the print of ours' `group_action_move_to` shows the
  sub-group of one. **Holds**: it shows `[60, 67, 68, 69]`, called from
  `group_action_siege_attack_to`, where the sub-group built at its head
  was `[60]`.

### 27.3 The cause

Army 1's list is **`[60, 69]`** on both sides: `1/69` is the squad's
tail, listed without its captain `1/67` (§23's shape). The siege arm
builds its sub-group `[60]` carrying the parent's `army`, so it reads
and writes a record (§26). `Form::categorize` begins with
`Group::sort@00708090`, which runs **on the group being laid out**, and
in the original that is the stack sub-group of one: a no-op.

This crate's `group_action_move_to` sorted **the seat's** list. `1/69`
comes after no captain of its own, so the sort killed the squad and
re-added it whole, `[60, 67, 68, 69]`. And a sort that changed the list
handed the layout the seat's group, so the sub-group became the whole
army. `1/60` was laid out as slot 0 of four, a `GROUP_ATTACK_TO`, and
stood as a leader waiting for its followers.

**Built** (`crates/sim/src/group.rs`): a value that names a seat but
holds another list is a group on the stack, read at entry, before the
clear. Its sort is `Sim::stack_sort`, over its own list. The seat's sort
still runs for a seated group. `the_anchor_s_sub_group_sorts_its_own_
list_and_walks_alone` fails with the seat's sort: the wagon walks under
a group order and the army's list comes back re-seated.

### 27.4 What it moved

**East Indies 17403 → 17501. Great Lakes holds at 20568.**

**The value diff, on 17403** (`run233_s_word_frame_is_widened_whole`):
`1/60` holds an `ATTACK_TO` to (38136, 42024) on both sides, and its
position, path and `orders_x/y` agree. Army 1's list, compared on
`(who, army)` and never the slot, is `[60, 69]` on both sides on 17401
to 17405. Before the fix it was `[60, 67, 68, 69]` here. All 52 of
`1/60`'s rows go, and so do `1/15`'s two and the squad's escort rows.
The window's floor goes 293/312/324/1,036 → 293/312/324/446.

**The new word, 17501**: ours 13 draws against 11, parting at index 3.
Ours spends `Guy::set_anim+0x97a < Guy::move+0x19f` where the original
spends `< Unit::do_idle+0x7d`. A scratch print of ours' sites names
**`1/57`'s walk step**, and one more gaia clock (`9/9` twice and `9/15`
here, two there). `1/57` is army 0's column. **What stands under it on
run233**: from block 17405 the column `1/48`..`1/58` parts on the order
army 0 issues on tick 17404. Its group id is 17404000 here and 17410801
there, which is 674's shape: `(id + frame·10)·100 + order_num` with id 0
and `order_num` 0 against 68 and 1. Their positions and headings part on
the same block. It is past run233's end, so run251 was taken
(`docs/RUNS.md`). No mechanism is named.

### 27.5 What this has *not* established

- **The stack sort's pointer writes.** The original's `kill` on the
  stack group clears `+0x80` on a squad member that names the group's
  id, which is the parent's. `normalize` prunes by that id.
  `stack_sort` writes neither. No capture has a sub-group that needs
  sorting: every siege arm on file has one anchor.
- **A seatless group's sort.** `Form::categorize` sorts every group it
  lays out. This crate sorts a seated group and a stack group that
  carries a seat, and never a player's selection or `action_guard`'s
  local group. The latter is built captain-first by `group_add`, so its
  sort is a no-op.
- ~~**The squad's birth push** (17363): the original's new squad sits in a
  pool slot of its own for a block before the army takes it. That is
  689's family and spends no draw.~~ `Unit::come_out`'s push, built by
  item 882 (§31).
- 744's other seams, the slot angles, the copy-back and the anchor that
  leads its parent (§26.6), are not touched.

### 27.6 Coverage

**Diff-backed** (run233, every block 16929..17440): the anchor's plain
order and its walk on 17403, the escort's posts, army 1's list on
17401..17405 and nothing of `1/60` or the squad parting after 17364 but
the pool id. **Reading only**: §27.5's pointer writes, which no run
reaches.

## 28. A pushed group keeps its slot's `facing` — East Indies 20007, named and not built (item 865, 2026-09-26)

*Established by run269's and run277's `GROUPDATA`, `copy_group`'s
listing, scratch prints and a payoff probe. Diff-backed on 19664 and
20002. **Not built**: the build needs the pool's numbering. That parts on
block 239, and on a branch (§28.4) it holds to 8369.*

### 28.1 The frame, read whole first

The word is 20007 (block 20008). Ours spends 8 draws against 7, parting
at index 0: ours spends `1/67`'s `Guy::set_anim+0x97a <
Unit::move_step+0x823`, a blocked step. Under it, `1/65`'s walk animation
is 7 here against 8 there from 20005. **The first killer is run277's
first block, 20002**, with `RON_STANDING`. Its first parting's field list,
on `1/64`..`1/66`'s `ATTACK_TO` orders:

| on 20002 | theirs | ours |
|---|---|---|
| `1/64`..`1/66` order `facing` | 0, 0, 0 | 1, 1, 1 |
| `1/65` order `x/y` (`off` (504, 648) / (696, 456)) | (35832, 42120) | (36024, 41928) |
| `1/66` order `x/y` | (36024, 41928) | (35832, 42120) |
| `1/65` `pos` | (34659, 40997) | (34663, 40991) |

**On run269's last block, 19664, nothing of theirs parts but the pool
id.** `1/64`..`1/66` sit in army 1's group, 69 there and 70 here (689).
The gap 19665..20001 is on no disk. The move itself is on 20002's own
`GROUPDATA`:

| slot 69 | 19664 (run269) | 20002 (run277) |
|---|---|---|
| `army` | 1 | −1 |
| list | sixteen, army 1's | `[64, 65, 66]` |
| `stamp` | 18976 | 20000 |
| `order_num` | 5 | **6** |
| `facing` | 1 | **1** |

Army 1 sits on 68 from `stamp 20000`, and who=1's `last_group` is 69.
In ours, a scratch print: on tick 20000 `add_to_army` (a joiner walking
to the army, ARMY §4.3) calls `go_to`, which pushes the squad `[64, 65,
66]` and moves it with `set_angle 0`. The destination is (35928, 42024),
the angle 1527906304 and the leader `1/64`'s heading −1093992448, so
`compute_form`'s toggle holds (§6.3). The pushed record's `facing` is
**0**, so the mirror is 1. Ours seats it on 69 too, over a fresh record.

### 28.2 The readings, and what killed each

Each killer tests its reading's own unit on tick 20000, the move's mirror
and slot table:

- **R1, the members' order into the seat differs.** Killed if both
  lists are `[64, 65, 66]` with the same leader. **Killed**: the
  original's slot 69 lists `[64, 65, 66]`, and ours' layout is called
  with the same list and leader `1/64` at slot 0.
- **R2, the seat's arithmetic differs on the same order.** Killed if,
  with the mirror forced to the original's, every field of the three
  agrees on 20002. **Killed** by the probe below: they agree whole.
- **R3, the facing flips the slots.** Holds: the orders' `facing` is
  the mirror the layout used (§6.3, run31), 0 there and 1 here, and a
  mirror trades `1/65`'s slot for `1/66`'s exactly.

**The writers of the slot on tick 20000, counted** (parked 823): one per
member, `group_action_move_to`'s order loop. `1/65` gets (36017, 41910)
and `off` (−144, 0) under `reverse`. Nothing rewrites the order before
block 20001. **No `SEAM:` on the chain** (`add_to_army`, `go_to`,
`push_group`, `pool_slot_for`, `group_action_move_to`) names what
`copy_group` leaves in the slot. The one on `pool_slot_for` names the
fallbacks.

### 28.3 The cause

`Groups::copy_group@006fa690` writes `who`, `num`, `ox`/`oy`, `o_dist`,
`o_angle`, `buildings`, `speed`, `stamp = frame`, the list, the angle
bytes and the four offset arrays. **It leaves `facing`, `order_num`,
`form`, `form_num`, `new_speed` and `army`**, as §3.2 says. So a push
onto a fresh slot inherits its previous occupant's `facing`. The
original's 69 held army 1's closed group (ARMY §22), which kept
`facing 1` and `order_num 5`. The go-to move reads `facing 1`. The
toggle holds, so the layout is square (`facing` 0 on the orders), and
the move counts `order_num` 6. The members join army 1 on the same tick
(`Group::add` on the army's live group), so no leader turn toggles 69
afterwards, and it prints its pre-call 1. Ours pushed a fresh record,
`facing 0`, so the mirror was 1.

**The payoff probe** changed only tick 20000's push, giving it `facing
1` and `order_num 5`. The word goes **20007 → 20782**. On 20002 every
field of `1/64`..`1/66` agrees, orders, path and position alike. Only
the pool-id rows remain (`group` 71 here, 68 there).

### 28.4 Why it is not built: the numbering parts on block 239

The build is two rules, both faithful, and each was measured on run54's
word:

- **`copy_group` into the slot's own record.** A push onto index `s`
  inherits that record's non-copied fields. The one entry per index is
  the record: a closed army's orphan on its own index, and a push over
  it. **Alone, or with the next rule, it breaks at 17530.** Bisected by
  field, the break is `facing`. On tick 17363 the squad `[67, 68, 69]`
  is pushed at its birth (§27.5). The original's slot 69 then held
  `order_num 5`, `facing 0` (run233, block 17363). Ours takes slot 70,
  which holds the tick-17146 `go_to` joiners' record, `order_num 12`,
  `facing 1`. A faithful inheritance of the wrong record is worse than
  a fresh one.
- **`get_open_slot`'s `get_num` rule.** A slot is empty when
  `Group::get_num@00714700` is zero. That is a `normalize` for a list of
  fewer than four, and the dead alone dropped for four or more. So a
  closed army's fifteen stale members hold its slot until the cursor
  prunes them. Ours asks "no live member points here", which passes that
  slot at once. **Alone, it holds 20007.**

Both rules need **the original's numbering**, and East Indies' parts on
**block 239** (`east_indies_pool_numbering_takes_who_1_s_building_groups`,
run45). The original's who=1 pushes **building groups**, `[2005]` onto
64 on frame 1 and `[2000]` onto 66 on frame 176. So `think_scout`'s
repush of `[0]` on tick 238 does not equal `last_group`, and
`get_open_slot` takes 64. That is a building group that is not
`last_group`, taken at once (§3.1), and `1/0` moves 65 → 64. Ours pushes
no building group, so the repush equals `last_group` and stays on 65.
Every pool-id row since, 689's forty, descends from this.

**The caller is the AI scripts.** In the listing, `Group::action_queue_up@006fdbb0`
has six call sites. Five are `ScenarioFuncSet`'s `train_unit`,
`train_unit_with_cost`, `train_unit_at`, `train_unit_at_with_cost` and
`research_tech_with_cost`, and the sixth is `CommandPackage::process_queue_up`.
Each script function first `Group::add`s the producing building to a
stack group and calls `push_group(who, g, 1)` (`009f4595`, `009ee881`,
…). run54's coverage enters `research_tech_with_cost` and
`train_unit_at_with_cost` on frame 1, `Group::action_queue_up` on frame 1,
and `process_queue_up` never. att-867's candidates are
`Object::take_damage`'s two `action_alarm` sites (`0x652677`,
`0x65284b`), with `CommandPackage::process_group` ruled out by the same
lane. run54 never enters `take_damage` in 24,000 frames, so those are
not the caller on this game. The fifteen `push_group` sites in the game's own
code, from `5e5f9f` to `704cd2`, all pass `force 1`.

**Built on a branch and measured, not landed** (`att-865-pool-wip`, two
commits):

1. **The numbering.** `ai_host`'s `action_queue_up` pushes the
   building group first. `get_open_slot` takes a building group at
   once, empties a slot by `get_num`, and skips the eviction on a
   building group's slot. `Sim::pushed` holds one entry per pool index.
   **East Indies holds 20007 and Great Lakes 20568.** who=1's `group`
   agrees with every East Indies dump, pointer for pointer, up to block
   **8369** (run98), where it had parted on 239. On 8369 the original's
   `1/11` goes 65 → −1 and this crate's stays on 65. `1/11` is a
   citizen of `think_civilian_transport`'s (flags 1) and prints nothing
   else that parts. The writer is not found. The pool-id pins across the
   suite were not re-pinned.
2. **`copy_group`'s kept fields**, on top of the numbering: **East
   Indies 17530 and Great Lakes 7213**, both on `facing` when bisected
   by field. From 8369 a slot's previous occupant is often another
   record.

**Item 870 landed the numbering** (§29): 8369's writer was
`do_non_flat_gather`'s every-call drop, and the pool agrees to the word.
The kept fields still fall to 17530/7213 (§29.4).

**What building it takes**: the writer of 8369's `1/11`, and then every
later pool-id parting in turn. The comparison is a script,
`group`-for-`group` against each East Indies dump in run order, first
mismatch per dump, and it reached 8369 in a minute. Great Lakes wants
the same walk. Then the kept fields, with both words measured. This
crate's slot 69 holds **two** records on 20002, army 0's orphan and the
squad
(`east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held`),
which is what one entry per index replaces.

### 28.5 What this has *not* established

- **The gap 19665..20001.** It is on no disk. The previous occupant of
  69 is read from 19664's and 20002's records and `order_num`'s
  arithmetic (5 → 6), not watched. In ours, army 1 closes on tick
  19706, army 0 takes the fifteen, army 1 re-forms on `[60]` on 19717,
  and army 0 closes on 19965. The original's 64 (`stamp 19706`, fifteen
  listed, `army −1` on 20002) and 68 (army 1) are the same events on
  other indices. No capture was taken (run283 and run284 are unused).
  The probe answered the frame, and what stands, 8369's writer, is
  already on run98's disk.
- **`get_num`'s own prune.** The rule above counts and does not write
  the list back.
- **The army path.** `Army::add_unit`'s `push_group` for a fresh army
  inherits the slot too. This crate's army record is its own.
- **A building's own `+0x80`.** `push_group`'s second walk points the
  building at its slot. The branch does not carry it.
- **Great Lakes' numbering.** It is not walked. It holds 20568 with
  the branch's numbering, and falls to 7213 with the kept fields.

### 28.6 Coverage

**Diff-backed**: slot 69's record on 19664 and 20002, who=1's
`last_group`, the three orders' `facing` on both sides and this crate's
two records on the index
(`east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held`),
and the pool on frames 2, 238 and 239 against this crate's `1/0` on 239
(`east_indies_pool_numbering_takes_who_1_s_building_groups`).
**Decompile-backed**: `copy_group@006fa690`'s nine fields,
`get_open_slot@006fa460`, `get_num@00714700`, `equals_group@00708000`.
**Listing-backed**: `action_queue_up`'s six call sites and the
`push_group` sites' `force`. **Coverage-backed** (run54): the script
functions entered on frame 1, and `take_damage` never. **Measured, not
asserted**: 20007 → 20782 under the probe. On the branch, 20007/20568
with the numbering, and 17530/7213 with the kept fields.

## 29. The pool's numbering is the original's to East Indies' word — the building groups and the gatherer's `group` (item 870, 2026-09-26)

*Established by run98's records, the listing at `5f0170`–`5f0245`, and a
walk of who=0's and who=1's `group` against every East Indies dump.
Diff-backed on every block of those dumps. **The word does not move**:
East Indies holds at 20007 and Great Lakes at 20568. §28's kept fields are
still not built.*

### 29.1 The numbering (from §28.4's branch)

865's first commit (`a05f90a6`) lands as it was measured:

- `ScriptHost::action_queue_up` pushes the producing building as a group
  first (`Sim::push_building_group`), as the scripts' `train_unit*` and
  `research_tech_with_cost` do.
- `open_slot` is §3.1 whole: an empty slot is one whose `get_num` is 0 (§28.4's
  rule), a building group is taken at once, the fallback is the oldest
  single-captain slot, and a building group's slot skips the eviction.
- `Sim::pushed` holds one entry per pool index. `seat_orphan` replaces
  its own index's entry.

Alone it holds both words, and who=1's `group` agrees with every East
Indies dump until block 8369.

### 29.2 Block 8369: the gatherer drops its group

On run98's 8369, `1/11` (a citizen, flags 1) reads `group` 65 → −1. **The
whole cast on the frame**: three of who=1's citizens, `1/2`, `1/6` and
`1/11`, each get a `MOVEORDER` pushed on top of their gather.

| | `1/2` | `1/6` | `1/11` |
|---|---|---|---|
| move `x`, `y` | (34872, 36984) | (37800, 36408) | (37800, 36456) |
| `angle` | −541917184 | 1325793280 | 1179123712 |
| `facing`, `dest` | −1, 0 | −1, 0 | −1, 0 |
| `group` on 8368 → 8369 | −1 → −1 | −1 → −1 | **65 → −1** |

Only `1/11` had a group to lose. This crate pushed the same three moves
on the same tick, with the same destinations and angles, from
`do_non_flat_gather`'s `add_move_order`, and kept `1/11` on 65.

**The writers of `+0x80`, counted by the offset**: `Group::action_move_near`,
`Group::kill`, `get_open_slot`, `push_group`, `Unit::~Unit`, `build_done`,
`do_attack`, `do_build`, `do_group_move`, `do_non_flat_gather`,
`Unit::init`, `set_group` and `think_peasant`. On tick 8368 `1/11`'s only
order is the gather it was given on 8367, so its own call reaches
`do_non_flat_gather` and none of the others. The listing is plain:
`Unit::do_non_flat_gather@005f0170` branches on `been_there` at `5f01ba`,
and both paths join at `5f0237` (`orl $-1, %eax`) and `5f023e` (`movw
%ax, 0x80(%ebx)`), before the `goto_build` test. **Every call drops the
gatherer's group, a computer's too.** That is parked 831, 824's measured
writer. The pool keeps listing the unit, and only the back-pointer goes,
as with `think_peasant`'s human arm.

**Built** (`crates/sim/src/orders.rs`, the head of `do_non_flat_gather`).

### 29.3 The walk

`east_indies_pool_walk` (an ignored probe in the harness) ticks run54's
game once and compares who=0's and who=1's `group` on every block of
every East Indies dump, run42 to run277. With 29.1 and 29.2:

- **Every dump agrees on every block to 20257**, run277's last, but two
  blocks.
- ~~**17363** (run233): `1/67`..`1/69` read −1 here and 69 there on their
  birth block. The original seats a new squad in a pool slot of its own
  for one block (§27.5), and this crate does not. The next block agrees.~~
- ~~**17575** (run251): `1/70`..`1/72` do the same, on 64 there. The next
  block agrees.~~ Both agree since item 882: the push is `come_out`'s
  (§31).
- run96's 23960 is past the word.

On block 20002, `1/64`..`1/66` point at 68 on both sides, and slot 69
holds the squad `[64, 65, 66]` on both sides. This crate now keeps one
record there, not §28's two.

### 29.4 The slot records: where §28's kept fields stand

The walk's second mode compares each `GROUPDATA` slot's `facing` and
`order_num` with this crate's record. Without the kept fields (a fresh
record on every push):

- they agree on all of run45 (blocks 1..901), run64 and run65
  (6164..6215);
- they first part on run221's first block, **15894**:

  | slot | list, `stamp` (both sides) | `order_num` here | there |
  |---|---|---|---|
  | 64 | `[48]`, 15888 | 1 | 6 |
  | 65 | `[31]`, 10952 | 1 | 3 |
  | 70 | army 0's ten, 15888 | 0 | 6 |
  | 71 | stamp 15887 | 1 | 8 |

  That is the previous occupant's `order_num`, kept by `copy_group`. The
  gap 6216..15893 prints no `GROUPDATA`.

With 865's second commit (`30e5b464`, `..prev` for the fresh record)
**on top of 29.1 and 29.2**, the words fall to **17530 and 7213**, as on
865's branch. The records part earlier, on **6164**: slot 64 (`[15]`,
stamp 5937) reads `order_num` 13 here against 1, and slot 68 (`[0]`)
reads `facing` 0 against 1. So this crate's kept-field accounting
carries a history the original's slots do not. The candidates are this
crate's army records, which are their own and not a pool slot's (§28.5,
parked 874), and the orphan seat. Neither is walked. The kept fields
stay unbuilt. **Item 880 walked it (§30)**: the first parting was run45's
block 377 (run42 had shadowed run45's blocks), and the cause was
`Group::kill`'s clear of an emptied record, then the army path's
`copy_group` and kill. The kept fields are built on it.

### 29.5 What it moved

Both long words hold: East Indies 20007 and Great Lakes 20568. Every
golden chapter holds. **The value diff at the item's own frame**, block
8369: `1/11`'s `group` −1 there, 65 here before, and −1 here now. On block 239
(run45), `1/0` is on 64 on both sides, slot 64 is stamped 238, and 66
holds the building group stamped 176.

Across the moved widenings, **302 parted keys close and none opens**:

- 295 `group` rows. Examples: `1/11` 64 against −1 on every East Indies
  window from 9960, `1/0` 66 against 65, `1/32`..`1/38` 68 against 66,
  and army 1's column 71 against 70 on 15894;
- five `order:move.facing` and two `mirror`, the scout's and `1/22`'s
  and `1/29`'s layouts.

run99's scout `order:move.facing` now first parts on **8481**, 0 here
against 1 there. That is the kept `facing` (§28), and it is past the
records' agreement. On Great Lakes, run79's 29 scout-mirror rows and
run83's `Move.facing` family close. On the golden controls, ch7c's and
ch7bc's `1/0` `group` on 605 (65 against 64) and ch7bc's `1/1` on 990 (64
against 65, §24's permutation) close.

**ENDPOINTS**: East Indies moves 38 → 40 off and 0 → 2 unlinked (`1/81`,
`1/82`), 3,994 frames past the word. Great Lakes holds 11/0.

### 29.6 What this has *not* established

- ~~**The squads' birth push** (17363, 17575): which call seats a new
  squad for one block. It is 689's family, one block each, and spends no
  draw.~~ `Unit::come_out@00617c10` (§31, item 882).
- ~~**Where the slot records' `order_num` part in 6216..15893.**~~ No
  capture was needed: the records part on block 377, and with §30 they
  agree on every dump, 15894 included.
- **Great Lakes' pool** (871). Its pins moved toward the original's and
  its word holds. It is not walked.
- **`get_num`'s own prune** (parked 872) and a building's `+0x80`
  (parked 873), as §28.5 says.

### 29.7 Coverage

**Diff-backed**:
- `1/11`'s `group` on 8369 and every East Indies dump's `group` rows
  (the widenings, the walk);
- the block-239 pool
  (`east_indies_pool_numbering_takes_who_1_s_building_groups`);
- slot 69 on 20002
  (`east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held`).

**Listing-backed**: `5f01ba`, `5f0237` and `5f023e`. **Measured, not
asserted**: the kept fields' 17530/7213 on this tree, and the record walk's
15894 and 6164.

## 30. A killed-empty slot is cleared, and an army's group is its slot's record — East Indies 20007 → 20782 (item 880, 2026-09-26)

*Established by run45's, run64's and run221..run277's `GROUPDATA`, the
decompiles of `Group::kill@00714110`, `Group::clear@00713e80`,
`Army::add_unit@006f9f40`, `Army::add_group@006f8c00` and
`Groups::push_group@0070f9e0`, and the listing of `Group::clear`.
Diff-backed: every `GROUPDATA` slot's `facing` and `order_num`, and
every unit's `group`, on every East Indies dump to 20257. §28's kept
fields are built on top of it (`30e5b464`).*

### 30.1 The parting, read whole first

With §28's kept fields on §29's tree, `east_indies_pool_walk`'s record
mode printed **6164** first only because run42 (no `GROUPDATA`) was
listed before run45 and took blocks 1..901. With run45 first, the
records part on **block 377** (run45): who=1's slot 65, the building
group `[2000]` stamped 376, reads `order_num` 1 here against 0 there.
The walk's slot history (`RON_POOL_WALK_HIST=1`) against the dump's:

| who=1 slot | block | theirs | ours (kept fields, before 880) |
|---|---|---|---|
| 65 | 239 | `order_num` 0, `facing` 0, `form` −1, `stamp` 238, `num` 0 | `order_num` 1, `form` 0, `stamp` 0, list `[]` |
| 65 | 377 | `[2000]`, `order_num` 0, `stamp` 376 | `order_num` 1 |
| 64 | 414 | `order_num` 0, `form` −1, `stamp` 413, `num` 0 | `order_num` 1, `facing` 1, `stamp` 238 |

On tick 238 `think_scout` repushes `1/0` from 65 onto 64, and the
original's 65 is **reset**, stamped with the tick it emptied. On 413 the
same happens to 64. So the slot's previous occupant is not always what
it held: an emptied record is cleared.

### 30.2 The writers, counted

**`order_num` (`+0x2c`) on a pool record**, by the offset: `Group::clear`
(0), and the `++` in `Group::action_move_near`, `action_patrol` and
`action_attack`. `Army::find_target`, `find_muster_spot` and `init` write
the `Army`'s own `+0x2c`, not a group's. **`facing` (`+0x48`)**:
`Group::clear` (a `movw` that zeroes `buildings` too), `compute_form`,
`action_air_patrol`, `action_flight` and `action_move_near`'s copy into
a sub-group. `copy_group` writes neither (§28.3).

**`Group::clear`'s callers on a pool record**: `Groups::clear` at the
start, and **`Group::kill`**, which clears when its list reaches zero
and otherwise writes `stamp = frame`. The listing of `Group::clear`
(`713e80`–`713f13`): `who` 0, `army` −1, `num` 0, `form` −1, `stamp =
game->frame`, `ox`, `oy`, `o_dist`, `o_angle` 0, the `facing`/`buildings`
halfword 0, and `disband`, `order_num`, `priority`, `role`, `new_speed`,
`speed`, `form_num`, `think_frame` and `march` 0. The pool index (`+4`)
stays when called with −1. `Group::normalize` and `Groups::process`
shrink a list without clearing.

**`Group::kill`'s callers on a pool record** are
`push_group`'s second walk (`0070f9e0`), which asks **the group the
member's `+0x80` names**, when it is not the slot being written, `(*groups[+0x80].vtbl+0x10)(o, who, 0, 0)`; `Group::add` with
`keep_captain`; and `Group::sort`.

### 30.3 The two rules

- **A kill that empties a record clears it** (`Sim::kill_from_named`,
  `GroupState::clear`). `push_group`'s walk asks the seat each member's
  `+0x80` names, takes the member's squad out of it, and writes `stamp`,
  or clears the record when the list empties. The broader removal this
  crate did before (every pushed list and every army that holds the
  squad, §3.3) stays after it.
- **`Army::add_unit`'s fresh group goes through the same `push_group`**
  (`Army::add_unit@006f9f40`: `Group::clear` a stack group, `Group::add`,
  `push_group(who, g, 1)`, `add_group`, which writes only `army`). So the
  army's group **is** the slot's record: `copy_group` keeps the previous
  occupant's `form`, `order_num`, `facing`, `form_num`, `new_speed` and
  `march`, an equal group is not copied, and the walk's kill runs.
  `GroupState::copied` is the one `copy_group` for `push_group`,
  `push_building_group` and the army path.

Each was measured with the record walk, East Indies' list in run order
with run45 first:

| tree | first record parting |
|---|---|
| 870 + kept fields | block 377 (run45), slot 65 `order_num` 1/0 |
| + the clear in `push_group` | run45, run64 and run65 whole; 15900 (run221): army records only, slot 70 `order_num` 0 against 6 |
| + the army's copy | 17496 (run251): slot 64 `[0]` stamp 17191 `order_num` 7 against 1 |
| + the army path's kill | **none**: every slot's `facing` and `order_num` on every East Indies dump to 20257 |

17496's was the army path's missing walk. On tick 17156 `1/60`, the last
member left on army 0's orphan on 64, joins army 1's fresh group on 71.
The original's 64 is cleared (`stamp` 17156, run233's 17157). Ours kept
`order_num` 6, and `1/0`'s push onto 64 on 17191 inherited it.

Every unit's `group` agrees on every East Indies dump too, but run96's
`1/60` on 23960, past the word. The `[2]` markers the walk still prints
(a stale `Pushed` entry beside an army on one index) carry no field
difference.

### 30.4 What it moved

**East Indies 20007 → 20782.** Great Lakes holds at 20568. Every golden
word and control holds (the whole suite at 509 passed; the five
failures were the re-pins below and the thread-width guard).

**The value diff on the old word's frame, block 20002** (run277):

| | theirs | ours before | ours now |
|---|---|---|---|
| slot 69 `order_num`, `facing` | 6, 1 | 1, 0 | **6, 1** |
| `1/64`..`1/66` order `facing` | 0, 0, 0 | 1, 1, 1 | **0, 0, 0** |
| `1/65` order `x/y` | (35832, 42120) | (36024, 41928) | agrees |
| `1/65` `pos` | (34659, 40997) | (34663, 40991) | agrees |

- run277: 916 → 299 keys. The first block's 37 slot-swap rows close
  (323 → 286), all 21 under the old word close, and 580 later keys
  close. **None opens** (a set difference of both trees' key lists).
- run99: 174 → 173, the scout `1/0`'s `order:move.facing` on 8481 (0
  here against 1 there, §29.5).
- `ENDPOINTS`: East Indies 40 → 37 off, `build_diverged` 0 → 2.

**The new word, 20782**: ours spends 8 draws against 1 at index 0, ours
`Leader::use_market+0x1ed` and theirs `Farms::inc_time+0x1ae`. It is
past run277's end; run289 was taken for it (`docs/RUNS.md`,
`run289_s_word_frame_is_widened_whole`). The first keys to part under it
are who=1's make list on 20782 (three city-1 entries here, empty there),
then this crate's extra building `1/2030` on 20783. **It is not a pool
mechanism**, and it makes Great Lakes (20568) the lower map.

### 30.5 What this has *not* established

- **`o` on a pushed record.** The listing's `Group::clear` writes 0 to
  `ox`/`oy` (`713eb4`, `713ebb`), and `copy_group` copies a stack
  group's 0. This crate seeds −1 (`GroupState::default`), which
  `group_o` reads as "never moved". `GroupState::clear` writes the
  listing's 0 on a cleared slot; `copied` keeps −1. Unmeasured.
- **The kill's other callers.** `Group::add` with `keep_captain` and
  `Group::sort` reach `seat_kill`, which does not clear on empty. No walk
  parted on them.
- **The broader removal.** This crate still takes a pushed squad out of
  every list that holds it, where the original asks only the group
  `+0x80` names. The walk does not see the difference.
- **`who` and `army` on a cleared record** (0 and −1). This crate keeps
  the pool index and does not model them.
- **Great Lakes' records** (871): its walk (`RON_POOL_WALK_MAP=greatlakes`)
  agrees on every unit's `group` to 20500 and on every slot's `facing` and
  `order_num`. The one parting row is run46's, a human-click game, on
  who=0's selection.

### 30.6 Coverage

**Diff-backed**: every `GROUPDATA` `facing` and `order_num` and every
`group` on East Indies (the walk), slot 69 and the orders on 20002
(`east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held`),
and run277's and run99's floors. **Decompile-backed**: `Group::kill`,
`Army::add_unit`, `Army::add_group`, `push_group`. **Listing-backed**:
`Group::clear`. **Measured, not asserted**: the walk's partings in 30.3,
and Great Lakes' walk.

## 31. A trained squad's pool push, and a command's building group — chapter twenty-four's `group` and pool rows (item 882, 2026-09-26)

*Established by the listing at `618900`..`6189aa` and `617c33`, the
decompiles of `Unit::come_out@00617c10`, `Group::add@00714350`,
`Groups::push_group@0070f9e0` and `Unit::init@00612100`, and run285's
`UNITDATA` and `GROUPDATA` (`docs/GOLDEN.md` §33). Diff-backed on every
block of run285 and run208. The golden words hold: chapter twenty-four
at 1560, thirteen at 1000, twenty-three at 1840.*

### 31.1 The push

**The booking's "human arm" was
877's hypothesis, and the listing killed it.** `come_out`'s push is
`618900`..`6189aa`: the unit is a captain (vslot `0xe8`, `o_up < 0`), its
host index is ≥ 0 and the host's vslot `0x20` answers (a building, not a
transport), and the type's `uber_size` (`+0x308`) is over 1; then
`Group::add(o, who, 0, 0)` (the captain and its `o_down` chain) into the
stack group `Group::clear` opened at `617c33`, and `push_group(who, g,
1)`. **There is no owner test**: a computer's trained squad takes its own
slot the same way, which is East Indies' one-block birth rows on 17363
and 17575 (`docs/GROUPS.md` §27.5, §29.3, parked 689). The push follows
the members' own exits (`:535`'s recursion runs first) and precedes the
gather-point block (`BuildData +0xcc`), which no command in run285 sets,
so `come_out` issues the group no order here. **The writers on the birth
block, counted by offset** (823, 869): `+0x80` — `push_group`'s second
walk, and nothing else in `come_out`; `+0xaa` (`form`) —
`Unit::init@00612100` (9 for types `0x32`–`0x35`, 0 for the rest),
`Group::action_move_near@00704990`, `Group::action_form@00707220`,
`Unit::do_form_change@005e8670` and the cast and scenario writers, and
no layout or cast reaches these units in run285, which leaves `init`; `+0x70`/`+0x74` (`orders_x/y`) —
`Unit::init`'s zeroing and its `update_action`, which sets them to the
point it inits the unit at, tile-centred (`div_3(x >> 4)·0x30 + 0x18`),
and `come_out`'s `update_action` on the captain.

### 31.2 Two events, and which row needs which

**So the slot numbers need two events**, and each row was asked which:
the Barracks' building group at each command (`CommandPackage::
process_group@0094a0c0`, `0x94a6cf`; §32's standing row) and the squad's
push. Without the first, the Hoplites take slot 1 (ours) against 0
(theirs): slot 0 is `last_group`'s. **Built**: `Sim::come_out` pushes
the squad (`crates/sim/src/garrison.rs`), and
`Sim::push_command_buildings` seats a one-building command group through
`push_building_group` before the eject, the build mask and the queue-up
(`crates/rondata/src/input.rs`). The pool widening reads a building group
(`Sim::pool_building_group`), which it could not see before.

### 31.3 What it moved

**The value diff, both sides** (run285 and this crate):

| block | unit or slot | `group` / held, theirs | ours before | ours now |
| --- | --- | --- | --- | --- |
| 622 | slot 1 | `[2007]`, `buildings 1`, stamp 621 | empty | `[2007]` |
| 856 | `0/10`–`0/12` | 0 | −1 | 0 |
| 856 | slot 0 | `[10, 11, 12]`, stamp 855 | empty | the same |
| 1060 | `0/13`–`0/15` | 2 | −1 | 2 |
| 1060 | slot 1 | `[2007]`, stamp 901 (the press on 900 re-seats it) | empty | `[2007]` |
| 1060 | slot 2 | `[13, 14, 15]`, stamp 1059 | empty | the same |
| 1272 | `0/16`–`0/18` | 1 | −1 | 1 |
| 1302 | slot 3 | `[2007]`, stamp 1301 | empty | `[2007]` |

With the push alone the squads read 1, 0 and 2: the first measure, which
named the building group. **The widening goes 34 → 25 rows and the pool
4 → 8**, all of one mechanism each, and none is the push:

- **25 rows, `Unit::init`'s** (parked 646, not built: it re-pins `form`
  on every birth of both long captures, which is att-880's harness):
  `form` 0 there and −1 here on chapter thirteen's four births and the
  nine trained (13); and the six followers' `orders_x/y`, (2712, 14232)
  there — the Barracks' point (2688, 14208) tile-centred — against the
  point itself here (12), kept only until the follower's own `work` on
  the next block, where both sides read its position. Readers: `form`
  the next layout (`action_move_near`, `action_form`), none in run285;
  `orders_x` `Unit::check_target_path@005e22d0`, and nothing targets
  these squads.
- **8 pool rows, the push record's `o`**: every slot's `ox`/`oy` on its
  first seat (622, 856, 1060, 1302) prints 0 there and −1 here. That is
  `push_group`'s record (`GroupState::default().o`), att-880's fence;
  chapter twenty-three's six `ox`/`oy` rows are the same shape. Its reader is a layout's
  `o`, and no group here is laid out.

Chapter thirteen's three `903 group` rows and chapter twenty-three's
`1442 slot 0 held` close with it, and on the long captures (every row
closing, none opening): Great Lakes' `1/62`..`1/64` on 11424 (69, parked
561) and East Indies' `1/67`..`1/69` on 17363 (69) and `1/70`..`1/72` on
17575 (64), the birth push of §27.5 and §29.3. Both long words hold,
East Indies 20007 and Great Lakes 20568. The word stays **1560,
closed**.

### 31.4 What this has *not* established

- ~~**A command group of two or more buildings.** `process_group` seats
  one group of every listed building; this crate seats one building
  (`Sim::push_command_buildings`, a `SEAM:`). No capture selects two.~~
  **Built and diff-backed by run304** (item 888): one record, in the
  command's order, reused when equal in order — `docs/PRODUCTION.md`,
  "The command on a selection of buildings", and `docs/GOLDEN.md` §37.
- **The gather-point arms** of `come_out` (`BuildData +0xcc`), which
  move the pushed group with `Group::action_move_to@0070fba0` (the calls at
  `6190ce`, `619a04` and `619e98`): no capture sets a gather point.
- **`Unit::init`'s birth values** (parked 646): `form` and the
  tile-centred point, above. Not built; both reach every birth on the
  long captures.
- **The push record's `o`** on a slot no layout has touched: −1 here, 0
  there. `push_group`'s record, not built here.
- **A transport's `come_out`** (vslot `0x20` answering 0): no push, by
  the listing. This crate's disembark is `transport.rs`'s and pushes
  nothing, which agrees; no capture compares it.

### 31.5 Coverage

**Diff-backed**: every `group` row and pool slot of run285 (956 blocks)
and run208 (903), the long captures' 11424, 17363 and 17575 blocks, and
the pinned floors that carry them. The mutation (the push dropped) puts
every one back. **Listing-backed**: `618900`..`6189aa` (the four tests,
the `Group::add` and `push_group` calls and their arguments).
**Decompile-backed**: `Group::add`'s chain walk, `push_group`'s second
walk, and `Unit::init`'s `form` and position writes. **Unit-tested**:
`a_trained_squad_is_pushed_into_a_pool_slot_of_its_own_and_a_single_unit_is_not`.

## 32. A verified line reads the stack again — Great Lakes 20568 → 20800 (item 795, 2026-09-26)

*Established by run294 (`docs/RUNS.md`), a 160-block capture of the four
walkers' departure from the far point, widened whole
(`run294_s_departure_is_widened_whole`), and by the listing at
`5f8c1d`..`5f8c5d` in `Unit::do_move@005f7b30`. Diff-backed on every block
of run294 and run243. The section sits here because the group move is
where it showed; the rule is `do_move`'s (`docs/ORDERS.md` §4.4, whose
pseudocode already had the line).*

### 32.1 The frame, and what the gap held

The word was **20568**: the original's `1/40` stands blocked by the
animal `8/0` on block 20569 (`collide 1`, `collide_who 8`, `collide_o 0`,
`stopped 1`) and ours, three frames behind on the same path, walks. The
lag was whole on run243's first block, 20500, so it came from the gap
(parked 796), and the disk said this much of it before any capture:

- each walker's last collision stamp is ours to the frame (`1/40` 17819,
  `1/41` 18044, `1/42` 18075, `1/60` 19853), so every collision in the gap
  lands on the original's frame, and the draw stream agrees on all of it;
- `1/40`'s move `last` is ours, (3144, 31704): the far point it left from;
- `1/40`'s path entries agree on every block of run243, and its lag is a
  constant three frames there;
- on ours, the four stand at the far point under an `ATTACK` each until
  19853..19883, leave as one group move led by `1/40` (whose grid roll,
  `Unit::do_move+0xe84`, is on 19875 on both sides), and on 19892 a
  follower's formation slot is refused (`1/41`'s, tile (17, 162), the
  city centre `0/2000`'s footprint), which ungroups all three
  (`do_group_move`'s arm 9, `5e838a`); `1/40` then detours by
  (3732, 31380).

`1/41`'s `last` read (3207, 31449) here against (3440, 31381) there, ten
frames apart, so the departure was the place to look, and nothing on disk
held it. run294 was booked for it.

### 32.2 The first parting

On run294 the four walkers stand agreed through the attack. **The first
row to part on the departure is `1/40`'s own, on block 19876** — frame
19875, the leader's grid roll:

| field | ours | theirs |
|---|---|---|
| `order:move.dest` (the waypoint) | (3912, 30984) | (3732, 31380) |
| `tolerance` | 384 | 0 |
| `pos` | (3162, 31687) | (3165, 31692) |

Both path stacks hold the same 54 entries, and **the top of both is
(3732, 31380), `tolerance` 0**: `go_around_building` pushed it and
`find_path`'s recursive check accepted it (`last` = (3144, 31704) on both
sides). The world entry under it is (3912, 30984), `tolerance` 384. Ours
walked at the world entry, past its own detour, for seventeen frames. On
19892 a follower's slot fell on the city centre, and the ungroup, the
re-plan and the detour followed. The original walked the detour first,
and its group move still stands on block 19893 (`MOVEORDER` with its
`GROUPORDER`, `orig` (44851, 22480)).

### 32.3 The writer

The order's `dest_x`/`dest_y` (`MoveOrder +0x2c`/`+0x30`) and the unit's
`+0x60` tolerance, **counted by offset** (823, 869). On the leader's frame
19875 the writers are `do_move`'s three (the `dest == 0` arm's take of
the top, `5f8532`; the first line check's `peek`, `5f8944`; and TAKE's
second `peek`, `5f8c51`), `find_path`'s pull-back
(`local_24 + 0x2c`, only while the waypoint is the goal), and
`resolve_unit_collision` (no collision on this frame);
`do_group_move`'s slot write is a follower's. This crate carried every one
but **TAKE's second `peek`**:

```
5f8c1d  call find_path(top)            ; r2
5f8c26  or   [ebx+0x68], 8             ; r2 == 0: the line is verified
5f8c35  test [ebx+0x68], 8 ; je 5f8c90
5f8c3d  call Stack<PathData>::peek     ; the top, *again*
5f8c51  mov  [edi+0x2c], x             ; dest_x
5f8c57  mov  [edi+0x30], y             ; dest_y
5f8c5d  mov  [ebx+0x60], tolerance
```

`find_path` itself writes no waypoint on the detour arm: it re-pushes the
detour, writes `last` (`+0x34`/`+0x38`) and sets `unit_masks |= 8`. So
what TAKE hands the step is whatever is on top after the check, which is
the detour when one was pushed. Built at the end of TAKE in
`Sim::do_move` (`orders.rs`).

### 32.4 What moved

- **run294**: every row of `1/40`, `1/41` and `1/42` first parting on
  19841..19999 closes (72 keys), and none opens. What parts there is
  `1/60`'s own route (below) and the standing rows of 19840. Among those,
  the three walkers' `order:group.id` (17662401 here, 17662501 there)
  stands from before the departure and still stands. The id is
  `(group + frame × 10) × 100 + order_num`, so it is group 4 here against
  5 there, on the frame (17662) and `order_num` (1) both sides agree on:
  Great Lakes' pool numbering, parked 871. Its readers
  (`still_group_move`, `ungroup_move_order`, `kill_group_move`,
  `group_rewrite_leader`) match it by equality only, so nothing here
  reads its value.
- **run243**: `1/40`, `1/41` and `1/42` close on every block (the 31 rows
  carried onto 20500 and 25 of the 26 up to the old word; the 26th is
  `1/60`'s on 20510), and none opens.
  **The value diff on the old word's block, 20569**: `1/40` reads
  `collide 1`, `collide_who 8`, `collide_o 0`, `stopped 1` and `pos`
  (17256, 26760) on both sides; ours was 0, −1, −1, 0 and (17203, 26773).
- **Great Lakes 20568 → 20800.** East Indies holds at 20782.

### 32.5 The new word, 20800

On 20800 ours spends 36 draws against 35, parting at index 1: ours
`Guy::set_anim+0x97a < Unit::move_step+0x823`, a blocked step, where the
original spends `Animal::think_bird+0x82`. On block 20801 **`1/60` stands
blocked by `1/64` here** (`collide 1`, `collide_who 1`, `collide_o 64`,
`stopped 1`) and walks there. `1/60` is on another route: its world path
parts on run294's first block already (slots 5..50; slot 15 (32640, 24960)
here against (32640, 24192) there), planned before 19840 while it stood at
the far point. That is the gap's again, below run294 (17351..19839). No
mechanism is named.

### 32.6 What this has *not* established

- **`1/60`'s world plan.** Where in 17351..19839 it parts, and which
  `find_wpath` input differs, is on no disk.
- **The other two `dest` writers** (`5f8532`, `5f8944`) were already
  built. This adds the third; no capture has been read for the other two
  since.
- **`group.id`'s group number** (above, 871): not traced to its writer.
- **`resolve_block`** on a top with `flags & 0x10`: TAKE's `SEAM`, not
  reached by any capture.
- Chapter-level: no golden chapter walks a detour on TAKE, as far as the
  gate shows (every golden word holds).

### 32.7 Coverage

**Diff-backed**: every record of run294 (160 blocks) and run243 (319),
and the long word on run53's trace. **Listing-backed**: `5f8c1d`..`5f8c5d`.
**Unit-tested**: `a_detour_the_line_check_pushes_is_the_waypoint` stages
the march's order (`PATHED`, only its `FINAL` entry, no waypoint) against
a barracks across the first leg: the grid roll plans, and the step walks
at the detour with tolerance 0. **Mutation**: with the re-read taken out
on the built tree, the unit test reads (2424, 4728) and 384 against the
detour (1644, 4332) and 0, and run294's `1/40` rows on 19876 come back
(item 795's journal).

## 33. The second pair's widening compares the group record and the attack order's row (item 1061, 2026-09-28)

DECISIONS 54 §2, parked 1062 folded. Until this item the second pair's
widening compared the pool's **lists** and nothing else of `GROUPDATA`,
and no shared site compared the `ATTACKORDER`'s own row. The pair's word
stood on a group's order in a war.

### 33.1 What is compared

`diff::second::widen_records`, on every block of a widening that asks for
it (run356's and run373's), both directions:

- **Every player's 64 pool slots** that either side holds: the list,
  `num`, `army`, `form`, `order_num`, `ox`/`oy`, `o_dist`, `o_angle`,
  `facing`, `form_num`, `speed`, `new_speed`, `stamp`, `buildings`,
  `disband`, `priority`, and each slot's `off`, `curr` and `angle`.
  `role` is compared on an army's group, against `Army::role`
  (`docs/ARMY.md` §3.1). `disband` and `priority` are compared against 0,
  which this crate implies by writing neither. A slot one side holds alone
  is a `held` row. Keys are `(who, -3, "group:<id>.<field>")`.
- **The `ATTACKORDER`'s own row** on every order slot both sides hold as
  an attack: `defensive`, `in_range`, `ever_in_range`, `new_ord`, `def_x`,
  `def_y`, and on the front-most attack `mandatory`, which this crate keeps
  on the unit (`combat::State`). Keys are `(who, o, "attack[<slot>].<field>")`.
- **Each figure's aim**, `GuyData`'s `ox`/`whom`. A building is named by
  its `(owner, index)`, as `widen_block` links it.

`think_frame` is the one scalar left, because this crate does not carry
it.

### 33.2 What it read

On run356 (4550..4806) and run373 (4841..5097), from run347's start:

- **Standing from 4550**:
  - army 0's group (slot 65) `role`, ours 0 against 1379331: this crate's
    `Army::role` is never filled;
  - the Town Center's building group (slot 64) `ox`/`oy`, ours −1 against
    0.
- **4600**: the army group's `curr[*]` parts by one to three units, on the
  block `1/19`'s and `1/20`'s headings part. That family was already in
  the widening.
- **4753** (`1/24`), **4841** (`1/9`'s second attack), **4853** (`1/24`,
  the block after its one-in-five re-search, §70.7 of `docs/COMBAT.md`),
  **4898** (`1/9`), and on to the word: the attack row parts, always one
  shape. Ours reads `new_ord 0, ever_in_range 1` (and `in_range 1` from
  4853), where the original reads `new_ord 1, ever_in_range 0,
  in_range 0`.
- **4924, the word's block**: `1/11`'s head attack reads `in_range 1,
  new_ord 0` here against `in_range 0, new_ord 1`. Its figure aims at the
  citizen `0/2` here and at the Town Center `0/2000` in the original.

No row of the group record parts between 4600 and 4987 but `role` and
slot 64's point, which stand throughout. On 4987 slot 66, a second
group, has its point and angle part, and on 4997 army 0's `facing`, both
past the word.

### 33.3 The compared pin

`coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned`
walks `second::great_lakes_word_window` since this item: run373's
4923..4927, the word's block with two on either side. Before, it walked
the first pair's run202 15383..15387.

Its pinned rows were re-read against that window's dump:

- the attack row's seven fields, the `GROUPDATA` scalars bar
  `think_frame`, `GroupMemberDump` whole, `BuildDump`'s `job_counter` and
  `queue`, `QueueItemDump` whole, and `Guy`'s `ox`/`whom` left the pin;
- the guard order's six fields arrived. No guard stands on these blocks.
  The dump's order blocks there are `UNITORDER`, `MOVEORDER`,
  `TARGETORDER`, `ATTACKORDER`, `ATTACKTOORDER`, `GATHERORDER`,
  `BUILDORDER`, `EXPLORETOORDER` and `FLEETOORDER` only.

### 33.4 What this has *not* established

- **Why the attack row parts.** One reading fits every instance: the
  original's attack is a fresh order where this crate's has struck. That
  is item 1061's part two, `docs/COMBAT.md` §71.
- **`role`**: `Army::role` is not written by `add_group` here. Nothing in
  this crate reads it yet.
- **Slot 64's point**: a building group's `(ox, oy)`, −1 here against 0,
  is `copied`'s default against `Group::clear`'s. Nothing reads it.

### 33.5 Coverage

**Diff-backed**: every block of run356 and run373, pinned in both
widening tests. **Mutation**: see item 1061's journal.
