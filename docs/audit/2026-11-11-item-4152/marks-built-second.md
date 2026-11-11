# marks-built — blind second reading (item 4152, wave 2)

- **Slug:** `marks-built`
- **Leaves** (`LeaderData`, all `int`): `fort_mark` (+0x428), `oil_well_mark`
  (+0x430), `supply_mark` (+0x434), `wonders_built` (+0x854),
  `wonders_held` (+0x858), `senates_built` (+0x85c), `forts_built` (+0x8f0).
- **Reader:** Claude Opus 5.5 (`claude-opus-5-5`), blind: the first reading
  of this family was not opened.
- **Sources:** the decompile export (`~/ghidra-projects/decomp`), the PE
  listing (`tmp/r4152/aa/exe.s`, llvm-objdump of `.text`), `crates/sim`.

## Common ground (applies to every leaf below)

- **Base.** `leaders.list` sits at `0xe3a390`, stride `0x6eec`, and a
  `Leader`'s `field_0xNNN` is `LeaderData +0xNNN`: `LeaderData` is at
  `Leader +0` and the 8 extra bytes are at the tail. Proven by
  `Fort::init (0073eac0)`, decompiled `leaders.list[w].field_0x8f0 += 1`,
  listed `incl 0xe3ac80(%eax)` with `0xe3ac80 − 0xe3a390 = 0x8f0`.
- **Whole-record writers** (every leaf of the family, not repeated per leaf):
  - `LeaderData::walk_data (006d6750)`: for a slot with `leader_flags & 1`
    it walks `[+0x8, +0x692a)` as one byte range. On load (`input != 0`)
    that range is overwritten from the stream, so a save/restore writes all
    seven leaves; on output it is the sync checksum's leaders line
    (`crates/sim/src/checksum/leaders.rs` header names
    `CheckSums::check_leaders`). Called per slot by
    `Leaders::walk_data (006e38e0)`.
  - `Leaders::clear (006ed5d0)`: `memset(record, 0, 0x6929)` for every
    slot — covers all seven. Called from `Game::close (00594b80)` and
    `Game::run_oos_recovery (005869e0)`.
  - `Leader::close (006b7f80)`: the same `memset(this, 0, 0x6929)`; no
    direct `call` to it in the listing (not chased further).
  - `ScenarioRead::load_leaders_chunk (009acde0)` stores at
    `0x428/0x430/0x434(%edx)`; listed but the base register's type was not
    read — scenario path only (see each leaf's "plain game?").
- **Slot-list shape** shared by the three `*_mark` leaves (same as
  `wonder_mark` and the crate's `units::slots::SlotList`): each leader owns a
  `PtrArray` of small records with an in-use bit; the mark is one past the
  highest in-use record. *Take*: first record below the mark whose bit is
  clear; else, if the array already has a record at the mark, the mark
  itself (mark += 1); else append a new record (slot = old length). Then
  `mark = max(mark, slot + 1)`. *Give back*: clear the bit, then while
  `mark > 0` and record `mark − 1` is free, `mark −= 1`. Units: a count of
  records (int).
- **Non-leader false positives** at the same displacements, by owner
  function, ruled out by base type: `GraphicWinXML::*`, `Constants::init`
  (`0x56c5xx`, a `Constants*`), `PlayerProfile::*`, `ConquestLeader::*`,
  `ConquestGame::*`, `TileSet::init`, `RiverOut::render`,
  `EventGroup::EventGroup`, `String::operator*` (`%esp`), `Game::loop`,
  `Game::calc_graphic_resolution`, `ObjectsOut::pop_all_zoomed_in`,
  `GraphicPieces::*`, `CP2PAuthPlayer::*`, `GameSpyGroup::GameSpyGroup`.
  Readers through the global at `0xc061f0` (`Build::plunder`,
  `Object::disband`, `Leader::gain_tech`, `Object::do_damage`) are a
  non-leader object, not `LeaderData`.

## fort_mark (+0x428)

1. **Writers.**
   - Export: no `->fort_mark` writer (only `LeaderData::log_data (006e5110)`
     reads it). `field_0x428` writers on a leader: `Forts::init_fort
     (0073ed70)` (two stores), `Forts::close_fort (0073ec90)`,
     `Leader::init (006e3930)`.
   - Listing: `0xe3a7b8(%reg)` stores at `73ed51` (close_fort), `73edee`,
     `73ee84` (init_fort); `0x428(%ebx)` at `6e4136` (Leader::init,
     `ebx` = `this`). Plus the whole-record writers above.
2. **Event.** `Forts::init_fort(who, o)` — the take rule above on the
   leader's forts list (`forts.lists[who]`, record 0x10 bytes, in-use bit
   `+6 & 1`). Its one caller is `Build::activate (00623e20)` at `6245a9`,
   reached for **every** activation (no `captured`/`param_1` test) of a
   building whose type answers vtable `+0xfc` true. The PDB names that
   slot `is_fort` (`LF_ONEMETHOD`, vftable offset 252);
   `BuildTypeData::is_fort (00472ba0)` is `is(FORTX 0x1bb, 0)`, and
   `ObjectTypeData::is (0065f7d0)` matches the type itself or any entry of
   its `is_list` — so the whole Fort line counts. `Forts::close_fort(who,
   slot)` — `Fort::close`, then the give-back trim — is called from
   `Build::close (00628980)` at `628ce7` when the type is a fort and the
   building's `+0x78` slot index is ≥ 0. Step: +1 or more (a trim can drop
   it by several at once).
