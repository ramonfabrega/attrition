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
   (`crates/sim/src/territory/supply.rs:171`), `register` at unit birth
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

## Build::activate's callers (for the three `*_built` leaves)

`Build::activate (00623e20)` is vtable `+0x1a8` (`vtables.txt`, `Build`),
called only virtually. Its first argument is "captured" (`param_1`); the
call sites found in the export:

| caller | args | what |
|---|---|---|
| `Wall::do_construct (006434d0)` | `(0,1,1)` | construction finished |
| `Leader::produce_building (006e1400)` | `(0,param_2,0)` | instant build |
| `Setup::build_cities (005ab910)` | `(0,0,0)` | start cities |
| `Cities::capture_city (00733380)` | `(1,1,0)` | every active building of a taken city, cloned to the captor through vtable `+0x94` then activated |
| `Build::check_capture (006276a0)` | `(0,1,0)` | captured city's object, new owner |
| `City::find_buildings (007384c0)` | `(0,1)` | buildings a city adopts, new owner |
| `Leader::defeat_by (006d1c80)` | `(0)` | holdings handed to the victor |
| `ConquestGame::build_cities`, `ScenarioRead::load_city_chunk`/`load_build_chunk`, `ScenarioEditor::*`, `ScenarioFuncSet::create_building`/`switch_teams`, `Console::hurry_action`, `ConsoleWin::run_cmd`, `UnitBalance::next`, `SpellType::cast_bribe` | various | CtW, scenario, editor, cheat, tool, bribe |

## wonders_built (+0x854)

1. **Writers.**
   - Export: no `->wonders_built` store (`BuildData::construct_time
     (0062d5c0)` reads it). `field_0x854` on a leader: `Build::activate
     (00623e20)` line 1265 (`puVar2 = &…field_0x854; *puVar2 += 1`),
     `Leader::reset_score (006e37f0)`.
   - Listing: `incl 0xe3abe4(%eax)` at `625b4b` (Build::activate),
     `movl %eax, 0x854(%ecx)` at `6e3883` (reset_score, `ecx` = `this`).
     Whole-record writers.
2. **Event.** In `Build::activate`, after the oil-well arm and the
   `+0x1e8` counter, a building whose type is a wonder: when the object's
   vtable is `Build::vftable` and the type's is
   `BuildType::vftable_for_Type_` (`0xb42b94`), wonder iff the type id is in
   `0x20e..=0x21e` (PYRAMIDS..SPACEPROGRAM; listing `625adf`–`625af2`);
   otherwise the virtual `BuildData::is_wonder` / `TypeData::is_wonder_type`.
   Then `UnbuiltWonders::remove_unbuilt_wonder`, **`+= 1`**, then
   `Wonders::init_wonder`. **No test of `captured`** on the path from `LAB_00625a45` (`625a45`)
   to the increment — the region is top-level in the function, and the
   next `param_1` read is `625b6d`, after it, for the achievement event — so a
   wonder taken with a city (`capture_city`, `(1,1,0)`) or handed over by
   `defeat_by` counts again for its new owner. Never decremented.
3. **Reset.** `Leader::reset_score`, called by `Leader::init (006e3930)`
   line 140 and again by `Setup::build_game (005ac190)` line 731 for every
   slot with `leader_flags & 1` (after all `Leader::init`s and the graphics
   preload, before goods placement). Whole-record writers.
4. **Readers.** `BuildData::construct_time (0062d5c0)`: the
   `has_tribe_bonus(0x14)` first-wonder arm requires
   `get_wonders() == 0 && wonders_built == 0` (and the type not 0x21d/0x21e).
   No other `LeaderData` reader found by either net.
5. **Crate.** No such counter. Name collision to avoid:
   `Cities::wonders_built` (`crates/sim/src/cities.rs:63`) is
   `Game::wonders` (`Game +0x618`), a per-type bitmask, not this leaf.
   `Sim::american_free_wonder` (`crates/sim/src/cities/city.rs:1827`) stands
   in the census `wonder_mark` for `wonders_built == 0`, which differs once
   a leader has built a wonder and lost it (the mark trims, this never
   falls). Builder: `+= 1` in `Sim::activate`
   (`crates/sim/src/cities/city.rs:2420`) under the existing
   `self.is_wonder_building(ty)` test (~line 2490), for `captured` true and
   false alike, zeroed where `reset_score` is modelled; a `"wonders_built"`
   arm in `checksum/leaders.rs`.
