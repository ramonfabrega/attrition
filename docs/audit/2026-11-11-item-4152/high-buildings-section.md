## 20. `high_buildings`, `territory_high` and `reg_terr`

### 20.1 Leaves

- `territory_high`: +0x9dc, `int`. Territory's high-water mark, in tiles.
- `reg_terr`: +0x145e, `ushort[64]`. Tiles held per land region, indexed by
  the original's region number 0..63 with no bias.
- `high_buildings`: +0x5660, `ushort[129]`. A building type's high-water
  count, indexed by `basic_type(T) − 0x19e` (the root of T's `from` chain).
  The executable folds the bias into the displacement, so every access is
  spelled at +0x5324 (absolute `0xe3f6b4`). The export's
  `high_buildings[x + 0x4f]` hits are `num_units` (+0x5762), not this leaf.

All three are zeroed by both bulk writers (`Leader::init (006e3930)`'s
tribe<0 memset and `Leaders::clear (006ed5d0)`). The tribe<0 arm runs for
every slot at game creation, because `Leaders::init (006ed850)` calls
`Leader::init(i, −1, i)`. `Game::run (00584590)` reaches it before
`Game::init`, which runs setup's territory recompute. `Leaders::clear`
covers all ten slots and runs from `Game::close (00594b80)` and
`Game::run_oos_recovery (005869e0)`.

### 20.2 The rule

**`reg_terr`.** Every writer is inside the border pass,
`World::compute_reg_territory (006b0bb0)`, called as `(r, reset)`:

1. If `reset`, set region r's cursor (`Region +0x2c`) to 0.
2. **Zero block.** If the cursor is 0, then for every slot 0..7 with
   `leader_flags & 1`, set `reg_terr[s][r] = 0`. The same block zeroes
   `reg_known_rares[s][r]` (+0x4d4) and the slot's live cities' `bordering`
   (+0x65). Slots without the live bit keep their last counts, and the
   checksum walks them.
3. **Early return** when the cursor ≥ the region's size, or when the
   frame's tile budget (`GameDaemon +0x24`) is above 0xff. The test is
   signed, so −1 means unlimited. It runs **after** the zero block, so a
   region the budget denies is re-zeroed every frame until its first tile.
4. **Count.** For each tile the pass assigns, if the winner is ≥ 0, then
   `reg_terr[winner][r] += 1`, wrapping as a u16. Two winners count for
   nobody: −1 (no claim) and −2. A tile is −2 when the winning city's
   `CityData +0x5f` (`race`, a `char`) is not the slot whose city list held
   that city.

`World::compute_all_territory (006b5700)`, setup's wholesale recount,
zeroes all 64 entries for every live slot 0..7. It then sets the budget to
−1 and calls `compute_reg_territory(r, 1)` for every region r with
`Region.size (+0x14) != 0`.

`Leader::init`'s ordinary path also zeroes all 64 entries.

The reader that decides is `BuildTypeData::blocked_location (006375b0)`.
City spacing relaxes by `CITY_SPACING − RELAX_CITY_SPACING` when
`size·9/10 <= reg_terr[who][r]`. The two `territory` folds below also read
it.

**`territory_high`.** It has two writers, both the same fold:

- `World::compute_all_territory (006b5700)`, at setup. Its callers are
  `Setup::build_cities` and `Setup::build_game`, which calls it twice.
- `GameDaemon::check_borders (00732060)`, every frame through
  `GameDaemon::process_all`. It folds only when both hold: some
  `compute_reg_territory` call this frame returned bit 2, and no sized
  region 0..63 still has its cursor below its size.

The predicate is slots 0..7 with `leader_flags & 1`. The step, signed:

```
territory      = Σ reg_terr[r]   over r in 0..63 with Region.size != 0
territory_high = max(territory_high, territory)
```

It is never lowered between the resets. There is no sim reader. The
readers are `AchieveChartPane::set_chart_data (007f2c40)`, a UI float, and
`LeaderData::log_data`.

**`high_buildings`.** The only writer is `Build::activate (00623e20)`,
called as `(p1, p2, p3)`. It runs after the building is counted: the base
`Wall::activate` runs `Wall::increment_stats` first.

- `n = Σ num_buildings[U]`, over T and every record on T's `to` (+0x2e0)
  chain. This is `num_buildings[T] + LeaderData::get_buildings
  (006e0680)(to(T))`, recursive while `to >= 0`.
- **Predicate:** `p1 == 0` and `n > high_buildings[root]`, as a signed
  compare. Then `high_buildings[root] = n as u16`.
- Inside the raise branch:
  - `T == 0x21a` sets `leader_flags |= 0x10000000`;
  - `p3 != 0` pays the nation's free units, subject to the semaphore gates
    `0x821 & 0x10` and `0x822 & 2` and the scenario byte `0xcb195b`.
- The reset is all 0x81 entries, by `Leader::init`'s ordinary-path loop
  (which zeroes `num_buildings` alongside) and both bulk writers.
  `Wall::decrement_stats` leaves the mark alone, so it is never lowered.

The callers and their `(p1, p2, p3)`, from all 16 listing sites:

| caller | p1, p2, p3 |
|---|---|
| `Setup::build_cities` | 0,0,0 |
| `Build::check_capture` | 0,1,0 |
| `Wall::do_construct` | 0,1,1 |
| `SpellType::cast_bribe` | 0,1,0 |
| `Leader::defeat_by` | 0,1,0 |
| `Leader::produce_building (006e1400)` | 0,p2,0 |
| `Cities::capture_city` (two sites) | 1,1,0 |
| `City::find_buildings` | 0,1,0 |
| `Console::hurry_action` | 0,1,1 |
| `ScenarioEditor::add_build`, `activate_selected` | 0,0,0 |
| `ScenarioRead::load_build_chunk` (two sites) | 0,0,0 |
| `ScenarioFuncSet::switch_teams` | 1,1,0 |
| `ScenarioFuncSet::create_building` | 0,1,1 |

`ConquestGame::build_cities (00796140)`, `ScenarioRead::load_city_chunk`,
`UnitBalance::next` and `ConsoleWin::run_cmd` also call it. So p1 = 1 only
for `capture_city` and `switch_teams`, and every other caller can raise the
mark.

The readers are `TypeData::get_cost (00664090)` and
`BuildData::construct_time (0062d5c0)`. Both run the Iroquois quick-Senate
test, `high_buildings[T − 0x19e] == 0`, indexed by T itself.

### 20.3 In the crate

- **`territory_high`: not held.** The fold belongs in
  `Sim::update_territory_holdings` (`crates/sim/src/territory/upkeep.rs:129`).
  That function is called from `start_border_pass`
  (`territory/border_pass.rs:56`, pre-play), `settle_borders` (`:170`) and
  the pass-complete tail of `advance_border_pass` (`:248`), and those are
  the two original sites. One `Sim` is one game, so zeroing at creation is
  the reset. A crate that reuses a `Sim` across games must zero the field at
  game start. The crate's `holdings.territory` counts cells over every land
  region (per its doc comment). The original sums `reg_terr` over sized
  regions 0..63, so the checksum agrees only if the two counts agree.
- **`reg_terr`: not stored.** `Sim::reg_terr` (`cities/place.rs:344`)
  recounts on demand, so it parts from the original mid-pass. The crate
  already reads the spacing rule at `cities/location.rs:305`.
  `zero_region_rares` (`border_pass.rs:125`) notes that the +0x145e write
  "has no field in this crate". A store goes in three places:
  - the cursor-0 arm of `advance_border_pass` (`:184`, arm `:205`);
  - its per-cell loop (`:222`, `who.player()`);
  - the wholesale recount in `wholesale_bordering` (`:136`),
    `settle_borders` (`:166`) and `start_border_pass` (`:48`).

  `Sim::reg_terr` should then read the store. **Trap:** `zero_region_rares`
  zeroes for every `ai` leader without a liveness test. A `reg_terr` zero
  placed there must be gated on `leader_flags & 1`, or a dead slot's row
  parts.
- **`high_buildings`: held**, as `Cities::building_high`
  (`crates/sim/src/cities.rs:51`). It is written by
  `Sim::claim_building_high` (`cities/nations.rs:219`) from `Sim::activate`
  (`cities/city.rs:2420`, call `:2578`). The guard at `nations.rs:234`,
  `building_high[w][root] >= n || captured`, is the original's predicate,
  with `captured` as p1. Only the checksum emitter is missing. It maps
  `building_high[who][rec]` to the original's `type − 0x19e`, zero-padded
  to 129, and the `"num_buildings"` arm is the model. The crate callers
  checked against the table:
  - `capture_view.rs:236` passes `(true, false)`, `city_views.rs:276`
    passes `(false, false)` and `city.rs:680` passes `(false, false)`. All
    three match.
  - `Wall::do_construct` routes through `cities/construct.rs:49`/`:108` as
    `activate(0, 1, 1)`. Not traced further.
  - **Unported:** `Build::check_capture`'s non-city swap branch.
    `SimCapture` answers `swap_team` with −1, and its `activate_new` is
    empty (`cities/capture.rs:427` ff., `:661`). The (0,1,0) raise there
    never happens. Only the `capture_city` arm is live.
  - **Unported:** a building arm of `cast_bribe`. `spells/bribe.rs` ports
    the unit arm only.
  - **Trap:** `ai/ai_place.rs:1035` passes `activate(o, false, true)` where
    `Leader::produce_building` pushes p3 = 0. The mark is unaffected,
    because p1 = 0 on both sides. But `counted` gates `free_units_for`
    (`nations.rs:238`), so the crate can pay free units on a frame-0
    building where the original pays none. Book it against the units
    score, not this leaf (§19 names the same flag).
- **Checksum.** The rows are at `crates/sim/src/checksum/leaders.rs:236`
  (`territory_high`), `:294` (`reg_terr`) and `:297` (`high_buildings`).
  Each leaf is `(unheld)` in `LEADERS_HELD_OUT` with count 2
  (`crates/rondata/src/diff/checksum_lines.rs:1794, 1820, 1840`), and each
  port removes its entry.

### 20.4 Reached by a plain game

All three leaves are reached.

- `reg_terr` and `territory_high` move at setup and on every border pass.
- `high_buildings` moves on starting buildings (`Setup::build_cities`),
  every finished construction (`Wall::do_construct`), frame-0 production,
  `City::find_buildings`, `Leader::defeat_by`, and buildings changing
  hands.
- Not yet checked: whether a plain game reaches `check_capture`'s non-city
  swap or a building bribe.
- Off the plain path: `Console::hurry_action` and `ConsoleWin::run_cmd`
  (console), the `ScenarioFuncSet` calls (triggers), the editor and
  scenario loaders, `UnitBalance::next`, and `ConquestGame::build_cities`
  (Conquer the World).

### 20.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/high-buildings-*.md`); no diff backs it
yet.

| leaf | confirmed | corrected | open |
|---|---|---|---|
| `territory_high` | 9 | 1 | 0 |
| `reg_terr` | 12 | 2 | 1 |
| `high_buildings` | 12 | 0 | 2 |

The writers, predicates, steps and resets are settled at listing
addresses. Confidence is high on all three leaves.

The corrections that changed a rule:

- **`territory_high` is reset per game.** The claim that nothing resets it
  across games in one process is wrong. The tribe<0 memset (`006e398c`, a
  `_memset` thunk per `rise_z.map`) and `Leaders::clear` (`006ed5e0`) both
  zero `[+0, +0x6929)`. Both readings' nets missed `Leaders::clear`,
  because its size is 0x6929, not the struct size.
- **The −2 arm tests `CityData::race` (+0x5f)**, not an owner byte. The
  crate's `Owner::Ambiguous` doc (`crates/sim/src/world.rs:172`) has the
  same naming gap.
- **`compute_all_territory` clears live slots only**, with eight 16-byte
  stores (64 ushorts).

No kept-dump value is cited as a witness.

### 20.6 Not established

- **What `CityData::race` holds after a capture.** It decides when a tile
  counts for nobody (−2). A capture with `reg_terr` diffed, or a
  reading of `race`'s writers, settles it. Whether the crate's recount
  already parts in a capture is booked as `docs/PARKED.md` 568.
- **`Wall::increment_stats`' predicate**, which decides that
  `num_buildings[T]` already counts the building when `n` is taken. Both
  readings read it from a folded body (`006432e2`); confidence is medium.
- **Plain-game reach of `check_capture`'s non-city swap branch and of a
  building bribe.** Both are unported in the crate. A skirmish capture that
  swaps a non-city building, diffed on `high_buildings`, settles it.
- **Cells versus tiles in `territory`.** This is a property of
  `territory`, not of this leaf. It decides whether `territory_high`'s
  checksum value agrees.
- All three leaves stay unpinned until the crate holds them and their rows
  on a kept capture diff clean.
