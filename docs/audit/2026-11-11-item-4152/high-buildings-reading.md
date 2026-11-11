# Family 20 — `high-buildings`: first reading

- Item 4152, wave 1 (first reading). Reader: Claude Opus 5.5, 2026-11-11.
- Leaves (offsets are `LeaderData`'s, `types.txt` line 42088 ff.):
  `territory_high` (+0x9dc, int), `reg_terr` (+0x145e, ushort[64]),
  `high_buildings` (+0x5660, ushort[129]).
- Layout fact used throughout: `leaders` is at 0x00e3a390 (`rise_z.map`,
  `?leaders@@3VLeaders@@A`), `Leaders::list` is `Leader[10]` at +0, and
  `Leader` (0x6eec) carries `LeaderData` (0x6ee4) at its offset 0 — the 8
  extra bytes are at the tail. So a `Leader*`'s `field_0xNNN` is
  `LeaderData + 0xNNN` with no shift, and slot `i`'s leaf is at
  `0xe3a390 + i·0x6eec + off`: `territory_high` 0xe3ad6c, `reg_terr`
  0xe3b7ee, `high_buildings` 0xe3f9f0 for slot 0.

The README row lists no writers for this family ("out by name; a mechanic
of its own"), so every writer below was found from scratch.

## `territory_high` (+0x9dc, int)

**Index, value.** One int per slot. The most tiles the slot has ever held
as territory at one fold of `territory` (+0x9d8) — a **high-water mark of
`territory`**, in tiles (the same unit as `territory`).

### 1. Writers, both nets

- Export: `->territory_high` appears only in `LeaderData::log_data
  (006e5110)` (a read). `field_0x9dc` hits are `HotKeyGroup`,
  `AchieveChartPane::set_chart_data (007f2c40)` (a read) and, under the
  `+8` guess, `Leader` methods' `field_0x9e4` — but `Leader` carries
  `LeaderData` at offset 0 (header above), so those are `other_team_terr`,
  not this leaf. The export net finds **no writer**.
- Listing: no store to `0x9dc(%reg)` from any leader/world function (every
  `0x9dc(` store maps to `HotKeyGroup`, `SelectGroup`, `Constants::init`,
  `Game::init`, `Console`, `TileSet`, `TerrainMap`,
  `CommandPackage::process_hotkey`, `ConquestSetupWin`), and no absolute
  `0xe3ad6c`. **Both writers store through a pointer to `territory`, at
  `+4`**, which is why neither net catches them:
  - `World::compute_all_territory (006b5700)`: `edx = 0xe3ad68`
    (`territory` of slot 0) at `006b5834`; the fold ends `cmpl %ecx,
    0x4(%edx); cmovgl 0x4(%edx),%ecx; movl %ecx, 0x4(%edx)` at
    `006b5877`–`006b5880`.
  - `GameDaemon::check_borders (00732060)`: the same loop, `edx =
    0xe3ad68` at `0073211b`, decompile lines 51–73.
- Clears: `Leader::init (006e3930)` **only on its `param_2 < 0` path**,
  `memset(this, 0, 0x6929)` (`push $0x6929` at `006e398c`, call
  `006e3995`). Its ordinary path zeroes `territory` (`006e4a7e`),
  `explored`, `effective_pop`, `other_team_terr` … but **skips +0x9dc**
  (the run `006e4a78 0x9d4`, `006e4a7e 0x9d8`, `006e4a84 0x9e0`).
  `Leaders::init (006ed850)` calls `Leader::init(i, -1, i)` for all ten
  slots (`006ed8c5`–`006ed8dd`, bound `0xe7f8c8`), from
  `Game::init_rules_and_teams (00589bb0)`; `WalkDataGame::walk_data
  (005a2360)` does the same on one record at `0xe789dc`.
- Save/load: `LeaderData::walk_data (006d6750)` hands the whole
  [+0x8, +0x692a) run over (`docs/CHECKSUM.md` §8), so a loaded game
  restores it; not a rule.

### 2. Event, step, predicate

Both writers run the same fold over slots 0..7 (bound `0xe724c8` =
`0xe3ad68 + 8·0x6eec`), **only slots with `leader_flags & 1`**:

1. `territory = Σ reg_terr[r]` over land regions `r = 0..63` whose
   `Region::size` (+0x14) is non-zero (regions are `0x88` apart, bound
   `0x2200`).
2. `territory_high = max(territory_high, territory)` (signed compare).

When each runs:

- `compute_all_territory`: unconditionally after re-running
  `compute_reg_territory(r, 1)` to completion on every land region with
  `size != 0` (game_daemon `borders = -1` = unlimited budget). Callers in
  a plain game: `Setup::build_cities (005ab910)` and
  `Setup::build_game (005ac190)` twice — so before frame 1. CtW's
  `ConquestGame::build_cities` and the scenario editor/triggers
  (`ScenarioFuncSet::create_building`, `gain_upgrade`, `lose_upgrade`,
  `ScenarioRead::import`, `ScenarioEditor::*`) also call it.
- `check_borders` (per frame, from `GameDaemon::process_all (00732700)`):
  it spends the 256-tile budget (`borders` zeroed at its head; a call
  returns 1 = budget hit / unfinished, 2 = region finished). The fold runs
  **only on a frame where some call returned bit 2** (a region finished
  this frame) **and** no land region with `size != 0` still has
  `Region::borders` (+0x2c, the cursor) `< size`; otherwise it returns
  before the fold (decompile lines 34–49).

So `territory_high` moves only at the frame a whole border pass completes
(or at setup), and never decreases except through the memset.

### 3. Reset

`Leader::init(·, -1, ·)`'s memset, i.e. `Leaders::init` at game creation.
Not `Leader::init` with `param_2 >= 0` (`Setup::build_leader (005abc70)`,
`Setup::build_game`, `ScenarioRead::load_leader_chunk`, `MapScenario::make`,
`ConquestGame::build_conquest_game`, `ScenarioEditor::new_leader`), and no
per-frame fold.

### 4. Readers

`AchieveChartPane::set_chart_data (007f2c40)` (end-game chart:
`territory_high · 100.0f / worldc +0x78` — UI, float), `LeaderData::log_data`.
No sim reader found (no `0x9dc(` load in any leader/AI function; no
absolute `0xe3ad6c`). It changes no decision.

### 5. Crate site

`territory` itself is `economy::Holdings::territory`
(`crates/sim/src/economy.rs:554`), written by
`Sim::update_territory_holdings` (`crates/sim/src/territory/upkeep.rs:128`),
which `crates/sim/src/territory/border_pass.rs` calls at the two
original sites: `start_border_pass` pre-play (line 56, the setup
`compute_all_territory`) and `advance_border_pass`'s tail when every land
region is done and one finished this frame (line 248, `check_borders`'
fold); also `settle_borders` (line 170). A builder adds a per-player
`territory_high` beside `Holdings::territory` and folds
`max(high, territory)` inside `update_territory_holdings`, **for live
slots only**; it is never cleared except at sim creation. Checksum: the
row `("territory_high", 0x9dc, 4, 1)` already exists at
`crates/sim/src/checksum/leaders.rs:236`; the emitter needs one arm next
to `"territory"` (line 1145), and `crates/rondata/src/diff/checksum_lines.rs:1840`
holds it out.

Caveat for the builder: the crate counts cells where the original counts
tiles (`upkeep.rs` doc says the unit cancels for the tax ratio). For
`territory_high` the **absolute** value is checksummed, so whether
`territory` is already exact in the original's units decides whether this
leaf can agree; `territory` is not on the held-out list, which suggests it
already does.

### 6. Plain game?

Yes: setup's `compute_all_territory` and every frame's `check_borders`.

### 7. Confidence, not read

High on the rule (listing-confirmed at both stores). Not read: the bodies
of `ScenarioFuncSet::*`, `ConquestGame::build_cities` beyond the call;
whether any code reaches +0x9dc through a pointer other than
`&territory + 4` (both nets are blind to that shape; I searched the
`0x9d8(`/`0xe3ad68` sites only, which are the natural bases).

## `reg_terr` (+0x145e, ushort[64])

**Index, value.** `reg_terr[r]`, `r` = the original's region number
0..63 (the slot in `Regions::list`, records `0x88` apart; the land half —
`compute_all_territory`'s second loop writes owner `0xff` into every tile
of slots 64..127, so sea tiles are never anyone's). Value: **the number of
tiles of region `r` the border pass has so far assigned to this slot** in
the region's current pass. It is a per-region territory count, **not** a
high-water mark, and mid-pass it is a *partial* count (see §2).

### 1. Writers, both nets

- Export: `->reg_terr` no hits. `field_0x145e` hits (all through
  `leaders.list[...]`, i.e. `Leader`, offset 0 = `LeaderData`):
  `World::compute_reg_territory (006b0bb0)` lines 82–88 (store 0) and
  620–621 (`+= 1`), `World::compute_all_territory (006b5700)` lines 16–54
  (store 0) and 100 (read), `GameDaemon::check_borders (00732060)` line 60
  (read), `BuildTypeData::blocked_location (006375b0)` line 183 (read).
- Listing (`0x145e(`, `0xe3b7ee`): stores at `006b0c17`/`006b0c2c`
  (`movw %cx,(%eax)` through `eax = 0xe3b7ee + 2r`), `006b185b`
  (`incw 0xe3b7ee(,%eax,2)` with `eax = owner·0x3776 + r`),
  `006b570a`… (`compute_all_territory`'s 32-dword clear per slot).
  Reads at `00637911`, `006b5860`, `00732140`.
- `Leader::init (006e3930)` ordinary path: zeroed by the per-index loop
  based at `this + 0xd64` (`puVar11[0x37d]`, `0xd64 + 2·0x37d = 0x145e`,
  for `i < 0x40`, decompile line ~698); the `param_2 < 0` path's memset
  covers it too.

### 2. Event, step, predicate

All in `World::compute_reg_territory(r, reset)`; its only callers are
`compute_all_territory` (`reset = 1`, `006b5777`) and `check_borders`
(`reset = 0`, `0073209f`).

- **Zero.** If `reset`, `Region[r].borders` (+0x2c, the pass's tile
  cursor) is set to 0 first. Then **if the cursor is 0** — a fresh pass on
  this region — for each slot 0..7 (bound `cities + 0x10 + 8·0x1c =
  0xc09a50`) with `leader_flags & 1`: `reg_terr[r] = 0`,
  `reg_known_rares[r]` (+0x4d4) `= 0`, and every live city's byte +0x65
  cleared. This happens **before** the budget test, so a region whose turn
  is cut off still gets its row zeroed on its first call.
- **Return early** if cursor `>= Region::size` (nothing to do) or
  `game_daemon->borders > 0xff` (budget spent this frame).
- **Step, one tile at a time** from the cursor (cursor `+= 1`, budget
  `+= 1` unless the budget is `-1` = unlimited): the tile's best claim
  over the eight slots (start slot rotates with the tile's x, `(k + x) & 7`)
  and their cities and forts gives an owner `local_14`: a slot, `-1` (no
  claim within `TERRITORY_BASE << 8`), or `-2` (the winning city's own
  owner byte +0x5f differs from the slot whose list held it). The owner
  byte (`World +0x134` tile record `+0xf`) and runner-up (`+0x10`) are
  written; then **if `local_14 >= 0`: `reg_terr[local_14][r] += 1`**
  (`006b185b`, a 16-bit `incw`). `-1` and `-2` tiles count for nobody.
- The influence arithmetic is the territory mechanic already ported
  (`crates/sim/src/territory.rs`); I did not re-derive it for this leaf.

So between a border fix (which resets every region's cursor to 0,
`Regions::fix_all_borders`) and the frame the pass reaches the end of
region `r`, `reg_terr[·][r]` holds **only the tiles reached so far**
(zero until the region's first call). After the pass completes, it equals
the count of tiles in `r` whose owner byte is that slot.

### 3. Reset

The fresh-pass zero above (per region, on its cursor-0 call);
`compute_all_territory`'s blanket clear of all 64 for every live slot
before it re-runs every region; `Leader::init` (both paths). No per-frame
fold.

### 4. Readers

- `compute_all_territory` / `check_borders`: summed into `territory`
  (see `territory_high` §2).
- **`BuildTypeData::blocked_location (006375b0)` line 183** — a decision:
  when placing a city, the spacing `CITY_SPACING` drops by
  `RELAX_CITY_SPACING` if `Region[r].size · 9 / 10 <= reg_terr[who][r]`.
  A transiently zero row (mid-pass) therefore changes city placement.
- `LeaderData::log_data`.

### 5. Crate site

- The border pass: `crates/sim/src/territory/border_pass.rs`,
  `Sim::advance_border_pass` (line 184): its `if *at == 0` arm (where
  `clear_bordering` and `zero_region_rares` already model the cursor-0
  zero) is where `reg_terr[p][r] = 0` for live slots goes, and its per-cell
  loop (`set_owner`, then `if let Some(p) = who.player()`) is where
  `reg_terr[p][r] += 1` goes. `Owner::Ambiguous` (the `-2`) and
  `Owner::None` already map to "nobody" (`crates/sim/src/world.rs:168`).
  `start_border_pass`'s pre-play branch and `settle_borders` are the
  wholesale (= `compute_all_territory`) sites: rebuild the full count.
- Today the crate **derives** it: `Sim::reg_terr`
  (`crates/sim/src/cities/place.rs:344`) counts the region's cells whose
  visible owner is the player, which is what `cities/location.rs:305`
  (the `blocked_location` rule) reads. That equals the original only when
  no pass is open; mid-pass the original's count covers reached tiles only
  while the crate's covers reached tiles at the new owner plus unreached
  tiles at the old owner. `docs/PARKED.md` (568) already records "`reg_terr`
  is still read live, not through the sweep", and
  `crates/sim/src/cities/city.rs:1004–1014` names the transient.
- Checksum: row `("reg_terr", 0x145e, 2, 64)` exists
  (`crates/sim/src/checksum/leaders.rs:294`); an emitter arm would use
  `leader_region_array` (line 574) to place values by the original's region
  number; held out at `crates/rondata/src/diff/checksum_lines.rs:1820`.
  Emitting the derived count would agree on every frame no pass is open
  and part on the frames one is.

### 6. Plain game?

Yes: setup's `compute_all_territory`, and `check_borders` every frame after
any border fix (city founded, building finished/lost, Civic tech, rare
claimed …).

### 7. Confidence, not read

High on the zero and `+= 1` rule and the predicate `local_14 >= 0`
(listing at `006b0c21`–`006b0c69` and `006b184a`–`006b185b`). Medium on
the exact meaning of `-2` (city +0x5f read as the city's own owner byte;
the crate's `Owner::Ambiguous` doc agrees). Not read: the full per-tile
winner arithmetic (lines 320–585), the fort arm, the goods/rares tail.

## `high_buildings` (+0x5660, ushort[129])

**Index, value.** `high_buildings[k]`, `k = basic_type(T) − 0x19e` (414):
the building record of the **root of T's upgrade line**. Building type
ids run 0x19e.. (the same bias `num_buildings` and `reg_buildings` use;
`num_units` is biased by 0x32). `basic_type` is the type vtable's `+0xe4`
slot — `LF_ONEMETHOD basic_type, vftable offset = 228` in
`ObjectTypeData`'s field list (`rise.pdb` type `0x14662`/`0x14664`);
bodies `BuildTypeData::basic_type (00639970)` / `ObjectTypeData::basic_type
(006619e0)` follow `TypeData::from` (+0x3c) until it is negative and
return that record's `type` (+0x4). Value: **the most finished buildings
of T's line-from-T this slot has had at one activation** — a high-water
mark, 16-bit.

### 1. Writers, both nets

- Export: `->high_buildings[x + 0x4f]` in `LeaderData::get_support_count
  (006da110)`, `get_units (006e07d0)`, `has_general (006e1050)`,
  `IFaceOptions::setup_selected (008165b0)` are **not this leaf**:
  `0x5660 + 2·(x + 0x4f) = 0x5762 + 2·(x − 0x32)` = `num_units[x − 50]`
  (the biased-base trap). `LeaderData::log_data` line 2385 is the dump
  loop (reads). `field_0x5660` hits: `Leader::init` line 726 (clear) and
  `TerrainOut`/`TerrainData` (other classes).
- Listing: `0x5660(` stores are `TerrainData::TerrainData`,
  `~TerrainData`, `TerrainOut::one_time_init`, and `Leader::init`'s `lea`
  (`006e4bad`). No absolute `0xe3f9f0`. The writer is reached with the
  type-id bias folded in, `0x5660 − 2·0x19e = 0x5324` /
  `0xe3a390 + 0x5324 = 0xe3f6b4`:
  - **`Build::activate (00623e20)`**: `movw %bx, 0xe3f6b4(,%ecx,2)` at
    `00624c16`, `ecx = owner·0x3776 + basic_type(T)` (`00624bf0`–`00624bfa`).
    The only store found under either bias.
  - Other `0x5324(`/`0xe3f6b4` sites are reads: `BuildData::construct_time
    (0062d5c0)` at `0062d8d8`, `TypeData::get_cost (00664090)` at
    `00665762`.
- Clears: `Leader::init` ordinary path, the 129-entry loop at decompile
  726–732 that zeroes `num_buildings[i]` (`puVar11[-0x81]`) and
  `high_buildings[i]` together; and the `param_2 < 0` memset.
  `Build::activate` is virtual only (vtable slot `+0x1a8`, no direct call).

### 2. Event, step, predicate

`Build::activate(p1, p2, p3)` (decompile 603–617; listing
`00624ba4`–`00624c16`), for a building of type `T` owned by slot `s`
(`Object +0x9`):

1. `n = num_buildings[s][T] + get_buildings(to(T))`, where `to` is
   `BuildTypeData::to` (+0x2e0, a `TypeIndex`) and
   `LeaderData::get_buildings (006e0680)` recursively adds
   `num_buildings[to] + get_buildings(to(to)) …` while `to >= 0`. So `n`
   counts T and every record downstream of T in its line — not the line's
   records upstream of T.
2. **If `high_buildings[s][basic_type(T)] < n` and `p1 == 0`**:
   `high_buildings[s][basic_type(T)] = n` (as ushort).
3. Inside the same branch: `T == 0x21a` sets `leader_flags |= 0x10000000`;
   and if `p3 != 0` and (`game->semaphore.ptr[1] & 0x10` and `ptr[2] & 2`
   both clear, or `ScenarioData::building_unit_bonus`), the nation's free
   units (Nubian caravan on `0x1b4`, Iroquois scouts on `0x1ab`, … to
   ~line 1150). The mark is what stops a rebuilt replacement from paying
   again.

`num_buildings[T]` already includes this building when it is counted:
`Wall::activate (0063e4b0)` runs first (activate line 158) and calls
`Wall::increment_stats (00643270)` — which does `num_buildings[T] += 1`
(`incw 0xe3f5b2(,%ecx,2)` at `006432e2`) and `reg_buildings[region][T] += 1`
for region < 64 — when `build_flags & 0x10`, or vslot `+0x20` is non-zero
and the object's `+0x72` short is `>= 0` (Wall::activate lines 56–62).

**The `p1` predicate, by caller** (pushes read from the listing; the last
push is `p1`):

| caller | (p1, p2, p3) | raises the mark? |
|---|---|---|
| `Wall::do_construct (006434d0)` `00643663` — construction finished | (0, 1, 1) | yes, and pays free units |
| `Setup::build_cities (005ab910)` `005ababf` — starting buildings | (0, 0, 0) | yes, no units |
| `Build::check_capture (006276a0)` `00627f82` | (0, 1, 0) | yes |
| `City::find_buildings (007384c0)` `007388a8` | (0, 1, 0) | yes |
| `SpellType::cast_bribe (00672d00)` `006734e2` | (0, 1, 0) | yes |
| `Leader::defeat_by (006d1c80)` `006d20ec` | (0, 1, 0) | yes |
| `Leader::produce_building (006e1400)` `006e2e0a` | (0, p2, 0) | yes |
| `Cities::capture_city (00733380)` `00733687`, `0073390a` | (1, 1, 0) | **no** |
| `ScenarioFuncSet::switch_teams (009f46b0)` `009f4833` | (1, 1, 0) | **no** |
| `ScenarioFuncSet::create_building`, `Console::hurry_action` | (0, 1, 1) | yes (trigger / cheat) |
| `ConquestGame::build_cities`, `ScenarioEditor::*`, `ScenarioRead::*` | (0, 0, 0) per decompile | yes |

`UnitBalance::next`, `ConsoleWin::run_cmd` print no arguments; not read.

### 3. Reset

`Leader::init` (both paths). Never lowered otherwise — a building's loss
(`Wall::decrement_stats (00642da0)` lowers `num_buildings`) leaves the mark.

### 4. Readers

- `TypeData::get_cost (00664090)` `00665762`: `high_buildings[T] == 0`
  is one arm of the Iroquois free first Senate (with tribe bonus `0x12`,
  the Senate test and `IROQUOIS_QUICK_SENATE`). **Indexed by T itself,
  not `basic_type(T)`** — the writer writes at the root; for the Senate
  root = T.
- `BuildData::construct_time (0062d5c0)` `0062d8d8`: the same predicate,
  compared against a constant, for the construction clock.
- `Build::activate` itself (the compare in §2).
- `LeaderData::log_data`.

### 5. Crate site

**Already held**: `Cities::building_high: Vec<Vec<i32>>`
(`crates/sim/src/cities.rs:45–51`), raised only by
`Sim::claim_building_high` (`crates/sim/src/cities/nations.rs:219`),
called from `Sim::activate` (`crates/sim/src/cities/city.rs:2420`, call at
2578) with `captured` = `p1` and `counted` = `p3`; it computes `n` with
`buildings_of_line` and indexes by `base_build_type(rec)` (the `from`
chain root). Read by `iroquois_free_senate` (`city.rs:1253`) and
`construct_time_of` (`city.rs:1795`). What is missing is the **checksum
emitter**: the row `("high_buildings", 0x5660, 2, 129)` exists
(`crates/sim/src/checksum/leaders.rs:297`) but no arm emits it, so it is
held out as `(unheld)` (`crates/rondata/src/diff/checksum_lines.rs:1794`).
The `"num_buildings"` arm (`leaders.rs:1201`) shows the indexing: crate
build record = original index (type − 0x19e), so the arm is
`building_high[who][rec]` for `rec < 129`, zero-padded. A builder should
also check that each crate caller of `activate` passes `captured` as the
table above does (bribe, `defeat_by`, `check_capture`, `find_buildings`
pass `p1 = 0`).

### 6. Plain game?

Yes: setup's starting buildings and every finished construction; also
city-radius pickup, building capture, bribes and a defeated leader's
handover. Only `capture_city` / trigger team-switch are exempt.

### 7. Confidence, not read

High on the writer, index and predicate (listing at `00624ba4`–`00624c16`;
`basic_type` slot settled from the PDB's `LF_ONEMETHOD`, not the folded
symbol). Medium on `to` meaning "next record in the upgrade line"
(name from `types.txt`; consistent with `get_buildings`'s recursion). Not
read: the free-unit arms past line ~660, `Wall::activate`'s vslot `+0x20`
identity, the `UnitBalance`/`ConsoleWin` callers' arguments, and whether
any `Build` subclass overrides slot `+0x1a8`.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `territory_high` | World::compute_all_territory (006b5700) `006b5880`; GameDaemon::check_borders (00732060) — both via `&territory + 4` | `max(high, territory)` after `territory = Σ reg_terr[r]` (live slots, regions with size ≠ 0); check_borders only on the frame the last open region finishes | `Leader::init(·, −1, ·)` memset only (Leaders::init); the ordinary `Leader::init` path skips it | new field beside `Holdings::territory`, folded in `update_territory_holdings` (`territory/upkeep.rs:128`); emitter arm at `checksum/leaders.rs:1145` | yes | high |
| `reg_terr` | World::compute_reg_territory (006b0bb0) `006b0c2c` (zero), `006b185b` (`+= 1`); World::compute_all_territory (006b5700) (clear) | per land region: zero for live slots when the region's cursor is 0; `+1` per tile the pass assigns to slot ≥ 0 (`−1`/`−2` count for none) | per-region fresh pass; `compute_all_territory`; `Leader::init` | `advance_border_pass` (`territory/border_pass.rs:184`) cursor-0 arm and per-cell loop; today derived live by `Sim::reg_terr` (`cities/place.rs:344`), parked 568 | yes | high (rule), medium (`−2`) |
| `high_buildings` | Build::activate (00623e20) `00624c16` (only store) | at `basic_type(T)−414`: raise to `n = num_buildings[T] + Σ downstream (to-chain)` if larger and `p1 == 0` | `Leader::init` (both paths) | held as `Cities::building_high` (`cities/nations.rs:219`); only the emitter arm is missing | yes | high |
