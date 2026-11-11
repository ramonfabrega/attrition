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

