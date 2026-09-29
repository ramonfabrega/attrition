# Transports and docks — the sea half of the AI

*First reading, 2026-08-25, in the main thread on Fable, from the Ghidra
export (`~/ghidra-projects/decomp`), the PDB type records (`types.txt`),
the listing where the decompiler dropped an argument, and run21's trace
(`docs/RUNS.md`, "run20 and run21"), which is the first capture to enter
this family. Second reading: §14.*

**What this is.** Rise of Nations has no transport ship a player builds. A
land unit ordered across water walks to the shore and *becomes* a transport
barge (or a merchant fleet) carrying itself, and walks back out on the far
side. This document is the machinery around that: what lets a leader's
units do it at all, which units may, where the docks are kept and what
they mark, how a citizen or a scout decides to leave its island, and how
an AI army goes "transporting". The boat itself — its movement, the
boarding path, the unloading — belongs to the pathfinder and the orders
(`docs/PATHFINDER.md` §4.6, `docs/ORDERS.md` §4.6) and is cited, not
re-read, here.

**Confidence.** The predicates and the registry (§2–§6) are read line by
line and are small; high. `think_civilian_transport` (§7) and
`do_transporting` (§8) are read whole, with the two `vector_dist` calls the
decompiler mangled settled in the listing; high on structure, medium on the
two score formulas until a capture with a transported unit in it exists
(§13). `find_target`, `find_muster_spot` and `cast_transport`'s tail are
read only as far as this mechanic reaches into them.

**Naming.** Offsets are the PDB's: `struct /rise.pdb/LeaderData`,
`Region`, `ArmyData`, `DockData`, `UnitData`, `UnitTypeData`. `Leader` is
the unpopulated shell over `LeaderData` (`docs/AI.md` §10). "Cell" and
"tile" are as everywhere else: a cell is 4 × 4 tiles, a tile is `0xc0`
coordinate units, a cell `0x300`.

## 1. Shape of the mechanic

Four state words, three of them per unit:

| state | where | who writes it |
| --- | --- | --- |
| the leader's **transport level** | `leader_flags` bits `0x100`, `0x200`, `0x400` | `Leader::check_transport` (§4); scenario hooks |
| the leader's **transport lock** | `leader_flags2 & 0x20` | `ScenarioFuncSet::force_transport_ability` only |
| the unit's **auto-transport bit** | `unit_masks & 0x800000` | `Unit::init`, `check_transport`, the player's toggle, `cast_transport` |
| the unit's **never-transport bit** | `unit_masks2 & 0x2000` | scenario script only (`disable_/enable_unit_transport`) |

and one registry: `docks`, eight `PtrArray<Dock>` (one per leader, 20
slots preallocated), whose entries carry the dock building's object, its
land region and its gull, and whose `init`/`close` are the only writers of
the census array `reg_docks[64]` (§5).

The flow, in the order a game reaches it: a dock is finished →
`Docks::init_dock` registers it and `check_transport` runs (§4) → every
unit that can ever transport gets `0x800000` (§3) → a unit whose path needs
water (`needs_transport`, §6) boards at the shore, which is
`SpellType::cast_transport` (§6) → the AI's civilians pick an island to go
to (§7) and its armies go "transporting" when the census tells them the
region wants expanding (§8).

## 2. The leader's transport level — `LeaderData::can_transport@006e0c60`

The enum is `TransportType`; its values are fixed by `can_transport`'s
return and by `check_transport`'s comparison:

| value | name | granted by |
| --- | --- | --- |
| 0 | none | — |
| 1 | `TRANSPORT_SCOUT` | `leader_flags & 0x400` |
| 2 | `TRANSPORT_MILITARY` | `leader_flags & 0x200` |
| 3 | `TRANSPORT_CIVILIAN` | `leader_flags & 0x100` |

`can_transport()` returns the **highest** of the three bits set — `0x100`
→ 3, else `0x200` → 2, else `(flags >> 10) & 1`. The same three-way test is
inlined in `check_transport`, `think_civilian_transport` and
`Group::action_set_transport`. A unit may auto-transport when its own
`transport_type` (§3) is **≤** the leader's level, so the ladder reads:
scouts first, then military, then civilians.

