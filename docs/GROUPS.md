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
  +0x30  priority    int     read by normalize's prune. A one-bit "I am a control
                             group": 1 from HotKeyGroups::clear@00715230,
                             HotKeyGroups::find_group@00714d00 and
                             Object::replace_hotunit@00643b70; 0 from
                             Group::clear@00713e80 and Groups::clear@00713f20
  +0x34  role        int     OR of the members' type roles (find_role)
  +0x38  think_frame int
  +0x3c  new_speed   int     \ the group's march speed, reset every frame by
  +0x40  speed       int     / Groups::process to the leader's own
  +0x44  form_num    int     the member count Form::compute_dests laid out
  +0x48  facing      uchar   the formation's mirror flag (§6.3). Three writers:
                             Group::clear (0), compute_form's toggle/restore
                             pair (§6.3), and — outside this family, and the
                             one no reading found — Unit::kill_current_order
                             @005e2cb0, which writes the dying move order's own
                             reverse flag onto the group when the unit is the
                             group's leader
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

`GroupData::is_on_map@0070c450` is true for a building group with members,
and for a unit group with at least one active on-map captain.

`GroupData::get_form@0070b9f0` returns the `unit +0xaa` shared by every
active on-map member, and **−1 as soon as two differ** — a mixed group has
no formation. `get_form_mod_option` is its width twin at `+0xab`.

`GroupData::get_stance_type@0070d370` seeds from **`find_leader`** for a
unit group and from **`list[0]`** for a buildings group, and reads its
`vslot 0x108` (`get_stance_type`); if that is `STANCE_NONE` or
`STANCE_CASTER` it walks the members and takes the first non-`NONE` that is
not `CASTER`, falling back to `CASTER` if one was seen. `docs/ARMY.md` §6's
`set_stance` acts only on `STANCE_COMBAT` groups.

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
  category. `game/Data/rules.xml` lines 1446–1487 give **ten** formations
  in document order — 0 Line, 1 Refused, 2 Envelop, 3 Echelon Right,
  4 Echelon Left, 5 Sparse, **6 Square**, **7 Wedge**, **8 Column**,
  9 Mob — and `Forms::init@0072e9a0` errors unless the count is 10.
  `compute_rows_and_columns@0072d910` branches on **6** (a square,
  `ceil(sqrt(n))`, line 21) and **7** (a Wedge: rows of 1, 2, 3, …,
  line 136); **8** is handled inside the general arm (line 158), where it
  sets `rows[c] = ceil(count[c]/3)` and leaves `cols[]` uninitialised.
  Otherwise the general arm is
  `span = max over c of (count[c] < 6 ? count[c]·width[c] :
  count[c]·width[c]/2)`, then `cols[c] = clamp(1, count[c],
  ((span · form_mod)/50)/width[c])` and `rows[c] = ceil(count[c]/cols[c])`;
- `compute_dests` walks the members and lays each category's block out
  around the destination, rotating every offset by the formation angle
  through `sin_table`; a **non-captain** is placed relative to its captain
  rather than the block; `is_modern_infantry` scatters by a
  position-derived `% 3`; and the categories are **stacked in rank** rather
  than shifted as one block — walking `0 ..= cat`, subtracting `depth[c]/2`
  between adjacent non-empty categories and `trunc((rows[c] − 0.5)·depth[c]·k)`
  for every category strictly before the member's.

**It is not implemented, and the only reason left is cost.** Both of the
reasons the first reading gave are void:

- **There is no float barrier.** Over the whole `Group` family
  (`00704990`–`00708000`, `00708000`–`0070b9f0`, `0070b9f0`–`0070ea70`,
  `0070f8f0`–`00710000`, `00711540`–`00715400`) there are **zero** float
  instructions. Over `0072cba0`–`0072ed30` there are **fourteen**, and they
  are two identical seven-instruction copies at `0072d00a`–`0072d036` and
  `0072d846`–`0072d872`, both computing `trunc((rows[c] − 0.5f) · depth[c] ·
  k)` with `k = 2 − (x != 0) ∈ {1, 2}`. The constant at `0xb694c0` reads
  `00 00 00 3f` out of the PE — exactly `0.5f`. The whole expression is
  `((2·rows − 1) · depth · k) / 2` truncated toward zero, **integer-exact**
  for anything a 128-member group can reach.
