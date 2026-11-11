# Item 4152, second (blind) reading: `queued`

- Slug: `queued`
- Leaves (all `int`, `LeaderData` offsets): `scholar_militia` (+0x9f0),
  `barracks_queued` (+0xa10), `stable_queued` (+0xa14), `factory_queued`
  (+0xa18), `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued`
  (+0xa24).
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.
- Sources: the Ghidra decompile export (`~/ghidra-projects/decomp`), the PE
  listing (`llvm-objdump` of `riseofnations.exe`, pre-dumped `.text`), and
  `crates/sim`.

## Layout facts used throughout

- `Leader` (size `0x6eec`) puts `LeaderData` (size `0x6ee4`) at offset 0:
  `Leader::track_unit_type (006e0dd0)` reads `who` as `this->field_0x8` and
  ORs `leader_flags` at `leaders.list + who` offset 0, and the listing
  (`6e0eeb`) rebuilds that pointer as `0xe3a390 + who*0x6eec`. So a
  `field_0xNNN` on a `Leader*` is the `LeaderData` field at `+0xNNN`, and the
  absolute form of slot `i`'s field is `0xe3a390 + 0x6eec*i + off`
  (`scholar_militia` `0xe3ad80`; the six queued leaves `0xe3ada0`,
  `0xe3ada4`, `0xe3ada8`, `0xe3adac`, `0xe3adb0`, `0xe3adb4`).
- `TypeIndex` (export `enums/TypeIndex.txt`): `SCHOLARS 0x34`, `MILITIA
  0x42`, `SPECIALFORCES 0x4b`, `BARRACKS 0x1ab`, `STABLE 0x1ac`,
  `SIEGEFACTORY 0x1ae` (not `FACTORY 0x1af`), `DOCK 0x1b0`, `LIBRARY 0x1b3`,
  `FIGHTERBOMBER 0x134`, `DISBAND 0x29a`; unit types are `0x32..0x19d`
  (`TypeData::is_unit_type (004707e0)`), building types start at
  `BASE_BUILDTYPES 0x19e`.
- `UnitType` fields read by the family (`types.txt`, `TypeData` and
  `ObjectTypeData`): `+0x40 where` (the `TypeIndex` of the building that
  trains it), `+0x1e8 attack`, `+0x218 domain` (`2` is the air domain by
  every use here), `+0x2f0 control_cost` (population), `+0x2c8 role`.
- `Type` vtable (`vtables.txt`, `Type::vftable 00b43cbc`): `+0xc
  is_unit_type`, `+0x60 is(t, strict)`. On an object type `is` is
  `ObjectTypeData::is (0065f7d0)`: own `TypeIndex` equal, or `t` in the type's
  `is_list` (non-strict).