`LeaderData::locked_transport@006d5230` is `leader_flags2 & 0x20`. Nothing
in the skirmish path sets it; `ScenarioFuncSet::force_transport_ability`
sets `leader_flags |= 0x700` and `leader_flags2 |= 0x20` together, which
freezes the level at 3 against `check_transport` (§4's first test).

## 3. The unit's side

### 3.1 `UnitData::transport_type@0046f790`

By the unit's type: `is(SCOUT)` (0x45, the lineage test) →
`TRANSPORT_SCOUT`; a citizen or scholar (base type `0x32..0x35`), a caravan
(`is(0x3b)`, lineage) or a merchant — the **exact** base types `MERCHANT`
(0x3d), `MERCHANTDUTCH` (0x3e), `FURTRAPPER` (0x190) → `TRANSPORT_CIVILIAN`;
anything else → `TRANSPORT_MILITARY`.

### 3.2 `UnitData::can_transport@0046f960` — the bit, read

`(unit_masks & 0x800000) && !(unit_masks2 & 0x2000)`, **or** the type's
`unit_flags & 0x10` (letter `e` in `unitrules.xml`'s `FLAGS`,
`docs/DATALAYER.md`): a type that always transports, whatever the leader's
level. `docs/PATHFINDER.md` §4.6 cites this as the "can take a transport"
test in `calc_cost`; `GroupData::can_transport` requires it of **every**
on-map unit in the group for the toggle (§3.4).

### 3.3 `UnitData::can_ever_transport@0046f290`

A land unit (`type+0x218 domain == 0`) → 1. Otherwise only a unit whose
type has `carry != 0` (`UnitTypeData +0x2d4`) and is not
`is(AIRCRAFTCARRIER)` (0x15f) → 1 — the vslot `+0x18` in the middle is the
folded `return 1` (`Buffer::is_pending_load`'s body, audit A.12), so it is
vacuous. So the barge and the fleet themselves qualify; warships and
aircraft do not.

### 3.4 Who writes `0x800000`

- **`Unit::init@00612100`** (lines 318–338): a new unit whose
  `transport_type` is **≤ the leader's current level** (the inlined
  three-way test, §2) and that `can_ever_transport` gets the bit, unless it
  `is(SCOUT)` and the leader's option bit is clear — `LeaderOptionData.flags
  & 2`, set by `LeaderOption::init` (on by default) and the game-options
  window; the "scouts auto-transport" preference. A unit born before the
  level is granted starts without it; `check_transport` (§4) is what
  catches those up.
- **`Leader::check_transport`** (§4): sets it on every unit when the level
  goes to 3 (the same scout exception), clears it on every unit when the
  level goes to 0.
- **The player's toggle**: `Options::do_transport` →
  `issue_set_transport(!GroupData::can_transport())` →
  `CommandPackage::process_set_transport` →
  `Group::action_set_transport(flag)`: if the leader's level is non-zero,
  every alive `can_ever_transport` unit of the group gets the bit set (flag
  1) or cleared (flag 0); with level 0 the flag is forced to 0. It travels
  the order stream (`docs/COMMANDS.md`'s `SetTransportCommand`).
- **`SpellType::cast_transport`** (§6): the barge created at the shore gets
  the bit.
- **`ScenarioFuncSet::force_transport_ability`**: every alive
  `can_ever_transport` unit, with the leader forced to `0x700 | lock`.

`unit_masks2 & 0x2000` is written only by
`ScenarioFuncSet::disable_unit_transport` (set) and
`enable_unit_transport` (clear) — a per-unit veto for scripted scenarios.

## 4. `Leader::check_transport@006bc5f0` — the level, maintained

Called from `Build::activate` and `Build::close` when the building
`is(DOCK)` (0x1b0, the lineage test — so a Shipyard or any later upgrade
counts), from `Leader::gain_building` / `lose_building` under the same
test, unconditionally at the end of `Leader::gain_tech`, and from
`Leader::fix_tech_flags`; and by the scenario editor's three transport
toggles. Run21 enters it at frame 201 — a `gain_tech`, long before the
first dock.

```
if leader_flags2 & 0x20: return                              # locked
want = has_preq(TRANSPORT_BONUS) ? 3 : 0                     # TypeIndex 0x2ae
if want:
    docks = num_buildings[DOCK] + get_buildings(DOCK.to, num_buildings[DOCK.to])
    if docks:
        if can_transport() == want: return
        leader_flags |= 0x100 | 0x200 | 0x400
        for u in my units, alive, can_ever_transport(u):
            if !(option_flags & 2) and u.is(SCOUT): continue
            if transport_type(u) <= want: u.unit_masks |= 0x800000   # always, at want == 3
        return
if leader_flags & 0x700:
    leader_flags &= ~0x700
    for u in my units, alive: u.unit_masks &= ~0x800000
```

Three things worth stating plainly, because they are not what the three
bits suggest:

1. **The skirmish game knows one level, 3.** `want` is 3 or 0; the bits are
   set and cleared as a block. The finer levels exist for scenarios
   (`set_transport`, `toggle_always_/never_transport` in the editor) and
   for the shipped scenario scripts, not for the AI.
2. **The gate is a prerequisite and a dock.** `has_preq(TRANSPORT_BONUS)`
   is the tech side (`docs/TECH.md`, "has_preq"; which shipped row grants
   the bonus is §13's first item); the dock count is
   `LeaderData::get_buildings` over the `to` chain
   (`docs/DATALAYER.md`, "the `to`/`upgrade` back-links"): a dock **or any
   building it upgrades into**. `num_buildings` is `LeaderData +0x555e`,
   `ushort[129]`, indexed by `type − BASE_BUILDTYPES`; the decompiler folds
   the base and prints `field_0x5582` for the dock's slot.
3. **Losing the last dock revokes it for every unit at once** — the second
   block. A unit at sea keeps sailing (nothing here touches a boat), but
   nothing new boards.

The `has_preq` inside the `docks != 0` arm is a repeat of the first and
always true there.

## 5. The docks registry — `Docks`, `Dock`

`docks` is a global `DocksData`: `PtrArray<Dock>[8]`, `0x1c` a leader
(`+0x4` count, `+0x8` capacity, `+0x10` list). `Docks::init@00741580`
preallocates **20** `Dock`s per leader. A `Dock` is `DockData`, ten bytes:

| offset | field | meaning |
| --- | --- | --- |
| `+0x0` | `dock` | its slot |
| `+0x2` | `o` | the building's object number, −1 when free |
| `+0x4` | `reg` | the building's cell region (`WData.region` at the building's position) |
| `+0x6` | `gull_o` | the gull's object number, −1 if none |
| `+0x8` | `dock_flags` | bit 1 active |
| `+0x9` | `who` | owner, −1 when free |

### 5.1 `Docks::init_dock(who, o)@00740fc0` — the slot rule

Called from `Build::activate` (line 560) for a building whose type is
**not** a fort (vslot `+0xfc`) and **is** a dock (vslot `+0x108`,
`BuildTypeData::is_dock` = `is(0x1b0)`); the slot is stored on the building
at `BuildData +0x78`. The rule is the forts' (`docs/CITIES.md`): the first
inactive slot below `dock_mark` (`LeaderData +0x42c`) is reused; else the
slot at `dock_mark` if the array has it, growing the array by one `Dock`
otherwise; then `dock_mark = max(dock_mark, slot + 1)`; then `Dock::init`.

### 5.2 `Dock::init(slot, who, o)@00740a80`

`dock = slot; who; o; dock_flags = 1; reg = tregion(building)` — the
building's position `>> 8` through `div_3_table` to a cell, and the cell's
`WData.region`. **If `reg < 0x40`** — a land region —
**`leaders[who].reg_docks[reg] += 1`**. ~~Which a dock's own cell always
is.~~ **It is not, and run22 says so** (§10): the AI's first dock stands at
`(44160, 41856)`, cell `(57, 54)`, and its `DOCK` record carries **`reg 65`**
— the ocean. A dock's centre is on water, so the guard skips the increment
and `reg_docks` stays 0 for it; only a dock whose centre cell is a land
cell would count. Whether the shipped placement ever produces one is
§13's; the register's `reg` itself is the sea region, which is what
`find_dock`'s same-region flag and `send_navy`'s coast test then read.
Then the gull:
`Objects::init_unit(who = 9, GULLBIRD, x − 0xc0, y − 0xc0)` — Gaia, one tile
up and left of the dock — and if it was created, `Random::get(game_random,
0, 0xffff) % 7 × 0x0aaaaaaa − 0x40000000` as its angle (`set_angle(…, 7,
0)`) and `add_strafe_order(o, who, …)` to circle the dock; `gull_o` = its
number, or the −1 `init_unit` returned.

**Two `game_random` draws per dock, in this order**, and run21 shows both
at frame 3579 (`report.py … draws 3579`): draw 0
`Guy::init_real+0x52 < Unit::init+0xb97 < Animal::init+0x1a` — the gull's
creation — and draw 1 `Dock::init+0x125 < Docks::init_dock+0x128 <
Build::activate+0xcbf`. This is a per-frame site `docs/SYNC.md` §3 does not
list; it is added there with this document.

**And `gull_o` is 15** in run22's block 3580, which is the one field of the
`DOCK` record nothing compared for a week. Owner 9 is absent from a dump, so
the gull's own record is not there — but its *object number* is, and it is
what says this crate hands gaia its slots where the original does: the
pasture's five plus ten birds are allocated before it, and
`rondata::diff`'s run54 test asserts the 15 at the end of 24,000 frames.

### 5.2.1 The gull, flying (2026-09-01)

`Dock::init` is not the end of the gull. It is a **bird** — `Animal`, one of
`Guy::set_anim`'s three gaia bird types — and from the frame after its
birth it is in the unit loop with a `StrafeOrder`, which is East Indies'
word at 3579 and the whole of item 133.

- **`do_job` never reaches `do_idle` for it.** `Unit::do_job@00617a10`
  dispatches on the order: `STRAFE → Unit::do_strafe@005eab00`, where a
  wild bird's `AIR_PATROL → Unit::do_air_patrol@005ea620`. Both call the
  `+0x180` virtual (`Animal::think_bird`) and then
  `Unit::do_air_physics@005e86d0`. `think_bird`'s `0x194` arm returns after
  an `order_type` call and **draws nothing**, so the gull's think is free
  where the wild bird's is three draws every eighth frame (`docs/SYNC.md`
  §3.9).
- **`do_air_physics`'s tail is `set_anim(CHAR_WALK, 0, 1)`**, and for the
  gull that is one draw at birth and nothing after: `Guy::set_anim`'s
  gaia-walker early return (`set_anim:141` — `who >= 8`, the request is
  `CHAR_WALK`, the guy's category already is, and it is inside its length)
  catches every later frame. run54's is frame **3580**, `Guy::set_anim+
  0x104b < Unit::set_anim+0x56 < Unit::do_air_physics+0x683`, and the
  chain occurs exactly **12 times in 24,000 frames** — ten wild-bird
  births, this gull, and one at 15458.
- **After that the gull is on the wing beat**, `Guy::set_anim+0x104b <
  Guy::inc_time+0x271`, like any gaia bird: `set_anim:620` throws the coin
  for `0x192`, `0x193` **and** `0x194`, and `rnd % 100 > 0x31` takes
  *Bird Flap* over *Bird Soar*. The two identity tests in `set_anim` are
  **different sets** and that is not a slip — `set_anim:221`'s
  same-category early return names `0x192` alone — but a gull never reaches
  221, because 141 returns first.
- **`do_air_physics` spends no draw of its own** on either bird here. Its
  one draw site, `+0x639` (the `piVar2[4] == 0` re-bank after
  `UnitData::invalid_loc` refuses the projected point), fires **three times
  in run54's 24,000 frames** — 5437, 5732, 5919 — and all three are under
  `do_air_patrol`, never under `do_strafe`. That draw is queue item 120,
  Great Lakes' word at 1802.

**What this crate models, and what it does not.** `Sim::do_idle`'s gaia arm
stands in for both air orders (the seam `docs/SYNC.md` §3.9 already named
for the wild bird): the gull is parked on the tile it was born on, and what
reaches the stream — the walk request at birth and the wraps after it — is
spent where the original spends it. The **flight** is not modelled:
`bank_aircraft`, `pitch_aircraft`, the `project`/`invalid_loc`/
`set_new_location` step, and `do_strafe`'s own sixteen-frame
`find_new_air_target` are all unwritten. Nothing dumps owner 9, so the
gull's position has no oracle but the draw stream — and the draw stream is
matched. `Guy::move`'s body follow is skipped for all three bird types
(`anim.rs::is_air_gaia`), because a bird has no ground body: without that
the standing body asks it to idle every frame and spends draws the original
never spends.

### 5.3 `Dock::close@007409f0`, `Docks::close_dock@00740f50`

`close`: if `o ≥ 0`, the building is still active, and `reg < 0x40`,
`reg_docks[reg] -= 1`; then `dock_flags &= ~1`, `o = −1`, `reg = 0`,
`who = −1`, and the gull, if any, gets `Unit::close(0, −1, 0)` (vslot
`+0x150`) — **`gull_o` itself is left as it was** (audit B.13), so a closed
slot's record carries a stale gull number until the slot is reused. `close_dock(who, slot)` calls it and then walks `dock_mark`
**down** past every trailing inactive slot. `Build::close` (line 227)
calls it for a dock. So `reg_docks` is exact at every moment for docks that
were active when they died, and is **not** decremented for a dock closed
while unfinished — which never registered.

`docs/AI.md` §2.3 step 3 says `reg_docks` is "kept by
`gain_/lose_building`"; those two only call `check_transport`. The writers
are the two above. Struck there and pointed here.

### 5.4 `Docks::remask_docks(who, o)@00740c90` — the water apron

Called from `BuildType::mask_me` after a building's own mask is redrawn:
for every active leader's every active dock **except** `(who, o)`, take the
dock building, its type's `x_size`/`y_size` (`+0x234`/`+0x238`), its
centre tile with the odd-size parity correction, and for every tile from
`−3` to `size + 2` on both axes that is in bounds and **water**
(`TData.mask & 0x30 == 0x20`): if the tile does not yet carry `0x2000`,
the cell's `WData.bad` (`+0x12`) is incremented; then the tile's `0x2000`
is set. So the pathfinder's "rough" bit (`docs/PATHFINDER.md`, tile
`0x2000`) is also the **three-tile water apron around every dock**, and
`bad` counts apron tiles per cell. `BuildType::mask_me`'s own unmask
direction is what erases a dying dock's box (through `World::set_bad_path`),
and `remask_docks` is its last step: it restores the halo where another
dock's box overlapped (audit B.16–B.18; `World::set_blocked_at` keeps the
same bit and counter for blocked tiles).

### 5.5 `ObjectsData::find_dock@0065cfd0`

The nearest active dock of any leader that passes `Search::valid_search`
for `(search, who)`, by `vector_dist`, within `max_dist` if given, through
the filter; with flag `0x200`, only docks whose `reg` equals the point's
cell region — the same "same region" flag `find_city` takes
(`docs/AI.md` §2.3 step 10). Returns the slot, with `find_who` /
`find_dist` set on the objects record. No caller in the export outside the
UI and the scenario functions.

### 5.6 `BuildTypeData::is_dock_tile(wx, wy, ·)@00636700` — the tile rule

Whether a **tile** could take a dock; the census counts the first such
water tile in each city's radius (`docs/AI.md` §2.3 step 13, `dock_tile`).
The tile must be in bounds, its cell not flagged coastal (`WData.flags &
0x100`) and its cell's `land` 1 or 2 (SANDY or OCEAN — a water cell). Then
for each of the four orthogonal directions `orthog_x/y[1..4]` = N `(0,−1)`,
E `(1,0)`, S `(0,1)`, W `(−1,0)` (`.rdata` at `0x00add250` / `0x00add210`):
step one **cell** (four tiles) that way; if the cell there is in bounds
and is itself an uncoastal water cell, this direction is not a shore —
next direction. Otherwise the tile stepped to must not be a mountain
(`mask & 3 == 2`) nor forest (`mask & 0x30 == 0x30`); then a strip of four
tiles — along the axis perpendicular to the step, two rows deep back
toward the starting tile — must have no `0x80` (placed) and no `mask & 3
== 3` (building footprint). The first direction that survives returns 1;
none → 0. Its **only caller is the census** (`Leader::plan_strategy`, audit
B.29) — nothing on the placement path asks it — so the run20 frame-1
records (`dock_tile 1` for the AI's city, 0 for the human's) are its whole
check, and they pass (`rondata::diff`, the run20 `CITY` widening).

## 6. Boarding — `needs_transport` and `cast_transport`

**`UnitData::needs_transport(x1, y1, x2, y2)@00609920`** (tiles): 0 if the
two are the same tile or both on the same side of the shore (`TData.mask &
0x30 == 0x20` on both, or on neither); else `(from is not water) + 1` —
**1 = disembark** (from water to land), **2 = embark** (from land to
water). `PathFinder::calc_cost` is its only caller (`docs/PATHFINDER.md`
§4, the "this step embarks" flag).

**`SpellType::cast_transport(o, who)@00670db0`** is `SpellType::cast`'s
case `0x28a`, the transport spell — the shore conversion. For an active,
on-map unit of a land domain (`domain ∉ {1, 2}`): the boat type is the
leader's `current_upgrade(TRANSPORTBARGE)` (0x140) for anything that is
not a caravan, `current_upgrade(MERCHANTFLEET)` (0x13e) for a caravan
(`is_caravan`, vslot `+0xd0`), each falling back to the base type when the
upgrade is −1. `UnitType::find_nearby_spot` looks for water within
`constants.unit_board_distance` of the unit; **no spot → nothing happens**
(and `S_NOT_NEAR_OCEAN_FOR_TRANSPORT` for the console player). With a
spot: `Objects::init_unit(who, boat, unit's x, y)`; the boat takes the
unit's damage (`same_damage`), position (`set_new_location`), angle; a
`MARINES` (0x77) unit's boat gets `unit_masks2 |= 0x200` and
`update_speed`; a caravan's boat inherits the caravan slot and the
`0x200` flag; the unit's whole order list moves to the boat; the boat's
current order is killed; the unit's **path stack is inverted and copied**
to the boat, the first waypoint re-pushed with its embark flag (`4`)
cleared when its region equals the boat's; the boat gets `unit_masks |=
0x800000`; the unit's orders are cleared; the selection group swaps the
unit for the boat; `replace_hotunit`; and `Unit::go_inside(boat)` — the
land unit is now cargo. Run21 enters `cast_transport` at frame 3608, 29
frames after the dock, with `detect_boat_collision` the same frame: the
AI's first citizen sailing.

### 6.1 Who casts it, and where the order comes from (2026-09-01)

**`Unit::set_new_location@005f8d20` is the adder**, and it is the only one:
a step that would put a unit across the waterline is *converted* rather
than taken. The arm sits behind two gates and answers `0` — which is what
makes `move_step` end the unit's frame there (`005fb3a3`: `if
(set_new_location(...) == 0) return 1`, after the walk's own
`set_anim`).

```
local_10 = on_map and the step changes world cell
bVar2    = on_map and the step changes tile
if !local_10:
    if !bVar2: just move                       # inside one cell of dry land: never asks
    if !(WData(old cell).flags & 0x100): just move      # HALFLAND — the shore itself
if can_transport(this):                        # §3.2's predicate, inlined
    domain 0 and the new tile's surface == 0x20 → add_cast_order(-1, -1, -1, -1,
        0x28a, QUEUE_FIRST, 0); return 0       # embark
    domain 1 and it != 0x20 → eject_contents(0, -1, 1, 1); die if nothing is
        left inside; return 0                  # disembark
```

`add_cast_order@005e4a60` with `param_7 = 0` **clears** the action bit
rather than setting it, and `QUEUE_FIRST` rotates the list head onto the
new order, so the cast is stepped on the *next* frame and the move it
interrupted is still behind it.

**`Unit::do_cast@005ebfe0`**, along the untargeted arm (`spell_flags &
0xe == 0`, which the transport craft has — `craftrules.xml`'s `Transport`
row is empty of `FLAGS`, `COST`, `COST2` and `MANA`, and its `JOB_TIME` is
**0**):

1. `pay_cast_costs` once per order (`+0x1c`), which an empty cost never
   refuses;
2. on the first frame only (`spell_time == 0`) `set_anim(CHAR_DEFAULT, 0,
   1)` — **one draw a figure**, and `Unit::set_anim@00616f40` walks the
   squad guys (`0..UnitData+0xb5`) and then the crew (`type->squad_size
   ..UnitData+0xe8`), so a scout and its dog spend two;
3. still on that frame, `find_nearby_spot` on `unittypes[TRANSPORTBARGE]`
   within `constants.unit_board_distance` (`UNIT_BOARD_DISTANCE`, `3/1
   tile` = 576) — nothing → `kill_current_order`. Both this call and
   `cast_transport`'s pass `FILTER_NOT_ME` with `not_o = not_who = -1`, and
   **that pair is what decides which collision test the sweep makes**: the
   pairwise `find_collision`/`find_ordered_collision` needs both to be
   non-negative, so these two sites take `find_unit_with_radius` instead —
   reject iff `vector_dist(spot − it) <= its big_radius + r_coll`, with
   `r_coll` the barge type's own `+0x240` — and never ask the ordered
   variant at all (`docs/COLLISION.md` §5.2.1). Reading it as the pairwise
   pair puts the barge's birth cell two tiles wrong, because a Chebyshev
   `3 + 1` cells refuses the caster at offsets the 192-unit reach clears;
   run59's frame 5342 is where that was measured, and East Indies' long
   word went 5819 → **6164** when it was put right;
4. `spell_time += 1`; below `get_job_time` it returns and waits. Transport's
   is 0, so it casts on its first frame;
5. `SpellType::cast`, whose `is_castable@00675bc0` for `0x28a` is
   `ObjectData::is_cargo` (vslot `+0x110`) **and nothing else** — an on-map
   unit of a land type in `0x32..=0x19d`; the head's "who may cast this"
   test exempts `0x28a` by name.

`cast_transport` then **does not kill the order**: it has already moved the
whole list onto the boat, and the boat's own `kill_current_order` throws
this cast away there. `0x28a` is the *only* index that gets that; every
other craft is killed at the tail of `do_cast` once it has cast, which is
what a fishing boat's deploy needs and used not to get
(`docs/ORDERS.md` §6.9, the same arm read whole). The `JOB_TIME` here is
no longer a literal either — the craft table is loaded, and `Transport`'s
own row is where the 0 comes from.

The three draws are frame 3608 of run54/run57 exactly: two
`Guy::set_anim+0x97a < Unit::set_anim < Unit::do_cast+0xc89` and the boat's
`Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd`.

### 6.2 What `cast_transport` does, in order

The boat is born at the **caster's own** (48-snapped) position and walked
to the water; the caster never enters it.

1. `Objects::init_unit(who, boat, unit.x, unit.y)` — one `Guy::init_real`;
   and the boat is initialised like any other unit, so its cached speed is
   **`Unit::update_speed`'s**, not the type's raw `MOVES`
   (`docs/MOVEMENT.md` §1). run86 measures the difference on the only ride
   ever captured: East Indies' `1/22` prints `myspeed` **30** on a
   `MOVES` of 25 — the Whales rare's `+WHALES_SHIPS_MOVE%` — and steps
   `(−27, −13)` a frame. This crate set the raw 25 here and nowhere else,
   which cost the passenger 933 units over 190 frames of water and held
   East Indies' word at 7448 for three sessions;
2. `same_damage` — the boat takes the caster's damage as a fraction of its
   own hit points, in 256ths;
3. `set_new_location(boat, spot, 1, 1)` — onto the water the spot search
   found. run57 block 3609: the barge `1/14` at `(41112, 33695)`, whose
   `los_x/los_y` `(40872, 33720)` is the snapped birth point it was moved
   from. The pushes are `x, y, 1, 1` (`671027`..`67102f`), so a move that
   crosses a half-cell lights the boat's **whole disc at the spot**
   (`docs/VISION.md` §11);
4. `set_angle(boat, caster.angle, ·, 1)` — the snapping form, guy included;
5. the caster's **whole order list** moves onto the boat, in order, and the
   boat `kill_current_order`s the cast at its head;
6. the caster's **path stack** is inverted and popped onto the boat, which
   restores the order it was in, so the boat inherits the route whole; the
   boat's `unit_masks |= 0x800000`;
7. the top waypoint's embark flag (`4`) is cleared when
   `get_tregion(top)` equals `get_tregion(spot)` — the boat is already on
   that side of the shore and must not board a transport of its own;
8. `clear_orders(caster)`, the selection swap, `replace_hotunit`, and
   `go_inside(caster, boat)`.

run57 block 3609 backs 5–7 field for field: the barge's eleven `PATHDATA`
entries are the scout's, and the top's `flags` is `0` where the scout's
was `4`.

### 6.3 What lets the path cross water at all

Three predicates, and none of them is `can_transport` alone:

- **`UnitData::invalid_loc@00607c30`** splits on the type's domain. The
  land arm refuses `surface == 0x20` unless `(param_6 || param_7)` **and**
  the raw `unit_masks & 0x800000` — not `can_transport`, which would also
  read the veto and the type flag. Its head is the other half: a path stack
  whose **top** carries `flags & 4` forces `param_6 = 1`, which is what
  lets `move_step`'s all-zero call step onto the water at all. The sea arm
  is the mirror — dry land refuses a boat unless the caller asked, the boat
  `can_transport`, and the cell is not `WData.flags & 0x70` — and that is
  the **disembark**.
- **`PathFinder::calc_cost`**'s embark tail (`docs/PATHFINDER.md` §5) is
  gated on `can_transport`.
- **`PathFinder::find_wpath@00688fc0`'s pull-back is skipped entirely for
  a unit that can board** (`00689375`: the walk runs only when `domain < 2
  && (!is_on_map() || !can_transport())`). This is the load-bearing one,
  and it is easy to miss: without it the goal is dragged toward the unit
  until its tile region matches, which for an island target means back onto
  the unit's own island, and the search then plans a route to the shore
  and stops. `docs/PATHFINDER.md` §14.

### 6.4 Coming ashore — `eject_contents` and `come_out`, the cast run backwards (2026-09-01)

**How a barge reaches the shore at all** (item 1143). The boat's goal is
on land, and a sea unit's `invalid_loc` refuses land unless the path's top
carries `flags & 4` (§6.3). `do_move`'s waypoint take sets that bit on a
final entry in another tile region than the boat's (`docs/ORDERS.md` §4.4,
`5f865b`), so `find_path` keeps the land goal rather than pulling it back to
the water, and the step that crosses the waterline is this arm. East Indies'
barge `1/42` takes its final leg to (38016, 11136) on frame 6271 and prints
it `flags 5` on 6316; without the bit this crate walked it to (38028, 11542),
the last water tile, and stood it there loaded (`docs/AI.md` §90).

`set_new_location`'s sea arm calls `Object::eject_contents(0, -1, 1, 1)`
(`5f8fc8`: the four pushes are `param_4 = 1, param_3 = 1, param_2 = -1,
param_1 = 0`) and then dies if `num_inside(1)` is 0. For each contained
object, in order:

1. **`param_4 != 0`** — `unit_masks &= ~0x4000000`, `path.length = 0`,
   `close_orders(0)`, `clear_partial_path`, `update_action` on the
   *passenger*. Vacuous after a `cast_transport`, which emptied both;
   kept because it is the arm the call passes.
2. **`Unit::come_out(passenger, 0)`** — the spot, and §4.1's army coin at
   its tail.
3. **`Unit::same_damage(passenger, boat)`** — the mirror of §6.2's line 2.
4. **`param_3 != 0`**, and then the passenger's type's `uber_size`
   (`+0x308`) splits it:
   - **`== 1`**: the boat's whole order list is moved onto the passenger,
     one `remove_current`/`add` at a time; then `Stack<PathData>::invert`
     on the **boat's** path and a pop-and-push of the whole stack onto the
     passenger's, which restores the order it was in; then the passenger's
     **top** waypoint is popped, `flags &= ~4`, and pushed back — the
     embark flag cleared unconditionally, where §6.2 step 7 clears it only
     on a region match.
   - **`> 1`**: `Unit::reset_move_orders(boat)` instead, and — for a
     passenger that is not AI-driven or has no army — a `Group` insert that
     moves the boat's group membership onto the passenger
     (`Group::set_up_insert` / `push_group` / `finish_insert`). **Not
     modelled**; no capture disembarks a squad.
5. The transport bit back: with `transport_type(passenger) <= ` the
   leader's level, and unless `is(0x45)` without `leader_options.+0x1c &
   2`, `passenger.unit_masks |= 0x800000`. Then the selection swap and
   `replace_hotunit`.

**`come_out`'s host arm — every term of the spot is the boat's.** The
function splits on the host's vslot `0x1c` (a unit, or a building). For a
**unit** host:

| term | value |
|---|---|
| centre | the host's position |
| bearing | the **host's** `angle` (`+0x50`) |
| inner | the **host's** type `block_radius` (`+0x240`) |
| outer | inner `+ UNIT_DISEMBARK_DISTANCE` (`3/1 tile` = 576) |
| arms | on the **passenger's** `block_radius`: `0` → `FILTER_ALL`, the doubled ring, then the host's own point; non-zero → `FILTER_NOT_ME`, then the same ring with `nocoll 1`, then **refuse** (the passenger stays aboard) |

All three come off the host object in `eax` at `61845c`..`618483`
(`0x50(%eax)`, then `0x240` off `0x18(%eax)`, then `constants->+0x9c`
added); the arm test at `618490` reads `0x240` off `0x18(%ebx)`, and `ebx`
is `this`. The decompiler folds the two into one local, which is the easy
thing to misread — and the two answers differ by a whole ring. Then
`set_angle(passenger, host->angle, ·, 1)` at `6191f4`,
the snapping form, which turns the crew with it; and
`set_new_location(passenger, spot, 1, 1)`, whose `param_3` seats each crew
guy **on** its track offset rather than letting it walk there.

**run57 pins the whole of it on one frame.** The barge `1/14` stands at
`(35740, 26706)` with `angle -13303808`, a degree and a quarter west of
due north, and `BLOCK_RADIUS 3` makes the ring `[144, 720]` with the
sweep's own step of `(720 − 144) / 8 = 72`. The first ring at the first
bearing projects to `(35737, 26562)`, which snaps to **`(35736, 26568)`** —
block 3979's scout, exactly; and its second `GUY` sits at `(35640,
26616)`, the track offset `(−96, 48)` at that facing. Block 3979's
`MOVEORDER` and both `PATHDATA` entries are block 3978's barge's, field
for field, with the top's `flags` `4 → 0`.

**And the ring only reaches land because the barge marks no collision
cells.** `docs/COLLISION.md` §2's region gate is `get_tregion` of the
figure's *tile* against the cell's `region`, and for a boat on the water
half of a coastal cell those differ — so the boat's `BLOCK_RADIUS 3` disc
marks nothing at all. With the crate's earlier plain `region_of` the barge
filled its own cell and the first three rings were refused.

**`come_out`'s tail, for a computer player's unit** (item 1164). Past
`set_new_location` (`6186c1`), `6187c8` skips on `leaders.list[who] & 4`
(`is_human`), and `6187e4`..`618811` skip a plane and the types `0x32..0x35`.
Every other unit takes step 1's four again **from the spot** (`618813`..
`618836`), so `orders_x/y` is the landing point; `dest_angle` stays the
boarding heading, as the `set_angle` comes after. A human's unit keeps its
boarding point (run249's `0/6`).

**And the passenger keeps its speeds** (item 1164): `GuyData +0x80`/`+0x84`
are written by `Guy::move`, `Guy::clear` and `Guy::init_real` alone, never by
either `set_new_location`, so the passenger lands at the average the ride
froze, and its first turns ashore are `turn_speed`'s divided ones
(`docs/AI.md` §95 has run420's measurement).

**And the boat's figures move on the frame it dies** (item 1191).
`Object::die(boat, 0)` → `Unit::close` → `Object::close` writes neither the
guy array nor its counts, and `Unit::process@00610bc0` runs `Guy::process`
right after the think it died in: a boat standing on its `des` on the walk
with `stopped` set pays the arrival stand on its last frame — a draw no dump
shows. East Indies' `1/56` on 8519 (the journal, item 1191).

`crates/sim/src/transport.rs`'s `disembark` is steps 1–4 in the original's
own order. What is **not** established: step 5's `is(0x45)` clause and the
`leader_options` bit (the crate's `auto_transport` is never cleared by
boarding, so the passenger keeps the flag either way and no capture
separates them); the `uber_size > 1` arm; and the placement's own
fallbacks, which no capture has forced.


## 7. The civilian's island — `Unit::think_civilian_transport(colonise)@005f40d0`

Two callers, both in the unit AI (`Unit::think` → `think_peasant` /
`think_scout`, `docs/ORDERS.md` §4.5):

- `think_peasant` (line 53): an AI-controlled citizen (`unit_masks &
  0x40000`, base `0x32`/`0x33`) with nothing to do calls it with
  **`colonise = 1`**; a 1 return ends the think.
- `think_scout` (line 580): once the scout's own search has failed
  (`best > 99,999,998`): a citizen without `unit_masks & 0x100` in a region
  with no city of mine returns 0 instead; otherwise **`colonise = 0`**.

Run21 enters it at frame 275 — a citizen's idle think, long before any dock,
and it returns without sending anyone (the leader's level is 0).

**The gate.** A sea unit (`domain == 1`) whose type lacks `unit_flags &
0x10` returns 0 at once. ~~`domain == 2` always.~~ **An air unit skips the
capability gate altogether** — the `domain != 2` test wraps only the gate,
not the search (audit A.23), so a plane reaching this from `think_scout`
goes straight to the region search. For a land unit: `leader level <
transport_type(u)`, or
`!can_transport(u)` (§3.2), or `!is_cargo(u)`
(`ObjectData::is_cargo@00653600`: on the map, a unit type in
`0x32..=0x19d`, `type->domain == 0`) → **no**: return 0, or
`think_scout(1)` when `unit_masks & 0x100` (an exploring unit).

**The colonise gate**, `colonise = 1` only: `xport_peasants < city_num`
(`LeaderData +0x9c0` / `+0x3f8`), the unit's cell is **mine** (`WData.who
== who`) and my `reg_cities[unit's region] != 0`. Otherwise fall through to
the scout tail (return 0 / `think_scout(1)`).

**The candidate regions.** For every land region `r` in `1..=0x3f` with
`size != 0` — ~~and `r != mine`~~ **the loop makes no such test**
(`005f4283` gates on `Region.size` alone, 2026-09-01); what keeps a scout
from choosing home is the `scouted` bit its own tail has just set — let
`g = Region::go_here(r, who)` (§9.4):

- `colonise = 1`: `g & 1` (a free resource region) and my
  `reg_xport_peasants[r] == 0` — nobody already sent there; or, failing
  `g & 1`, nothing.
- `colonise = 0`: `g & 1`, or `g & 2` (an enemy holds it and I am not
  stronger); and the region's `scouted` bit for `who` is **clear**
  (`Region +0x34 BitMask<8>`, data at `+0x40`).

Then a sea region `s` in `0x41..=0x7e` with `size != 0` that `is_coast` of
**both** my region and `r` (§9.2); none → skip `r`.

**The cell.** Sample `r`'s cell list (`Region::coords`, a `WCoordList` —
every cell of the region, `size` of them) with stride `max(16, size /
50)`, starting at `(o + frame) % stride` and, when a whole pass finds
nothing, the next phase, up to `stride` passes; for each sampled cell that
is `coast_here(r, s, x, y)` (§9.3) and has `WorldData::num_waterhalf ==
0` — a cell not flagged `0x100`, or one with no water tile among its
sixteen: the shore cell on the **land** side (audit A.32): `score =
vector_dist(|dx|, |dy|) × max(1, danger)` — `dx, dy` from the
unit's own cell (`coord >> 8` through `div_3_table`), in cells, and
`danger` the leader's grid at half resolution (`WorldData +0x13c`,
`danger[who][(y/2) × reg_xs + x/2]`); keep the minimum (`≥ 0` only — the
listing at `0x5f4578` confirms `|dx|`, `|dy|` in `ecx`, `edx`). The first
pass with any hit ends the sampling for that region.