6. **Plain game?** Yes.
7. **Confidence.** High (decompile and listing agree; one store). Not read:
   `UnbuiltWonders::remove_unbuilt_wonder`; whether `Leader::defeat_by`'s
   handover reaches a wonder in a plain game; whether scenario-loaded
   wonders activate before or after `build_game`'s second `reset_score`.

## wonders_held (+0x858)

1. **Writers.**
   - Export: no `->wonders_held`. `field_0x858` on a leader:
     `Wonders::init_wonder (0073c860)` lines 99–103, `Leader::reset_score`.
   - Listing: `0xe3abe8(%ebx)` load/store at `73ca11`/`73ca1f`
     (init_wonder); `0x858(%ecx)` at `6e3889` (reset_score). Whole-record
     writers. (Other `0x858(%reg)` stores belong to `Game::loop`,
     `ObjectsOut`, `GraphicPieces`, `Constants` — not leaders.)
2. **Event.** `Wonders::init_wonder(who, o)` (called only from
   `Build::activate` at `625b5b`, right after the `wonders_built`
   increment, so the same predicate and no `captured` test): after taking
   the wonder's slot (the slot-list rule on `wonder_mark`, `+0x424`) and
   stamping the record (frame, `o`, timer, in-use bit `+0xc & 1`), it
   counts the in-use records in `[0, wonder_mark)` — the wonders the leader
   **stands in right now**, this one included — and sets
   `wonders_held = max(wonders_held, count)`. A high-water mark of
   simultaneous holdings, in wonders. `Wonders::close_wonder (0073c7e0)`
   does not touch it.
3. **Reset.** `Leader::reset_score` (same two call points as
   wonders_built). Whole-record writers.
4. **Readers.** `AchieveChartPane::set_chart_data (007f2c40)` (post-game
   chart). `TerrainOut::render`'s `field_0x858` is on a different object.
5. **Crate.** No such counter. `Sim::wonders_held`
   (`crates/sim/src/economy/holdings.rs:572`) is a **bitmask of standing
   wonder types**, not this count. The crate models `init_wonder` in
   `Sim::note_wonders` (`crates/sim/src/ai/ai_census.rs:1276`), once per
   frame after the building pass, closes first then opens. Builder: after
   each insertion there, count the `Some` entries of `cs.wonder_slots[..
   wonder_mark]` and raise a per-leader high-water. Caveat: the batch
   order (all closes, then all opens) can under-count against the
   original's event order when one wonder falls and another activates for
   the same leader in one frame.
6. **Plain game?** Yes.
7. **Confidence.** High on the rule. Not read: `Wonder::get_timer`.

## senates_built (+0x85c)

1. **Writers.**
   - Export: `->senates_built` only read (`LeaderData::log_data`).
     `field_0x85c` on a leader: `Build::activate (00623e20)` line 227,
     `Leader::reset_score`.
   - Listing: `incl 0xe3abec(%eax)` at `6246ec`; `0x85c(%ecx)` at `6e388f`
     (reset_score). Whole-record writers. (`GraphicPieces::_GraphicPieces`
     `0x85c(%edi)` is not a leader.)
2. **Event.** `Build::activate`, the non-city arm (`flags & 0x20` clear):
   an `ObjectData::is` (vtable `+0xb8`) chain tests GRANARY 0x1a7,
   LUMBERMILL 0x1a8, SMELTER 0x1a9, REFINERY 0x1aa, TEMPLE 0x1b5, MARKET
   0x1b4, then **SENATE 0x1b6** (`is(t, 0)`: the type or its `is_list`).
   In the Senate arm, after the government-hero training head, **`+= 1`**
   iff the building has a city (`+0x72 ≥ 0`) **and** `captured == 0`
   (listing `6246cf`–`6246dc`: `cmpw $0,0x72(%esi); jl …; testl %edi,%edi;
   jne …`, `edi` = `param_1`). Callers with `param_1 = 0` that are
   transfers — `check_capture`, `find_buildings`, `defeat_by` — would still
   count.
3. **Reset.** `Leader::reset_score`. Whole-record writers.
4. **Readers.** `LeaderData::log_data`. (`Object::do_damage`'s
   `0x85c(%eax)` and the `TerrainOut::render` field are other objects.)
5. **Crate.** The event is there and the counter is a SEAM:
   `senate_arm_steps` (`crates/sim/src/cities/activate.rs:251`) calls
   `v.senates_built_inc()` under exactly `city_index ≥ 0 && captured == 0`;
   `SimArms::senates_built_inc` (`crates/sim/src/cities/activate.rs:394`)
   is an empty body, "SEAM: no reader here carries `senates_built`".
   Builder: give it a per-leader counter and a `"senates_built"` arm.
