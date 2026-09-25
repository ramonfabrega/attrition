# Armies — the AI's military state machine

*First reading, 2026-08-25, from the Ghidra export, the PDB type records,
the listing wherever the decompiler dropped a register argument, and the
four traced islands runs of §16 (run21, run23, run24 and its `DUMP_ALL`
windows); the story is in `docs/JOURNAL.md`. The transport audit's army
rows (B.44–B.68, `docs/audit/2026-08-25-transport.md`) are folded in
where they stand. Second reading: §19.*

**What this is.** A computer leader's soldiers are not driven one by one.
Every military unit the AI owns is put into an **army** — one of sixteen
slots per leader — and each army is a small state machine that runs every
256 frames: it gathers at a muster spot beside a city, decides when it is
big enough, picks a target city or fort with a scored search, walks its
groups there in a line, and hands the fighting to the units' own attack
orders once enough of them are engaged. This document is that machine:
the record, the pool, who joins, the cadence, each state, the target
search, and the queries the rest of the AI asks of it. The army's
**transporting** state — how an island AI picks another island — was read
with the sea half and stays in `docs/TRANSPORT.md` §8; it is cited here,
not repeated.

**Confidence.** The pool, the record, the cadence, the counts, the
disband/merge rules, mustering and its release, defending, the
besieged-city and muster-spot searches, and the `Armies` queries are read
line by line: high, and the parts a run reaches are diffed (§16, §17).
`find_target` (§12) is read whole — 1,219 lines of decompile — and its
multipliers are each cited to the expression that carries them; high on
the shape, medium on the exact order of two of the multipliers until a
capture with an army marching on an enemy city exists (§18). The three
places the decompiler dropped a register argument (`vector_dist`,
`find_angle`, `sin_table`, `project`) were settled in the listing and are
marked. Nothing here draws `game_random` except `find_target` (§12) and
the units' own `come_out` (§4), and those draws are named.

