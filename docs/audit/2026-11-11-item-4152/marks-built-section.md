## 13. The slot marks and build counters

### 13.1 Leaves

- `fort_mark` +0x428, `int`: high-water mark over the leader's fort records.
- `oil_well_mark` +0x430, `int`: the same, over oil-well and platform records.
- `supply_mark` +0x434, `int`: the same, over supply-unit records.
- `wonders_built` +0x854, `int`: wonder activations since the last reset.
- `wonders_held` +0x858, `int`: the most wonder records ever live at once.
- `senates_built` +0x85c, `int`: Senate constructions.
- `forts_built` +0x8f0, `int`: fort activations.

The leaders array is `Leader[10]` (`types.txt`, `struct Leaders`, size
`0x4554c`), so every per-slot rule below runs over ten slots, not eight.
`Leaders::clear (006ed5d0)` has two callers only, `Game::close` and
`Game::run_oos_recovery`; it zeroes all seven leaves. The
`ScenarioRead::load_leaders_chunk (009acde0)` stores at +0x428/+0x430/+0x434
are off the `Game`, not a leader.

### 13.2 The rule

#### The three marks (shared rule)

Each mark indexes a per-leader registry of small records with an in-use bit.
The registry's index for a building is stored at `Build +0x78`; for a unit at
`Unit +0x84`.

- **Take:** reuse the first free record below the mark; else the allocated
  record at the mark; else append a new one. Then
  `mark = max(mark, idx + 1)`. A take raises the mark by 0 or 1, never more.
- **Give back:** clear the record's in-use bit, then walk the mark down past
  every free record at the top. A give-back can lower the mark by several.
- **Reset:** `Leader::init (006e3930)` zeroes all three after its
  `tribe < 0` early return, so only for seated slots; `Leaders::clear`.

Per mark:

- **`fort_mark`.** Take in `Forts::init_fort (0073ed70)`, called from
  `Build::activate (00623e20)` on every activation of an `is(FORTX 0x1bb)`
  type (`BuildTypeData::is_fort`, type slot `+0xfc`), `captured` not tested.
  Give back in `Forts::close_fort (0073ec90)`, from `Build::close (00628980)`
  when `is_fort` and `+0x78 ≥ 0`. Readers that change a decision:
  `find_fort`, `compute_reg_territory`, `Army::find_target`,
  `Object::do_launch`, `Leader::produce_unit`, `blocked_location`; also
  `log_data`.
- **`oil_well_mark`.** Take in `init_oil_well (0073f6f0)`, from
  `Build::activate` on every activation of `is(OILWELL 0x1a5)` or
  `is(OILPLATFORM 0x1a6)`, `captured` not tested. Give back in
  `close_oil_well (0073f670)`, from `Build::close` only when
  `build_flags & 0x40` (`is_gather_type`, type slot `+0x90`) **and** the type
  is a well or platform **and** `+0x78 ≥ 0`. The give-back writes the record:
  `+6 &= ~1`, `+7 = 0xff`, `+2 = 0xffff`, `+4 = 0`. Reader:
  `Leader::calc_gather (006ceee0)` (loop bound; the sum is order-free) and
  `log_data`.
- **`supply_mark`.** Take in `init_supply (0073ad40)`, from `Unit::init` when
  the unit `is_supply` (`unit_flags2 & 0x40`), and from `Unit::set_type` into
  a supply type when `+0x84 < 0`. Give back in `close_supply (0073acc0)`, from
  `Unit::close` (supply, `+0x84 ≥ 0`) and from `Unit::set_type` out of a
  supply type (`+0x84 ≥ 0`). Extra reset: `Setup::build_game (005ac190)`
  zeroes it for all ten slots, unconditionally, before any unit exists.
  Reader: `find_supply (0073aba0)`, through `UnitData::in_supply`,
  decision-changing; `log_data`.

#### `wonders_built`

- **Writer:** `Build::activate`, `wonders_built[owner] += 1` on **every**
  activation of a building whose type index is in `0x20e..=0x21e` (Pyramids
  to Space Program, 17 types; the vtable fast path and
  `TypeData::is_wonder_type (00470780)` name the same range). No test of
  `captured`, `announce` or `counted`, and every path through `activate`,
  the city branch included, reaches it. Order: `remove_unbuilt_wonder`, the
  increment, then `Wonders::init_wonder (0073c860)`.