- **A capture pins its output, and it is already on disk.**
  `GroupData::log_data@0045e1d0` writes `off_x`, `off_y`, `curr_x`,
  `curr_y` and `angles` **per member**, plus `form`, `form_num`, `o_dist`
  and `o_angle`, every frame the `GROUPDATA` category is on. run29's window
  carries a four-member army group in formation 0 — `id 66`, frame 15100,
  `off_x = [0, −14, 13, −28]`, `off_y` all zero, `curr = [(0,0), (473,480),
  (−440,−446), (946,960)]` — and `unit_formation_spacing 12` is printed in
  the same dump. §12's ninth check is the diff that reads it.

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
consequence.** The `QUEUE_NEW` clear at `70524f` exempts a shooting siege
unit only when `local_44 == 0` — i.e. only when the army is **not**
hurrying — while the order loop at `7054c7` takes the "leave it alone" arm
when `local_44 == 0 **or** local_48 < 0`, i.e. also when the army *is*
hurrying but `find_city` returned −1. So a **hurrying AI army with no
friendly city near its destination clears its shooting siege unit's orders
and then issues it nothing**: the unit is left standing with an empty
queue. Not obviously intended, and reproduced deliberately in
`crates/sim/src/group.rs` rather than smoothed over
(`a_hurrying_army_with_no_city_strands_its_shooting_siege`).

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
is `cmpl 0x44(%eax)` — **`form_num`, not `num`**. run29's group `id 66`
confirms it: `off_x = [0, −14, 13, −28]` becomes
`curr = [(0,0), (473,480), (−440,−446), (946,960)]`, magnitudes 674, 626,
1348 — `|off_x| × 48` to the sine table's granularity. This is what
`do_group_move` adds to the leader's position each frame
(`docs/ORDERS.md` §8.3).

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
| `Form::compute`'s slot table (§6.4) | where in the formation each member stands | every member is given the **same** destination; the group arrives as a heap, not a line. **Diffable**: `GROUPDATA` logs `off_x`/`off_y`/`curr_x`/`curr_y`/`angles` per member and `form`/`form_num`/`o_dist`/`o_angle` on the group, so closing this seam is checkable against run29 the day it is written — which changes what several rows below cost too |
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

   Then the second reading's seven, each written to fail first and each of
   which did (`docs/audit/2026-08-25-groups.md`, items 26–34):
   `halt_clears_the_group_s_form_and_leaves_every_member_s_own` (§7),
   `stance_cycles_from_the_modal_option_not_the_leader_s` (§8),
   `stance_skips_a_plane_but_not_a_helicopter` and
   `a_helicopter_takes_a_group_move_where_a_plane_does_not` (§7, §8, §6.6),
   `attack_keeps_a_lone_shot_and_a_march_into_range_and_re_orders_the_rest`
   (§10's two sub-arms),
   `the_siege_anchor_scores_only_members_that_are_on_the_map` (§9), and
   `a_hurrying_army_with_no_city_strands_its_shooting_siege` (§6.5's
   clear/order asymmetry).
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

Still open:

- **`Form::compute`'s slot table** (§6.4), the largest gap, and now a gap
  of *work* rather than of evidence. ~~*Capture:* a `UNITS=3
  COMMANDMANAGER=1` dump of a `do_forming` frame.~~ **Closed by the second
  reading:** the output is in `GROUPDATA`, per member, and run29 already
  has a four-member group in formation 0 (§6.4). What remains is writing
  `categorize` / `compute_rows_and_columns` / `compute_dests` and diffing
  them against it.
- **`FormData::type_cat`** (§4.4) decides which member leads a mixed group;
  the simulation takes the group's first on-map captain, which is right for
  a one-type army and wrong for a mixed one. The blind reading read the
  field whole — `FormCatIndex` has 18 values of which `type_cat` returns
  eight, from the PDB's own `LF_FIELDLIST` — so `find_leader`'s key is
  named; what is not established is *which* category each shipped type
  falls in. *Capture:* a `GROUPDATA` frame for an army holding a wagon and
  a hoplite; the leader is the member at slot offset `(0, 0)`.
- **The 18 pool slots above `who*64 + 45`** (§1). Nothing allocates them
  and nothing searches them; `Groups::process` still cycles them. What they
  are for is open.
- **`pathfinder +0x70`** (§6.7) is set to 1 around an army group's
  `find_wpath` and cleared after; what reads it is inside `astar_path`,
  which `docs/PATHFINDER.md` did not reach.
- **`action_guard`** (§9) is 300 lines and unread; `GUARD` is not
  implemented in the simulation either (`docs/ORDERS.md` §14).
- **The formal name of object vslot `+0x1c`.** Its four values are settled
  (Unit 0, Animal 0, Build 1, Wall 1) and nothing here depends on the name,
  but `?is_wallbuild@BuildData@@UBEHXZ` does not appear in `rise_z.map` at
  all, which a value of 1 for `BuildData` would need. Left as the audit's
  one surviving `FABLE:` marker.
- **The blind list this document adds:** `Groups::get_open_slot`'s
  "UH OH, NEED MORE GROUPS" path, `Group::sort`, `refresh_group_order`,
  `distribute_attack`, `kill_group_move` — none of which any traced game
  has executed. `tools/trace/report.py … blind docs/` will list them.
  `distribute_attack` and `kill_group_move` were read blind (§14) and the
  reading is still the only evidence for both.

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

Only the formal name of vslot `+0x1c` is still marked, and nothing depends
on it.

**And one finding that came out of settling marker 3, which no reading
had:** run29 shows live groups with `facing = 1`, which `compute_form`
alone cannot produce in a game with the editor closed. The third writer is
**`Unit::kill_current_order@005e2cb0`** — outside the `Group` family
entirely, which is why the brief never reached it. → §1, §8.