- `UnitData`: `+0x9 who`, `+0xa o` (slot index), `+0x54` an unnamed union
  (`UnitData_u_84`; the crate calls it `rare`, holding a gather good, or the
  former type), `+0x64 queue_time`, `+0x68 unit_masks`, `+0xa0 num_queued`
  (short, the unit's own training queue).
- `LeaderData +0x5a22 num_queued` is `ushort[806]` by `TypeIndex`; absolute
  `0xe3fdb2 + 0x6eec*who + 2*t`.

Nets run for every leaf: the export by name (`->scholar_militia`,
`->*_queued`), the export by `field_0x9f0`/`field_0xa10..a24`, and the
listing (`exe.s`) by every register-relative store to those displacements
(`, 0xNNN(%` destination, `inc`/`dec`), by `lea` of them, by indexed stores
reaching them from a nearby array base (`0x9xx/0xa0x(%r,%r,4)`), and by the
absolute `0xe3ad80`/`0xe3ada0..b4`. Every hit was mapped to its function
through `INDEX.tsv`; hits whose base is not a `Leader`/`LeaderData`
(`Constants::init`, `TerrainMap`, `GraphicPieces`, `ConquestSetupWin`,
`TileSet`, `HotKeyGroupOut`, `ScenarioEditor::modify_terrain_map`,
`ConsoleWin::run_cmd` through `MiscAccess::graphic_pieces` at `0xc06214`,
`Game::do_frame`'s `prevFrameKeystate` copy, stack locals at `-0xNNN(%ebp)`)
are other records' fields and are not listed again below.

## scholar_militia (+0x9f0)

### 1. Writers

| writer | site | store |
|---|---|---|
| `Leader::track_unit_type (006e0dd0)` | `6e0ecd addl %edi,0x9f0(%esi)` | `+= delta` |
| `Leader::init (006e3930)` | `6e4c14 movl %eax,0x9f0(%ebx)` (`eax` = 0 from `6e4c09`) | `= 0` |

No other store: no absolute `0xe3ad80` anywhere in `.text`, no `lea` of
`0x9f0` off a leader, no indexed store reaching it. The other
`0x9f0(%reg)` stores are `Constants::init`, `HotKeyGroupOut`,
`ConquestSetupWin`, `TerrainMap`, `TileSet` (not leader records).
`Leader::Leader (006d2bf0)` references the `LeaderData` size `0x6ee4`
(a construction-time clear, not read in detail).

Callers of `track_unit_type` (every `calll 0x6e0dd0`, 6 sites; no vtable
slot holds it):

- `Unit::set_type (00612fa0)` `613097` (old type, `-1`, only under
  `SET_TYPE_NORMAL`) and `61361f` (new type, `+1`, under any mode but
  `SET_TYPE_LOAD`); so a fresh unit's first `set_type` adds, a conversion
  moves one from the old type to the new.
- `Unit::close (0060ee50)` `60f42a` (`-1`, the unit's own type, on removal).
- `Objects::init_unit (0065e0c0)` `65e228`, `65e4e4` (`-1`: a squad
  follower is taken back out).
- `SpellType::cast_create_decoy (00674370)` `67473b` (`-1`: a decoy with
  population is taken back out).

Each call site is gated the same way: the unit is its own captain (the
`+0x8e` sign test the export prints behind the folded name
`UnitData::is_captain`), `unit_masks & 1` clear, and the type has
population (`control_cost != 0`) or `is(FIGHTERBOMBER)` or is a government
hero. The last argument is always the unit's own slot `o`.

### 2. Event, step, predicate

Inside `track_unit_type(type t, delta d, slot o)`, after the
production-building tallies and the civilian tallies, the listing
(`6e0e87`..`6e0ed3`) does:

1. if `o < 0`: skip to the `SPECIALFORCES` leader-flag arm (not this leaf);
2. if not `types[t].is(MILITIA 0x42, 0)`: same skip;
3. read `f = units[who][o].+0x54` (signed); if `f < 0`: return;
4. if `types[f].is(SCHOLARS 0x34, 0)`: `scholar_militia += d`.

So the leaf counts **live, counted units whose current type is in the
Militia line and whose `+0x54` former type is a Scholar type**. Step is
`d` = `+1`/`-1`, a unit count. No floor, no clamp.

Where `+0x54` gets a Scholar type: `SpellType::cast_to_arms (00670880)`
stores the unit's current `ptype->type` into `+0x54` and then calls
`set_type(current_upgrade(MILITIA), 0)`. In that `set_type` the `-1` for
the old (Scholar) type fails step 2, and the `+1` for the new (Militia) type
passes step 2 and finds the Scholar in `+0x54`: **+1 on To Arms cast by a
Scholar**. `SpellType::cast_civilian (006704a0)` reads `+0x54` (filling in
Citizen `0x32`, or a tribe graft, when it is 0), and calls
`set_type(current_upgrade(+0x54), 0)`: the `-1` for the Militia type finds
the Scholar and gives **-1 on the return to civilian**; the `+1` for the
Scholar type fails step 2. `+0x54` is not cleared there. A Scholar militia
that dies goes through `Unit::close`'s `-1`: **-1 on death**.

A militia trained in a building has `+0x54 = 0` from `Unit::init (00612100)`;
`types[0]` is a good type (`FOOD`), whose `TypeData::is (004771c0)` is an
equality test, so it never counts. A Citizen militia's `+0x54` is
`0x32..0x33`, not a Scholar type. The `is` test is non-strict, so the
Korean Scholar (`0x35`) counts if its `is_list` names `SCHOLARS`; that list
is data and was not read.

Not checked: `SpellType::cast_immortal_hinf (00670600)` and
`cast_immortal_arch (00670740)` also write `+0x54` before a type change; if
either could ever target a Militia-line unit whose `+0x54` holds a Scholar,
the `-1` would miss and leave the count high. Their target predicate was not
read.

### 3. Reset

`Leader::init (006e3930)` zeroes it with its neighbours `+0x9f4..+0xa34`
(one run of `movl %eax` at `6e4c0e`..`6e4c74`). `Leader::init` is called
from game setup (`Leaders::init`, `Setup::build_game`,
`Setup::build_leader`, `MapScenario::make`, `WalkDataGame::walk_data`), the
scenario editor/reader, and `ConquestGame::build_conquest_game`. Nothing
else zeroes it.

### 4. Readers

- `TypeData::get_cost (00664090)`, the unit price ramp: for a Citizen
  (`0x32`, Korean `0x33`) the count is raised by
  `support(0x42) + support(0x43) - scholar_militia + support(0x44)`, for a
  Scholar (`0x34`, `0x35`) by `scholar_militia`. So a Scholar turned
  Militia is priced as a Scholar, not as a Citizen.
- `LeaderData::log_data (006e5110)`.

### 5. Crate

The event lives in `Sim::track_unit_type` (`crates/sim/src/units/store.rs`)
with `Unit::set_type`'s callers; its signature is `(who, ty, delta)` and has
no unit slot, so the former-type test has no input there. The crate instead
derives the value at the read: `Sim::worker_support`
(`crates/sim/src/economy/pricing.rs`) counts live, captain, non-decoy units
of `type_index 0x42..=0x44` whose `rare` is `0x34 | 0x35`. To emit the leaf
as a counter a builder either passes the unit to `track_unit_type` and adds
`types[t].is(MILITIA) && is_scholar(units[o].rare)`, or exposes the same
derivation to the leaders checksum (`Sim::leader_long`,
`crates/sim/src/checksum/leaders.rs`, beside the `*_units` arm). The
crate's `0x42..=0x44` range is a stand-in for `is(MILITIA)`; the two agree
only if the Militia `is_list` is exactly those three types (data, not
read).

### 6. Plain game

Yes. To Arms (`TOARMSSPELL 0x294`) is a Citizen and Scholar command in an
ordinary skirmish (the crate already routes it, `spell::TO_ARMS` in
`crates/sim/src/ai/transport.rs`); death and the civilian return are
ordinary too. No trigger, CtW or multiplayer gate on any writer. Whether
the computer player ever casts To Arms with a Scholar was not read.

### 7. Confidence

High on the writer set (both nets, the absolute form, and the one call
graph agree) and on the arithmetic (listing read end to end for
`track_unit_type`). Medium on which units can hold a Scholar in `+0x54`
while their type is a Militia (the immortal spells and the `is_list` data
are unread).

## The six queued leaves: shared writer set

The six leaves have one writer set. Every store to any of them sits in a
copy of one rule, **track-queued(t, d)**, which is
`Leader::track_queued (006e0f30)` out of line and inlined (with `d` fixed at
`+1` or `-1`) in ten other functions. Every copy also moves
`num_queued[t]` (`+0x5a22`) on the same statement, and the set of functions
that store `num_queued` in the listing is exactly this set plus
`Leader::init`. That is a check the nets agree: nothing stores the six
without storing `num_queued`, and the reverse holds too.

### The rule (listing `6e0f30`..`6e1044`)

Given type `t` (a `TypeIndex`) and delta `d`:

1. `t < 0`: nothing.
2. `num_queued[t] += d`, **unless `num_queued[t] == 0 and d < 0`** (16-bit).
3. `t` must be a unit type (`types[t]` vtable `+0xc`; devirtualized in the
   inline copies as `0x32 <= t <= 0x19d`) and `unittypes[t].attack
   (+0x1e8) != 0`; else stop.
4. By `w = unittypes[t].where (+0x40)`, first match:
   - `w == 0x1ab` (Barracks): `barracks_queued += d`, then `combat_queued += d`;
   - `w == 0x1ac` (Stable): `stable_queued += d`, then `combat_queued += d`;
   - `w == 0x1ae` (Siege Factory): `factory_queued += d`;
   - `w == 0x1b0` (Dock): `dock_queued += d`;
   - else if `unittypes[t].domain (+0x218) == 2`: `air_queued += d`;
   - else nothing.

   Each add is skipped **when that counter is 0 and `d < 0`**, tested per
   counter: `combat_queued`'s skip looks at `combat_queued` itself, not at
   the Barracks or Stable counter beside it. With `d = -1` this is a floor
   at zero; with `d < -1` (only the carrier paths below) a counter at 1 can
   go to `1 + d < 0`. The `+1` inline copies are bare `incl` with no test.

Units: counts of queued items (one per queued unit). The step is per
event, not per frame.

### Writers (`Name (addr)`, the listing's store sites, `d`, the event)

| writer | stores (listing) | d | event |
|---|---|---|---|
| `Leader::track_queued (006e0f30)` | `6e0fa7`, `6e0fc6`, `6e0fdc`, `6e0ffb`, `6e101a`, `6e103b` | arg | out of line; callers below |
| `Build::queue_up (00620f40)` | `incl` at `621a2f`..`621a79` | +1 | an entry appended to a building's queue |
| `Build::unqueue (006207c0)` | `6209dc`..`620a57` | -1 | an entry removed (vtable `Build +0x1c8`) |
| `Build::clean_queue (00620b60)` | `620d92`..`620e0d` | -1 per entry | a building's whole queue emptied, last entry first |
| `Group::action_spell (006fe1a0)` | `incl` at `6fe80c`..`6fe856` | +1 | a carrier's plane queued |
| `Unit::process (00610bc0)` | absolute `0xe3ada0(%esi)`..`0xe3adb4(%esi)` at `611362`..`6113dd` (`esi = who*0x6eec`) | -1 | a carrier's plane finished |
| `Unit::action_unqueue (005e1f20)` | `5e1ff3`..`5e206e` | -1 | a carrier's plane cancelled |
| `Wall::increment_stats (00643270)` | `incl` at `6433cc`..`64341a` | +1 | dead for these leaves (below) |
| `Wall::decrement_stats (00642da0)` | `642f22`..`642fa0` | -1 | dead for these leaves |
| `Wall::activate (0063e4b0)` | `63e668`..`63e6e3` | -1 | dead for these leaves |
| `Wall::close (0063f6e0)` | `63f93a`..`63f9b5` | -1 | dead for these leaves |
| `Leader::init (006e3930)` | `6e4c3e`..`6e4c5c` | `= 0` | reset |

Calls of `track_queued` (every `calll 0x6e0f30`; no vtable slot holds it):

- `Unit::close (0060ee50)` `60efed`: a unit with `num_queued (+0xa0) != 0`
  is removed: `track_queued(current_upgrade(FIGHTERBOMBER), -num_queued)`,
  then the unit's `num_queued = 0`. So a carrier lost with `n` planes queued
  takes `n` off at once (a `d < -1` site).
- `Leader::gain_tech (006dcb60)` `6dda28`/`6dda47`: on gaining a unit
  upgrade, a carrier's queued planes are moved from the old Fighter-Bomber
  type to the new: `(old, -n)` then `(new, +n)`.
- `Leader::gain_tech (006dcb60)` `6ddd04`/`6ddd4a`: every building queue
  entry of the upgraded line is rewritten to the new type: `(x, -1)`, then
  `BuildQueue::set_queue`, then `(new, +1)`. The listing settles `x`: on the
  direct match (`6ddca6 cmpl %eax,%esi; je 6ddcf4`) `esi` is the queued type;
  on the graft-chain match (`6ddcce`..`6ddceb`) the loop exits only with
  `esi == param_1`, so `x` is the **new** type and the old type's counts are
  not taken down. The decompile prints the same variable in both arms. On
  these six leaves the two arms net 0 whenever old and new types share
  `where` and `domain` (the `-1` cannot hit a floor, since the entry being
  rewritten is already in that counter); they differ only if an upgrade
  changes the training building.

Callers of the inline copies, by role:

- `Build::queue_up`: `Build::do_queue (0061e410)` (the repeat flag
  re-queues the item just finished), `Leader::produce_unit`,
  `produce_tech`, `produce_upgrade`, `produce_spell` (the computer's
  orders), `Group::action_queue_up` (the player's queue command),
  `Group::action_spell`, `Group::action_disband` (queues `DISBAND 0x29a`, not
  a unit type, so no step here), and the scenario layer
  (`ScenarioEditor::queue_unit`, `ScenarioRead::load_city_chunk`,
  `load_build_chunk`).
- `Build::unqueue` (only through `Build +0x1c8`, `calll *0x1c8(%r)`):
  `Build::do_queue` `61ec35`/`61eda0` with `(slot, 0)`, after
  `Build::finished` (`+0x1b0`) reports the head item done, i.e. **an
  item completes**; `Build::action_unqueue (00620280)` with
  `(last, 1)` (a cancel, refunded); `Build::activate (00623e20)` three
  sites with `(find_in_queue(...), 1)` (a matching entry in another
  building's queue cancelled when a building activates; predicate not
  read); `ScenarioEditor::unqueue_unit`; and `Build::unqueue` itself, which
  forwards from a non-first Library to the first Library's queue (the
  decrement then happens in the first Library's call).
- `Build::clean_queue`: `Build::close (00628980)` (two sites),
  `Build::swap_team (00628210)` (before the capture changes owner, so the
  old owner's counts fall; skipped for a Library), `City::close
  (00737550)`, `Leader::defeat (006ecb00)`, `Leader::victory (006ec9b0)`,
  `ScenarioEditor::delete_leader`, `ScenarioFuncSet::clear_build_queue`.
  Inside, a plain `Build` takes the inline path; any other vtable calls
  `+0x1c8`.
- Carrier copies: `Group::action_spell` is reached from
  `CommandPackage::process_spell (00948340)` and
  `Group::finish_insert (0070e620)`; per member that is castable for the
  carrier's craft, with `num_queued < 10` and room aboard, it pays, then
  `num_queued += 1` and the inline `+1` on `current_upgrade(FIGHTERBOMBER)`.
  `Unit::process`, when `num_queued != 0` and `leader_flags & 2`, adds the
  train acceleration to `queue_time`; once it reaches `train_time` and the
  population has room, `num_queued -= 1`, `queue_time = 0`, and the inline
  `-1` on `current_upgrade(FIGHTERBOMBER)` at that frame.
  `Unit::action_unqueue` (from `CommandPackage::process_unqueue`,
  `Build::train`, `ScenarioEditor::unqueue_unit`,
  `ScenarioFuncSet::add_unit`): returns at `num_queued == 0`, else
  `num_queued -= 1` and the inline `-1`. `Build::train`'s call is on a
  freshly trained Aircraft Carrier (`is(0x15f, 1)`), whose `num_queued` is 0,
  so it changes nothing.
- The `Wall` copies apply the rule to the wall's own type
  (`ptype->type`) while the wall is not active (`flags & 4` clear): a
  wall foundation counts in `num_queued` of its building type. A wall's type
  is a building type (`>= 0x19e`), so step 3 fails and **none of the six
  moves**. Not checked: that no wall type's vtable `+0xc` overrides
  `TypeData::is_unit_type`; the inline devirtualization takes the range
  test whenever the slot is the base one.

The Fighter-Bomber line (`FIGHTERBOMBER 0x134`) is built at the Airport,
not one of the four buildings; with `domain 2` it lands in `air_queued`.
That `where` was not read from the data.

### Reset

`Leader::init (006e3930)` zeroes all six (`6e4c3e`..`6e4c5c`, `eax` = 0),
at game setup, scenario load and CtW setup (callers listed under
`scholar_militia`). Nothing else zeroes them; in particular no frame-level
recount exists, so any floor-skipped decrement or `d < -1` overshoot
persists to the end of the game.

### Readers

- `LeaderData::get_support_count (006da110)`: for an armed unit type
  (`attack != 0`) whose progression flag `unittypes[t] +0x2f4 & 1` is set,
  returns `<building>_queued + <building>_units` by the same `where` /
  `domain` chain (Barracks, Stable, Siege Factory, Dock, air), else
  `num_queued[t] + high_buildings[...]` and its extras. `combat_queued` has
  no arm here. Its caller is `TypeData::get_cost (00664090)`, the unit
  price ramp.
- `LeaderData::log_data (006e5110)` prints all six (lines
  `2134`..`2159`).
- `ConquestLeader::*`, `ConquestSetupWin::*` and `ConquestGame::build_units`
  read `0xa10..0xa24` off their own records (CtW), not a leader.

### Crate

The crate already keeps the type half and a per-building half:

- `Muster::queued_by_type` (`num_queued`) and `Muster::queued_by_group`
  (`crates/sim/src/units/unit.rs`), where a type's `group` is its
  `where_` when `attack != 0` and `None` otherwise
  (`crates/rondata/src/load.rs`, `let group = if c.attack != 0 { c.where_ }`).
- `+1`: `Sim::queue_up_with` via `Sim::track_tree_queued`
  (`crates/sim/src/cities/queue.rs`); `Sim::carrier_action_spell`
  (`crates/sim/src/air/carrier_queue.rs`); the re-target
  `Sim::retarget_queued_to` (`cities/queue.rs`, which models the chain-arm
  register quirk above).
- `-1` floored: `Sim::untrack_queued` (`cities/queue.rs`), called from the
  completion and cancel paths, `Sim::untrack_item`/`clean_queue`
  (`crates/sim/src/cities/city.rs`), `Sim::carrier_queue_tick` and
  `Sim::unit_action_unqueue` (`air/carrier_queue.rs`).
- SEAM: `Sim::upgrade_units_to` (`cities/queue.rs`) names the carrier arm of
  `gain_tech` (`6dd9ec`, the `(old, -n)`, `(new, +n)` pair) as not built.
- Not found in the crate: `Unit::close`'s `track_queued(FB, -num_queued)`
  for a carrier lost with planes queued. Grepping `num_queued` and
  `untrack_queued` finds no close path; no capture has a carrier
  (`carrier_queue.rs` module doc), so no diff has asked for it.

What a builder adds: four of the six are one `queued_by_group` entry each,
provided the group index is the `where_` building (Barracks `0x1ab`,
Stable `0x1ac`, Siege Factory `0x1ae`, Dock `0x1b0`), and the crate's
per-group floor is the original's per-counter floor for those four.
`combat_queued` and `air_queued` are not a single group: `combat` is
Barracks + Stable with **its own** floor, `air` is every armed `domain 2`
type outside those four buildings (possibly several `where_`s) with its own
floor. Either add two counters moved at the `track_tree_queued` /
`untrack_queued` sites with the original's per-counter skip, or derive all
six in the leaders checksum (`Sim::leader_long` in
`crates/sim/src/checksum/leaders.rs`, beside `leader_trained`, which does
the same for the `*_units` leaves from `by_type`) from `queued_by_type`
with the same type filters. **Deriving from `queued_by_type` is not
exact in a plain game.** On `gain_tech`'s graft-chain arm the `-1` lands on
the new type: if `num_queued[new]` is 0 it is skipped there, while the
group counter beside it (which already holds the entry) does fall; the `+1`
then raises both. Net: `num_queued[old]` stays one high for good,
`num_queued[new]` is right, and the six leaves are right. A sum over
`queued_by_type` would then read one high. `queued_by_group` follows the
original (the crate's `untrack_queued` floors the type and the group
separately), so derive the four single-building leaves from it, and keep
`combat_queued` and `air_queued` as counters moved at the same sites with
their own per-counter floor. The `d < -1` overshoot needs the missing
carrier-loss path.

## barracks_queued (+0xa10)

1. **Writers**: the shared set above. Stores: `track_queued` `6e0fa7`;
   `Build::queue_up` `621a2f`; `Build::unqueue` `6209dc`;
   `Build::clean_queue` `620d92`; `Group::action_spell` `6fe80c`;
   `Unit::process` `611362` (absolute `0xe3ada0`); `Unit::action_unqueue`
   `5e1ff3`; `Wall::*` (`6433cc`, `642f22`, `63e668`, `63f93a`, dead);
   `Leader::init` `6e4c3e`.
2. **Predicate**: armed unit type (`attack != 0`) with `where == 0x1ab`
   (Barracks). `+1` per queued entry, `-1` per entry finished, cancelled or
   cleaned, skipped at 0. Always moved together with `combat_queued`.
3. **Reset**: `Leader::init` only.
4. **Readers**: `LeaderData::get_support_count` (Barracks arm, with
   `barracks_units`) feeding `TypeData::get_cost`; `LeaderData::log_data`.
5. **Crate**: `Muster::queued_by_group[Barracks]`, moved by
   `Sim::track_tree_queued` / `Sim::untrack_queued`
   (`crates/sim/src/cities/queue.rs`); expose to `Sim::leader_long`.
6. **Plain game**: yes (player and computer queues, completion, cancel,
   building loss, defeat).
7. **Confidence**: high; listing read for the out-of-line rule and every
   inline copy's store run.

## stable_queued (+0xa14)

1. **Writers**: `track_queued` `6e0fc6`; `Build::queue_up` `621a44`;
   `Build::unqueue` `6209f6`; `Build::clean_queue` `620dac`;
   `Group::action_spell` `6fe821`; `Unit::process` `61137c`
   (`0xe3ada4`); `Unit::action_unqueue` `5e200d`; `Wall::*` (dead);
   `Leader::init` `6e4c44`.
2. **Predicate**: armed, `where == 0x1ac` (Stable). Same step and floor;
   always moved with `combat_queued`.
3. **Reset**: `Leader::init` only.
4. **Readers**: `get_support_count` (Stable arm, with `stable_units`);
   `log_data`.
5. **Crate**: `queued_by_group[Stable]`, same sites.
6. **Plain game**: yes.
7. **Confidence**: high.

## factory_queued (+0xa18)

1. **Writers**: `track_queued` `6e0ffb`; `Build::queue_up` `621a59`;
   `Build::unqueue` `620a21`; `Build::clean_queue` `620dd7`;
   `Group::action_spell` `6fe836`; `Unit::process` `6113a7`
   (`0xe3ada8`); `Unit::action_unqueue` `5e2038`; `Wall::*` (dead);
   `Leader::init` `6e4c4a`.
2. **Predicate**: armed, `where == 0x1ae` (`SIEGEFACTORY`; `FACTORY 0x1af`
   does not match). Not part of `combat_queued`.
3. **Reset**: `Leader::init` only.
4. **Readers**: `get_support_count` (with `factory_units`); `log_data`.
5. **Crate**: `queued_by_group[Siege Factory]`, same sites.
6. **Plain game**: yes.
7. **Confidence**: high on the predicate; the building a given unit's
   `where` names is data and was not read.

## combat_queued (+0xa1c)

1. **Writers**: `track_queued` `6e0fdc` (one store reached from both the
   Barracks and Stable arms); `Build::queue_up` `621a35` and `621a4a`;
   `Build::unqueue` `620a07`; `Build::clean_queue` `620dbd`;
   `Group::action_spell` `6fe812`, `6fe827`; `Unit::process` `61138d`
   (`0xe3adac`); `Unit::action_unqueue` `5e201e`; `Wall::*` (dead);
   `Leader::init` `6e4c50`.
2. **Predicate**: armed and `where` is `0x1ab` or `0x1ac`. Step `+1`/`-1`;
   the skip at 0 tests `combat_queued` itself, independently of the
   Barracks and Stable counters.
3. **Reset**: `Leader::init` only.
4. **Readers**: `LeaderData::log_data` only. `get_support_count` has no
   combat arm; no other leader read in `.text` (every `0xa1c(%r)` read was
   mapped; the rest are CtW records, spell tables and graphics).
5. **Crate**: no single group holds it. Either a counter moved at the
   `track_tree_queued` / `untrack_queued` sites with its own floor, or
   `queued_by_group[Barracks] + queued_by_group[Stable]` (equal so long as
   both start at 0 and only paired `+1`/`-1` reach them, which holds without
   the `d < -1` carrier-loss path).
6. **Plain game**: yes.
7. **Confidence**: high.

## dock_queued (+0xa20)

1. **Writers**: `track_queued` `6e101a`; `Build::queue_up` `621a68`;
   `Build::unqueue` `620a3b`; `Build::clean_queue` `620df1`;
   `Group::action_spell` `6fe845`; `Unit::process` `6113c1`
   (`0xe3adb0`); `Unit::action_unqueue` `5e2052`; `Wall::*` (dead);
   `Leader::init` `6e4c56`.
2. **Predicate**: armed, `where == 0x1b0` (Dock). Ships trained at a Dock;
   not part of `combat_queued`.
3. **Reset**: `Leader::init` only.
4. **Readers**: `get_support_count` (with `dock_units`); `log_data`.
5. **Crate**: `queued_by_group[Dock]`, same sites.
6. **Plain game**: yes, on a map with water.
7. **Confidence**: high.

## air_queued (+0xa24)

1. **Writers**: `track_queued` `6e103b`; `Build::queue_up` `621a79`;
   `Build::unqueue` `620a57`; `Build::clean_queue` `620e0d`;
   `Group::action_spell` `6fe856`; `Unit::process` `6113dd`
   (`0xe3adb4`); `Unit::action_unqueue` `5e206e`; `Wall::*` (dead);
   `Leader::init` `6e4c5c`. Also through `track_queued` from
   `Unit::close` (`-num_queued`) and `gain_tech`'s carrier pair.
2. **Predicate**: armed, `where` not one of `0x1ab`/`0x1ac`/`0x1ae`/`0x1b0`,
   and `domain == 2`. Airport queues and the Aircraft Carrier's own plane
   queue (always `current_upgrade(FIGHTERBOMBER)`). The only leaf of the
   six with a `d < -1` writer: a carrier removed with `n` planes queued
   subtracts `n` at once, guarded only by `air_queued != 0`.
3. **Reset**: `Leader::init` only.
4. **Readers**: `get_support_count` (air arm, with `air_units`);
   `log_data`.
5. **Crate**: the airport queue is `queued_by_group[<airport where_>]` via
   the same sites, and the carrier's queue moves the same group through
   `Sim::carrier_action_spell`, `Sim::carrier_queue_tick`,
   `Sim::unit_action_unqueue` (`crates/sim/src/air/carrier_queue.rs`).
   Missing: the carrier-loss `-num_queued` (`Unit::close (0060ee50)`
   `60efed`), and the `gain_tech` carrier pair (a named SEAM in
   `Sim::upgrade_units_to`). If more than one `where_` trains an armed
   air type, `air_queued` is the sum of their groups with one floor, so a
   separate counter is the safer port.
6. **Plain game**: yes (Airport from the Industrial age; carriers later).
   No capture has a carrier.
7. **Confidence**: high on the rule; medium on which buildings train
   `domain 2` types (data, not read).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `scholar_militia` +0x9f0 | `Leader::track_unit_type (006e0dd0)` (callers `Unit::set_type (00612fa0)`, `Unit::close (0060ee50)`, `Objects::init_unit (0065e0c0)`, `SpellType::cast_create_decoy (00674370)`); `Leader::init (006e3930)` | `+d` when the tracked type `is(MILITIA)` and the unit's `+0x54` former type `is(SCHOLARS)`: +1 on a Scholar's To Arms, -1 on its return to civilian or its death; no floor | `Leader::init` | `Sim::track_unit_type` (`units/store.rs`, needs the unit) or the existing derivation in `Sim::worker_support` (`economy/pricing.rs`) | yes | high (writers, arithmetic); medium (who can hold a Scholar in `+0x54`) |
| `barracks_queued` +0xa10 | `Leader::track_queued (006e0f30)`, inline in `Build::queue_up (00620f40)`, `Build::unqueue (006207c0)`, `Build::clean_queue (00620b60)`, `Group::action_spell (006fe1a0)`, `Unit::process (00610bc0)`, `Unit::action_unqueue (005e1f20)`, `Wall::increment_stats (00643270)`/`decrement_stats (00642da0)`/`activate (0063e4b0)`/`close (0063f6e0)` (dead); `Leader::init (006e3930)` | armed, `where 0x1ab`: +1 queued, -1 finished/cancelled/cleaned, skipped at 0 | `Leader::init` | `queued_by_group` via `track_tree_queued`/`untrack_queued` (`cities/queue.rs`) | yes | high |
| `stable_queued` +0xa14 | same set | armed, `where 0x1ac`; same | `Leader::init` | `queued_by_group[Stable]` | yes | high |
| `factory_queued` +0xa18 | same set | armed, `where 0x1ae` (Siege Factory); same | `Leader::init` | `queued_by_group[Siege Factory]` | yes | high |
| `combat_queued` +0xa1c | same set | armed, `where 0x1ab` or `0x1ac`; same, own floor | `Leader::init` | none yet: new counter at the same sites, or Barracks + Stable groups | yes | high |
| `dock_queued` +0xa20 | same set | armed, `where 0x1b0`; same | `Leader::init` | `queued_by_group[Dock]` | yes (water maps) | high |
| `air_queued` +0xa24 | same set, plus `track_queued` from `Unit::close (0060ee50)` (`-num_queued`) and `Leader::gain_tech (006dcb60)` | armed, none of the four `where`s, `domain 2`; same, and `-n` on a carrier lost with `n` queued | `Leader::init` | `queued_by_group[airport]` + `air/carrier_queue.rs`; carrier loss and the `gain_tech` carrier pair missing | yes | high (rule); medium (which buildings train air) |

### What this reading did not read

- The `is_list` data for the Militia and Scholar lines (which types
  `is(MILITIA)` / `is(SCHOLARS)` accept), and which building each unit's
  `where` names.
- `SpellType::cast_immortal_hinf (00670600)` / `cast_immortal_arch
  (00670740)` target predicates (could they strand `scholar_militia`).
- `Build::activate (00623e20)`'s cross-queue cancel predicate, and whether
  the computer player casts To Arms with Scholars.
- `Leader::Leader (006d2bf0)`'s clear, and any save/load path (none found by
  the `0x6ee4`-size grep beyond the constructor).
- No diff was run; everything here rests on the decompile and listing.