- **Capture credits the captor.** `Cities::capture_city (00733380)` converts
  a member of the fallen city only when its type lacks `build_flags &
  0x2000`, and activates an active member as `activate(1, 1, 0)`. Fifteen
  wonders carry `BUILD_FLAGS` `a` alone and are city members, so a finished
  one in a captured city steps the captor by 1. The Forbidden City (`0x213`)
  carries `0x2000` but is itself a city object: `capture_city`'s head
  activates it `(1, 1, 0)`, and the captor is credited too. The Red Fort
  (`0x216`) carries `0x2000` and `e` and is not counted on capture. An
  unfinished wonder site is closed or disbanded, not activated, so it does not
  count. `Leader::defeat_by (006d1c80)` hands over military trainers only, so
  it never moves a wonder.
- **Never decremented.** Reset to 0 by `Leader::reset_score (006e37f0)`,
  called from every `Leader::init` and from `Setup::build_game`'s loop over
  seated slots, after the per-slot `Leader::init` and before
  `build_empire`; and by `Leaders::clear`.
- **Reader that changes a decision:** `BuildData::construct_time
  (0062d5c0)`, tribe bonus `0x14`'s first-wonder arm: free only when
  `get_wonders() == 0 && wonders_built == 0` and the type is not `0x21d` or
  `0x21e`. No other reader.

#### `wonders_held`

At `init_wonder`, after the new wonder's record is taken, count the leader's
in-use wonder records in `[0, wonder_mark)` (the new one included) and store
`wonders_held = max(wonders_held, count)`. Never lowered; `close_wonder` does
not touch it. Same predicate as `wonders_built`, since `init_wonder`'s one
caller follows that increment, so capture raises it too. Reset:
`reset_score`, `Leaders::clear`. Readers: `AchieveChartPane::set_chart_data
(007f2c40)` (UI) and the checksum; `log_data` does not print it.

#### `senates_built`

`Build::activate`'s non-city arm, `senates_built[owner] += 1` when the type
is in the SENATE `0x1b6` lineage, the building's city index `+0x72 ≥ 0`, and
`captured == 0`. `announce` and `counted` are ignored. Reset:
`reset_score`, `Leaders::clear`. Reader: `log_data` and the checksum.

#### `forts_built`

`Fort::init (0073eac0)`, `forts_built[who] += 1`, unconditionally. Its one
caller is `init_fort`, whose one caller is `Build::activate`, so the rule is
+1 per activation of an `is(FORTX)` type, Fort line and Red Fort (which is
also a wonder and steps `wonders_built` too). An upgrade in place does not
count. Never decremented. `Fort::init` also steps another family's
per-region short at `+0x12de + region·2` when `region < 0x40`. Reset:
`reset_score`, `Leaders::clear`. Reader: `set_chart_data` (UI), checksum.

### 13.3 In the crate

- **Marks.** `fort_mark` and `oil_well_mark` have no registry.
  `units::slots::SlotList` (`crates/sim/src/units/slots.rs:24`) is the exact
  shape: one per leader per mark, taken in `Sim::activate` beside the
  `is_fort` test (`crates/sim/src/cities/city.rs:2533`) and given back in
  `Sim::close_building`'s fort arm (`~2905`). Wells are found by kind in
  `economy/holdings.rs` today. `supply_mark` is already modelled:
  `SupplyList` (`territory/supply.rs:171`) pops free tails, so `len()`
  (`:207`) **is** the mark; opened `units/store.rs:145`, closed
  `territory/upkeep.rs:62`; the `set_type` pair is stubbed
  (`units/set_type_adapter.rs:207–211`).
- **`wonders_built`: no counter.** The site is `Sim::activate`
  (`city.rs:2420`) beside `self.is_wonder_building(ty)` at line 2490,
  **before** the `building_is_city` split (~2506), for `captured` true and
  false alike. Not the non-city tail: the Forbidden City runs the city
  branch. The capture member loop (`cities/capture_view.rs:156–170`) already
  gates on `not_civilian` (`0x2000`) and activates with `captured = true`, so
  a counter there reproduces the capture rule with no further work.
