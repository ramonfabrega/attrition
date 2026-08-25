# Armies — the AI's military state machine

*First reading, 2026-08-25, in the main thread on Fable, from the Ghidra
export (`~/ghidra-projects/decomp`), the PDB type records (`types.txt`,
`vtables.txt`), the listing (`llvm-objdump`) wherever the decompiler dropped
a register argument, and four traced runs of the islands lobby — run21 (no
attack), run23 (the null result of §16.3), run24 (the AI's capital raided)
and the `DUMP_ALL` windows staged from run24's frames (§16). The second
reading's army rows from the transport audit (B.44–B.68,
`docs/audit/2026-08-25-transport.md`) are folded in where they stand.
Second reading: §19.*

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
XOR `0x63187`, the leader's current age; `diplos[a][b]` is `LeaderData
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
unit's idle think joins the nearest one (`Unit::add_to_army`, §4) → every
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

`get_num_cap@007145c0` counts the group's active units for which
`UnitData::is_captain@0046ceb0` holds — `o_up < 0`, a unit **not inside
another object** (not garrisoned, not carried; `docs/ANIM.md`'s
`inside_up`). So `num_captains` is "units on the map" and
`num_standard` the fighting line: captains less spellcasters, supply
wagons, decoys and anti-air. `GroupData::count@00711720` is the
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
`Unit::think@005f6e40:338` at the end of a computer unit's idle think —
after the supply, hero and spy branches, for any type that is not a supply
wagon, a hero, or a spy without the special flag (those scout instead if
they have no army); `Unit::think_attack@005f5a80:155`, an idle attacker
that is not a merchant (`0x3d`, `0x3e`, `0x190`), not a caravan and not a
special (`unit_flags2 & 0x10`), when its city is not one the census marks
weak; `think_supply` and `think_hero`, unconditionally;
`think_scout@005f6010:571`, a **sea** unit whose region is scouted;
`Unit::come_out@00617c10:1614`, a unit leaving a building, **unless a
draw says otherwise**: `Random::get(game_random, 0, 0xffff)` — `frame &
(draw & 1) == 0` for a non-`is` type, `frame & (draw % 3) == 0` for the
other — skips the join. A `game_random` site the sync accounting
(`docs/SYNC.md` §3) now lists. Run21's first `add_to_army` is frame 5823,
`add_group`/`add_unit` 10187.

**`SpellType::cast_create_decoy@00674370:157`** adds the decoy it creates
to the caster's army; **`Unit::fight@005fd4d0:293`** and
**`Object::do_damage@0064a480:1373`** call `Army::charge` (§14) for an
AI-controlled siege unit's army when it is fought or damaged;
**`City::close@00737550:73`** closes every army mustering at a dying city
and clears `target_o` on every army targeting it;
**`Cities::capture_city@00733380:250`** → `Armies::update_city` (§15.7).

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
   siege or not.
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
is **validated**, and any failure goes to the retarget below. With
`target_o >= 0`, `t = objects[target_who][target_o].data()` (vslot
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
(A.62), the stance rule (§8.2), the same
siege / move choice as §8.3–8.4 with **`hurry` alone** selecting the move
(`bVar12`), and then per group: a **building target at 90 % or more of
its hits** (`damage < hits × 9 / 10` is false) gets `action_move_to` to
the **building's own position**, `ATTACK_TO`; anything else gets the
§8.4 order at `(x', y')` with `angle`; then the half-cell step back
(§8.5). `status |= 4` so it is not repeated while the fight lasts.

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
with `num_valid() != 0`. No target → nothing. Run24 never entered it
(§16.4): the AI's armies were forming at their muster spots when the
hoplites arrived and `is_engaged` never held.

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
  ways (or my ally-slot), **my** `weak[L] == 0`, `strong[L] != 0`, not a
  tribute period, and `defense_mod >= 0x100` — an ally's cities are scored
  as places to defend, the `attacked` multipliers below making them worth
  anything;
- **at war**: only if `defense_mod <= 0x100`; and if my `wonderwin_timer
  != 0 || popwin_timer != 0` set **`about_to_win = 1`** (a ×100 below).

So above `0x100` the AI considers allies only, below it enemies only, at
`0x100` both. A leader that passes goes through the **difficulty gate**:
at `diff < 2`, qualify only if `diff != 0 || L is human` and
`L.frame_attacked + 0x1c20 <= frame` (7,200 frames since I last took a
target against `L`) and either `find_aggressive_army(me)` is this army
or, with none, a coin — **`Random::get(game_random, 0, 0xffff) & 1 ==
0`**; at `diff >= 2`: skip if `diff == 1` and (`num_captains >= combat /
2` or `find_aggressive_army(me)` is another army); then proceed if `diff
< 2`, or `L` is my ally-slot, or allied, or (`age < 3` and
`pers.early_army != 0` and `diff != 2`), or `num_standard >= 7 − 2 ×
pers.raid`; then `diff != 2`, or `L.frame_attacked + 0x708 <= frame` and
(`team_style == 3` or `+ 0xe10 <= frame` or `L.attacked_by == me` or
`my.attacked_by == L`). A leader that fails any of these is skipped this
pass.

*Team play* (`team_style == 2`): `L` must be the next leader after my
ally-slot in the start list that is alive and in use, or me, or my
ally-slot, or allied — otherwise skipped.

*Every city of `L`* below `L.city_mark`, `city_flags & 1`, and for a navy
in a region `is_coast` of mine with `ocean != 0`; then at `diff <= 1`,
for me or an ally, skip unless the city's `founder` is me (B.25: on the
two easiest difficulties the AI defends only cities it founded). The
score:

```
v = Random::get(game_random, 0, 0xffff) % 200 + 900                  # one draw per candidate
if have_capital and L is an enemy (not my ally-slot, at war either way):
    if weak_army:                                                     # a weak army only retakes
        skip unless city.founder == me and its building is_unassimilated   # B.27: founder
    v −= 50 × vector_dist(|cell(c.x) − cap_x|, |cell(c.y) − cap_y|) / world.xs   # listing 6f7281
if my pop_issues != 0 and L is an enemy: v ×= 4
if L.wonderwin_timer != 0 and the city has a wonder (num_wonders(c, 0)) and Game::wonder_winning() == L: v ×= 10
if L is me or allied both ways:
    if allied and !(city_flags & 2):                                  # an ally's untroubled city
        if num_captains < 20 or count(SIEGE) < 2:
            if num_captains < 10 or count(SIEGE) == 0:
                v ×= (TOWN → 2 | METROPOLIS, FORBIDDENCITY → 3 | else 1)
        else: v = v × (4 − city_level) / 4
        if city_flags & 0x2000: v /= 20                                # §13's "no muster spot" mark
    elif team_style == 2: v ×= 5
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
if L is not me but my ally-slot or allied: v = v × 3 / 4; and if !(city_flags & 2): v /= 2
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
0x1a1)`: the farm's `ever_seen |= 1 << me`, `action_stance(5)`,
`action_attack(farm, L, 1, QUEUE_NEW, 0)`, then `action_move_to` back to
the first unit's position, `MOVE_TO`, `QUEUE_LAST`; else
`action_move_to(target, QUEUE_NEW, ATTACK_TO)`. Then `normalize()`,
`L.frame_attacked = frame`, `L.attacked_by = me`. Two units go and poke a
farm.

**No target after both passes**: return with `target` unchanged (−1 from
the retarget); `do_marching` sets `status = 8` (§9).

**The draws.** One `Random::get(game_random, 0, 0xffff)` per candidate
city that reaches the score, one per candidate fort, and one in the
difficulty gate's coin — all in the sync stream (`docs/SYNC.md` §3).

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
  `vector_dist(|cx − A.muster_x|, |cy − A.muster_y|) < 4` if `who` is me
  or my ally, `< 2` otherwise, ends the candidate's walk before it scores;
  it scores 0 and cannot become the best;
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
  neighbour, 2304 at most, and the walk order breaks every tie;
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
8. **`emergency(who)@006f3250`** (from `Object::do_damage@0064a480:952`,
   when an object of an AI leader takes damage and the damage was not
   attrition/friendly — `local_30`): for the in-use, non-human leader
   with `leader_flags2 & 0xa == 0`, **every valid army** has `target =
   (−1, −1)` and `Army::process(army, 1)` at once — the whole tick, cadence
   bypassed. Run24: 12129, the first hoplite's first blow.
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