3. **Reset.** `Leader::init (006e3930)` stores 0 unconditionally (after the
   `tribe < 0` early return; `6e412a`..`6e4166` block also zeroes hero,
   wonder, dock, oil-well, supply, special marks and the popwin/wonderwin
   words). Not in `Setup::build_game (005ac190)`'s per-slot clear. The
   whole-record writers.
4. **Readers.** `Forts::find_fort (0073ebe0)`, `Forts::init_fort` itself,
   `World::compute_reg_territory (006b0bb0)`, `Army::find_target
   (006f69b0)`, `Object::do_launch (0064f3b0)`,
   `BuildTypeData::blocked_location (006375b0)`, `Leader::produce_unit
   (006cb9e0)`, `LeaderData::log_data`. All use it as the loop bound over
   the leader's forts.
5. **Crate.** No forts slot list: `Sim::reg_forts`
   (`crates/sim/src/cities/place.rs:326`) counts live active fort buildings
   by region instead; `checksum/leaders.rs` `leader_long` has no
   `"fort_mark"` arm (it falls to `_ => None`). Builder: a
   `units::slots::SlotList` per leader, `take()` in `Sim::activate`
   (`crates/sim/src/cities/city.rs:2420`, beside the
   `build::is_fort` test in its non-city arm — but note the original takes
   for any activation, captured included) storing the slot on the building,
   `give_back()` in the building close beside `self.dock_close(b)`
   (`crates/sim/src/cities/city.rs:2829`), and a `"fort_mark"` arm reading
   `mark`. A count is not enough: holes below the top survive.
6. **Plain game?** Yes — any fort a player or the AI builds.
7. **Confidence.** High on the take/give-back rule (decompile and listing
   agree) and on the two event sites (the only callers in the listing).
   Not read: `Build::close`'s outer guards in full (only that the fort arm
   sits inside the `flags & 1` in-use block and tests `+0x78 ≥ 0`); the
   order of object creation versus `Leader::init`'s zeroing at game start
   (if a fort existed before `Leader::init`, its slot would survive while
   the mark was zeroed — scenario only).

## oil_well_mark (+0x430)

1. **Writers.**
   - Export: no `->oil_well_mark` writer. `field_0x430` on a leader:
     `OilWells::init_oil_well (0073f6f0)` (two stores),
     `OilWells::close_oil_well (0073f670)`, `Leader::init (006e3930)`.
   - Listing: `0xe3a7c0(%reg)` stores at `73f6d1` (close), `73f76e`,
     `73f7fa` (init); `0x430(%ebx)` at `6e4142` (Leader::init).
2. **Event.** `OilWells::init_oil_well(who, o)`, the take rule on
   `oil_wells.lists[who]` (record 0x10, in-use bit `+6 & 1`); it also runs
   `OilWell::init`, which records the tile's region. One caller:
   `Build::activate (00623e20)` at `625a9f`, for every activation of a
   building that `is(OILWELL 0x1a5, 0)` or `is(OILPLATFORM 0x1a6, 0)` —
   no `captured` test. `close_oil_well` (clear bit, region 0, then trim)
   from `Build::close (00628980)` at `628daf`, same `+0x78 ≥ 0` guard.
3. **Reset.** `Leader::init` (unconditional zero, as fort_mark). Not in
   `Setup::build_game`'s clear. Whole-record writers.