- **Trap: two bitmasks with counter names.** `Cities::wonders_built`
  (`cities.rs:63`) and `Sim::wonders_held` (`economy/holdings.rs:572`) are
  type bitmasks, not these counters. Do not merge them.
- **Trap: the American proxy.** `american_free_wonder` (`city.rs:1827`, tail
  ~1855) reads `census.wonder_mark == 0` and no standing wonder in place of
  `wonders_built == 0`. They part when a leader has ever activated a wonder
  (built, or captured then lost) and now holds none: the census mark trims
  to 0 (`Sim::note_wonders`, `ai/ai_census.rs:1276`), the counter does not,
  so the crate grants the free arm the original refuses. Feed it from the
  counter.
- **`wonders_held`: no counter.** `Sim::note_wonders` batches every close
  before every open, which can under-count against the original's event
  order; count at `Sim::activate`'s wonder test instead.
- **`senates_built`.** `senate_arm_steps` (`cities/activate.rs:251`, call
  `:261`) reaches `SimArms::senates_built_inc` (`:394`), an empty SEAM.
- **`forts_built`.** None; increment with the `fort_mark` take.
- **Checksum.** `checksum/leaders.rs` lists all seven leaves (`:105–108`,
  `:175–177`, `:189`); `leader_long` (`:949`) has an arm for none. Each sits
  in `LEADERS_HELD_OUT` at count 2 (`crates/rondata/src/diff/
  checksum_lines.rs:1787, 1788, 1817, 1835, 1837, 1850, 1851`); a port of a
  leaf adds its arm and drops its entry. The verdicts give no figure beyond
  that.

### 13.4 Reached by a plain game

A skirmish reaches every writer: fort and oil-well takes and give-backs by
construction and destruction (oil from the Industrial age); supply takes at
supply-unit births and deaths; `wonders_built` and `wonders_held` by
construction (`Wall::do_construct`, `(0, 1, 1)`) and by capturing a city that
holds a finished wonder or is a Forbidden City; `senates_built` and
`forts_built` by construction. `Leaders::clear` runs at game close and in
multiplayer out-of-sync recovery. `LeaderData::walk_data (006d6750)`
overwrites all seven on a save load, outside a plain game. Whether the
`set_type` supply pair or a `(0, 1, 0)` Senate transfer is reached is open
(13.6).

### 13.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/marks-built-*.md`); no diff backs it yet.

Verdicts: 63 confirmed, 7 corrected, 5 open. The corrections that changed a
rule:

- the array is ten slots, and `build_game` zeroes `supply_mark` in all ten;
- `Leaders::clear` has two callers, not four;
- `oil_well_mark`'s give-back also needs `build_flags & 0x40`, and writes
  four fields of the record;
- the `wonders_built` crate site is beside line 2490, outside the city
  split, not the non-city tail;
- `defeat_by` hands over no wonder;
- a fort taken with a city is **not** recounted for the captor: forts carry
  `0x2000`, which the member loop exempts, and no capture route to a fort
  was found.

Confidence is high throughout. The `wonders_built` increment, its missing
gate and its single return are listing-read; the capture route is
listing-read at `capture_city`'s gate, its member activation and its head
activation, and the wonders' flags are the shipped `buildingrules.xml`'s,
held to the program's loaded words by an existing guard (`docs/DATALAYER.md`).
`senates_built`'s `captured` register was traced through the listing. Every
leaf's absolute-address writer net was swept, and the register-relative
hits outside a leader are named in the verdicts.

### 13.6 Not established

- **A capture route to a Red Fort**, or to any fort, for `wonders_built` and
  `forts_built`. None was found; a sweep of every caller of the `activate`
  slot (`0x1a8`), its predicate against `is_fort`, would settle it.
- **`is_wonder_building`'s equivalence to `0x20e..=0x21e`.** The crate asks
  the tech tree's `wonder` kind; a test that the tree files exactly those 17
  types settles it.
- **Whether a plain game retypes a unit across `unit_flags2 & 0x40`**, which
  is what reaches `supply_mark`'s `set_type` pair. A capture with a supply
  unit retyped, or a reading of every `set_type` caller, settles it.
- **Whether a Senate travels a `(0, 1, 0)` transfer** (a cityless foreign
  Senate adopted by `City::find_buildings`). The predicate is settled; the
  route is not.