**Naming.** Offsets are the PDB's: `struct /rise.pdb/ArmyData` (0x98
bytes), `ArmiesData`, `GroupData`, `CityData`, `FortData`, `LeaderData`,
`Personality`. "Cell" and "tile" as everywhere: a tile is `0xc0`
coordinate units, a cell `0x300` (4 × 4 tiles); a `WCoord` is a cell. An
angle is `docs/MOVEMENT.md`'s signed 32-bit binary angle, north 0, east
`0x40000000`. `get_diff()` is the difficulty the AI plays at
(`docs/AI.md` §2.14: the lobby's in a solo game, `multi_diff` in a
multiplayer one under the semaphore bits); it is inlined a dozen times in
this family and written `diff` below. `age` is `data_encrypted->epoch[0]`
XOR `0x63187`, ~~the leader's current age~~ **the Military library
level, which is not the age** (§20: the age is `+0xdc ages` under
`0x62766`, and only §12's forts pass reads it); `diplos[a][b]` is `LeaderData
+0x74`, 0 war, 1 peace, 2 alliance, and "at war" means `diplos[a][b] == 0
|| diplos[b][a] == 0` (`LeaderData::is_enemy@006ebaa0`), "allied" means 2
both ways or `a == b` (`is_ally@006edb50`).

## 1. Shape of the mechanic

```
ArmyData  (types.txt, 0x98)
  +0x0  valid          short   slot in use
  +0x2  army           short   the slot number
  +0x4  status         int     a bit word, §5
  +0x8  reg            int     the army's region (a city's on init; a sea's for a navy)
  +0xc  role           int     OR of the groups' roles
  +0x10 num_units      int     sum of the groups' unit counts
  +0x14 num_captains   int     units not inside anything (§3.3)
  +0x18 num_standard   int     captains − casters − supply − decoys − AA guns
  +0x1c num_decoys     int
  +0x20 city           int     the city it musters at, −1 once released
  +0x24 navy           int     1 for a navy (docs/TRANSPORT.md §8.3)
  +0x28 human_frame    int     a countdown during which a human's ping drives it (§14)
  +0x2c hurry          int     find_muster_spot found an enemy at the target (§13)
  +0x30 target_o       int     the target object, −1 none
  +0x34 target_who     int     its owner
  +0x38 x, +0x3c y     Coord   where the army is going, in coordinate units
  +0x40 angle          int     the formation's facing
  +0x44 rally_dist     int     0x1200 when a new target is taken, never read (§18)
  +0x48 muster_x, +0x4c muster_y  WCoord  the muster cell
  +0x50 muster_angle   int
  +0x54 list[16]       int     group ids, −1 free
  +0x94 who            short
  +0x96 num_groups     short
```

`status` bits — the PDB's `ArmyStatus` enum (audit A.14: `ARMY_MUSTERING 1,
ARMY_MARCHING 2, ARMY_OFFENSE 4, ARMY_IDLE 8, ARMY_FORMUP 16, ARMY_DEFENDING
32, ARMY_NEED_TRANSPORT 64, ARMY_FORCE_PROCESS 128`), from every writer in the
family (§5–§10, `docs/TRANSPORT.md` §8.1):

| bit | name | state | entered by |
| --- | --- | --- | --- |
| `0x01` | `MUSTERING` | **mustering** | `Army::init` |
| `0x02` | `MARCHING` | **marching** | `do_mustering`'s release; `process` when nothing else is set; `do_defending` on a besieged city; the muster-spot retarget |
| `0x04` | `OFFENSE` | `march_to_target` has issued its orders once | `march_to_target`; cleared whenever the army is not engaged |
| `0x08` | `IDLE` | one tick's rest after `find_target` failed | `do_marching` |
| `0x10` | `FORMUP` | **forming** | a muster spot found; the midpoint step; a target taken |
| `0x20` | `DEFENDING` | **defending** | `do_mustering` on a region the census marks weak, at difficulty < 3 |
| `0x40` | `NEED_TRANSPORT` | **transporting** | `do_mustering` (`docs/TRANSPORT.md` §8.1) |
| `0x80` | `FORCE_PROCESS` | consumed by `process_all` into `process`'s argument | `Armies::update_city` (§15) |

The other enums the family reads are the PDB's too (audit A.15, A.56,
A.57): `strategy[]`'s `STRATEGY_EXPAND 1, ATTACK 2, DEFEND 4, TRANSPORT 8`;
`city_flags`' `CITY_VALID 1, CITY_UNDER_ATTACK 2, CITY_ATTACKING 4,
CITY_EVER_ATTACKED 8, CITY_CAPITAL 16, CITY_ALARM 64`; the object flag
byte's `OBJECT_VALID 1, OBJECT_ACTIVE 4, OBJECT_CITY 32` — so every `flags &
0x20` on an object below asks "is this a city centre", not "a building";
`leader_flags`' `LEADER_VALID 1, ACTIVE 2, HUMAN 4, COOP_SOLO 8, DEFEATED 64,
CAN_TRANSPORT_CIV 256, _MIL 512, _SCT 1024`; `leader_flags2`'s
`LEADER_UNIT_AI_OFF 2, PROD_AI_OFF 4, COMBAT_AI_OFF 8`.

The states are **not exclusive** — `process` (§6) dispatches each set bit's
handler in turn, so an army can be mustering and forming in the same tick
(`0x11`), or forming and marching (`0x12`, the common walking state).

The flow, in the order a game reaches it: the census seeds an army at a
city (`Armies::init_army`, `docs/AI.md` §2.3 step 16) → every military
supply wagon's or hero's idle think joins the nearest (§4) → every
256 frames `Army::process` recounts, disbands or merges, and runs the
state (§5–§6) → mustering until `release_mustering` says so (§7), forming
at the muster spot (§8) → `find_target` scores every enemy city and fort
(§12), `find_muster_spot` picks the cell beside it (§13) → marching walks
the groups there in a line (§9) → once a quarter of the units are fighting
within 0xc00 of the army's point, `engagement` points every group at the
first engaged unit's target (§11).

## 2. The pool

**`Armies::init@006f3c10`**: eight `PtrArray<Army>` (`0x1c` a leader —
`+0x4 count`, `+0x8 capacity`, `+0x10 list`), each preallocated with
**16** `Army`s (`0xa0` bytes each, an `ArmyData` behind a vptr), so
`count == 16` from the start and the dump prints sixteen headers per
leader whether or not any is valid (§16.1). `Armies::close@006f3b80`
frees them.

**`Armies::init_army(who, city)@006f36a0`** — audit B.59, doubly read:
scan the sixteen slots in order; the first with `valid == 0` is taken.
If none is free, the slot with the **smallest `num_units`** is re-`init`ed
over its old contents — `<=` on the scan, so a tie goes to the **later**
slot — and the units it held keep their `group.army` pointing at the
recycled slot until `normalize` (§3.3) or `close` sorts them out. Returns
the slot. It never fails.

**`Army::init(slot, who, city)@006f9000`**: `army = slot; who; city`.
With a city: `reg = city.reg`, `x = city.x`, **`y = city.y + 0x300`** —
one cell south of the city's own point — and the muster cell is that
point's cell: `muster_x = cell(x)`, `muster_y = cell(y)` (`div_3_table[v
>> 8]`, i.e. `v / 0x300`). Without one (`city < 0`): `reg`, `x`, `y`,
`muster_x`, `muster_y` all −1. Then `num_groups = num_units =
num_captains = num_standard = num_decoys = human_frame = role = 0`,
`valid = 1`, **`status = 1`** (mustering), `angle = muster_angle =
rally_dist = 0`, `target_o = target_who = −1`, `navy = hurry = 0`, every
`list[i] = −1`. Returns 0. Run20's frame-1 record is this function's
output for the AI's first city: `x 39264 y 40800, muster 51,53` — the
city at `(39264, 40032)`, `40032 / 0x300 = 52`, so the muster row is one
below the city's (§16.1).

**`Army::close@006f8ea0`**: for every listed group whose `army` is this
slot, `group.army = −1` and `Group::action_halt(group, 0)` — the units
stop where they are; then `valid = 0`, `status = 0`, `human_frame = 0`,
`num_groups = 0`. Nothing else is cleared, which is why a dump can show
`target_o`, `x`, `muster_x` on an invalid slot; the dump only prints valid
ones (§16.1).

**`Armies::init_navy`** is `init_army` plus `navy = 1` and `reg = the sea`
(`docs/TRANSPORT.md` §8.3, audit B.60).

## 3. Membership — groups, units, counts

An army owns **groups** (`GroupData`, `0x9d4` apart in `groups.list`), and
a group owns units (`+0xc num`, `+0x8cc list[128]` of unit numbers, `+0x8
army` the owning slot or −1, `+0x4a who`, `+0x49 buildings`, `+0x34
role`). The army never touches a unit's orders directly: every order it
issues is a `Group::action_*` on one of its groups (§8, §9, §14).

### 3.1 Adding and removing groups

**`Army::add_group(g)@006f8c00`**: refused with an error box when
`num_groups == 16` ("Army has too many groups") or `group.who != who`;
otherwise `list[num_groups++] = group.id`, `group.army = army`,
`num_units += group.num`, `num_captains += group.get_num_cap()`, `role |=
group.role`.

**`Army::remove_group(g)@006f8b50`**: only if `group.army == army` and the
id is in `list`: shift the tail down over it, `num_groups -= 1`,
`group.army = −1`, then `normalize` (§3.3).

**`Army::member(o, who)@006f8de0`**: the unit `o` of leader `who` is in one
of my groups and its object is active. **`Army::member(g)@006f9a50`**: the
group id is in `list`. `Object::get_army@00649d70` is the unit's own
view: its group (`UnitData +0x80`) must list it (`GroupData::member`), and
that group's `army` must list the group; else −1.

### 3.2 Adding a unit — `Army::add_unit(o)@006f9f40`

Nothing if the object is not active or the unit is already a member.
Otherwise, if the army has **no group, or its first group id is −1**: a
fresh one-unit `Group`, `Groups::push_group(who, group, 1)`, and
`add_group`; else the unit is `Group::add`ed to **`list[0]`**, the first
group, and `num_units` grows by the group's count change, `num_captains`
by one. Then `Unit::set_group(unit, group, 0)`. So an army is normally
**one group** with everything in it; a second group only ever arrives
through `add_group` from outside, and §3.3 keeps the biggest first.

**"Everything" is not every figure** (item 557, `docs/GROUPS.md` §23).
`Group::add` opens each step with `get_num`, which normalizes a group of
fewer than four, so a squad whose pointers still name its `come_out`
group joins a small army as its **tail alone**. `set_group` then points
the whole squad at the army anyway. `Army::member`, which is this
function's own guard, reads the **list**; `Object::get_army` needs both
the pointer and the list.

### 3.3 The counts — `Army::normalize@006f9b50`

Zero `role`, `num_units`, `num_captains`, `num_standard`, `num_decoys`.
Then over the groups **from the last to the first**: `Group::normalize`;
if the group now has **no units, or is a building group** (`+0x49`),
`remove_group(it)` and **return** — one removal per call, the rest waits
for the next tick. Else:

```
num_units    += group.num
captains      = group.get_num_cap()             # Group vslot +0x8 (vtables.txt)
num_captains += captains
num_decoys   += count(group, COUNT_DECOYS)
num_standard += captains − count(COUNT_CASTERS) − count(NON_DECOY_TYPE, SUPPLYWAGON 0x3f)
                         − count(COUNT_DECOYS) − count(NON_DECOY_TYPE, AAGUN 0x119)
role         |= group.role
```

`get_num_cap@007145c0` counts the group's active units (`flags & 1`) for
which `UnitData::is_captain@0046ceb0` holds — and that function is
**`return this->o_up < 0;` and nothing else**, the head of an uber squad.
~~a unit not inside another object (not garrisoned, not carried;
`docs/ANIM.md`'s `inside_up`)~~ was this section's gloss until
2026-09-07 and it is wrong: `o_up`/`o_down` (`UnitData +0x8e`/`+0x90`)
are the **squad** chain and `inside_up`/`inside_down` (`ObjectData`) the
garrison one, and the dump prints both under those names in the same
record (`docs/ORACLE.md`, run79, "The chain IS in the dump"). So
`num_captains` is "**squads** on the map, one per squad" — three
Longbowmen are one — and `num_standard` the fighting line: captains less
spellcasters, supply wagons, decoys and anti-air.

The difference is a whole item. Great Lakes' AI army holds six units in
two three-figure squads at frame 7162; the original counts
`num_standard` **2** and this crate counted **6**, so §7's `n < 5` kept
the original mustering and sent this crate marching, and the
`GROUPATTACKTOORDER` the 256-frame tick issues there never went out
(item 261, `run79_s_window_is_every_unit_s_whole_record`). `GroupData::count@00711720` is the
`CountIndex` switch; the cases this family uses:

| `CountIndex` | counts a unit that is |
| --- | --- |
| `COUNT_DECOYS` | active, `unit_masks & 1` (a decoy, `cast_create_decoy`'s bit), and a captain |
| `COUNT_CASTERS` | active, `SubObjectData::is_spellcaster`, and **not** a decoy |
| `COUNT_NON_DECOY_TYPE t` | not (active and a decoy), and `is(t, 0)` — the lineage test |
| `COUNT_TYPE t` | `is(t, 0)`, decoy or not |
| `COUNT_SIEGE` | a unit, its type `is_siege` (type vslot `+0x10c`), and `unit_masks & 0x40001 != 0x40001` — not an AI-controlled packed one |
| `COUNT_ATTACK` | a captain whose type has an attack (`type +0x1e8 != 0`); or a `TRANSPORTBARGE` (0x140) on the map carrying one (`o_inside +0x28`, its owner `+0x3e`) |
| `COUNT_CATEGORY c` | `type +0x14 == c` |

Then the sort: groups are ordered by the **category of their leader
unit's type** (`GroupData::find_leader`, or `list[0]` for a building
group; `UnitTypeData +0x14`, the same field `COUNT_CATEGORY` compares),
**ascending** — the pass swaps a pair when the earlier key is the greater
(audit A.13; the first reading had it descending) — so `list[0]` is the
group whose leader has the **lowest** category, and §3.2 adds every new
unit to it.

### 3.4 Leaving without being removed — `push_group`'s second walk (2026-09-18)

Nothing in §3.1 is the *usual* way a unit leaves an army. This is:

> **`Groups::push_group` kills every member out of the group it was in.**

`0070f9e0`'s second walk, per member, is `if ((-1 < old) && (old != slot))
(*old->vtbl+0x10)(unit.o, unit.who, 0, 0)` — `Group::kill@00714110` through
the vtable (`vtables.txt`) — and only then `unit.+0x80 = slot`. `docs/GROUPS.md`
§3.2 has had the bullet since the first reading; what had never been
written down is **what it means for an army**, because an army's one group
(§3.2) is the thing being emptied.

A unit killed out of the army's group stops being `Army::member` (§3.1) and
stops being counted by `Army::normalize` (§3.3) — `num_units`,
`num_captains`, `num_standard`, `role`, all of it — and, decisively, every
later `Group::action_*` the army issues (§8, §9, §14) walks the group's own
list and so **goes out without it**. The army has no idea it shrank; there
is no `remove_unit`, and nothing in the family notices.

`Group::kill(o, who, 0, 0)` mirrors `Group::add`'s walk exactly
(`docs/GROUPS.md` §4.1, §4.2): a **non-captain is replaced by its captain**,
and a captain's `o_down` chain goes with it. So what leaves is always a
**whole squad**, never a figure.

**Great Lakes 8186 is where it bites.** §12's probe builds a stack group
from the army group's first and last units, `Group::add` brings both
squads, and `push_group(force = 1)` installs it — and the six leave. The
dump says so at a glance: on run97's block 8443 (sim 8442) `1/27`, `1/28`,
`1/29`, `1/40`, `1/41` and `1/42` carry `group 65` with an order `id`
`8192502`, and `1/31`–`1/39` carry `group 64` with `id 8448408`. One `id`
per group order: the army's `GROUPATTACKTOORDER` on that frame reaches
**nine** units, not fifteen.

This crate had all fifteen in the army until item 350 (§17). The cost was
not the probe's own frame — the probe's orders were already right
(`great_lakes_8186_sends_the_probe_s_six_where_the_original_does`) — but
every frame after it: the army's counts were 15/5/5 where the original's
are 9/3/3, and when 8442's `find_target` retargeted, this crate turned six
units the original leaves walking. It is the whole of the residue item 347
had parked as a walk-slot band "from block 8443": the band's frame is the
frame the six were re-ordered, and it fell from 5,853 fields to 1,659 when
they stopped being.

**What it is not.** It is not a *removal*: the group object stays in the
army's `list[16]`, `num_groups` does not change, and `Army::remove_group`
is not called. The army simply issues orders to a shorter list.

## 4. Who joins — `Unit::add_to_army@005f7740`

The one writer of membership outside the family. If `Object::get_army`
already answers, return it. Otherwise pick an army:

- a **sea** unit (`type +0x218 domain == 1`): `Armies::find_army(who, x, y,
  −1, ·, −1)` — any army in the unit's region, no distance cap, no unit
  filter (§15.1); none → give up;
- a land or air unit that is **undamaged** (`damage == 0`):
  `find_local_army(who, x, y, ·, o)` (§15.2); damaged:
  `find_army(who, x, y, 0x2400, ·, o)` — within 12 tiles, filtered by the
  unit (§15.1). None: the nearest **friendly city** within `0x200`
  (`ObjectsData::find_city(SEARCH_FRIENDLY, FILTER_ALL)`) seeds a new army
  there with `init_army(who, city)` — this is the second place armies are
  born, beside the census — and the unit is added without the walk below.

With an existing army: `normalize` it, and if it has units, the unit
**walks to the army's first unit** (`go_to_unit(get_unit(0))`, if that
unit is active) — then `add_unit`. Returns the slot.

**Its callers**, all in the unit AI (`docs/ORDERS.md` §4.5):
`Unit::think@005f6e40:338`, the tail of an AI unit's idle think, where
it is **only a supply wagon or a hero** — `if (!is_supply &&
!is_hero) { if (!(role & 0x10) && !is(SPY, 0)) return; if (get_army() < 0)
think_scout(0); return } add_to_army(this)` at `005f7615` — a scout or
spy takes `think_scout`, everything else returns with no army
(`docs/SCOUT.md` §2; read here as its complement till 2026-08-29,
journal 68); `Unit::think_attack@005f5a80:155`, an idle attacker
that is not a merchant (`0x3d`, `0x3e`, `0x190`), not a caravan and not a
special (`unit_flags2 & 0x10`), when its city is not one the census
marks weak — behind its city search, unreached by any trace;
`think_supply` and `think_hero`, unconditionally;
`think_scout@005f6010:571`, a **sea** unit whose region is scouted;
and `Unit::come_out@00617c10:1614`, below.

### 4.1 `come_out`'s tail — the army coin (2026-09-01)

The fifth caller, and the only one with a draw in it. It is the last thing
`come_out` does, after the exit spot is taken, and the two `is` calls the
decompiler leaves unnamed are settled in the listing
(`llvm-objdump 0x61a0a0..0x61a230`):

```
options->rebuild = 1
if !(unit_masks & 0x40000):                        return   # not AI-driven
if is_caravan():                                   return   # unit_flags2 & 8
if type_index in {MERCHANT 0x3d, MERCHANTDUTCH 0x3e, FURTRAPPER 0x190}: return
if !is_special() and !is(BARK, 0):                          # 0x2b8 & 0x10, 0x143
    join = is(SPY, 0) != 0                                  # 0x3a — no draw
else:
    r    = Random::get(game_random, 0, 0xffff) % (is(BARK) ? 3 : 2)
    join = (game->frame & r) != 0
if join: add_to_army(this)
```

`is_special` is `is(SCOUT)` by the loader (`docs/DATALAYER.md`,
`unit_flags2`), so **the two coin arms are the scout line and the
naval-scout line and nothing else**: a citizen, a soldier or a boat leaves
a building spending nothing, which is why the site went unnoticed for
3,977 frames of East Indies. The `% 2` arm is written `& 0x80000001` with
a sign fixup the draw's own range makes dead. Note the sense: the coin
**joins** when `frame & r` is non-zero, so `r == 0` — half the throws on
one arm, a third on the other — never joins.

**Both arms are diff-backed on run54.** Its 24,000 frames reach the site
eleven times: nine `+0x25ca` (`61a1da`, the `% 2`) under `Object::
eject_contents < Unit::set_new_location`, a passenger put ashore; and two
`+0x25b0` (`61a1c0`, the `% 3`) under `Build::train < Build::finished`,
which is the AI's Bark being trained on frames 10323 and 10465. The two
addresses are the two arms, and that is what names them. Run21's first
`add_to_army` is 5823 — the second of the nine, where `12357 & 1 = 1` and
the frame is odd; run33's 1,850 frames enter neither this nor
`Army::add_unit`.

`crates/sim/src/army.rs`'s `come_out_join_army` is the tail, called from
`garrison::come_out` (the trained unit's way in) and from
`transport::disembark` (the passenger's).

**What is diff-backed and what is not.** The `% 2` arm is: East Indies'
word runs through frame 3978, where the site is spent and named. The `%
3` arm's first appearance is frame 10323, well past the word, so the
*label* `+0x25b0` and the modulus behind it rest on the trace's own
addresses and the listing — not on a run the diff reaches. The predicate
that separates the arms (`unit_flags2 & 0x10`, `is(BARK)`, `is(SPY)`) is a
unit test, `come_out_s_army_coin_is_thrown_by_the_two_scout_lineages_alone`.

**`SpellType::cast_create_decoy@00674370:157`** adds the decoy it creates
to the caster's army; **`Unit::fight@005fd4d0:293`** and
**`Object::do_damage@0064a480:1373`** call `Army::charge` (§14) for an
AI-controlled siege unit's army when it is fought or damaged;
**`City::close@00737550:73`** closes every army mustering at a dying city
and clears `target_o` on every army targeting it;
**`Cities::capture_city@00733380:250`** → `Armies::update_city` (§15.7).

### 4.2 `think_attack`'s head — how the AI's first soldier joins (2026-09-04)

The **sixth** caller, and the one every capture on disk actually reaches
first. `Unit::add_to_army@005f7740` is entered on frame **6612** of run53
(Great Lakes) and **5823** of run54 (East Indies); `Unit::think_attack`
itself on **6612** and **10187**. Run53's is the AI's free British archer
squad, born the frame its first Barracks completes (`docs/CITIES.md`
§4.3), and thirty-eight frames later that squad is marching — which is
what made the site worth reading.

**Who enters `think_attack` at all.** `Unit::think@005f6e40`'s step 3
(`docs/ORDERS.md` §2.4), the arm at `5f70fd`, past the first-idle-frame /
one-in-thirty-two cadence:

```
if is(0x3e, 1):                                        # the merchant lineage
    if (unit_masks & 0x80000) and think_merchant(): done
    if !is_packing_or_unpacking() and think_attack(): done
if type.attack != 0 and (role & 0x10000): think_attack()      # 5f7152
```

`+0x1e8` is `attack` (the base column `ObjectData::attack@006469f0` reads
first) and `role & 0x10000` is the **military** bit. Both halves matter:
an armed citizen has the first and not the second, and taking only the
first put this crate's woodcutters in an army on frame **307** of run53.

**The head, and its five gates** (`llvm-objdump 0x5f5a80..0x5f5db0`; the
decompiler drops the `esi` dance that carries the answer):

```
manual = !(unit_masks & 0x40000)                       # not AI-driven
if !manual and !(leader_flags & 2):   manual = 1       # not in play
if leader_flags2 & 8:                 manual = 1
if manual: goto find_melee_target                      # 5f5c4d — no army
range = 0
if tile(x, y) & 0x100 and cell(x, y).who == who:       # inside my own city's radius
    find_city(x, y, SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)     # answer discarded
    if damage != 0 and ((obj_masks & 0x1020) or healing != 0):
        range = -1                                     # 5f5d10 — stay and heal
    else:
        weak = leader.strategy[my region] & 4          # dead: see below
if role & 0x10:                       range = -1       # 5f5d49 — a scout
if type_index in {0x3d, 0x3e, 0x190} or is_caravan(): skip
elif range >= 0:  army = add_to_army(this)             # 5f5d8c
```

`0x1020` is `MOUNTED | FOOT` (`docs/COMBAT.md` §3), `+0x24` is
`ObjectData::damage` and `+0x38` its `healing`. So the only unit the head
turns away, once it is AI-driven and military, is **a damaged foot or
mounted unit standing inside one of its own cities' radius** — it stays
to heal.

Two readings the listing settles and the decompiler does not:

- **`weak` is dead.** The decompiler prints `if (bVar17) iVar7 = -1;`
  after the join, which reads as a sixth gate. It is not: every path
  through the block reaches `5f5d94`'s `or esi, -1` or jumps past it with
  `esi` already negative, so the argument `find_melee_target` is handed
  is `-1` on **every** path of the function, and the weak-region flag
  changes nothing. The whole `range` variable exists to gate the
  `add_to_army` call and nothing else.
- **The `find_city` answer is discarded.** `local_10` is written `1` at
  `5f5ce8`, *before* the call at `5f5cef`, and `eax` is never read. The
  flag is "I am standing in my own territory", not "a city was found",
  and the call is the original's own dead code. Its only reader is the
  function's tail — `go_to_city`, which fires when the unit is not at
  home, joined no army and found no target, and which this crate does not
  model.

**What this crate carries** (`Sim::think_attack_join_army`, called from
`Unit::think`'s step 3 in `crates/sim/src/orders.rs`): the whole gate but
`leader_flags2 & 8` (no capture sets it), `ObjectData::healing` (not a
field here — it only widens the stay-and-heal arm, and only for a damaged
unit that is neither foot nor mounted), the merchant-lineage entry arm,
and the tail's `go_to_city`. `think_attack_s_head_joins_a_military_unit_
and_leaves_an_armed_citizen` is the unit test.

**What it buys, and what it does not.** With the join, run53's archer
squad is in leader 1's **army 1** — every army being empty,
`find_local_army`'s `90,000,000` and its `<=` hand the last valid slot the
win (§15.2) — and that army's tick is `frame ≡ 250 (mod 256)`, which is
6650. There `Army::do_forming` (§8) issues
`Group::action_siege_attack_to`, and the original spends **one**
`Unit::do_move+0xe84`. This crate spends **three**, because it stands a
group move up as N independent moves (`docs/ORDERS.md` §8.4) and every
member calls `do_move`; in the original only the *leader* does and the
followers take `move_step`. So the word is still 6650, and what stands
there now is `Unit::do_group_move` rather than the missing order.

### 4.3 The walk to the army — `go_to_unit` and `go_to` (2026-09-07)

`add_to_army` is not only a join. Between picking the army and adding the
unit it **orders the newcomer to walk to the army's first member**, and
that limb is the whole of Great Lakes 6994 (§16.7).

```
army = find_army / find_army(0x2400) / find_local_army   # §4, unchanged
if none: army = init_army(nearest friendly city); goto ADD   # 005f7891
normalize()
if num_units != 0:
    u0 = get_unit(0)                                      # ArmyData::get_unit@006f9df0
    if u0 >= 0 and objects[who][u0].is_active(): go_to_unit(this, u0, who)
ADD: add_unit(army, this.o)
```

`get_unit(k)` walks `list` in order, skips a building group, and takes the
`k`-th entry of the groups' own member arrays — so `get_unit(0)` is the
**first unit that ever joined**, not `find_leader`'s pick and nothing on
the muster record. Note the seeded arm's `goto`: an army the unit itself
creates has no first member and issues no walk, which is why run53's
*first* squad (frame 6612, §4.2) spends nothing and its second does.

**`Unit::go_to_unit(o, who)@005f78c0`** — the listing `5f78c0`–`5f79b3`,
because the decompiler loses both `vector_dist` arguments:

```
c = get_captain()                                   # this.o when o_up < 0
while c >= 0: units[who][c].unit_masks |= 4; c = units[who][c].o_down
d = vector_dist(target.x − this.x, target.y − this.y)      # 5f7987–5f7991
if d > 0x480: go_to(this, target.x, target.y, ATTACK_TO, 0, 0x300)
```

The mask walk runs whatever the distance says. `unit_masks & 4` is
`docs/GROUPS.md` §6.6 step 6's "no `GroupMoveOrder`" bit, so a squad sent
this way marches as plain move orders rather than as a formation — and
run84's `1/31`, `1/32` and `1/33` all carry the bit at 6995 while their
orders are plain `ATTACKTOORDER`/`MOVEORDER` records, which is what names
this function rather than any other writer of it.

**`Unit::go_to(x, y, orders, min, max)@005f7a50`** — the listing
`5f7a50`–`5f7b21` for the argument order:

```
if find_nearby_spot(type, x, y, &x, &y, min, max, step 0, angle 0x55555555,
                    FILTER_NOT_ME, this.o, this.who, 0, uber 1, -1, 0, -1) == 0:
    g = Group(); g.add(this.o, this.who, 0, 0)
    slot = push_group(who, g, force 1)
    action_move_to(groups[slot], x, y, QUEUE_NEW, set_angle 0, angle 0,
                   orders, action 0, form −1, width −1, disembark 0)
```

A refusal issues **nothing at all**. `Group::add` on a captain takes the
whole squad (`docs/GROUPS.md` §4.1), so one call moves three figures; the
bias angle is a third of a turn and the rings are an eighth of the span
apart, `min` and `max` being 0 and `0x300`; and because `set_angle` and
`angle` are both 0 the heading is `action_move_near`'s own.

**`uber_unit = 1` is load-bearing**, and it is what a first
implementation gets wrong (`docs/ORDERS.md` §10): it grows the collision
block to `block_radius + ((uber_size − 1) × guy_spacing) / 2 + 0x30`, and
it leaves `bVar17` clear, so the sweep asks `find_unit_with_radius` and
`find_unit_ordered_with_radius` rather than the pairwise pair. Without it
the ring at 192 answers where the original walks out to 288 and the
anchor lands two tiles off — **with the draw stream unchanged**, which is
exactly the kind of error only a coordinate diff catches.

**Great Lakes 6994, end to end.** `1/31`–`1/33` are born on 6993 and
placed by `come_out`; on 6994 the captain's first idle frame takes
`think`'s step 3 into `think_attack` (§4.2), `add_to_army` finds army 1 —
already holding the marching squad — and walks it to `1/27`. The sweep
answers `(41352, 22920)` — the ring at 288, its `k = 0` bearing, and
nothing to do with the army's own destination `(36312, 23352)`; the
formation lays the other
two either side at `(41256, 23016)` and `(41448, 22824)` on one heading;
`Army::add_unit` then moves all three from their `come_out` group into the
marching squad's. `1/32` and `1/33` are stepped later in the same frame
and each pays `do_move`'s grid roll; `1/31`, whose turn the order spent,
pays its own on 6995. Those are 6994's two draws and 6995's one.

**What is diff-backed.** `great_lakes_6994_issues_the_second_squad_s_walk_
to_the_army` reproduces all three destinations, the shared heading and the
order index against run84's own block, and leaves the marching squad's
order point where the original leaves it. The mask walk, the `0x480`
threshold and the `MOVE_TO` arm of `go_to` are read and not run: no
capture on disk holds a joiner nearer than `0x480` to `get_unit(0)`, and
`go_to`'s only other caller, `Unit::go_to_city@005f79c0`, is
`think_attack`'s tail and is not modelled here (§4.2).

## 5. The frame hook and the cadence

**`Game::do_frame@00591ef0:272`**: after `Leaders::strategy_all` and
`GameDaemon::process_all`, unless `semaphore[1] & 8` (the AI-off bit
`docs/AI.md` §2.1 names), `Armies::process_all`. Before the objects.

**`Armies::process_all@006f3b00`**: for each leader with `leader_flags &
1` (in use), `(leader_flags & 0xc) != 4` (not a human without the 8 bit —
`docs/ORDERS.md` §8) and `leader_flags2 & 0xa == 0`: for each valid army,
**`hurry = status & 0x80; status &= ~0x80; Army::process(army, hurry)`**.
The bit is consumed into the argument, which bypasses the cadence.

**`Army::process(bypass)@006f93d0`**, the gate (audit B.45, doubly read):

```
if human_frame: human_frame -= 1                          # every call, before anything
if !bypass:
    s = (army + who × 2) × 2
    if (frame − 30 + s) % 128 == 0:                       # MSVC signed modulo
        if leader defeated (leader_flags & 0x40): return
        normalize()
        if num_captains == 0: return
        if num_standard and count(NON_DECOY_TYPE, GENERAL 0x36): use_generals()
        if count(NON_DECOY_TYPE, SPY 0x3a):   use_spies()
        if count(NON_DECOY_TYPE, SCOUT 0x45): use_scouts()
    if (frame + s) % 256 != 0: return
if leader defeated: return
… the tick, §6
```

So an army ticks **every 256 frames**, on a frame that depends on its slot
and owner — leader 1's army 0 at `frame ≡ 252 (mod 256)`, its army 1 at
250, army 2 at 248 — and its spellcasters get a look **every 128**,
thirty frames earlier in the cycle. Run21's `do_mustering` at 252,
`do_transporting` at 14586 (`14586 mod 256 = 250`, army 1), run24's
`do_marching` at 12024 (248, army 2) and `do_defending` at 15100 (252,
army 0) are this arithmetic. `use_generals`/`use_spies`/`use_scouts`
(§14) each walk the units and give the first hero / spy / special unit a
`Unit::think_spellcaster` turn, stopping at the first that returns 1.

## 6. The tick — `Army::process`, after the gate

In order, each an early return:

1. **Disband.** `normalize()`. If `num_standard < 1` and the army is valid
   and **not mustering**: `city = 0`, then the first active city of mine
   (below `city_num`) — none → `close()`; found →
   `send_here(city.x, city.y, MOVE_TO)` (§14), then `close()` anyway. The
   survivors walk home and the slot is freed.
2. **A human's ping.** If `human_frame != 0`: `send_here(x, y, ATTACK_TO)`
   and return — the AI is suppressed while the countdown runs (§14,
   `Leader::action_ping`).
3. **Merge** (audit B.67, doubly read). If `num_standard < (num_captains −
   num_decoys) / 2` — fewer than half the line is standard: scan my
   sixteen slots for a valid army that is **not a navy**, in **my
   region**, with `num_standard > 4` and `(num_captains − num_decoys) / 2
   <= num_standard` (the same test, passing); the first found takes every
   active unit of every one of my groups — groups walked from the last,
   units within each from the last — by `add_unit`, and I `close()`. The
   test cannot match the army itself.
4. **Retarget the muster point.** If `!is_moving() && !is_engaged()`
   (§11): `status &= ~0x12` (not forming, not marching). Then with a
   target (`target_o >= 0 && target_who >= 0`):
   - the target is **not an enemy, or I am a navy**:
     `find_muster_spot(target_o, target_who, 0)` (§13), `status |= 0x12`;
   - an enemy, land army: **`muster_x = (cell(target.x) + muster_x) / 2`,
     `muster_y = (cell(target.y) + muster_y) / 2`** — the muster cell
     moves halfway to the target — and `status |= 0x12`.
   This is the march: every 256 frames the muster cell halves the distance
   to the target, and `do_forming` (§8) orders every group to it.
5. **`if (status & 0x18) == status: status = 2`** — an army whose word has
   nothing but the forming/no-target bits (0, 8, 0x10, 0x18) is set
   marching. This is what turns the one-tick rest (`0x08`) back into a
   `do_marching` → `find_target` retry.
6. **Dispatch**, each `if` independent: `& 1` → `set_stance(1)`,
   `do_mustering` (§7); `& 0x20` → `set_stance(1)`, `do_defending` (§10);
   `& 2` → `set_stance(0)`, `do_marching` (§9); `& 0x10` → `do_forming`
   (§8), **else** `if is_engaged(): engagement()` (§11); `& 0x40` →
   `do_transporting` (`docs/TRANSPORT.md` §8.2); finally a navy gets
   `set_stance(3)`. `set_stance(s)` (§14) is `Group::action_stance(s)` on
   every group whose `get_stance_type` is `STANCE_COMBAT`.

## 7. Mustering

**`Army::do_mustering@006f4260`** is `docs/TRANSPORT.md` §8.1 (audit
B.51, the `num_captains` field): with `release_mustering() == 0`, a city
that is active and has a muster spot (`find_muster_spot(city.o, who, 1)`)
→ `status |= 0x10` and **return 1** (still mustering, now forming there);
else the expand-and-transport test → `0x40`; else **`status = 2`** — an
army at a city with no muster spot marches. Released and not a navy: the
weak-region test (`strategy[reg] & 4`, `diff < 3`) → `0x20`; the
expand-and-transport test → `0x40`; else `2`. A released navy → `2`. Every
path but the first ends in the common tail: `city = −1`, `x = muster_x ×
0x300 + 0x180`, `y = muster_y × 0x300 + 0x180` (the muster cell's centre),
`angle = muster_angle`, return 1.

**`Army::release_mustering@006f87c0`** — 1 means "go". `c` is my
`cities[city]`, `n = num_standard`:

```
if c.who != who or !(c.city_flags & 1): return 1        # the muster city is gone
if rush_rules and !Game::war_allowed(): return 0         # docs/CITIES.md's pre-war gate
if leader_flags & 8: return 0                            # §18
if n < 2: return 0
for each active leader i that is my ally (is_ally, so i == who counts):
    for each of i's cities below city_mark with city_flags & 3 == 3:      # active and ATTACKED
        if i == who: return 1                            # my own city under attack: go now
        if n > 4: return 1                               # an ally's: go if five or more
if n < city_num: return 0                                # fewer soldiers than cities
if n < 5: return 0
if navy: return 1
p = pop_cap
if p × 7 / 8 <= effective_pop: return 1                  # population nearly capped
if p / 8 <= n: return 1
if p / (2 × num_armies(who, MUSTERING, reg)) <= n: return 1
if diff == 0 and n > 6: return 1
if diff == 1 and n > 10: return 1
if diff == 2 and n > 12: return 1
if pers.rush == −1: return n >= 33
if pers.rush == 1:
    if age < 2: return 1
    if age < 3: return n >= 16
    if age < 5: return n >= 21
elif pers.early_army != 0:
    if age < 2: return 1
    if age < 3: return n >= 16
return n >= 26
```

`age` here is the Military level, `epoch[0]` (`6f8997`, `6f89c3`), as the
glossary above says. The implementation read `ages` until item 657
(§20).

`num_armies(who, mask, reg)@006f3200` counts my valid armies with `status
& mask` and, when `reg >= 0`, that region — here the mustering ones in
mine, so the population share each is expected to reach shrinks as more
armies muster in the same region. The city-under-attack test walks
**every** allied leader's cities including my own; the attacked bit is
`city_flags & 2`, `docs/CITIES.md`'s.

## 8. Forming — `Army::do_forming@006f43c0`

Returns 0 at once if `is_engaged()` (§11). Otherwise the formation's
origin: the muster cell's centre `(muster_x, muster_y) × 0x300 + 0x180`,
**projected** `num_groups × 0xc0` — a tile per group — along
`muster_angle` (`project@0092cf40`, the listing at `6f43e9`–`6f441c`:
`ecx = muster_angle`, `edx = num_groups × 0xc0`; `x' = x +
sin_table(angle, d)`, `y' = y − sin_table(angle + 0x40000000, d)`,
`docs/MOVEMENT.md`'s polar step). Then for each group with units:

1. `group.army = army`.
2. **The stance rule**, shared with §9 and §11: with a target owner that
   is not my ally-slot (`LeaderData +0x8 who`, i.e. not me), at war either
   way, `hurry == 0`, `age < 4` and `count(COUNT_SIEGE) == 0` →
   `set_stance(3)`.
3. `siege = find_building(x', y', SEARCH_ENEMY, who, 0x300, 0x200,
   FILTER_COMBAT) >= 0 && count(COUNT_SIEGE) != 0` — an enemy combat
   building within a cell of the origin, and I have siege.
4. If `siege`: `Group::action_siege_attack_to(group, x', y', ·, ·,
   muster_angle)` — the third and fourth arguments are **never written**
   (`sub esp, 8` at `6f45df`; `action_siege_attack_to@0070d830` reads
   only its first, second and fifth), so the leftover stack is harmless.
   Else if `hurry != 0`, or the target is mine or an ally's and its city
   is active and attacked (`Build +0x72 city`, `city_flags & 3 == 3`):
   `set_stance(0)`, `Group::action_move_to(group, x', y', QUEUE_NEW, 1,
   muster_angle, ATTACK_TO, 1, −1, −1, 0)`. Else (an enemy target, or no
   attacked city): `action_siege_attack_to` as above — the same call,
   siege or not. The friendly test is **`is_ally`** (`6f4559`), so a leader
   merely at peace does not qualify; `target_who == me` skips it.
5. **Step** for the next group: `x' += 0x180 · sin(muster_angle)`, `y' −=
   0x180 · cos(muster_angle)` — half a cell **along** the muster angle.
   The listing's leading `sub eax, 0x80000000` is the first step of
   `docs/MOVEMENT.md`'s sine fold (the far half of the circle negates the
   distance), not a reversal; the first reading read it as one, and both
   second readers (A.36, A.62, A.67) as the fold. So the groups stand in a
   column from the origin onward, lowest category first (§3.3).