**The order.** With a best `(r, cell)`: a fresh `Group` of this one unit,
`Groups::push_group`, and `action_move_near` to the cell's centre (`cell ×
0x300 + 0x180`), `QUEUE_NEW`, order `EXPLORE_TO` when `unit_masks & 0x100`
else `MOVE_TO`. Then, `colonise` only: `xport_peasants += 1` and
`reg_xport_peasants[region of the cell] += 1` — the **destination cell's
own `WData.region`**, not `r`; `coast_here` accepts a cell whose region is
`r` or `s`, so a sea-side cell would book against a sea index, past the
64-entry array (audit A.36; whether `num_waterhalf == 0` ever admits one is
§13's). Return 1. No best: return 0, or `think_scout(1)` for an exploring
unit.

So the census's `xport_peasants` / `reg_xport_peasants` (`docs/AI.md` §2.3
steps 3, 8, 10) are zeroed by the sweep and **incremented here between
sweeps** as a running count of citizens already dispatched — the "one
colonist per city, one per region" throttle. `docs/AI.md` §2.3 does not
name this writer; amended.

**This is not the pathfinder's `go_here`.** The tile chosen is on `r`'s
coast; the walk there crosses `s` by the ordinary path with `0x800000`
set, boarding at the near shore (§6).

**Its caller, and the tail that reaches it (2026-09-01).**
`Unit::think_scout@005f6010`'s own tail, at `005f6d74`, when the whole
search came back above `99,999,998`: a non-air unit sets
`Region.scouted`'s bit for its leader on **its own region** (`+0x40 +
who/8`, the `BitMask<8>` at `+0x34`) and clears the mask's `flags`; a sea
unit goes to `add_to_army` instead; a citizen (`0x32`/`0x33`) that is not
exploring and has no city in its region returns 0; everything else calls
this with `colonise = 0`. So the mark and the read are one mechanism, and
a scout gives up on home exactly once.

**Landed 2026-09-01, and the diff backs the choice.** run54's frame 3584
is the AI scout `1/0`'s: this crate picks cell **(46, 33)** and issues a
`MOVE_TO` to its centre `(35712, 25728)`, which is the destination
run57's block 3585 prints, and the eleven-waypoint route the pathfinder
then plans is that block's `PATHDATA` list entry for entry.

**And the `colonise = 1` caller landed with it (2026-09-01, later the same
day).** `Unit::think_peasant@005f5760`'s head, at `005f5809`, after the
idle gate and **before** `find_build_spot` and the gather search: an
AI-driven unit (`unit_masks & 0x40000`) whose **base type** is `0x32` or
`0x33` asks, and a `1` back ends the think. The test is the base type and
not the worker category, so a scholar (`0x34`/`0x35`) never asks even
though `think_peasant` serves it. Until this the colonise gate, the
`xport_peasants` throttle and `reg_xport_peasants` had no writer between
the census's sweeps, and §7's whole `colonise = 1` half was reachable only
from a test.

**And the cell it picks is diff-backed too, since 2026-09-01.** run57's
`1/15` — the AI's second colonist — is idle on 3735 and takes a
`MOVE_TO` to `orders 30360, 26520`, cell **(39, 34)**'s centre plus the
order's own 24. This crate now picks that cell, on that frame, from that
unit's position: the scan is region 8, `size` 100, stride `max(16,
100/50) = 16`, phase `(o + frame) % 16 = 6`, and of the six cells the
first pass samples the four `coast_here` accepts score 23, 19, **15** and
18 — the minimum is (39, 34). It had been (37, 36) until §9.3's
`get_tregion` correction landed, and `1/15` then stood where the
original's does for all four thousand of run57's frames.

run57's `1/11` is the diff: it finishes a build on 3580, and on **3581**
the original holds it in a fresh `group 65` with a `MOVE_TO` whose
`orig 38784, 24192` is cell **(50, 31)**'s centre — this function's
`cell × 0x300 + 0x180`. With the arm in place this crate sends the same
citizen to the same cell on the same frame, and East Indies' long word went
**3687 → 3978** on it (`docs/SYNC.md` §3.23).

## 8. The army's transporting — `Army::do_transporting@006f4690`

`ArmyData`'s fields, for the reader (`types.txt`): `+0x0 valid`, `+0x2
army`, `+0x4 status`, `+0x8 reg`, `+0xc role`, `+0x10 num_units`, `+0x14
num_captains`, `+0x18 num_standard`, `+0x1c num_decoys`, `+0x20 city`,
`+0x24 navy`, `+0x28 human_frame`, `+0x2c hurry`, `+0x30 target_o`, `+0x34
target_who`, `+0x38/0x3c x, y`, `+0x40 angle`, `+0x44 rally_dist`,
`+0x48/0x4c muster_x, muster_y`, `+0x50 muster_angle`, `+0x54 list[16]`,
`+0x94 who`, `+0x96 num_groups`. `status` is a bit word: `1` mustering,
`2` marching, `0x10` forming, `0x20` defending, **`0x40` transporting**;
`Army::process` (line 182) runs `do_transporting` when `0x40` is set,
after the other four.

### 8.1 How an army gets there — `Army::do_mustering@006f4260`

The transporting state is entered from mustering, in two places, and both
read the census's **expand bit**, `strategy[reg] & 8` (`docs/AI.md` §2.3
step 15: set for a `sea_map ≥ 3` map's regions with no war or ally bits,
and for `sea_map == 2` when some resource region is empty):

- still mustering (`release_mustering() == 0`), with no active muster city
  or no muster spot: `strategy[reg] & 8 && num_captains > 7 && leader_flags
  & 0x300` → `status = 0x40` (`+0x14` is `num_captains`; `+0x10` is
  `num_units` — audit B.51 corrected the field);
- released, not a navy: `strategy[reg] & 4` on difficulty `< 3` → `0x20`
  (defend); else `strategy[reg] & 8 && leader_flags & 0x300` → `0x40`;
  else `2` (march).

`leader_flags & 0x300` is "level ≥ military" — the AI checks it directly
rather than through `can_transport`. Run21's first `do_transporting` is
frame 14586, eleven thousand frames after the dock.

### 8.2 The step itself

Runs only when the army stands in its own region (the cell region at
`(x, y)` equals `reg`); otherwise returns 0 — an army already at sea, or
landed elsewhere, is left to the other states. Then, over every land
region `r` in `1..=0x3f`, `r != reg`, `size > 0`:

```
s = strategy[r]; g = go_here(r, who)                      # §9.4
if !(s & 2 || g & 2 || g & 4): continue
v = size × (g & 4 ? 3 : s & 2 ? 2 : 1)
v /= vector_dist(reg.coords[0] − r.coords[0]) / 4 + 1        # listing 0x6f478e: the two regions'
                                                             # FIRST cells, in cells