6. **Plain game?** Yes.
7. **Confidence.** High.

## forts_built (+0x8f0)

1. **Writers.**
   - Export: no `->forts_built`. `field_0x8f0` on a leader: `Fort::init
     (0073eac0)` (`puVar1 = &leaders.list[who].field_0x8f0; += 1`),
     `Leader::reset_score`.
   - Listing: `incl 0xe3ac80(%eax)` at `73eaea` (Fort::init);
     `0x8f0(%ecx)` at `6e38b9` (reset_score). Whole-record writers.
     (`GameSpyGroup`, `Constants::init`, `ConquestGame::close` hits are other
     objects.)
2. **Event.** `Fort::init (0073eac0)`, whose one caller is
   `Forts::init_fort (0073ed70)` (`73ee93`), whose one caller is
   `Build::activate` (`6245a9`): **`+= 1` per fort activation**, the
   `is_fort` predicate of fort_mark, **no `captured` test** — a fort taken
   (e.g. in a captured city's radius) counts again for the captor. The same
   `Fort::init` also counts the fort into the leader's per-region short
   table at `+0x12de` (region < 64), which `Fort::close` undoes; that table
   is not this leaf. Never decremented.
3. **Reset.** `Leader::reset_score`. Whole-record writers.
4. **Readers.** `AchieveChartPane::set_chart_data (007f2c40)`. Not printed
   by `LeaderData::log_data` under its name.
5. **Crate.** No counter. The crate's `reg_forts` (the `+0x12de` table) is
   recomputed from buildings (`crates/sim/src/cities/place.rs:326`).
   Builder: `+= 1` in `Sim::activate` (`crates/sim/src/cities/city.rs:2420`)
   where the fort is taken (with fort_mark's slot), for captured and fresh
   activations alike.
6. **Plain game?** Yes.
7. **Confidence.** High.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `fort_mark` +0x428 | Forts::init_fort (0073ed70), Forts::close_fort (0073ec90), Leader::init (006e3930); whole-record | slot-list mark: take on fort `Build::activate` (any `captured`), trim on `Build::close` | Leader::init; Leaders::clear; walk_data load | none; add `SlotList` in `Sim::activate` (cities/city.rs:2420) + give-back beside `dock_close` (city.rs:2829); arm in checksum/leaders.rs | yes | high |
| `oil_well_mark` +0x430 | OilWells::init_oil_well (0073f6f0), OilWells::close_oil_well (0073f670), Leader::init (006e3930); whole-record | slot-list mark on OILWELL/OILPLATFORM activate/close | Leader::init; Leaders::clear; walk_data load | none; same two sites as fort_mark | yes (Industrial+) | high |
| `supply_mark` +0x434 | Supplies::init_supply (0073ad40), Supplies::close_supply (0073acc0), Leader::init (006e3930), Setup::build_game (005ac190) (`_1076_4_`, 10 slots); whole-record | slot-list mark on supply-unit birth/death/retype (`unit_flags2 & 0x40`) | Setup::build_game; Leader::init; Leaders::clear; walk_data load | `SupplyList::len()` (territory/supply.rs:171) already equals it; add arm; `set_type` pair stubbed (units/set_type_adapter.rs:207) | yes | high |
| `wonders_built` +0x854 | Build::activate (00623e20), Leader::reset_score (006e37f0); whole-record | +1 per wonder activation, captured included | reset_score (Leader::init, Setup::build_game) | none (`Cities::wonders_built` is `Game::wonders`, a different field); add in `Sim::activate` under `is_wonder_building` | yes | high |
| `wonders_held` +0x858 | Wonders::init_wonder (0073c860), Leader::reset_score (006e37f0); whole-record | max(held, in-use wonders below `wonder_mark`) at each init_wonder | reset_score | none (`Sim::wonders_held` is a type bitmask); add in `Sim::note_wonders` (ai/ai_census.rs:1276) | yes | high |
| `senates_built` +0x85c | Build::activate (00623e20), Leader::reset_score (006e37f0); whole-record | +1 per Senate activation with a city and `captured == 0` | reset_score | SEAM `SimArms::senates_built_inc` (cities/activate.rs:394) | yes | high |
| `forts_built` +0x8f0 | Fort::init (0073eac0), Leader::reset_score (006e37f0); whole-record | +1 per fort activation, captured included | reset_score | none; add with the fort slot in `Sim::activate` | yes | high |