Returns 1. Note what it does not do: it never checks the army has
arrived; `process` step 4 moves the muster cell and this issues the
orders, every tick, until `is_moving`/`is_engaged` says otherwise.

## 9. Marching

**`Army::do_marching@006f3df0`**: `count(COUNT_ATTACK) < 1` → `close()`,
return — an army with nothing that can attack disbands. Then the target
is **validated**, and any failure goes to the retarget below.

**Holding no target is itself a failure, and it is the common one.** The
function's third instruction is `iVar4 = field_0x30` and its fourth
`if (iVar4 < 0) goto LAB_006f3fb2` (`006f3df0`+0x1d) — the same label
every validation failure below jumps to — so an army that reaches
`do_marching` with `target_o == -1` goes **straight to `find_target`**
and never touches `march_to_target`. That is not an edge case: it is
what happens on the very tick `do_mustering` releases an army, because
§6's dispatch `if`s each re-read `status` and `do_mustering`'s `status =
2` (§7) leaves `target_o` at `-1`. Release and first target are one
tick, not two. Great Lakes 7930 is that tick for the AI's first army and
the crate spent nothing there against the original's three draws, which
is what item 317 closed (§16.8).

With `target_o >= 0`, `t = objects[target_who][target_o].data()` (vslot
`+0xac`):

- `t.flags & OBJECT_VALID`, else *keep marching* (a dead target is left
  to `find_muster_spot`); and
- `OBJECT_CITY` clear (not a city centre — a fort, or a unit): if the
  owner is not an enemy, the object must answer its "under attack" slot
  (`Build::vftable` → `build_masks & 0x20`; else vslot `0x188`), or →
  **retarget**;
- a city centre with a city (`+0x72 >= 0`): the city must be active; a
  **navy** targeting a non-enemy's city needs it attacked (`flags & 2`);
  and a non-enemy, non-attacked, assimilated city whose building is
  damaged less than three quarters (`damage < hits() × 3 / 4`) is kept
  only if I am not its ally, `weak[target_who] == 0`,
  `strong[target_who] != 0` and not `is_tribute_period(target_who)` —
  otherwise **retarget**.

**Retarget:** `target_o = target_who = −1`. If `status & 0x20`
(defending): `status = 0x30`, return. Else `find_target()` (§12); found →
`status |= 0x10`, `do_forming()`, return; not found → **`status = 8`**,
return (§6 step 5 turns it back into 2 next tick). Then, if still `status
& 2`: `march_to_target()`.