4. **Readers.** `Leader::calc_gather (006ceee0)` (loop bound over the oil
   wells, which pay outside the city loop), `LeaderData::log_data`.
5. **Crate.** No oil-well slot list; `economy/holdings.rs` finds wells by
   building kind (`Ident::OilWell | Ident::OilPlatform`, lines ~286, 426,
   546). No `"oil_well_mark"` arm in `checksum/leaders.rs`. Builder: a
   `SlotList` taken in `Sim::activate` (`cities/city.rs:2420`) for the two
   kinds, given back beside `dock_close` (`cities/city.rs:2829`).
6. **Plain game?** Yes, from the Industrial age (oil wells/platforms).
7. **Confidence.** High; same evidence as fort_mark. Not read: whether the
   `+0x78` shared by dock, fort, oil well and farm animals can ever carry a
   dock's index into an oil-well close (the type tests are disjoint, so
   presumably not).

## supply_mark (+0x434)

1. **Writers.**
   - Export: no `->supply_mark` writer. `field_0x434` on a leader:
     `Supplies::init_supply (0073ad40)` (two stores),
     `Supplies::close_supply (0073acc0)`, `Leader::init (006e3930)`; and the
     unnamed form `leaders.list[0..9]._1076_4_ = 0` (1076 = 0x434) in
     `Setup::build_game (005ac190)` lines 195–231.
   - Listing: `0xe3a7c4(%reg)` stores at `73ad21` (close), `73adbe`,
     `73ae51` (init); `0x434(%ebx)` at `6e4148` (Leader::init); and the
     absolute stores `movl $0x0, 0xe3a7c4` at `5ac373` and nine more at
     `+i·0x6eec` (through `0xe78e10`, slot 9) in `Setup::build_game`, an
     unrolled clear of `+0x408` (city_mark), `+0x420` (hero_mark),
     `+0x434` and `+0x43c` (special_mark) for all **ten** slots.
2. **Event.** `Supplies::init_supply(who, o)`, the take rule on
   `supplies.lists[who]` (record 0x10 per `malloc(0x14)`, in-use bit
   `+4 & 1`); the record gets its own index, `o`, `who`, and the bit. Callers:
   `Unit::init (00612100)` at `612943` when the unit is a supply unit, and
   `Unit::set_type (00612fa0)` at `613796` when the new type is supply and
   the unit holds no slot (`+0x84 < 0`). "Supply unit" is
   `UnitData::is_supply (0046ce80)`: `UnitTypeData.unit_flags2 & 0x40`;
   every vtable checked carries the base at `+0xcc`. `close_supply` (clear
   bit, `who = 0xff`, `o = −1`, then trim) from `Unit::close (0060ee50)` at
   `60f7cd` (supply unit with `+0x84 ≥ 0`) and `Unit::set_type` at `613750`
   (new type not supply, slot held). Slot stored at `UnitData +0x84`.
3. **Reset.** `Setup::build_game`'s ten-slot clear (before units are
   placed), then `Leader::init`'s unconditional zero for each seated slot.
   Whole-record writers.
4. **Readers.** `Supplies::find_supply (0073aba0)` (loop bound for the
   nearest-supply search attrition uses), `Supplies::init_supply`,
   `LeaderData::log_data`.
5. **Crate.** Present in substance: `territory::supply::SupplyList`
   (`crates/sim/src/territory/supply.rs:172`), `register` at unit birth
   (`crates/sim/src/units/store.rs:145`, gated on
   `unit.kind.supply_unit`) and `close` on death
   (`Sim::close_supply_in`, `crates/sim/src/territory/upkeep.rs:62`).
   Because `close` pops trailing free slots, `slots.len()` equals the
   original's mark, and `register`'s "first free slot, else push" equals the
   take rule. Missing: a `"supply_mark"` arm in `checksum/leaders.rs`
   (`self.territory.supply[w].list.len()`), and the `set_type` pair —
   `crates/sim/src/units/set_type_adapter.rs:207`–`211` stubs
   `supply_id`/`close_supply`/`open_supply`, so a unit retyped into or out
   of supply does not move the list.
6. **Plain game?** Yes — supply wagons.
7. **Confidence.** High on the rule and the five call sites. Not read: what
   `Unit::set_type` retypes a supply unit in a plain game (an upgrade of a
   wagon line keeps the flag, so both tests would no-op); the order of
   starting units against `Leader::init` (a supply unit present before
   `Leader::init` would keep its slot while the mark is zeroed, and the
   next take would reuse record 0).