### 16.2 run21 — the family without an enemy

The islands game to 24000 (`docs/ORACLE.md`): `do_mustering`,
`do_forming`, `release_mustering`, `find_muster_spot`, `is_moving`,
`is_engaged`, `set_stance`, `count` from **252** (army 0's first tick);
`add_group`/`add_unit`/`member` 10187, `get_unit` 10232,
`center_of_gravity` 10488, `num_armies` 14074, `do_transporting`,
`find_target`, `find_aggressive_army` **14586**, `do_marching` and
`Army::close` **14838**, `use_generals` 15898, `leader_defeated` at the
quit. Never: `engagement`, `march_to_target`, `do_defending`,
`find_besieged_city`, `remove_group`, `stop`, `send_here`, `charge`,
`use_scouts`, `use_spies`, `find_waiting_unit`, `find_useful_army`,
`find_city`, `update_city`, `emergency`, `diplo_change`, `send_navy`.

### 16.3 run23 — the null result

The same lobby with `6000 war who=1` in `rontrace.cmd`: the line ran
(`INFO cmd`, `Leader::set_diplo` entered at 6000) and **every one of the
24,001 `game_random` words is identical to run21's** — a Quick Battle
already starts at war (run16 had to declare *peace* first,
`docs/ORACLE.md`), so the command changed nothing, and run21 was already
a war in which the AI never marched on an idle human. Worth a line
because it is the cheapest possible check of "did the scenario take": the
per-frame RNG word in the trace's `FRAME` records
(`tools/gamelog/rngcmp.py`).

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

## 17. What the simulation carries, and what checks it

`crates/sim/src/army.rs`: the record and the pool (§2 — `init_army`'s
slot rule and eviction, `Army::init`'s muster cell from the city, `close`),
the counts as a function of the groups' units (§3.3), `add_to_army`'s
army choice and `find_army` / `find_local_army` / `find_useful_army` /
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
`create_units`' `seam_army_at` (`ai_units.rs`) are wired to it. **Not
implemented**: the order-issuing half — `do_forming`, `march_to_target`,
`engagement`, `send_here`, `charge` and `find_besieged_city` — which
needs the sim's group orders (`docs/ORDERS.md`'s `Group::action_*`), not
yet modelled; they are documented above and the state machine stops at
"the orders this tick would issue".

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
   `do_defending` — read into §16.5; the assertions they can carry
   (`status`, `target_o/who`, `x/y`, `hurry`) wait on a harness that can
   stage a frame-12000 state, which is item 13 of the queue.
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

## 18. What is not established

- ~~**`leader_flags & 8`**~~ — the PDB names it `LEADER_COOP_SOLO` (audit
  A.56): a human seat the AI plays for; `production_ai` runs for such a
  leader and the armies do not (B.16). Never set in a skirmish.
- ~~**The `combat` average's field.**~~ `sea_combat`, from the listing's
  base register (B.20, §12) — the first reading's own "settlement" was the
  error.
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
  the family reads it. *Same.*
- **The order of two multipliers in §12** — the enemy-capital
  `GLOBAL_GOVERNMENT_BONUS` block and the `bordering` block — is as the
  decompile lists them; the listing was not read for these two, and the
  product is order-independent unless a division truncates between them.
  *Capture:* run26's `ARMY` records give the chosen target, not the score;
  a `LEADERS=9` window would need a score line the dump does not have.
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
- **`engagement` has never executed** in any traced game (§16.4); §11 is
  the reading alone. *Capture:* a run where the AI's army meets an attacker
  *at its muster spot* — the hoplites at (203, 207) reached the capital,
  not the army mustering one cell south.
- The blind list after run24: `engagement`, `send_here`, `charge`,
  `use_scouts`, `use_spies`, `find_waiting_unit`, `find_useful_army`,
  `find_city`, `diplo_change`, `Armies::send_navy`.

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