**`Army::march_to_target@006f4d80`** — with a target: if
`!is_engaged()`: `status = (status & ~4) | 0x12` and return — the walk is
§6 step 4 plus §8. If engaged and **`status & 4` clear** — the first tick
of the fight — the formation origin is `(x, y)` stepped **one cell along
`muster_angle`** (`0x300 · sin`, `−0x300 · cos`; listing `6f4daa`–`6f4e2b`,
the same fold as §8.5), `angle = find_angle(x' − target.x, y' −
target.y)` (`6f4e70`) — the direction from the target to the origin
(A.62), the stance rule (§8.2 — here without the `!= me` clause
`do_forming` carries, which `is_enemy`'s diagonal makes redundant),
~~the same siege / move choice as §8.3–8.4 with **`hurry` alone**
selecting the move (`bVar12`)~~ — **the choice is §8.3–8.4's whole**:
`hurry != 0`, *or* the target is mine or an ally's (`is_ally`, `6f4f55`)
and its building's city is active and attacked, exactly the two clauses
`do_forming` tests (corrected 2026-08-25, from the decompile while
`group.rs` was built) — and then per group: a **city-centre target**
(`flags & OBJECT_CITY`, `6f4fd9`) that has lost **nine tenths of its
hits** (`damage < hits × 9 / 10` is **false**, `6f5007`) gets
`action_move_to` to the **building's own position**, `ATTACK_TO`;
anything else gets the §8.4 order at `(x', y')` with `angle`; then the
half-cell step back (§8.5). `status |= 4` so it is not repeated while
the fight lasts. (The first reading had the 90 % test's sense inverted
and applied it to any building, not only a city centre.)

All three of those 2026-08-25 corrections — `is_ally` rather than
`!is_enemy`, the `>=` sense of the 90 % test, and its restriction to
`OBJECT_CITY` — were **ratified** by the group orders' adjudicator, who
re-derived each from the decompile, the listing and the PDB's own
`LF_FIELDLIST` (`docs/audit/2026-08-25-groups.md`, "The `docs/ARMY.md`
carry-over"). One substitution to say out loud rather than leave implicit:
`army.rs`'s `nearly_dead` models "city centre" as
`bd.city.is_some_and(|c| self.cities[c].building == b)`, where the original
reads the object's own `OBJECT_CITY` flag byte. They coincide for
everything the simulation builds; the flag is the original's answer.

## 10. Defending

**`Army::do_defending@006f4070`**: `count(COUNT_ATTACK) < 5` → `close()`.
If not engaged: `status &= ~4`; then, marching with a valid target whose
object is active: a **building target whose city is active and attacked
→ return** (keep it); otherwise drop the target (`−1, −1`), `muster =
cell(x, y)`, `status = (status & ~0x22) | 0x10` — forming where it stands
— and return. Then `find_besieged_city()` (below) → `status |= 2`, return.
Else `find_city(x, y, SEARCH_FRIENDLY, who, ·, 0x200, FILTER_ALL)`:
`target = −1`, **`city = the result`**; found → `status &= ~0x22`, `x, y =
city.x, city.y + 0x300` restricted to the world (`WorldData::restrict`),
`find_muster_spot(city.o, who, 1)` → `status |= 0x10`, else `close()`.
Not found → `muster = cell(x, y)`, `status = (status & ~0x22) | 0x10`.

**`Army::find_besieged_city@006f58b0`** — the city to relieve. Over every
active leader `i` that is me or my ally (both `diplos` at 2), every city
`c` of `i` below **my** `city_mark` (the loop bound is
`leaders[who].city_mark`, not `i`'s — §18) that is active, in **my
region** (a land army) or in a region `is_coast` of my sea (a navy), and
**attacked** (`flags & 2`):

```
v = 1000 × (TOWN 0x19f → 2 | METROPOLIS 0x1a0, FORBIDDENCITY 0x213 → 3 | else 1)   # the city building's type
if navy and no enemy unit within 0x1200 of the city (find_unit SEARCH_ENEMY, FILTER_DOMAIN 1): v /= 4
if i == who: v ×= 10
for each valid army of i, other than me, with city == c and num_standard > 4: v /= 4
v /= vector_dist(|x − c.x|, |y − c.y|) / 0x300 + 1                         # cells from my point
best = max v (>=, so the last equal wins)
```

With a best: a land army takes `angle = find_angle(c − muster centre)`,
`muster = cell(x, y)`, `x, y = c.x, c.y + 0x300` clamped to the world,
`target = (c.o, i)`, return 1; a navy calls `find_muster_spot(c.o, i, 0)`
and sets `x, y` to the muster cell's centre. Run24 enters it at 15100,
after the capital fell (§16.4).

## 11. Engagement

Three predicates over the army's units (`ArmyData::get_unit(i)@006f9df0`
walks the non-building groups' lists in order; `i >= num_units` is an
error box and −1), with the **centre of gravity**
(`center_of_gravity@006f8a20`: the mean cell of the active on-map units,
or `cell(x, y)` when there are none) computed lazily on the first unit
that needs it.

**`is_engaged@006f56d0`**: `normalize()` first. A unit counts if it is
alive (vslot `+0x8`), on the map (`+0xbc`), has a current action whose
`get_type()` is **`ATTACK` (10)** (`docs/ORDERS.md`'s order table), lies
within **`vector_dist(|dx|, |dy|) < 0xc00`** of the army's `(x, y)`
(listing `6f57a3`–`6f57c9`), and within `<= 0xc00` of the centre of
gravity's cell centre (`cog × 0x300 + 0x180`, `6f5815`–`6f5857`). Engaged
when `count > num_units / 4` and `count > 0`.

**`is_moving@006f5470`**: a unit counts if alive, on the map, with an
order list; if its current order's `is_move()` (vslot `+0x14`) is
**false**, it counts unless its cell's region equals the army's `reg`
(a unit standing still *outside* the region is "moving"); if true, it
counts when within `0xf00` of the centre of gravity's cell centre
(`6f568f`). Moving when `count > num_units / 6`.

**`engagement@006f5160`**: the first unit that passes `is_engaged`'s
tests **and whose attack order's target is a map unit**
(`update_action().get_target_order()` — vslot `+0x3c`, `docs/ORDERS.md`
§2 — its `ox`/`whom` both `>= 0`, `ObjectData::is_map_unit`) becomes the
army's target of the moment: the stance rule (§8.2), then
`Group::action_attack(group, ox, whom, 0, QUEUE_NEW, 0)` on every group
with `num_valid() != 0`. ~~No target → nothing.~~ Run24 never entered it
(§16.4): the AI's armies were forming at their muster spots when the
hoplites arrived and `is_engaged` never held.

**The `is_map_unit` test gates the break, not the use** — read from the
listing on 2026-08-26 because the decompiler's locals cannot show it.
`%edi` (`ox`) and `%ebx` (`whom`) are written at `6f531a` and `6f533f` by
**every** unit that reaches the two distance tests and are spilled to
`-0x4(%ebp)`/`-0x10(%ebp)` there; `6f5362`'s `is_map_unit` call decides
only whether `6f5369` jumps out of the loop, and the arm that falls
through to the next iteration (`6f536b jmp 6f5373`) does **not** restore
them. So the tail at `6f5383` accepts whatever the *last* qualifying unit
wrote, on `ox >= 0 && whom >= 0` alone. The rule is: the **first**
qualifying unit whose target is a map unit; failing that, the **last**
qualifying unit's target, map unit or not; and nothing when no unit
qualified at all — **or when the last qualifying unit's target order
carries a negative `ox` or `whom`** (2026-08-28, Fable, from the listing):
`6f5345`/`6f5349` jump straight to the next iteration on a negative
register without restoring the spill, so a qualifying unit whose target
has gone leaves `−1` in the registers and overwrites an earlier
building target. Only the arm that fails the centre-of-gravity distance
(`6f52f6 jg 6f536d`) reloads `%ebx`/`%edi` from the spills.

**And the test is on `get_action()`, not the front order** — `6f51fa`
calls `UnitData::get_action` and `6f520e` compares its `get_type()` with
10, so a unit walking a transit leg *in front of* its attack still counts.
That is the ordinary shape: run29's block 15100 has every member of army 0
holding `[MOVEORDER(pathed), ATTACKORDER]`, and a simulation that tested
the front order found nobody engaged on the one frame that reaches this
function at all.

**Diffed 2026-08-26** (`run29_s_engagement_seeds_its_attack_from_the_first_member_of_the_group_s_list`).
At 15100 the seven members' attack targets are a scatter — who 0's objects
15, 16 and 17 — and the group's `list` order is `[54, 53, 52, 51, 49, 48,
25]`, which is the order `ArmyData::get_unit` walks. The harness's seed is
`o 54`'s target, **object 15**; at 15101 six of the seven members hold 15,
every one has gained the action bit (`flags 0x10 → 0x14`) and `order_num`
has gone `0 → 1`. The seventh, `o 25`, holds 26 — `action_attack` gives
each member `Unit::find_melee_target`'s own nearest and the army's target
is only the fallback (`docs/GROUPS.md` §10), so what the record shares is
the **seed** and not the outcome.

## 12. The target — `Army::find_target@006f69b0`

Called from `do_marching`'s retarget (§9) and `do_transporting`'s no-city
path (`docs/TRANSPORT.md` §8.2). Run21 enters it at 14586 (the latter);
run24 at 12024, the first with an enemy in reach.

**The gate.** `leader_flags & 8` → return (§18). For a land army, the
strength test `strong = count(HOPLITES 0x84) + 2 × count(COUNT_SIEGE) +
count(CATAPHRACT 0xe3) >= 4` — hoplite-line and cataphract-line counted
by lineage, siege double — and, when `age > 2`, `type_avail(SUPPLYWAGON)`
and no supply wagon in the army (`count(NON_DECOY_TYPE, SUPPLYWAGON) ==
0`) **also** blocks; `weak_army = !strong || (that)`. A navy is never
weak. `supply = count(NON_DECOY_TYPE, SUPPLYWAGON)` is kept for later.

**The capital.** `LeaderData::find_capital(&city, &who, −1, −1)`; mine
(`who == me`) → `(cap_x, cap_y)` its cell and `have_capital = 1`.

**The two averages.** Over active leaders: mine and my allies'
**`sea_combat`** (`LeaderData +0x94c`) summed and divided by their count →
`ours`; every leader at war with me → `theirs`. The decompiler's
`puVar16[0x251]` is `+0x94c` from a base the loop keeps at the leader
**plus 8** (`cmp edx, 0xe71af8`, listing `6f6b5d`), so the `[edx + 0x944]`
at `6f6c5d` is `sea_combat` — the field that fits its one use, the
out-of-region discount below (audit B.20; the first reading's "settled:
`combat`" had missed the base).

**Two passes** (`local_5c` 0 then 1); the second only runs when the first
found nothing, and relaxes one multiplier (below). In each pass, over
every leader `L` in 0..8 with `leader_flags & 2` (alive):

*Which leaders qualify.* `L == me` always, straight to the scoring.
Otherwise **my** `defense_mod` (`+0x7a8`, `0x100` = ×1) is the switch
(audit B.22, from the listing — the decompiler inverts the sense):

- **not at war** (`diplos` both non-zero): qualifies only if allied both
  ways, **my** `weak[L] == 0`, `strong[L] != 0`, not a tribute period,
  and `defense_mod >= 0x100` — an ally's cities are scored as places to
  defend, the `attacked` multipliers below making them worth anything;
- **at war**: only if `defense_mod <= 0x100`; and if my `wonderwin_timer
  != 0 || popwin_timer != 0` set **`about_to_win = 1`** (a ×100 below).

(The decompiler's `leaders.list[me].+0x8`, which the first reading called
"my ally-slot" wherever it appears beside the allied test, is
`LeaderData::who` — the test is `L == me`, written twice. So "me or
allied both ways" throughout this section, and note that **`diplos` is 2
on its diagonal**, so `me` passes "allied both ways" wherever that test
is read on its own.)

So above `0x100` the AI considers allies only, below it enemies only, at
`0x100` both. A leader that passes goes through the **difficulty gate**,
whose shape the run26 replay corrected (§16.5): at `diff < 2`, first
`diff != 0 || L is human` and `L.frame_attacked + 0x1c20 <= frame` (7,200
frames since ~~I last took a target against `L`~~ **I last took a target
against `L` or any object of `L`'s last took a combat hit** —
`Object::take_damage` stamps the struck owner at `diff < 2`, `docs/AI.md`
§71) and either
`find_aggressive_army(me)` is this army or, with none, a coin —
**`Random::get(game_random, 0, 0xffff) & 1 == 0`** — and a leader that
passes **goes on** (`goto LAB_006f6dd1`) to the tests every difficulty
applies: skip if `diff == 1` and (`num_captains >= combat / 2` or
`find_aggressive_army(me)` is another army) — reachable, then, only at
difficulty 1; then proceed if `diff < 2`, or `L` is me or allied, or
(`age < 3` and `pers.early_army != 0` and `diff != 2`), or `num_standard
>= 7 − 2 × pers.raid`; then `diff != 2`, or `L.frame_attacked + 0x708 <=
frame` and (`team_style == 3` or `+ 0xe10 <= frame` or `L.attacked_by ==
me` or `my.attacked_by == L`). A leader that fails any of these is
skipped this pass. (The first reading had the `diff == 1` clause under
the `diff >= 2` arm, where it is dead: the decompiler's `else` hides the
`goto`.)

*Team play* (`team_style == 2`): `L` must be the next leader after me in
the start list that is alive and in use, or me, or allied — otherwise
skipped.

*Every city of `L`* below `L.city_mark`, `city_flags & 1`, and for a navy
in a region `is_coast` of mine with `ocean != 0`; then at `diff <= 1`,
**for me or an ally** (allied both ways — `me` included by the diagonal),
skip unless the city's `founder` is me (B.25: on the two easiest
difficulties the AI defends only cities it founded); **an enemy's city
always qualifies**. (The sim had this inverted — an enemy's city skipped
unless I founded it — until run26's replay drew one short, §16.5.) The
score:

```
v = Random::get(game_random, 0, 0xffff) % 200 + 900                  # one draw per candidate
if have_capital and L is an enemy (not me, at war either way):
    if weak_army:                                                     # a weak army only retakes
        skip unless city.founder == me and its building is_unassimilated   # B.27: founder
    v −= 50 × vector_dist(|cell(c.x) − cap_x|, |cell(c.y) − cap_y|) / world.xs   # listing 6f7281
if my pop_issues != 0 and L is an enemy: v ×= 4
if L.wonderwin_timer != 0 and the city has a wonder (num_wonders(c, 0)) and Game::wonder_winning() == L: v ×= 10
if L is not an enemy (diplos both non-zero; me included):
    if L is me or allied both ways:                                   # `me` passes: the diagonal is 2
        if !(city_flags & 2):                                          # an untroubled city, mine or an ally's
            if num_captains < 20 or count(SIEGE) < 2:
                if num_captains < 10 or count(SIEGE) == 0:
                    v ×= (TOWN → 2 | METROPOLIS, FORBIDDENCITY → 3 | else 1)
            else: v = v × (4 − city_level) / 4
            if city_flags & 0x2000: v /= 20                            # §13's "no muster spot" mark
        elif team_style == 2: v ×= 5
    # a neutral gets nothing — and never reaches the scoring
else:                                                                  # an enemy's city
    if team_style ∈ {2, 3}: v ×= 5
    if navy: v ×= 10
    k = by pers.raid: −1 → (TOWN 2 | METROPOLIS, FORBIDDEN 3 | 1)
                       1 → num_captains > 19 and count(SIEGE) > 2 and L.score × 4 / 3 < my score → city_level
                            else (TOWN 2 | METROPOLIS, FORBIDDEN 1 | else 3)     # a raider prefers small cities
                       else → num_captains > 19 and count(SIEGE) > 1 → city_level
                            ; num_captains > 9 and count(SIEGE) → skip the multiplier
                            ; else (TOWN 2 | METROPOLIS, FORBIDDEN 1 | 3)
    v ×= k
if c.reg != my reg and not a navy:
    v /= 2
    if !(leader_flags & 0x700): skip                                   # no transport level: unreachable
    if ours < theirs or ours < 5:
        v /= 2
        if ours < theirs − 2 or ours == 0: v /= 5
if pass 1, or diff > 2:
    if (L is me or ally) and !(city_flags & 2): v /= 3
if (L is me or ally) and city_flags & 2:
    if L == me: v ×= 10
    if city_flags & 0x10 (capital): v ×= 10
elif L is an enemy and city_flags & 0x10:
    v ×= has_preq(GLOBAL_GOVERNMENT_BONUS) ? 4 : (pers.raid < 0 ? 3/2 : 1)
if city.bordering & (1 << me):                                        # CityData +0x65, a bit per leader
    v ×= (pers.raid < 1 ? 4 : 2)
    if L is an enemy and diff > 2:
        if L.frame_attacked + 9000 < frame: v ×= 3
        if c.attack_stamp + 9000 < frame: v ×= 3
if L is not me but allied both ways: v = v × 3 / 4; and if !(city_flags & 2): v /= 2
if about_to_win: v ×= 100
if L == me and the city building is damaged: v ×= 2
elif damaged: v ×= (c.bordering & (1 << me)) ? 3/2 : 5/4
if !(c.was_capital_flags & (1 << me)):                                 # a city I never held
    if L is an enemy and not a navy:
        if age > 2 and diff > 2 and the building is assimilated:
            if count(SIEGE) == 0 and age > 3: v /= 10
            if supply < 2:
                if age > 4: v /= 10
                v /= 10
                if supply < 1: v /= 10
        if L is human (leader_flags & 4): diff == 4 → v ×= 5/4; diff == 5 → v ×= 4/3
else: v = 1,000,000                                                    # a city I once held
for each of my valid, non-mustering armies A already targeting (c.o, L):
    if A is me:
        v ×= (L is an enemy or the city is attacked) ? 8 : 2
        if L == me and the building is_unassimilated: v ×= 8
        if L == me and damage × 2 > hits: v ×= 8                       # my own city, half dead
    elif !(city_flags & 2): v /= 4                                     # another army has it, and it is quiet
    elif L is not me/ally and diff < 3 and not a navy: v /= 4
    else: v ×= 4
if v < 0: v = 1,000,000
best = max v (>=, the later equal wins), with (c.o, L)
```

*Every fort of `L`* — only when the army has **two or more siege** and
`supply != 0`, `L` is an enemy, and over `L.fort_mark` `FortData`s
(`fort_flags & 1`, in my region): `v = (Random::get(0, 0xffff) % 200 +
900) × (L.ages ^ 0x62766 + 1)` (`data_encrypted->ages`, the number of ages
`L` has entered), halved if the fort's building has no city; then the same
"already targeted" walk with ×2 for me, and for another army ×4 if the
building's "is a target" slot answers, else /4; then `v /= (d / 0x300 +
1)` where `d` is the **octagonal distance** from my `(x, y)` to the fort
— `max + min² / (2·max)` for `min < 60000`, else `(max + 2·min) / 2`
(`6f7ff0`ff, the same approximation `docs/PATHFINDER.md` names). Best on
the same `>=`.

**Taking it.** With a best `(o, L)`: if `L != me`, `hurry = 0`. Unless
`diff == 0 && L is human` (the easiest AI never attacks a human):
`rally_dist = 0x1200` if the target changed, `target = (o, L)`; if `L` is
an enemy: `L.frame_attacked = frame`, `L.attacked_by = me`, and a land army
whose region differs from the target's cell region asks the navy —
`Armies::send_navy(me, o, L, my region, its region)` (`docs/TRANSPORT.md`
§8.3). Then `x, y = target.x, target.y + 0x300` clamped to the world,
`find_muster_spot(o, L, 0)` (§13) → `muster_angle += 0x80000000` (face
away from the spot), return; no spot → `close()`.

**The probe.** `diff == 0` against a human: a group of the army's **first
and last units** (`get_unit(0)`, `get_unit(num_units − 1)`), pushed with
`Groups::push_group`; if `L.combat == 0` — the human has no soldiers —
`find_building(target, SEARCH_FRIENDLY, L, −1, 0x200, FILTER_TYPE, FARM
0x1a1)`. **The `−1` is the radius and the `0x200` is the flag word**, not
the other way round (item 328): a negative radius takes
`find_building@0065d260:36`'s *exhaustive* arm — every object, no
distance bound, nearest by **tile**-space octagonal distance, ties to the
last in walk order — and `0x200` is `param_6 & 0x200`, the **region**
filter that confines the answer to the asking point's tile region.
`docs/COMBAT.md` §17.1 has the citation; read the other way round the
search finds no farm at all. Then the farm's `ever_seen |= 1 << me`,
`action_stance(5)`,
`action_attack(farm, L, 1, QUEUE_NEW, 0)`, then `action_move_to` back to
~~the first unit's position~~ **the pair's `GroupData::find_leader`**
(`:1152`, read back at `:1191`; run19's block 8187 puts every member's
`GroupMoveOrder` origin on `1/40` and its `oxx` at 40 — `docs/COMBAT.md`
§17.5), `MOVE_TO`, `QUEUE_LAST`; else
`action_move_to(target, QUEUE_NEW, ATTACK_TO)`. Then `normalize()`,
`L.frame_attacked = frame`, `L.attacked_by = me`. Two units go and poke a
farm.

**No target after both passes**: return with `target` unchanged (−1 from
the retarget); `do_marching` sets `status = 8` (§9).

**The draws, and both addresses.** One `Random::get(game_random, 0,
0xffff)` per candidate city that reaches the score, one per candidate
fort, and one in the difficulty gate's coin — all in the sync stream
(`docs/SYNC.md` §3).

- **`find_target+0x7df`** (return `006f718f`, the call at `006f718a`) is
  the per-candidate score. The listing reads `cltd` / `mov $0xc8,%ecx` /
  `idiv %ecx` / `lea 0x384(%edx),%ebx` — the `% 200 + 900` above.
- **`find_target+0x410`** (return `006f6dc0`, the call at `006f6dbb`) is
  the difficulty gate's coin, read `and $0x80000001,%eax` / `jne` — so
  the leader is considered when the draw is **even**. The `jmp 0x6f6dcb`
  at `006f6dac` is `find_aggressive_army`'s non-negative arm skipping
  the call entirely, which is why the coin is thrown only when nothing
  is already aggressive.

run26's frame 12024 opens with three scores and no coin (§16.5); Great
Lakes 7930 is one coin and two scores (§16.8). Both are
[`sim::army::SITE_FIND_TARGET_COIN`] and `SITE_FIND_TARGET_SCORE` in the
crate and in `rondata::trace`'s table — the draws were always made, and
until item 317 neither side of the diff named them, so the trace spelled
them as bare addresses.

## 13. The muster spot — `Army::find_muster_spot(o, who, flag)@006f5cc0`

`hurry = 0` first. Three parts: the chase, the ring search, the fallback.
The callers and their `flag`: `do_mustering`, `do_defending` and
`do_transporting` pass 1; `process`, `find_target` and
`find_besieged_city` pass 0. *Built and replayed 2026-08-25* — the
paragraphs below are what the build settled, and §16.5 has the replays.

**The chase**, when the target's owner `who` is **mine or my ally** and I
am not a navy: `muster = (−1, −1)`; walk the target building and the
chain after it (`BuildData +0x74 city_down`, the next building of the
city) — for each that is **damaged** (`+0x24`) and has `build_masks &
0x20`: the nearest enemy combat unit within `0x900` of it
(`find_unit(SEARCH_ENEMY, who = me, 0x900, 0x200, FILTER_COMBAT,
FILTER_DOMAIN)`), else an enemy unit within `0x1200` **targeting it**
(`FILTER_TARGET o, who`), else with siege an enemy combat building within
`0x1200`; **found** → `muster = its cell`, `muster_angle = find_angle(muster
− cell(target))` — from the **target**, not the army (listing `6f602d`,
audit B.15) — **`hurry = 1`**, return 1. Not found: `muster` = the
damaged building's cell if unset. After the chain, `muster` unset → the
target's own cell; then `muster_y += 1`, or `−= 1` on the map's last row;
`muster_angle = find_angle(muster − cell(target))`. That muster is
**provisional**: the function goes on into the ring search whatever the
chase left, and a best there overrides it. (The first implementation
returned here, and every own-city search skipped the ring.)

Both enemy lookups are `find_unit`, whose leader loop stops at eight, so
**gaia's animals are not in their search space** and no diplomacy question
is asked about one. Asking it is what the first fuzzed seed panicked on
(`docs/ANIM.md` §6.1, `Sim::nearest_enemy_attacker`).

**The ring search.** Centre `(ax, ay) = cell(x, y)`, the army's point.
`inner`: 5 for a navy, or for a target that is not a city centre
(`OBJECT_CITY` clear); 4 for a city centre whose building
`is_unassimilated`; else `city.get_radius() / 4 + 1`, halved when I have
no siege and the target is not mine — 6 at a level-1 city of my own.
`outer = min(0x40, inner + (3 × navy + 1) × 2)`: two rings past `inner`
for a land army, eight for a navy. Walk the entries
`circle_x/y[circle_radius[inner] .. circle_radius[outer])` —
`circle_init@006817f0`'s tables, `ai_place::circle`: the rings `inner + 1
..= outer`, each in the table's order, `x` ascending and `y` ascending
within it — around `(ax, ay)`; an entry off the map is skipped outright.
For each candidate `(cx, cy)` on the map:

- **not too near another army of mine**: for each other valid slot,
  `vector_dist(|cx − A.muster_x|, |cy − A.muster_y|) **<= 4**` if `who` is
  me or my ally, **`<= 2`** otherwise, ends the candidate's walk before it
  scores; it scores 0 and cannot become the best. **The bound is
  inclusive** — the listing is `cmp $0x4` / `jle` at `6f633a` and `cmp
  $0x2` / the same `jle` at `6f6313`, so a candidate *at* the distance is
  dropped, which the decompiler's `SBORROW4` idiom reads as `< 4` if the
  `==` arm beside it is missed. Written `<` here and in the code until
  item 352, where it was half of Great Lakes 8442's wrong muster (§16.9);
- **the neighbourhood walk**, `move_x/y[0 .. k)` from the cell itself
  outward — `k = 0x31` (the 7 × 7, `world::MOVE_49`, read from the
  executable) with `flag`, 9 (the 3 × 3) without. Every neighbour must be
  on the map, else the candidate is **out**. For a land army, the
  neighbour's own neighbour in the same direction (`n + move[j]` again)
  whose `who` (`WData +0xf`) is not −1, not me, not mutually allied and
  not `who` → out — a third leader's cell (or a `−2`) two, four or six
  cells off, the reach growing with `flag`. Then, for `j < 9` (land) or
  `j < 0x19` (navy) only — the 3 × 3 or the 5 × 5 — the neighbour must be
  in my region (`WData +0x4 == reg`), or, for a land army whose leader
  has any transport level (`leader_flags & 0x700`), in any land region
  (`< 0x41`); its **class**: flag `COAST 0x04` → 3, `FOREST 0x20` → 4,
  `MOUNTAIN 0x10` or `0x40` → 5, `ROCK 0x08` → `((flags & 0x800) |
  0x3000) >> 11` (6, or 7 with `0x800`), else 7 if `is_ocean` and
  `0x800`, else the cell's own `land` byte; and **water** = `is_ocean` —
  not `HALFLAND 0x100`, and `land ∈ {1, 2}`
  (`WorldData::is_ocean@006b4830`; on the islands map the shallows print
  as `SANDY`, 1). A land army is **out** on water and on classes 3, 4 and
  5 — coast, forest and mountain, all three; not "scores nothing on 3",
  which the first reading had — and a navy is out on anything but water.
  The neighbour scores `lands[class].move_rate` (`LandData +0x100`)
  unless its cell is flagged `BUILDING 0x4000`, when it scores 0 — or 10
  for a navy's outer ring (`j > 8`). **`move_rate` is `0x100` for every
  land**: `Lands::init@0067e730` writes it, and `combat_bonus` at
  `+0x104`, as a constant to all nine, and no other function in the
  export writes either — so a candidate scores 256 per non-building
  neighbour, 2304 at most, and the walk order breaks every tie. **This is
  the ring's only source of difference between two admissible cells, and
  it is the whole of what makes an army muster clear of its own town** —
  `BuildType::mask_me@006312a0`'s first write sets the bit on the cell
  holding a building's own position (`docs/CITIES.md` §3.6), and until
  item 352 nothing in this crate wrote it, so every cell of the AI's
  town scored the flat maximum (§16.9);
- the best is by strict `>` against a running best that starts at 0 (a
  candidate whose 3 × 3 is all building cells never wins), and the walk
  stops at the first on-map candidate more than `0x28` entries past the
  best's.

The flag names are `WData::log_data`'s own words, solved from run20's
3,600 cells (`world::cell`); `0x40`, `0x400` and `0x800` have no word in
that dump and are cited by value.

With a best: `muster = it`, `muster_angle = find_angle(muster − (ax,
ay))` — from the **army's** point (listing `6f67aa`); a **city centre of
my own** with a live city: clear the city's `0x2000` bit. **The return is
`local_c`** — the last on-map candidate's verdict, 1 if it completed its
walk and 0 if it went out (a too-near drop leaves it as it was), or 1
after the early stop — **not whether a best was found**: `mov eax,
[ebp-8]` at `6f6824`, on this path and the marking one. A best whose ring
ends on an inadmissible cell returns 0 with the muster set, and
`do_mustering` marches on that 0. No cell: for a land army whose target
is a city centre that is not mine/allied, or that is damaged: `muster =
the city's cell, y ± 1`, `muster_angle` from the target, return 1; a city
centre of my own with a city that is active and not attacked (`city_flags
& 3 == 1`): **set `city_flags |= 0x2000`** — the mark §12 divides by 20 —
and return `local_c`. That is what closed army 1 at 12129 and army 0 at
15100 (§16.5), and the replays name the cell: around Norwich's (45, 48)
every entry of rings 7 and 8 but **(49, 54)** is coastal, water or
forest; at 12129 that one cell is three off army 0's chase muster (51,
52) and dropped as too near, and at 15100 the human, holding the captured
capital, owns it.

## 14. The helpers

**`send_here(x, y, order)@006f98a0`**: clamp to the world; if both differ
from the current point, `muster_angle = find_angle(x − this.x, y −
this.y)` (listing `6f98fc`–`6f9904`); `(x, y)` and `muster = cell(x, y)`
set; then every group with units gets `group.army = army` and
`action_move_to(group, p, QUEUE_NEW, 1, muster_angle, order, 1, −1, −1,
0)`, `p` stepping back half a cell along the reversed angle per group
(§8.5). Callers: `process` (§6 steps 1–2), `Leader::action_ping@006d18a0`
— a **human ally's ping** on the map: the nearest useful army of an allied
AI (`find_useful_army`, §15.3) is sent there with `MOVE_TO` and
`human_frame = 0xa8c` (2,700 frames of obedience).

**`stop@006f9180`**: every group's `action_begin` (vslot `+0x14`), then for
a non-building group `form = −1` and every active on-map unit that is not
a plane (or a plane that is entering/exiting) has `unit_masks &=
~0x4000000`, `+0xc0 = 0`, `close_orders(0)`, `clear_partial_path`,
`update_action`, `unit_masks &= ~0x100`. Called only by
`Armies::leader_defeated` (§15.9).

**`charge(o, who)@006f9a90`**: every group with units:
`action_move_to(objects[who][o].pos, QUEUE_FIRST, 0, 0, ATTACK_TO, 1, −1,
−1, 0)` and `target = (o, who)`. From `Unit::fight` and
`Object::do_damage` for an AI-controlled siege unit's army (§4): a siege
unit under attack drags its army onto the attacker.

**`set_stance(s)@006f8750`**: §6. **`find_waiting_unit(·, ·, filter, ·,
·)@006fa090`**: the first unit of the first group that is active, passes
`Search::valid_filter`, and has **no action or one of type 0**; no caller
in the export outside `Leader::action_attack`.

**`use_generals@006f4c30` / `use_spies@006f4af0` / `use_scouts@006f49a0`**:
walk every unit of every group; the first active hero
(`UnitData::is_hero`, `unit_flags2 & 0x20`) / `is(SPY, 0)` / special
(`is_special`, `unit_flags2 & 0x10`) whose `Unit::think_spellcaster`
returns non-zero ends the walk. Run21 reaches `use_generals` at 15898;
the other two never (§16).

## 15. The `Armies` queries

All scan a leader's sixteen slots in order; `ArmiesData::find_dist` is a
global the finders leave the winning distance in (`docs/AI.md` §2.3 step
16 reads it after `find_army`). "The region" is `WorldData::get_tregion`
of the point's tile.

1. **`find_army(who, x, y, max_dist, ·, unit)@006f34f0`**: valid, in the
   point's region, `vector_dist(|x − A.x|, |y − A.y|)` no farther than the
   best so far and `<= max_dist` if `max_dist >= 0`; with `unit >= 0`, a
   supply wagon (`is_supply`, `unit_flags2 & 0x40`) is only placed in an
   army with **fewer than three** supply wagons (`count(COUNT_TYPE,
   SUPPLYWAGON) < 3`), a hero (`is_hero`) in one with fewer than two
   generals (`count(COUNT_TYPE, GENERAL) < 2`). Nearest wins.
2. **`find_local_army(who, x, y, ·, unit)@006f32e0`**: valid, in the
   region, **`target_who < 0 || == who`** (not marching on someone
   else), and at `diff > 2` or when the army is **mustering**; distance as
   above but **`90,000,000` for an army with no units** (an empty one
   sorts last), `<=` so the later equal wins; the same supply/hero caps.
   The `diff` here is the leader's, inlined.
3. **`find_useful_army(who, x, y, ·, ·)@006f2fe0`**: valid,
   `num_captains != 0`, and either in the point's region or a land army
   of a leader with a transport level; score `dist × (3 if another
   region) / num_captains` — distance per captain, so a bigger army beats
   a nearer one — `<=`. The ping's finder (§14). (`+0x14`, audit B.49;
   the first reading had `num_units`.)