city = −1
for each of my cities c below city_mark, active, in region r, flagged 2 (attacked):
    v *= city_level(c); city = c                             # every such city multiplies
best = max v (strictly greater), with its r and city
```

With a best `v > 0`: `reg = r`. If `city < 0`: `find_target()`; if it set
no `target_o`, `reg` is put back to the cell region and the step returns
1 with nothing changed. If `city ≥ 0`: `target_who = who`, `city`,
`target_o = the city's object`, and `find_muster_spot(target_o, who, 1)`;
when that fails the muster tile is the city's own cell, one row below it
(or above it on the map's last row). Either way `status = 2` — marching —
and return 1.

So "transporting" is a **one-shot retarget**: it picks the region and hands
the army to marching, and the walk there is the units' own transport-aware
pathing. The weights: an enemy holds `r` and I have cities there and am
not weaker (`g & 4`) → ×3; my `strategy` says I am the stronger side of an
active war there (`s & 2`) → ×2; and the region's size is the base, divided
by a quarter of the first-cell distance plus one.

### 8.3 The navy — `Armies::init_navy`, `Armies::send_navy`

`init_navy(who, city, sea_reg)@006f31b0` = `init_army(who, city)` then
`navy = 1`, `reg = sea_reg`. Its one caller is `Leader::create_units`'
sea branch (`docs/AI.md` §2.18; `ai_units.rs`), when no army of mine is
near the city and a friendly city is found — run21 at frame 9981. A navy
army is one whose `navy != 0` and whose `reg` is a sea region.

`send_navy(who, target_o, target_who, reg_a, reg_b)@006f2c90`, from
`Army::find_target` (line 1084) when a **non-navy** army has taken a
target whose cell region differs from its own: for an AI leader that is
active and not human (`leader_flags & 5 == 1`), not `leader_flags2 &
0xa`, every valid navy army with `num_captains > 2` (`+0x14`) whose sea region
`is_coast` of both `reg_a` (the army's) and `reg_b` (the target's) is given
the same `target_o` / `target_who` and `Army::process(0)`ed at once. The
escort call. `Army::process` also forces `set_stance(3)` on a navy every
step (line 185).

`LeaderData::get_ships_speed_upgrade@006da800`: the count of
`SHIPS_FASTER_1 …` (0x2ec, 0x2ed, 0x2ee) held, where the middle one
(`BUY_SELL`, 0x2ed — the enum's name for that slot) is also granted by
tribe bonus 4. First entered at frame 4376 in run21 — the AI's first ship.

## 9. Regions: coasts

`Region` (`0x88` bytes): `+0x8 flags` (`4` player region, `8` resource
region — `docs/AI.md` §15.8), `+0x10 region`, `+0x14 size`, `+0x34 scouted
BitMask<8>`, `+0x44 coastal BitMask<64>`, `+0x58 coast BitMask<64>`,
`+0x6c coords WCoordList` (every cell, `size` of them; list at `+0x7c`).
A `BitMask<n>` is `bits, size, flags, data…` — the data at `+0xc`, so
`scouted` reads at `+0x40`, `coastal` at `+0x50`, `coast` at `+0x64`.

**Region numbers.** Land regions are `< 0x40`, sea regions `≥ 0x40`
(`is_coast`, `num_coasts`, `set_coastals`); run20's one ocean is region
**65** (`0x41`). `docs/AI.md` §2.3's "sea `0x3f..0x7e`" and the harness's
`n < 0x3f` split (`diff.rs`, `build_sim`) are both off by one at a boundary
no dump has yet hit; corrected in the harness with this document, the
`% 0x3f` storage note unchanged (`0x41 % 63 = 2`).

### 9.1 `Regions::set_coastals@0067fd70` — the writer

Clears every region's three masks. Then for every cell of a land region
`L`: each of the eight neighbours (`move_x/y[1..8]`, the compass table at
`.rdata 0x00adcaf0/0x00adc400`: `(-1,-1) (0,-1) (1,-1) (1,0) (1,1) (0,1)
(-1,1) (-1,0)`) that lies in a **sea** region `S` sets `L.coast[S − 0x40]`;
and, from such a coastal cell, each of the next forty offsets
(`move_x/y[9..48]`, rings 2 and 3) that lies in a **different land**
region `L2` sets `L.coastal[L2]` and `L2.coastal[L]`. Then
`Region::finalize_coastals` on every land region closes `coastal` over
shared seas (two land regions both coasting a sea that a third coasts).
`coast` is the mask this document uses; `coastal` is `find_target`'s and
is not read further here.

### 9.2 `Region::is_coast(r, other)@00680f90`

`r == other` → 1; land `r`, sea `other` → `r.coast[other − 0x40]`; sea `r`,
land `other` → `regions[other].coast[r − 0x40]`; two of a kind → 0.

### 9.3 `Region::coast_here(r, s, wx, wy)@00681020`

`is_coast(r, s)`, and the tile's cell region is `r` or `s` (the other
becomes the sought one), and one of the eight neighbouring **cells**
(`move_x/y[1..8]`, the tile stepped a whole cell and re-read at its centre
tile `×4 + 2`) lies in the sought region → that neighbour's index `1..8`;
else 0. A tile on the shore between the two.

**The two reads are different functions, and the crate had them the same
way round for a fortnight** (2026-09-01). The *own* read at `00681039` is
`WData +4` of the cell handed in — the plain cell region. The *neighbour*
read at `0068106a` is `WorldData::get_tregion` of that neighbour's centre
tile, and `get_tregion` answers a **coastal cell's `region2`** — the sea
region — when the tile it is given lies on ocean (§10, `docs/AI.md` §2.3
step 10). That is the whole point of the function: the water half of a
coastal cell is a *land* cell in `WData.region`, so a shore read with the
plain region can only ever see a wholly-ocean neighbour, and a land cell
one cell inland of the waterline never coasts anything. `crates/sim` asked
`World::tregion` — which is `region_of(cell_of_tile)` and **not**
`get_tregion`, whatever its name suggests; the refined one is
`World::tregion_alt`.

What it cost: `think_civilian_transport` (§7) samples a region's cells and
keeps only those `coast_here` accepts, so the colonist's destination was
drawn from the wrong set. run57's `1/15` is the diff — the original sends
it to cell **(39, 34)** on frame 3735 and this crate sent it to (37, 36),
which is nearer by the same scoring and simply was not the original's
candidate. `docs/SYNC.md` §3.25.

This is `docs/SYNC.md` §3.24's lesson in a second caller, and the two
together are worth stating as a rule: **wherever the original calls
`get_tregion`, `region_of` is not a substitute** — the difference is
invisible everywhere except the one case each gate exists for.
`World::tregion`'s remaining callers are unaudited and are the queue's.

### 9.4 `Region::go_here(r, who)@006810f0` — the AI's verdict on a region

Bit 1, **free to settle**: `r.flags & 8` (a resource region) and my
`reg_cities[r] == 0` and my `reg_peasants[r] == 0` (`+0xfde`), and no
*other* active leader (`leader_flags & 2`) has `reg_cities[r] × 200 >
size` — i.e. nobody has settled it densely enough to call it theirs.

**`Region.flags` is not derivable from the cells**, and until 2026-09-01
this bit was a seam because of it: the word is the map generator's
(`Map::region_flags`), and nothing in `WData` implies it. It is read from
the dump instead — `Regions::log_data@00681280` writes a `REGIONS` block
with every slot's `flags`, an `InitialDump` carries it, and the harness
installs it through the same map the cells were numbered by. East Indies:
`0xa8` on the eight middle islands (bit 8 set), `0xa4` on the two the
players start in, `0xa0` on the two smallest. Without the block a world's
regions keep `flags == 0` and `go_here` answers as it did before, so a
capture that lacks one simply never colonises.

Then over every other active leader `i` at war with me on either side
(`diplos[who][i] == 0 || diplos[i][who] == 0`, `+0x74 int[8]`) with
`reg_cities[i][r] != 0`: bit 2 if my `reg_cities[r] == 0` or my
`strategy[r] & 4` (I am the weaker there); else bit 4. So bit 2 is "an
enemy holds it and I do not, or not strongly", bit 4 "an enemy holds it
and so do I, and I am not the weaker".

**Every `reg_cities` read here is `LeaderData +0x125e`, the leader's
own, and a human's is live** (item 642, 2026-09-23). The loops index
`leaders.list[i].field_0x125e` for every slot. The original's census
runs for a human leader too (`docs/AI.md` §23.1), so the human's count
is there to read. This crate's census does not run for the human, so
until item 642 the human's `census.reg_cities` was all zero, and bits 1
and 2 never counted the human's cities. `Sim::go_here` now reads every
*other* leader through `Sim::leader_reg_cities`, the recount by region of
the four city types that the census's step 8 writes, and it falls back to
that recount only for a leader whose census has never run. The asker's
own count is its census, as before.

**What it cost, and the diff** (run159,
`run159_s_word_frame_is_widened_whole`). East Indies' AI scout `1/0` runs
out of land on 11549. Both sides spend `think_scout+0x941` and no
`+0xaba`, and the tail asks §7 with `colonise = 0`. The human's home
island, region 2 here (`flags 0xa4`, unscouted by leader 1), is a
candidate only through bit 2. The original sends the scout to cell
(19, 13) there, and block 11550's order is (15000, 10392). This crate
answered `g = 0` for region 2 and sent the scout to (25, 43), region 11,
score 45, where (19, 13) scores 36. It boarded a boat on 11747, 45
frames before the original, which is the long word 11747. With the read,
the order, the path and the boarding agree: both sides hold `1/44` from
block 11793, and the word is **13640**.

**Not established**: the recount answers at once where the census
answers at its next sweep. A human city founded or lost within 200
frames of a `go_here` call would read differently here. No capture has
put one there.

`Region::num_coasts(r)@00680760` counts the sea regions `0x41..=0x7d` in
`r.coast`. Sea index `0x40` is never assigned — `Regions::find_all` seeds
the sea counter at `0x40` and pre-increments, so the first sea region is
`0x41` (run20's 65) and land regions start at 1; `0` and `0x7f` are
excluded from the flood fill (audit B.32) — so the skip is exact. **For a
sea region the count is 1**, itself: the only arm it can take. The census's
"more than one coast" test (`docs/AI.md` §2.3 step 13) asks it of the water
cell's own region, so that clause never holds and the size test decides
(audit B.41).

## 10. The dump records

Under `DUMP_ALL` (run20, every frame block; `docs/ORACLE.md`):

- **`DOCKS`**: eight `length / size / increment` headers, each followed by
  its 20 `DOCK` records — `dock o reg gull_o who dock_flags` — free slots
  as `dock -1, o -1, reg 0, gull_o -1, who -1, dock_flags 0`
  (`DocksData::log_data`, `DockData::log_data`). Run20 has none active.
  **Run22** (`docs/ORACLE.md`; the run21 lobby with a `DUMP_ALL` window
  `[3579, 3582)`) has the first: block 3579 none, block 3580 `dock 0, o
  2010, reg 65, gull_o 15, who 1, dock_flags 1`, the building `o 2010` of
  leader 1 at `(44160, 41856)`, `orig_type 432` (`DOCK`). The same block
  shows every one of the AI's 14 units with `unit_masks & 0x800000` where
  block 3579 showed none, and the human's 6 without it in both — §4 to
  the unit. (A `FRAME n` block holds **two** `FULL DUMP`s, the end of
  `n − 1` and the start of `n`; a count over the block doubles.) Its trace
  repeats run21's frame 3579 to the seed: draws 0 and 1 are the gull's
  creation and `Dock::init+0x125`. **Owner 9 is not in the dump**: the
  block's 208 `ANIMALDATA` are all leader 8's, so the gull's record is not
  there to compare; the trace attests its creation.
- **`ARMY`**, sixteen per leader under `ARMIES`: `army who num_groups
  status reg role num_units num_captains num_standard num_decoys city navy
  human_frame hurry target_o target_who x y angle rally_dist [group ×
  num_groups] muster_x muster_y muster_angle` (`ArmyData::log_data`) —
  only `valid` ones. Run20's frame-1 block already carries the AI's first
  army: `status 1, reg 11, city 0, navy 0, x 39264 y 40800, muster 51,53`
  — `Armies::init_army` from the census's step 16.
- **`LEADERDATA`** (`LEADERS=9`): `naval`, `sea_combat`, `transports`,
  `fishermen`, `idle_fishermen`, `xport_peasants`, `reg_naval[scan]`,
  `reg_transports[scan]`, `reg_xport_peasants[scan]`. **`reg_docks` is not
  dumped**; the `DOCK` records' `reg` are, and `reg_docks[r]` is their
  count by region and owner — which is the assertion §12 names.
- **`CITY`**: `dock_tile` (`CityData +0x67`), run20: 1 for the AI's city
  from frame 1, 0 for the human's.
- **`UNITDATA`**: `unit_masks` / `unit_masks2` (`docs/ORDERS.md` §11), so
  the `0x800000` bit is visible per unit per frame.

## 11. Corrections to other documents, landed with this one

- `docs/AI.md` §2.3 step 3: `reg_docks` is kept by `Dock::init` /
  `Dock::close` (§5.2–§5.3), not `gain_/lose_building`.
- `docs/AI.md` §2.3 steps 3/8/10: `xport_peasants` and
  `reg_xport_peasants` have a second writer, `think_civilian_transport`
  (§7), between sweeps.
- `docs/AI.md` §2.3 preamble: sea regions are `≥ 0x40`, not `0x3f..`.
- `docs/SYNC.md` §3: a dock's completion is two draws (§5.2).
- `docs/AI.md` §9's family, cited there by name only, is now on the record
  by address: `Leader::check_transport@006bc5f0`,
  `Unit::think_civilian_transport@005f40d0`,
  `UnitData::can_ever_transport@0046f290`, `UnitData::transport_type@0046f790`,
  `UnitData::can_transport@0046f960`, `UnitData::needs_transport@00609920`,
  `LeaderData::can_transport@006e0c60`, `LeaderData::locked_transport@006d5230`,
  `LeaderData::get_ships_speed_upgrade@006da800`, `Docks::init_dock@00740fc0`,
  `Dock::init@00740a80`, `Dock::close@007409f0`, `Docks::close_dock@00740f50`,
  `Docks::remask_docks@00740c90`, `ObjectsData::find_dock@0065cfd0`,
  `BuildTypeData::is_dock_tile@00636700`, `SpellType::cast_transport@00670db0`,
  `Army::do_transporting@006f4690`, `Army::do_mustering@006f4260`,
  `Armies::init_navy@006f31b0`, `Armies::send_navy@006f2c90`,
  `Regions::set_coastals@0067fd70`, `Region::is_coast@00680f90`,
  `Region::coast_here@00681020`, `Region::go_here@006810f0`,
  `Region::num_coasts@00680760`, `Group::action_set_transport@007024b0`.

## 12. What the simulation carries, and what checks it

`crates/sim/src/transport.rs`: the level (§2, §4), the three unit
predicates (§3), `needs_transport` (§6), the dock registry as `reg_docks`
plus the two draws at activation and the decrement at close (§5), the gull
as a bird whose walk request and wing beat are on the stream (§5.2.1, with
`orders.rs`'s gaia arm and `anim.rs::is_air_gaia`),
`is_dock_tile` replacing the census's seam (§5.6), and the region coast
masks with `is_coast` / `num_coasts` computed from the cells (§9.1–§9.2).

**The boarding half landed 2026-09-01** and is `transport.rs` too, and the
**landing** half with it the same day (§6.4: `disembark` is
`eject_contents`' four steps, `come_out`'s host-arm ring and bearing, the
crew seated on its track offset, and the order list and inverted path
stack moving back):
`do_cast` and `cast_transport` (§6.1, §6.2), `board` and `disembark`,
`same_damage`, and `think_civilian_transport` (§7) with **both** of its
callers — the `think_scout` tail (`scout.rs`) and, since later the same
day, `think_peasant`'s colonist arm (`orders.rs`), which is what gives the
`colonise = 1` half a live writer. Around them:
`collide.rs::shore_step` is `set_new_location`'s conversion,
`orders.rs` carries a `CAST_SPELL` order and
`find_nearby_spot_type` (the `(not_o, not_who) = (-1, -1)` form, whose
collision half is `collide.rs::find_unit_with_radius` since 2026-09-02),
`path.rs` has `invalid_loc`'s three domain arms, `calc_cost`'s live
embark tail and `find_wpath`'s pull-back gate, `world.rs` has
`coast_here`, `num_waterhalf`, `Region.scouted` and `Region.flags`, and
`army.rs::go_here` answers bit 1.
`do_transporting` (§8.2) is `army.rs::do_transporting` since `docs/ARMY.md`
(2026-08-25), with the region-first-cell distance of B.55a; `init_navy`
on `create_units`' sea branch and `send_navy` are still seams there.

Checks, cheapest first:

1. **Run20, on disk**: the `CITY` record's `dock_tile` per city at frame 1
   (`rondata::diff` — the widening of the run20 test).
2. **Run22, on disk** (`rondata::diff::tests::run22_s_first_dock…`): the
   `DOCK` record's `reg` equals the harness's cell region at the building's
   position from the run's own `WORLD` block (the sea, 65); a dock placed
   and activated in the harness at that position takes slot 0, records the
   same region, and leaves `reg_docks` at 0 as the original's guard does;
   the `UNITDATA` masks show `0x800000` clear on every AI unit in block
   3579 and set on every one in 3580, and clear on the human's in both.
3. **run54, on disk** (`rondata::diff::tests::run54_s_24000…`): the word
   stands to **3608** with the gull spawned, angled at its own marked site
   and flying — 3579 and 3580 were the roll's missing mark and the birth
   walk request (§5.2.1) — and the dock's `gull_o` is run22's 15.
4. ~~The boarding and the sailing: a `UNITS=3` window over frames
   3600–3640 of the same game.~~ **Answered from run57, which was already
   on disk** (2026-09-01, and it is the queue's own "grep the dump before
   booking a capture" one level up): run57 is the same game at run39's
   detail for 4,000 frames, so blocks 3585–3609 carry the whole mechanic.
   What they say, and what this crate now reproduces: the caster is the AI
   **scout `1/0`**, not a citizen; its order list at block 3608 is
   `[CASTORDER spell 650 paid 0, MOVEORDER dest (41112, 33432)]` with an
   eleven-entry path whose top carries `flags 4`; the boat is `1/14`, guy
   `type 320` = `TRANSPORTBARGE`, at `(41112, 33695)` with the scout's
   path, the scout's orders minus the cast, and the top's flag cleared.
   run54's frame 3608 spends the three draws §6.1 names and this crate
   spends all three at the same sites.

   ~~The frame still parts, and by **one draw**: this crate spends a
   `Guy::set_anim+0x97a < Guy::inc_time+0x271` for the barge's brand-new
   guy and the original spends none, on 3608 or on any later frame.~~
   Settled by `docs/SYNC.md` §3.22 — the loop's bound is re-read, so the
   barge takes its own turn on the frame it is born and its guy is already
   walking when the clocks are stepped.

5. **run59's 5342, on disk** (2026-09-02,
   `rondata::diff::tests::run59_s_census…`): the barge `1/18` is born on
   the frame the original bears it and on the original's own cell, and the
   census window's 5,959 unit-frames all stand where the original's do.
   The capture had carried those positions for a day with nothing
   comparing them — the census read six goods a leader and left the rest
   of the file alone — and the widening is what found step 3's predicate.
   run63's window then stands whole too, 6,775 unit-frames with nothing
   ever off point.
6. **run57's 3581, on disk**: `1/11`'s colonist dispatch — the group, the
   `MOVE_TO` to cell (50, 31)'s centre, and the twenty-four-leg path —
   which is what §7's `colonise = 1` half is now checked by. With it East
   Indies' word is **3978**, and run57's four thousand frames have four
   units ever off the original's point rather than eleven
   (`rondata::diff::tests::run57_s_four_thousand…`).
7. **run86's 6924–7409, the only ride ever captured whole** (2026-09-06,
   `rondata::diff::tests::run86_s_window_is_the_transport_ride`): the
   cast, 190 frames of open water and the eject, 486 blocks at `UNITS=3`,
   with neither the barge nor its passenger ever off the original's
   position. It is what backs §6.2 step 1's speed, §6.4's ring on its one
   sample, and the freeze the passenger rides in. run82's own window and
   its shutdown dump at 6946 came with it
   (`run82_s_window_is_the_east_indies_ride_s_run_up`), which is where the
   792 was shown to be born inside the ride rather than before it.

## 13. What is not established

- ~~**The whole ride, end to end, is un-oracled** … the three places the
  792 can hide are the birth spot, the barge's plan and the ring.~~
  **Captured whole and settled, 2026-09-06 (run86, item 241): none of the
  three.** The barge's **speed** was the lag — `myspeed` 30 against the
  raw `MOVES` of 25, §6.2 step 1 — and with it right the birth spot, all
  190 frames of the crossing and `come_out`'s ring are exact on every one
  of run86's 486 blocks. East Indies' word 7448 → **7529**.
- ~~**A passenger's `x_internal`/`y_internal` while aboard.**~~
  **Witnessed on 191 blocks** (run86): `1/20` prints its boarding point
  `(34530, 32268)` from 7093 to 7283 and the ring spot on 7284, which is
  exactly what [`Sim::board`]'s freeze does. Not a seam.
- **`come_out`'s ring has one sample and cannot be told from a fallback.**
  run86's eject puts `1/20` at `(30408, 28152)` off a boat last seen at
  `(30521, 28243)` — §6.4's geometry reproduces it, and one sample cannot
  distinguish the first bearing of the first ring from a later one or from
  the host's own point. *Capture:* a second disembark, on any map, with a
  different approach angle.
- ~~**Which shipped row grants `TRANSPORT_BONUS`** (0x2ae).~~ **Settled
  from the data**: it is the third `BONUS` of `rules.xml`'s `TECHBONUSES`
  ("Units can be transported by sea", `preq0="Written Word"`), the bonus
  range being `0x2ac..` in file order (`docs/DATALAYER.md`). So
  `has_preq(TRANSPORT_BONUS)` is `has_tech(Written Word)` — the first
  Science technology — and run21's `check_transport` at frame 201 is that
  `gain_tech`. What `has_preq` does with a bonus row beyond its `preq0`
  (the government-tier arm, `docs/TECH.md`) is not exercised: the row has
  one prerequisite.
- ~~**`is_dock_tile`'s strip** (§5.6) … before the placement side relies
  on them.~~ Nothing on the placement side calls it (audit B.29); the
  census is its only reader, and run20's `dock_tile` is its check. The
  strip's two rows are as §5.6 says, doubly read.
- ~~**What clears the water apron** (§5.4) when a dock dies.~~ `mask_me`'s
  unmask, with `remask_docks` restoring overlaps (audit B.18).
- ~~**The barge's own guy does not wrap, and every other new unit's
  does.**~~ **Settled from the trace already on disk, 2026-09-01, and the
  booked `GUYS=4` window was not needed.** It is the loop, not the clock:
  `Objects::process_all@0065dce0` re-reads `unit_mark[who]` at the bottom
  of its inner loop, so the barge — cast at `o 14` by a scout at `o 0` —
  takes its own turn on the frame it is born. It steps the move order it
  has just inherited, `Unit::move_step` sets its guy to `CHAR_WALK`, and
  `Objects::inc_time` then finds `cur_time 1 < end_time`. Every other
  newborn in run57 is a **trained** unit, born in `Build::do_queue` in
  `process_all`'s second loop after every unit has moved, and each of the
  ten does wrap on its birth frame. `docs/SYNC.md` §3.22; East Indies'
  word 3608 → 3687.
- **`reg_xport_peasants` booked against a sea index** (§7, audit A.36):
  whether `coast_here` + `num_waterhalf == 0` can ever admit a cell whose
  `WData.region` is the sea's. If it can, the write lands in `reg_cities`,
  the next array. *Capture:* a `LEADERS=9` record after a colonist has
  been dispatched, `reg_cities[scan]` against the `CITY` count.
- **Whether `reg_docks` is ever non-zero in a shipped game.** Run22's dock
  registers in the sea (§5.2, §10); a dock whose centre cell is land would
  be one the placement rule put with its centre on the shore cell. If none
  exists, `reg_docks` is dead weight in the census's step 16 army score and
  everywhere else it is read. *Capture:* `DOCK.reg < 64` in any dump.
- **`vector_dist`'s arguments in `do_transporting`** are the two regions'
  first cells (`coords[0]`); whether `coords` is ordered so that the first
  cell is meaningful (row-major first, presumably) is `Regions::rebuild_
  coords`', unread. *Capture:* run22's `ARMY` records across a
  `do_transporting` frame (14586 in run21) — `reg` before and after.
- **`coords` holds cells.** `coast_here` indexes `wdata` with a coord pair
  directly and `set_coastals` walks cells, so a `WCoord` here is a cell,
  and `think_civilian_transport`'s `× 0x300 + 0x180` is a cell centre.
  Consistent; `rebuild_coords` itself is unread. *Capture:* the `MOVE_TO`
  target in a transported citizen's `UNITDATA` orders against the
  `WORLD` records — run22's window past frame 3608.
- The **`coastal`** closure (§9.1) and `find_target`'s use of it.
- **Where the gull flies** (§5.2.1). `do_air_physics`'s step —
  `bank_aircraft`, `pitch_aircraft`, `project`, `UnitData::invalid_loc`,
  `set_new_location` — and `do_strafe`'s sixteen-frame
  `find_new_air_target` are read in outline and not implemented, and no
  dump prints owner 9, so the position has no oracle. What *is* asserted
  is that neither spends a draw: `do_air_physics+0x639` fires three times
  in run54's 24,000 frames and all three are a wild bird's, under
  `do_air_patrol`. *Capture:* none available — the falsifier would be a
  `do_strafe`-chained draw anywhere in a trace, and there is none.
- **Whether a second dock's gull reuses a slot.** Every capture there is
  has exactly one dock; `Dock::close` leaves `gull_o` stale (§5.3) and
  `Unit::close` frees the object number, so the second gull should take the
  first's. *Capture:* a game with two docks and a `DOCKS` block.
- **The scoring term this crate cannot compute**: `think_civilian_transport`
  weighs a candidate cell by `vector_dist × max(1, danger)`, and `danger`
  is `WorldData::danger[who]@+0x13c` — an `int[reg_size]` **half-resolution
  cell** grid (`danger[who][reg_xs × div3(y >> 9) + div3(x >> 9)]`, as
  `Leader::produce_unit@006cb9e0:142` and four others index it). Nothing in
  this crate writes it, so every score here is its distance. Two things
  ride on that: the AI's four other readers index the same field **by
  region**, which is wrong and inert only because the grid is empty; and
  East Indies' frame 3584 picks the cell the original picks with the term
  at 1, which is evidence the grid was empty there too and not that the
  term does not matter. *Capture:* any dump with a `danger` block, if one
  exists; otherwise the writer has to be read.
- **The order of the region loops.** The original walks region *slots* —
  land `1..=0x3f`, sea `0x41..=0x7e` — and this crate walks its own dense
  numbering filtered by terrain, because the two bands do not exist here.
  The order decides a tie, `score < best` keeping the first. On East Indies
  the dump's land regions and the sweep's come out in the same order, one
  apart, so nothing has told them apart. *Capture:* a map whose regions the
  cell sweep meets out of the original's order.
- The blind list after run21: `Docks::remask_docks`, `ObjectsData::
  find_dock`, `Armies::send_navy`, `Group::action_set_transport` have not
  executed in any traced game. ~~`Region::coast_here`~~ — entered on run54's
  frame 3580, under the gull's first `do_strafe`. ~~`Unit::do_cast`,
  `SpellType::cast`, `SpellType::cast_transport`, `pay_cast_costs`,
  `SpellTypeData::get_job_time`~~ — all first entered on run54's frame
  3608, and all five are implemented from that frame's own draws.

## 14. Second reading — landed, 2026-08-25

Two blind readers on Opus 5, split §2–§4/§7 (A) and §5/§8–§9 (B), the
same day as the first reading; adjudicated in the main thread on Fable
against the decompile and the listing. `docs/audit/2026-08-25-transport.md`
is the record, verdict by verdict; the reports are at
`~/ghidra-projects/reading/transport-2026-08-25/`. The corrections above
carry their audit row (`A.23`, `B.13`, `B.51`, …) where they stand.

## 15. The player's toggle and the board line, measured (item 803, 2026-09-26)

`docs/GOLDEN.md` §28 is chapter twenty. It issued the toggle through the
original's own `CommandManager::issue_set_transport@00941910` on run249,
and this section records what that established about the mechanic.

- **§3.4's toggle is the whole command.** `process_set_transport
  @00948f60` → `Group::action_set_transport@007024b0` sets or clears the
  member's `unit_masks & 0x800000` and lays no order. The Chariot `0/7`:
  8388608 → 0 on 622, and back on 802. This crate's entry is
  `input::group_set_transport` → `Sim::set_transport`.
- **`BoardOrder` and `AwaitBoardOrder` are built by nothing a game
  reaches.** Their adders are reached only from `Group::action_board_ship
  @00700010`, which the never-issued `board_ship` command and
  `finish_insert`'s replay call, and from `Unit::check_meet_ship
  @00604550` under a `BoardOrder`'s own step. No dump on disk prints one.
  §6's Transport cast is what boards.
- **A human's move onto water without the bit.** The world plan runs
  straight onto the lake, with no pull-back and no embark flag. The unit
  walks it to the last land cell and its path and order end there: `0/7`
  at (8265, 35365) on 719. Nothing converts the step.
- **§6.4's step 1 is where `update_action` runs** (`Object::eject_contents
  @0064cd20`:115). It runs on the passenger at its boarding point with an
  empty list, and no second call follows when the boat's orders come back.
  So `orders_x/y` stay the boarding point and `dest_angle` the boarding
  heading. `come_out`'s `Unit::set_angle@00605400` writes `+0x50` and the
  guys, never `+0x58`. run249's `0/6` on 1160 shows both: (8428, 34200)
  and 1073741824. `crate::transport::disembark` has both.
- **A Dock placed on a refused tile moves along the spiral**
  (`BuildTypeData::snap_center@00636190`). A building of the Dock's
  lineage whose centred tile `blocked_site` refuses takes the first of
  `move_x/move_y`'s offsets 1–80 that clears. run249's Dock, asked at
  tile (53, 153), stands at (56, 155), offset 36. This crate has it as
  `Sim::snap_center_placed`, called from the harness's `add`.
  `Sim::init_build` keeps the plain snap. The same arm for a player's
  Woodcutter's Camp or Mine, and the oil types' cell snap, are not built.

**Not established.**
- Whether `init_build`'s callers other than the console's `add` — the
  AI's Dock and a player's `issue_build` — ever ask for a refused tile.
- ~~The boarding frame's figure update: the original ages `avg_speed` once
  more before the passenger freezes inside.~~ `Unit::process` gives every
  unit that goes inside in its own work its figures' frame
  (`docs/ANIM.md` §15, item 850).
- The disembarked figure's first animation and turn (run249 1162, 1165).