4. **`find_aggressive_army(who)@006f2e10`**: the first valid army with
   `num_captains >= 2` (`+0x14`, B.50), not mustering, whose `(x, y)` cell
   is **not** in the leader's own territory (`WData +0xf who != who`).
   §12's "is this army the one already on the offensive".
5. **`find_city(who, city, ·)@006f3160`**: the first valid army mustering
   at `city`. No caller in the game.
6. **`num_armies(who, mask, reg)@006f3200`**: §7.
7. **`update_city(old_city, old_who, new_city, new_who, ·)@006f2d70`**
   (from `Cities::capture_city`): every army of every in-use, non-human
   leader that has `target_o == old_city` and `target_who == old_who` is
   retargeted to `(new_city, new_who)`, and **its owner's own armies get
   `status |= 0x80`** — the hurry bit `process_all` turns into an
   immediate `process(1)` — when the new owner is that leader. The
   decompiler prints the first two comparisons against `+0x30`/`+0x34`;
   the third write is `+0x34`.
8. **`emergency(who)@006f3250`** — **the city alarm's, and a unit never
   reaches it** (corrected 2026-09-19, item 399; ~~"when an object of an AI
   leader takes damage and the damage was not attrition/friendly"~~ was this
   entry's gloss and it is wrong, as was item 395's "the emergency is
   reached on the golden record's 618", `docs/COMBAT.md` §23).

   The call at `Object::do_damage@0064a480:952` is

   ```
   if (local_30 != 0 && (leaders[param_2].leader_flags & 4) == 0)
       Armies::emergency(param_2)
   ```

   and `param_2` is the **victim**'s owner (`do_damage(A, o, who, …)`,
   `docs/COMBAT.md` §7.1; `this->field_0x9` is the attacker's). The second
   conjunct is `LeaderData::is_human@006ec170`, whose whole body is
   `return leader_flags & 4`, so "non-human" is the **computer** leader —
   that much of 395 stands. **The first conjunct is what fails at 618.**
   `local_30` is zeroed at `0064a8dc` and has exactly **two** writers before
   the test, both inside the **building** branch of `do_damage` (the `else`
   at `0064a9f8`, which resolves the target through vslot `0xac` to a
   `BuildData` and reads `+0x72 city`), and both beside a city alarm:

   - the **non-capital** arm, under `city_flags & 0x10 == 0` (`0x10` is
     *capital*, `docs/CITIES.md` §1.4), `local_34 != 0` and
     `300 < frame − city.attack_stamp` (`CityData +0x14`) — the
     `S_CITY_BEING_ATTACKED` notice;
   - the **capital** arm, the same 300-frame test — `S_YOUR_CAPITAL_ATTACKED`.

   `local_34` is the alarm's damage threshold, set 0 at the top of the
   `attacker != victim` block: with `local_14 == 0` it needs
   `building.damage + dmg >= hits / 4`, or `hits / 10` when the target
   answers type vslot `0x104` or carries `obj_flags & 0x20`; with
   `local_14 != 0` — a **siege** attacker (`type` vslot `0x10c`) against a
   target that answers vslot `0x1c` — it is set unconditionally.

   So the emergency means "**a city of mine is under attack**". `Unit::fight`
   hitting a soldier never reaches it, which is why the golden record's 618
   ticks no army: run110's group pool has group `64` (`army 0`, the three
   hoplites) at **`order_num 0` on every block 616..629**, and `order_num`
   is stepped by every `Group::action_*` (`docs/GROUPS.md` §10). Run24's
   12129 — this entry's original evidence — is a blow on the AI's **capital**,
   not on a unit.

   **What `crates/sim/src/fight.rs` carries**: the building-with-a-city
   conjunct alone. The threshold and the 300-frame cooldown are read-only
   and both only make the emergency *rarer*, so the crate still fires where
   the original would not on a lightly-damaged city building.
   *Falsifier:* a `DUMP_ALL` window over a frame on which an AI leader's city
   building takes a hit under a quarter of its hits, read for whether that
   leader's armies' `target_o` goes to `-1`.

   The body, for the in-use, non-human leader
   with `leader_flags2 & 0xa == 0`, **every valid army** has `target =
   (−1, −1)` and `Army::process(army, 1)` at once — the whole tick, cadence
   bypassed. ~~Run24: 12129, the first hoplite's first blow.~~ Run24's 12129
   is the first blow that reaches a **city building** — see above.
9. **`leader_defeated(who)@006f2f90`** (from `Leader::defeat`): `stop()`
   on every valid army. **`diplo_change(who)@006f30f0`** (from
   `Leader::set_diplo`, after both `diplos` are written): `process(1)` on
   every valid army of an in-use, non-human leader.

## 16. The captures

### 16.1 The record

`ArmyData::log_data@0045f660` (audit B.68) prints, for each leader,
sixteen `length / size / increment` headers (the `PtrArray` — `length 16,
size 16, increment −1`) and, for each **valid** slot, a `BEGIN ARMY`
block: `army who num_groups status reg role num_units num_captains
num_standard num_decoys city navy human_frame hurry target_o target_who x
y angle rally_dist [group × num_groups] muster_x muster_y muster_angle`.
The blocks are children of the `FULL DUMP` block, after the `CONSTANTS`
and before the `Line` records; there is no `ARMIES` parent. Run20's frame
1 has one: `army 0 who 1 status 1 reg 11 city 0 navy 0 x 39264 y 40800
muster 51,53`, every count 0 — `Armies::init_army` from the census's step
16 at frame 0, before any unit joined (§2).

### 16.2 run21 and 16.3 run23

run21 is the islands game to 24000 and run23 its null result (`6000 war
who=1` changed no word: a Quick Battle already starts at war). Their
coverage lists are in `docs/JOURNAL.md`, 2026-08-28, and
`report.py … blind docs/` regenerates them.

### 16.4 run24 — the raid

Run21's lobby with seven human hoplites dropped beside the AI's capital
from the cheat channel (`12000 add hoplite who=0 203,207`, and six more at
12002–16006; the capital is at tile (204, 208), from run20's city record).
The RNG words match run21's through frame 12000 and diverge at 12001; the
hoplites captured the capital at **13125** (`Cities::capture_city`,
`Armies::update_city`) and the AI was defeated at **16488**
(`Leader::defeat`, `leader_defeated`, `Army::stop`, `Leader::victory`,
`Game::end_game_close`). First entries: `do_marching`, `find_target`,
`find_aggressive_army` **12024**; `emergency` and `Army::close`
**12129**; `march_to_target` **12280**; `do_transporting` 12796;
`remove_group` 13466; `do_defending`, `find_besieged_city` **15100**.
Still never: `engagement`, `send_here`, `charge`, `use_scouts`,
`use_spies`, `find_waiting_unit`, `find_useful_army`, `find_city`,
`diplo_change`, `send_navy`.

### 16.5 run25–27 — the windows

`DUMP_ALL` windows of run24's game, staged with `tools/gamelog/window.py`
plus the hoplite lines (`~/.claude/jobs/…/runwin.sh`): **run25**
`[12129, 12132)` around `emergency`; **run26** `[12024, 12027)` around
the first live `find_target`; **run27** `[15100, 15103)` around
`do_defending`. A block `n` is the end of sim-frame `n − 1`; each run
also carries a late block (16007) written when the process was ended,
the sim having run on past the `!quit`. The records
(`tools/gamelog/armyrecs.py`):

- **Run26, block 12024** (before army 2's tick at 12024): three armies of
  the AI — army 0 at city 0 and army 1 at city 1, both `status 17`,
  counts 0, the muster cells `(44,51)` and `(49,54)`; and **army 2 with
  `reg 65`**, the ocean — a **navy** (`init_navy`, run21's 9981), `status
  17`, `city 0`, four units, muster `(56,42)`. **Block 12025**: army 2 is
  `status 18` (marching and forming), `city −1`, **`target_o 2000,
  target_who 1`** — the AI's **own capital**, the building the hoplites are
  hitting — `x 39264, y 40800` (the capital's point one cell south, §12's
  tail), muster `(46,58)`, `hurry 0`. The path: a released navy →
  `do_mustering`'s `status = 2` → `do_marching` → no target → `find_target`
  → the attacked capital (`L == me && attacked → ×10`, `capital → ×10`) →
  `find_muster_spot` (the ring search, a sea cell) → `status |= 0x10`.
- **Run25, block 12129** (before the `emergency` tick): army 0 `status
  17`, **`hurry 1`, muster `(51,52)`**, `muster_angle` SOUTH — §13's chase
  found an enemy at the capital and put the muster cell on it; army 1
  unchanged at city 1; army 2 as at 12025. **Block 12130**: **army 1 is
  gone** — closed in the `emergency` tick (§18); army 0 and army 2
  unchanged, army 2's target re-found (`emergency` had cleared it).
- **Block 16007** (both runs): after the capital fell, army 0 musters at
  what is now city 1 (the old city 1, `(34656, 36960)`) with seven units,
  `status 1`; a new army 1 at a new city 0 in region 5; army 2 still the
  navy, now `target_o 2007, target_who 0` — the human's building that is
  the captured capital — `status 18`.
- **Run27, block 15100** (before army 0's tick at 15100, the capital
  already lost): army 0 at city 1 — **seven units, `status 1`**, mustering
  with no muster spot, muster `(45,48)` (the init's), `muster_angle 0`;
  army 2 the navy, `status 18`, `target_o 2007, target_who 0`. **Block
  15101**: **army 0 is gone.** The window's coverage for 15100 names the
  path — `do_mustering` → `release_mustering` (with `num_armies`, so the
  population-share test was reached) → released into **defending**
  (`strategy[11] & 4`, difficulty < 3) → `do_defending` →
  `find_besieged_city` (none) → the nearest friendly city →
  `find_muster_spot` **failed** → `close`. No `find_target`, no
  `do_marching`. Block 16007: a new army 1 at a new city 0 in region 5,
  seeded by the census; army 0 back at city 1 with seven units.
- **Replayed, 2026-08-25** (`rondata::diff::army_tests::scene_at`: a
  block's own `WORLD` cells, its cities typed from their `BUILDDATA`, its
  leaders' transport bits and its `ARMY` records, and the search run on
  that). **Run22, 3579**: (44, 50) and (49, 54) come out of §13's search
  in the order the game ran it — the capital's army while it was the only
  one (its cell is two from Norwich's init cell, so with army 1 standing
  it would be dropped as too near), then Norwich's with army 0 settled.
  **Run25, 12129**: Norwich's search finds nothing — (49, 54), the only
  admissible entry of rings 7 and 8, is three cells from army 0's chase
  muster (51, 52) — and the `CITY` records show the mark going on,
  `city_flags 0x0001 → 0x2001` between blocks 12129 and 12130 (still
  `0x2001` at 15100, `0x3001` at 16007). The navy's search against its
  own capital lands on (46, 58) with the record's angle once
  `find_target`'s turn-about (§12's tail) is applied. **Run27, 15100**:
  Norwich again, and this time (49, 54) is out at its own cell — the
  human owns it, the capital having fallen; the navy's search against
  the captured capital lands on (46, 58) again, the record's angle plain.
- **Run26, block 12024, `find_target` whole (2026-08-25, later the same
  day).** The trace names the tick's draws: sim-frame 12024's first three
  `game_random` draws are all `Army::find_target+0x7df` — the
  per-candidate `% 200 + 900` — from `0x63ffe763`, which is block 12024's
  own `game_random seed 1677715299`, and there is no coin, so the gate's
  `find_aggressive_army` answered this army. Three candidates on a
  three-city map: the human's Napata (`who 0`, the capital, region 1),
  then the AI's London (`0x481f`: attacked, capital, `damage 12`) and
  Norwich (`0x0001`); the draws fall 1073, 1040, 1059 in that order.
  London's `1040 × 10 × 10 × 2 = 208,000` — attacked and mine, the
  capital, damaged — against Napata's `(1073 − 50 × 64 / 60) × 10 × 3/2`
  and Norwich's `1059 / 3`; the 12025 record has it: `target_o 2000,
  target_who 1`, `rally_dist 0x1200`, `x 39264, y 40800`, muster (46, 58),
  `muster_angle 541917184`, and `angle 292028416` — the block-12024
  `muster_angle`, which `do_mustering`'s tail (§7) had copied into
  `angle` before `do_marching` ran. That tail is the key to the gate:
  the record's point is one cell south of London, **the AI's own cell
  (51, 53)**, where the navy would not be aggressive and a coin would be
  drawn; `do_mustering` had moved the point to the muster cell's centre,
  (56, 42), ocean nobody owns. The replay
  (`run26_s_navy_targets_its_own_attacked_capital_with_three_draws`)
  applies that tail, seeds the sim's stream from the block's word, runs
  `find_target(1, 2)`, and asserts the target, the four written fields,
  the two stamps untouched (my own city: no `frame_attacked`), and **the
  stream three draws on, at the trace's `0xad038188`**. It failed first:
  the sim's diff-≤-1 gate skipped the human's city unless the AI had
  founded it — the inverse of the listing — and drew twice. Two more
  predicates came out of the same reading, neither observable on this
  block: my own untroubled city takes the size factor (the `diplos`
  diagonal), and the `diff == 1` clause is live at difficulty 1.

### 16.6 run28 — the army engaged while mustering

*2026-08-25, with the group orders (`docs/GROUPS.md`).* The path to
`engagement` is narrower than §6 reads at a glance: step 6 dispatches
`do_forming` **or** `engagement`, never both, so an army carrying `0x10`
can never reach `engagement` — and `march_to_target`'s engaged arm does
not clear `0x10`, so an army that arrives at a target and fights does not
reach it either. The one path is **`do_mustering`'s release**: its tail
overwrites `status` whole, so an army that is engaged when its mustering
tick runs comes out of `do_mustering` with `0x20` or `2` and no `0x10`,
`do_defending`/`do_marching` return early on `is_engaged`, and the
dispatch's `else` arrives.

Staged from that reading: run24's game with six more hoplites dropped on
**army 0's own point** — run27's block gives it as `x 34656 y 36960`,
tile (180, 192), seven units, `status 1`, mustering at city 1 — at frames
15020–15030, and the trace on with no dump (three minutes,
`~/.claude/jobs/…/run28.sh`). `Army::engagement@006f5160` is entered at
**15100**, army 0's own tick frame, and frame 15100's coverage names the
whole chain: `process` → `do_mustering` → `release_mustering` (released:
`diff == 0 && n > 6`, seven standard) → `do_defending` → `is_engaged` →
return → **`engagement`** → `GroupData::num_valid` → `Group::action_attack`
→ `Unit::find_melee_target` → `Unit::add_attack_order`. So §11 and
`docs/GROUPS.md` §10's `mandatory == 0` retarget are both executed, and
the frame was predicted from the state machine before the run.

**run29 — the records, and the assertion they carry.** The same scenario
under a `DUMP_ALL` window at [15100, 15103). Army 0's two blocks are the
tick, field for field: `status 1 → 32` (`DEFENDING`), `city 1 → −1`, and
the point from the city's `(34656, 36960)` to **`(34944, 37248)`** — the
muster cell's centre, `45 × 0x300 + 0x180` and `48 × 0x300 + 0x180` —
with `muster (45, 48)` untouched. That is `do_mustering`'s common tail
(§7) whole, and it is what clears `0x10`. Replayed
(`run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_
point`, §17 item 6): the harness's `release_mustering` agrees, its
`do_mustering` produces the record's `status`, `city`, point and angle,
and **no `FORMING` bit survives** — the gate the dispatch's `else` needs.
Made to fail first by ORing `0x10` into the tail. The one input the dump
does not carry is `strategy[reg]`, the census word that picks defending
over marching; it is set from the record's outcome and said so.

**A correction the same check produced.** §16.4 and §18 said `engagement`
had never executed in any traced game. It had: **run16** (the attrition
run, 2026-08-24) entered it at frame 6652. The claim was true of the
islands runs and was never checked against the corpus —
`tools/trace/report.py … blind docs` answers it in ten seconds and was
not asked. The lesson is the tool's own: a blind claim is only blind
against *every* trace on disk.

## 16 (continued) — the Great Lakes captures

The sub-numbers run on: §16.1–16.6 above are the record and the islands
windows, §16.7–16.9 here are Great Lakes'. The break is a `## ` heading
and nothing else — `docs_guard`'s size ceiling is per section and a
session reads one at a time.

### 16.7 run84 — the second squad's own order, and Great Lakes 6994

*2026-09-06, item 249.* Great Lakes' long capture parts at **6994** on two
draws of `Unit::do_move+0xe84 < Unit::do_attack_to+0x11 < Unit::do_job+0x4b`
— `sim::orders::SITE_MOVE_GRID`, the `% 5` grid roll a move spends when
`find_path` refuses its straight line (`docs/ORDERS.md` §4.4) — with a
third on 6995. The obvious reading, and the wrong one, is that they belong
to the **marching** squad: `1/27`–`1/29`, the Archers of §16's run76, the
only units on an `ATTACK_TO` when the frame opens.

run84's `UNITDATA` refuses it. All three open 6994 with `dest = 1` and
`unit_masks & 8` set, and `do_move@005f7b30` reaches the roll from exactly
two states — `dest == 0`, which takes a waypoint and clears the bit, or the
bit already clear. Neither holds. Their records across the frame are a step
and a tolerance arrival and nothing else, and this crate reproduces them to
the unit.

**The draws are the second squad's.** `1/31`, `1/32` and `1/33` are born on
6993 — the frame's three `Guy::init_real < Unit::init < Objects::init_unit`
draws — stand orderless when 6994 opens, and by 6995 each carries an
`ATTACK_TO` of its own to `(41352, 22920)`, `(41256, 23016)` and
`(41448, 22824)`, one shared heading (`-560070656`), a planned stack, and
**the marching squad's group**. A fresh move plans, takes its first
waypoint, clears `line_ok` and is precisely the shape that reaches the
roll; two of the three reach it on 6994 and the third on 6995. The
marching squad's own order point does not move across the frame, so this
was not a group order over all six.

So the seam is an **order this crate never issues**, not a pathfinder that
refuses a line this crate accepts. The membership is already right: by the
end of 6994 this simulation has all six in one army (§4), which is what
makes the gap an order rather than a bookkeeping bug.

~~What is missing is whatever gives a newly produced unit its own move —
`Unit::come_out@00617c10` is the candidate.~~ **It is not `come_out`, and
the three questions this section left open are §4.3's** (2026-09-07, item
250). `come_out` places the squad on 6993 and gives it its group; the
order is `Unit::add_to_army@005f7740`'s own middle limb on 6994, which
walks a joiner to `ArmyData::get_unit(0)` — the army's first member,
`1/27`. So the **anchor** is `find_nearby_spot`'s answer around `1/27`
rather than any point on the army record; the **heading** is
`action_move_near`'s own, `go_to` passing `set_angle 0`; and the two
**tolerances** are one planner and not two — `1/31` still carries the
tile-grid stack `action_move_near` planned, while `1/32` and `1/33` were
stepped later in the same frame, had their straight line refused, and
re-planned on the fine grid inside `do_move`. That is also why two of the
three draws fall on 6994 and the third on 6995.

Pinned by `run84_says_great_lakes_6994_belongs_to_the_second_squad` and by
run53's own membership row (§17); the crate's answer to it — the same
three destinations, the same heading, the marching squad untouched — is
`great_lakes_6994_issues_the_second_squad_s_walk_to_the_army`. The word
went to **7176** with the limb in.

### 16.8 Great Lakes 7930 — release and first target are one tick (2026-09-17)

Item 317; story in `docs/journal/2026-09-17-item-317.md`.

- **7930 is the first `find_target` draw of the whole 24,000-frame
  game**, every one after on `frame % 256 ∈ {250, 248}` — §5's gate,
  leader 1's slots 1 and 2, dated by `report.py <log> when
  Army::find_target`. Chain: `find_target < do_marching+0x248 <
  process+0x42c`.
- **Both sides release on the same tick**, so the muster was never it:
  run92's group 64 is `num` **12 on blocks 7673/7674, 15 on 7676**, this
  crate's `num_standard` 4 at 7674 and 5 at 7930.
- **It is `do_marching`'s empty-target arm** (§9). §6's `if`s re-read
  `status`, so `status = 2` reaches `do_marching` in the same tick with
  `target_o = -1`. With `retarget = target.is_none()` the frame agrees
  **ten for ten, entry for entry** — coin once, score twice (§12), both
  unnamed on both sides until now.
- **run93 checked the value**, `DUMP_ALL` at [7929, 7933). Block 7931:
  `status 18`, `city -1`, **`target_o 2007, target_who 1`**, the AI's own
  building; `x, y` stay **41568, 25440** — `find_target`'s tail, not the
  muster centre `do_mustering` wrote. Field for field here, `role`
  excepted (§18).

**Word 7930 → 8030**; 8318 frames draw for draw (8193). Successor: 8030.

### 16.9 Great Lakes 8442 — the muster cell, off a record that names no army (2026-09-18)

Item 352; story in `docs/journal/2026-09-18-item-352.md`.

**How an army's muster is read out of a dump that has no `ARMYDATA`.**
run97 block 8443 gives every member of group 64 a `GROUPATTACKTOORDER`
whose `MoveOrder::orig_x/orig_y` is **(44851, 22480)** for all nine — the
group's own destination before each unit's formation slot. §8's
`do_forming` builds that as `cell_centre(muster)` stepped `num_groups ×
0xc0` along `muster_angle`, and §12's tail has already added `0x80000000`
to that angle, so the point is one tile *back* from the muster's centre.
Swept over every centre and every muster cell on the map, **(44851,
22480) is `(58, 29)`'s and no other's** — a 192-unit residual pins the
cell exactly. The army's own point is London's `(target.x, target.y +
0x300)` → cell (55, 22), which makes (58, 29) offset `(3, 7)`, entry 26
of ring 7. This crate was mustering at `(50, 27)`, entry 10.

**Two predicates of §13's ring, and they compound.** The score is 256 a
neighbour with nothing to separate two open cells, so the *first* maximum
in walk order wins (`local_3c < local_18`, `jle` at `6f6720` — strictly
greater, as written):

- the same-owner spacing bound is **inclusive** (`jle`, `6f633a`). Army
  1/2's ring had a candidate exactly four cells from army 1/1's muster and
  kept it; with the bound right that army moves to (51, 26) instead, which
  is one cell from (50, 27) and takes (50, 27) out of army 1/1's ring;
- **`mask_me`'s `0x4000`** was unwritten, so the AI's own town scored
  flat. With it, ring 7's `y = 29` row scores 1792, 1280, 1024, 1536,
  1792, **2048** from x = 53 to 58 — the buildings at (54, 29), (55, 29),
  (54, 30), (55, 30), (56, 30) and (58, 30) — and the easternmost cell,
  the one with a single building beside it, is the best. Neither fix alone
  gets there: with only the spacing bound the muster is (50, 27) still,
  and with only the flag it is (55, 29).

**The value diff.** All nine of `1/31`–`1/39` stand on block 8443's own
`x_internal`/`y_internal` and hold its own path goal on the frame,
`1/37`'s ten-segment path included;
`great_lakes_8442_musters_the_army_where_the_original_does` is that
record. run97's **walk-slot band is empty** below the word — 1,747 fields
to 0 — and its point-and-goal residue 7,009 → 601, on eight units rather
than sixteen. **Word 8628 → 8663**; the successor is `Unit::do_move+0xe84`
against `Farms::inc_time+0x1ae` at 8663.

## 17. What the simulation carries, and what checks it

`crates/sim/src/army.rs`: the record and the pool (§2 — `init_army`'s
slot rule and eviction, `Army::init`'s muster cell from the city, `close`),
the counts as a function of the groups' units (§3.3), `add_to_army`'s
army choice **and its walk to `get_unit(0)`** — `army_get_unit`,
`go_to_unit`, `go_to` and `find_nearby_spot_squad` (§4.3) — plus
`find_army` / `find_local_army` / `find_useful_army` /
`find_aggressive_army` / `num_armies` (§15) over the sim's own units, the
cadence (§5), the tick's disband / merge / muster-midpoint / status
normalisation (§6), `release_mustering` and `do_mustering`'s table (§7),
`do_transporting` (`docs/TRANSPORT.md` §8.2, now that the army exists),
`emergency` / `update_city` / `leader_defeated` / `diplo_change` (§15),
`find_target`'s scoring over cities (§12) with its draws on the sync
RNG and the `0x2000` mark's division, and `find_muster_spot` whole (§13)
— the chase, the ring search over the loaded map's cells (`World`'s
`CellData`, `world::cell`, `world::MOVE_49`, `World::is_ocean`) and the
mark (`City::no_muster`). The census's step 16 (`ai_census.rs`) and
`create_units`' `seam_army_at` (`ai_units.rs`) are wired to it.

**The order-issuing half landed 2026-08-25** with `docs/GROUPS.md` and
`crates/sim/src/group.rs`: `do_forming` (§8, with §8.2's stance rule,
§8.3's siege test and the projected formation origin), `march_to_target`
(§9, the engaged arm and `MARCHED`), `engagement` (§11), `send_here` and
`charge` (§14), `set_stance` (§6) and `Army::close`'s halt. The army's one
group carries the live half of its `GroupData` as `army::Army::group`
(`group::GroupState`). What still writes the record and moves nothing is
`find_besieged_city`'s **navy** arm alone.

**And the one way out of an army landed 2026-09-18** (§3.4, item 350):
`Sim::push_group` calls `unseat_group`, which kills each pushed member's
whole squad out of the army group that held it and re-runs
`army_normalize`. `a_pushed_group_takes_its_members_out_of_the_army` is
the unit check and `run97_s_window_clocks_are_the_original_s` the
differential one — the latter now compares each dumped unit's **point and
its leading move order's goal** beside the four clocks, which is what
found this. The formation's slot table is
`docs/GROUPS.md` §6.4's seam: every member is sent to the group's own
destination.

The seams — every place the module stands in for the original — are one
table at the top of `army.rs` (`army::seams`). The ring search was the
first a capture reached (§16.5), and is a seam no longer.

Checks, cheapest first (`rondata::diff`, `army_tests`):

1. **Run20, on disk** — `run20_s_first_army_is_the_census_s_init_army_whole`:
   frame blocks 1–3's `ARMY` record against the harness's `init_army` at
   the census's step 16, **every field** (block 4 is the quit's, empty).
   Passes.
2. **Run22, on disk** —
   `run22_s_two_armies_are_init_s_records_with_the_ring_search_s_muster_cells`:
   block 3579's two records are `init`'s (`x, y` from the two cities, the
   counts zero, `status 17`), and their muster cells are the ring search's
   — moved from the init's, four cells apart. Asserted from the dump
   alone; the harness cannot reach 3579.
3. **Run25–27**: the `ARMY` records across `emergency`, `find_target` and
   `do_defending` — read into §16.5; ~~the assertions they can carry
   (`status`, `target_o/who`, `x/y`, `hurry`) wait on a harness that can
   stage a frame-12000 state, which is item 13 of the queue~~ — the
   scene loader (item 4) is that harness for one function of one block,
   and run26's `find_target` (item 5) is the first of them asserted.
4. **The ring search on the blocks' own maps** —
   `run22_s_muster_cells_are_the_ring_search_s_on_block_3579_s_own_map`,
   `run25_s_emergency_search_finds_no_cell_at_norwich_and_the_navy_re_finds_46_58`,
   `run27_s_defending_search_finds_no_cell_at_norwich_again`: §16.5's
   replays, four searches that find and two that fail, cells and angles
   equal to the records. `scene_at` is the loader: one function of one
   frame, not a replay of the frame — the first time a `DUMP_ALL` window
   block has fed the harness at all. Passes. Beside them the unit tests
   in `army.rs` pin what no block reaches: the walk order, each class
   exclusion, the `flag`'s widening, the spacing, the return quirk, the
   mark's set and clear, the fallback, and a navy's 5 × 5.
5. **`find_target` on run26's block 12024** —
   `run26_s_navy_targets_its_own_attacked_capital_with_three_draws`
   (§16.5): the loader now carries the block's frame, its sync word
   (`sim.rng`), the diplomacy table, the `LEADERDATA` words §12 reads and
   each city building's damage; `do_mustering`'s tail is applied by hand
   (the trace's call chain), `find_target(1, 2)` runs, and the target,
   `rally_dist`, the point, the muster cell and angle, the untouched
   stamps and the stream's word three draws on are the record's and the
   trace's. Passes; failed on the draw count before the diff-≤-1 gate was
   corrected.
6. **`do_mustering`'s release on run29's blocks 15100/15101** —
   `run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_point`
   (§16.6): the two blocks either side of the tick that put army 0 on the
   path to `engagement`; the sim's `do_mustering` reproduces the record's
   `status 32`, `city −1`, point and angle, and asserts the absence of
   `FORMING`. Passes; failed first when the tail ORed `0x10` in.
   `scenes()` parses one 250 MB window once for both blocks.
7. **Great Lakes 6994's own membership** —
   `run84_says_great_lakes_6994_belongs_to_the_second_squad` and the
   membership row inside
   `run53_s_24000_frames_put_the_ceiling_where_run33_did` (§16.7): the
   dump's three new `ATTACK_TO` orders, their anchor, heading, group and
   planners; and, on the long run, that this crate has all six archers in
   one army by the parting frame. Both pass; the first was failed by
   pointing it a frame later, the second by adding a citizen to the six.

## 18. What is not established

- ~~**Which city Great Lakes 8442 picks, and where the army musters on
  it**~~ — **settled by item 352 (§16.9)**, and without an `ARMYDATA`
  record: the order's `orig` reconstructs the muster cell uniquely through
  §8's one-tile step, and the answer is **London** (as this crate already
  had) mustering at **(58, 29)**. §13's ring was wrong on two predicates —
  the inclusive spacing bound and the unwritten `mask_me` flag — and the
  nine units now stand on run97's own coordinates on the frame.
  Still *not* in this crate's §12, still unchecked, and no longer
  blocking anything: the **wonder** clause (`num_wonders(city) != 0 &&
  Game::wonder_winning() == the owner` → `× 10`, `006f69b0`), and the
  own-city factor taken from the city **building's type** — `0x19f → 2`,
  `0x1a0`/`0x213 → 3`, else 1 — rather than from `CityData::get_level` as
  `size_factor` does. Neither is live at 8442 (London scores 347 against
  Norwich's 328, each `rnd % 200 + 900` under §12's `/3` with
  `size_factor(1) = 1`), so the choice below the word does not depend on
  either. *Capture:* a lobby whose AI holds a Large City with a wonder,
  where the two clauses disagree by a factor and the chosen target is the
  observable.
- ~~**Whether `mask_me`'s `0x4000` is written where the original writes
  it.**~~ **Two captures already say it is, cell for cell**, and both were
  on disk: run13's frame 95 pinned the missing bit as its *one* exception
  over 3,600 cells and now has none, and run93's block 7932 — the only
  mid-game world dump on disk for Great Lakes, 7,932 frames and every
  building the AI has started — went from 27 parting cells to **12**, the
  whole `0x4000` half of them gone.
  What those two do *not* reach is the **unmark**, and the placement paths
  this crate does not route: `mask_me` is also called from the scenario
  loader and from `Wall::mask_me`'s other callers, and a demolition is the
  only thing that clears the bit in a skirmish. **The capture that would
  refuse it**: a `WORLD ≥ 5` block after the AI demolishes or loses a
  building — `grep` its cells for `0x4000` and the flagged list should be
  the live buildings' cells exactly. Neither capture on disk contains a
  building death before its world block.
- ~~**Which target Great Lakes 7930 actually picks**~~ (item 317,
  §16.8) — **run93 settled it**: `target_o 2007, target_who 1`, the AI's
  own building, and this crate's record agrees field for field. The
  successor frame, **8030**, is still open: this crate spends one extra
  `Unit::do_move+0xe84` there — the marching army's own walk, one frame
  out — and the target is now ruled out as its cause.
- **`do_marching`'s inline `do_forming`, once or twice.** The original's
  `LAB_006f3fb2` calls `do_forming` itself before returning (§9), and
  `Army::process` then calls it again on its own `status & 0x10` arm —
  so on this path the original runs it **twice** and `army.rs` once,
  leaving it to §6's dispatch. It makes **no draw difference at 7930**
  (the frame agrees entry for entry with the single call), so nothing
  observed separates the two and the crate was left as it is rather than
  changed on a reading alone. The check that would settle it is a frame
  where `do_forming` draws — its `find_muster_spot` ring (§13) — reached
  from a retarget.
- ~~**`leader_flags & 8`**~~ — the PDB names it `LEADER_COOP_SOLO` (audit
  A.56): a human seat the AI plays for; `production_ai` runs for such a
  leader and the armies do not (B.16). Never set in a skirmish.
- ~~**The `combat` average's field.**~~ `sea_combat`, from the listing's
  base register (B.20, §12) — the first reading's own "settlement" was the
  error.
- ~~**The order a newly produced unit comes out with** (§16.7).~~ Not
  `come_out` at all: `add_to_army`'s walk to the army's first member,
  built 2026-09-07 (§4.3), and the word went 6994 → 7176. `come_out`'s
  own three `Group::action_move_to` sites remain unmodelled and unreached
  — the trained squad's **group** does come out of `come_out` (its
  `Group::add` + `Groups::push_group` pair, guarded by the type's squad
  size) and its **order** does not, which is what run84's block 6994
  shows: group 66 already set, no order yet. Left from the build:
  - `go_to_unit`'s `0x480` threshold and `go_to`'s `MOVE_TO` arm are read
    and never run — every joiner on disk is further than `0x480` from
    `get_unit(0)` and every walk is an `ATTACK_TO`. *Capture:* a joiner
    born beside its army, which a Barracks next to a mustering squad
    would give;
  - `go_to` has exactly two callers in the export, `go_to_unit` and
    `Unit::go_to_city@005f79c0`, and the second is `think_attack`'s tail
    (§4.2) — unmodelled here, so `go_to` has one caller in this crate.
- **`find_city`'s index in `do_defending`** (A.73): the search returns a
  per-leader city index and `do_defending` resolves it against the army
  owner's list; whether `SEARCH_FRIENDLY` can hand back an ally's index is
  `Search::valid_search`'s, unread. The sim searches the owner's cities.
- **`use_generals` / `use_scouts` gate on `is(GENERAL)` / `is(SCOUT)` and
  test `is_hero` / `is_special`** (A.80). Vacuous by the loader:
  `docs/DATALAYER.md`'s `unit_flags2 & 0x20` is `is(GENERAL)` and `& 0x10`
  `is(SCOUT)`.
- ~~**The ring search is what decides whether an army survives its first
  tick under attack.**~~ Built and replayed 2026-08-25 (§13, §16.5): the
  failing search at 12129 is the same-owner spacing rule against army
  0's chase muster, and at 15100 the human's territory on the one
  admissible cell; the harness now fails exactly where the original did.
  Left from the build, none observable in the four blocks:
  - the **return quirk** (§13, listing `6f6824`) — a best found with the
    ring's last on-map entry inadmissible returns 0. *Capture:* a
    `do_mustering` tick whose army keeps a moved muster and yet leaves
    mustering with `status 2`; none of the windows has one, and the unit
    test is the only evidence past the listing;
  - the region test's `< 0x41` admits the first sea region's number for
    a transport-level land army; the sim tests the terrain. A sea-region
    cell is water and out for a land army anyway, so nothing can differ
    unless a sea region carries `HALFLAND`;
  - a cell whose `who` is `−2` is "another leader" to the walk (the
    diplomacy read lands before the table); the sim treats
    `Owner::Ambiguous` so. No dump has shown a `−2`;
  - the class of a cell with none of the four flags is its `land` byte,
    0–3 on every map so far, and the score is the same for any class.
- **`find_besieged_city`'s loop bound** is the **calling** leader's
  `city_mark` for every ally's list (§10). Read twice, from
  `leaders.list[who].+0x408` with `who` the army's; either a bug in the
  original or a deliberate cap. *Capture:* run27's block, an ally with more
  cities than the AI — not this lobby's.
- **`rally_dist`** is written (`0x1200` on a new target) and never read
  in the family or anywhere the export shows. Dumped, so diffed, so kept.
- **`role`** is the OR of the groups' `GroupData::role` words; nothing in
  the family reads it — and, **unlike `rally_dist` above, this crate does
  not carry it**: `army_normalize` writes a flat `0` because
  `group::GroupState` has no role word to OR. run93's block 7931 is the
  first whole-record `ARMY` diff to reach it and `role` is its one
  skipped field (item 317); the original has `1379331` there, which is
  group 64's own `role` in run92's pool. Carrying it means giving the
  group state a role word, which is a `docs/GROUPS.md` job and not an
  army one.
- **The order of two multipliers in §12** — the enemy-capital
  `GLOBAL_GOVERNMENT_BONUS` block and the `bordering` block — is as the
  decompile lists them; the listing was not read for these two, and the
  product is order-independent unless a division truncates between them.
  *Capture:* run26's `ARMY` records give the chosen target, not the score;
  a `LEADERS=9` window would need a score line the dump does not have.
  The run26 replay (§16.5) is subject to the same limit: London wins by
  two hundred to one, so the **choice** is asserted and the draw count
  is, and the score arithmetic between them is not — every multiplier
  but the three London took (`×10 ×10 ×2`) could be off by a factor and
  the block would not say. *Capture:* a block where two candidates are
  within a multiplier of each other — an ally's city against an enemy's
  at `defense_mod == 0x100`.
- **Two §12 predicates corrected from the decompile on 2026-08-25 and
  observed by no block**: my own untroubled city taking the size factor
  and the `0x2000` division (every city in the windows is a small city,
  factor 1), and the `diff == 1` clause being live at difficulty 1 (every
  lobby so far is difficulty 0). *Capture:* a difficulty-1 lobby with a
  Large City of the AI's own; the draw count is the observable.
- ~~**`circle_x/y/radius`** (§13) are `circle_init@006817f0`'s tables, cited
  by name.~~ `ai_place::circle` rebuilds them and the replays of §16.5
  walk them; `move_x/y` are `world::MOVE_49`, read from the executable's
  `.rdata` (`rise.pdb` `0002:97008` / `0002:95232`, `int[441]` each).
- ~~**`lands[class].+0x100`** (§13), the per-land-class muster score, is
  a field `types.txt` does not name.~~ It does: `LandData::move_rate`
  (`+0x100`, beside `combat_bonus` at `+0x104`), a constant `0x100` from
  `Lands::init` with no other writer in the export. The class encoding's
  flag bits are named from run20's words (`world::cell`); `0x40`, `0x400`
  and `0x800` are not.
- ~~**`engagement` has never executed** in any traced game (§16.4); §11 is
  the reading alone.~~ **It has, twice**: run16 at 6652 (unnoticed since
  2026-08-24) and run28 at 15100, staged for it (§16.6). ~~What §11 still
  rests on the reading for is the *choice* of unit — the first engaged
  member whose target is a map unit — which no dump shows; run29's window
  carries the records around the frame.~~ **Settled 2026-08-26** by
  opening that window's `UNITS=3` half: the seed is `o 54`'s target and
  the next block carries it (§11). Two things the same capture corrected
  in the simulation — the test is on `get_action()` rather than the front
  order, and the walk is the **group's** `list` order.
- **`engagement`'s fallback**, new in §11 and observed by nothing: when no
  qualifying unit's target is a map unit the army adopts the **last**
  qualifying unit's target anyway. run29 breaks on the first, so the arm
  is unexercised. *Capture:* an army whose only attackers are pointed at
  **buildings** — every `TARGETORDER` in reach then fails `is_map_unit`
  and the army should still take the last one. ~~`FABLE:` the listing is
  the only evidence; the register spill at `6f5324`/`6f5342` and the
  fall-through at `6f536b` are the whole argument.~~ **Ratified
  2026-08-28, Fable, from the listing** (§11): the reading stands, with
  one refinement — a last qualifying unit whose target order is negative
  yields nothing, not the earlier target — and the simulation now does
  the same. Still reading-only; the capture above is still the check.
- **The blind list after run28**, from `tools/trace/report.py … blind
  docs` over **every** trace on disk (18 logs; 524 addresses cited under
  `docs/`, 428 entered, 96 never) — this document's share is just three:
  `Army::use_scouts`, `Army::use_spies` and `SpellType::cast_create_decoy`,
  plus `Leader::action_ping`, which is the human's ping and not the AI's.
  `Army::stop`, `use_generals`, `find_besieged_city`, `find_waiting_unit`,
  `find_useful_army`, `find_city`, `diplo_change` and `Armies::send_navy`
  are all covered by earlier runs. The list is the queue of runs, and it is
  only honest when it is asked against every log (§16.6).

## 19. Second reading — landed, 2026-08-25

Two blind readers on Opus 5, split §5–§11 and §14 (A) against §2, §12,
§13 and §15 (B), the same day as the first reading and briefed with the
captures; adjudicated in the main thread on Fable against the decompile,
the listing and the three windows. `docs/audit/2026-08-25-army.md` is the
record, verdict by verdict; the reports are at
`~/ghidra-projects/reading/army-2026-08-25/`. Eleven verdicts changed this
document and six changed `army.rs`; the corrections above carry their
audit row (`A.13`, `A.57`, `A.62`, `B.15`, `B.20`, `B.22`, `B.24`, `B.27`,
`B.49`, `B.50`) where they stand, and the largest — `sea_combat` — was a
claim the first reading had called settled.

**After the audit, from a diff (2026-08-25, later the same day).** The
run26 replay (§16.5, §17 item 5) changed three §12 predicates both
readings had passed — the diff-≤-1 founder gate's polarity for an
enemy's city (B.25 named the filter and did not read its sense), the
`diff == 1` clause's reachability, and `me` in the allied test — the
first of them on a failing draw count, the other two from the same
re-reading of the listing's structure. Three predicates, no arithmetic:
the audit README's recurring lesson again, and the reason the diff came
first.

**Ratified, 2026-08-25 (later still).** Those three §12 predicates, and
the three §8.4/§9 predicates the group-orders session changed from the
same listing — the friendly test being `is_ally` (`6f4559`, `6f4f55`),
the 90 %-damage test's sense (`damage >= hits × 9/10`, `6f5007`) and its
restriction to a city centre (`OBJECT_CITY = 32`, from the PDB's own
`LF_FIELDLIST`) — were adjudicated as a carry-over inside
`docs/audit/2026-08-25-groups.md`. **All three stand; no Rust changed.**
They had been written without an adjudicator, which
`docs/DECISIONS.md` entry 22 makes exactly the case a ratifying pass is
for.

## 20. The Military level, not the age — Great Lakes 12135 → 12184 (item 657, 2026-09-23)

`release_mustering` (§7), the stance rule (§8.2, and so `engagement` and
`march_to_target`) and `find_target` (§12) each read
`data_encrypted->epoch[0] ^ 0x63187`. That is `LeaderDataEncrypt +0xe8
epoch[4]`'s first entry, **the Military library level**. The type record
names the field. `ages` is `+0xdc`, under its own key `0x62766`, and in
this family only `find_target`'s forts pass reads it (`6f801a`,
§12). The glossary's gloss "the leader's current age" led
`army.rs` to read `tech.ages` at all three sites. The two fields differ
from the moment a leader researches Military ahead of its age. Great
Lakes' who=1 reads **`ages_get()` 1 and `epoch_get` 2** on its Military
line from before this window: `LEADERS=9` prints both, and this crate's
own leader rows agree on both.

### 20.1 The chain, from the word backwards

The word was 12135: ours spent 7 draws against the original's 6. The
extra was `Guy::set_anim+0x97a < Guy::inc_time+0x271`, `1/68` wrapping
its idle where the original's walks (`docs/ANIM.md` §11). Every step
below is a row of run163's widening or a value this crate prints:

1. **12058.** The original lists the newborn `1/68` in army 2's pool
   group 66 (ten members), with a move order and an 11-entry path. It
   also holds a one-member group 68 of `1/68`, with `order_num 4` and `o`
   (36264, 23688). That is `Unit::go_to`'s walk group, from
   `add_to_army`'s walk to the army's first unit (§4.3). This crate gave
   `1/68` a group 68 of its own and no order.
2. **12057, `add_to_army`.** This crate's `find_local_army` answered
   **army 0**, a valid slot mustering with no units, at 90,000,000
   (§15.2). It then seeded a stack group for it. Armies 1 and 2 stood at
   the same point, 3,819 away. Army 1 was at status 0x12 and army 2 at
   0x10, and `diff` is 0 (`GAME INFO`'s `DIFFICULTY 0`), so both failed
   `diff > 2 || status & 1`. The original took army 2, and a mustering
   army 2 is the only way it could have: the predicate is the one this
   crate carries (`Armies::find_local_army@006f32e0`, read whole), and the
   later equal wins.
3. **12024, army 2's tick** (its phase puts it on 11512 + 256k).
   This crate's `do_mustering` released it. `num_standard` 6,
   `city_num` 2, `pop_cap` 75, `effective_pop` 58 and two mustering
   armies put every threshold out of reach, and `pers.rush` is 1. So the
   personality's ladder decided: this crate read age 1, `< 2`, go. The
   weak-region arm then set 0x20, and `do_defending` left 0x10. The
   original reads Military 2. `< 3` asks `n >= 16`, and six stay.

No capture prints the army record on this line (`ARMY` is a full-dump
record, §16.1), so step 3 is the arithmetic of steps 1 and 2 plus the
listing. It is confirmed by what moved: with the one read changed, 12058
and everything under it agrees.

### 20.2 This crate

`Sim::release_mustering`, `army_stance_rule` and `find_target` read
`tech[w].military_level()`. The guard is
`release_mustering_s_rush_rule_reads_the_military_level_not_the_age`. It
was made to fail by putting `ages` back.

### 20.3 What it moved

| | before | after |
| --- | --- | --- |
| Great Lakes long word | 12135 | **12184** |
| run163 widening [11400, 12399], keys parted | 1,884 | **1,432** |
| 12058: pool group 66's list, `1/68`'s group, order, path, `form_mod` | parted | **agree** |
| `1/68` on 12136, the old word's block | 27 rows | **none** |
| a figure changing animation on one side only, near the words | 1 | **none** |

The keys under 11922 hold at 248. East Indies' 13640 holds.

**The new word, 12184** (block 12185, inside run163): ours spends 47 draws
against the original's 95, parting at index 0. Ours spends six pairs of
`Leader::create_buildings+0xffb`/`+0x1017`, and the original spends none.
The original spends one bird's thirty-round `Animal::think_bird+0x2aa`/
`+0x2d3` arm, and ours does not. The rows under it open on who=1's
production list (`MAKE[0]`, `MAKE[4]`) on 12181. On 12183 ours has queued
at `1/2019` and spent 100 food, and the original has not. Those rows
stood before this item, unchanged. No mechanism is named.

### 20.4 What this has *not* established

- **The stance rule's and `find_target`'s reads** changed with
  `release_mustering`'s. They are the same field, cited at
  `do_forming@006f43c0`, `engagement@006f5160`,
  `march_to_target@006f4d80` and `find_target@006f69b0`. No frame on the
  measured words turns on them yet. They are listing-backed, not
  diff-backed.
- **`find_target`'s head test**, `2 < epoch[0] &&
  type_avail(SUPPLYWAGON) && …`, joins its weak-army clause. This crate's
  `weak_army` does not carry it. It is a seam, reached by no capture.
- **Residue under the new word, no draw.** `1/67` and `1/68` are born
  with `form` −1. The original's is 0, because `Unit::init@00612100`
  writes `+0xaa` as 9 for the four citizen and scholar ids and 0 for
  every other type, and `+0xab` as −1. This crate starts every unit at
  −1. The same init value is behind the East Indies boat's row (parked
  646). Their birth-block `orders_x` is one block off. The original's
  walk group 68 is freed on 12086, and this crate keeps it.

### 20.5 Coverage

**Diff-backed**, in `run163_s_word_frame_is_widened_whole`: 12058's rows
and the old word's block agree, and the new word's chain is pinned from
12059 to its block. The coverage pin reads the new word's five blocks
on run163. **Listing- and type-backed**: the field at `+0xe8`, the key,
and the two reads in `release_mustering` (`6f8997`, `6f89c3`). **Inferred
from the move**: army 2's status on 12024 (§20.1, step 3).
