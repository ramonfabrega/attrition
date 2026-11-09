# Territory reading — group `world-2` (the type tables and the world's state)

The seventy-first tranche's reading fan-out (steer, 2026-11-06), group `world-2`: 60 functions
of the `type`, `types` and `world` stems from `tools/rubric.py --tables 0`'s open territory.
Read by name from the decompile export (`~/ghidra-projects/decomp`) against `crates/sim` and
`docs/`, by a reader on Opus 5.5. One ruling per function; the brief's ruling set. Nothing
here is transcribed; only the expression a ruling turns on is quoted.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00667570 | TypeData::can_pay_cost | layout | yes (run1016…) | twin | `crates/sim/src/cost.rs:995` `affordable`: both loop the six goods skipping `type_avail==0` and a zero charge, divide the purse (`bucket^0x8221`, less `escrow@+0x468` unless the escrow flag) by the charge, return 0 at once on a zero quotient (`== 0`, not `<= 0`, on both sides), keep the **maximum**, and answer 10 (`PLENTY`, `cost.rs:979`) when nothing constrained. Agree; the emulator sweep `crates/sim/tests/sweep/type_data_can_pay_cost.rs` asserts eax and both windows. The front arms (gamec semaphore bit 8 → 10; scenario `units_free`/`buildings_free`/`techs_free` → 10) are cheat/scenario switches a playing skirmish never sets; the port's `None => PLENTY` callers (`ai_host.rs:591`, `ai_types.rs:100`) cover the uncosted case. |
| 00667730 | TypeData::get_limiting_resource | layout | no | ui | Vslot `+0x88` (vtables.txt:29418…): the first available good whose charge exceeds the bucket, `-1` if two or more do. Its only caller by slot is `LeaderOut::warn_resources@006d3650` (line 77), the "not enough X" warning text. No sim effect. Census: the class `LeaderOut` (an `*Out` output class) already reads as UI; proposed pattern for the callee: a function whose every caller is in a `.*Out$` class or `IFace.*` — catches this one. |
| 006677c0 | TypeData::say_name | layout | yes | ui | Appends `display_name` to a `String`. Callers are text builders (`ObjectData::say_name`, `City::find_buildings`' message, `BuildTypeOut::draw_template`, `ConquestNews::add_*`). Entered because the logged runs print names; no state written. Proposed pattern: short name `^say_name$` (catches `TypeData::say_name`, `TechTypeData::say_name@0066d440`, `ObjectData::say_name@00645310`, `UnitData::say_name`, `ConquestLeader::say_name`). |
| 00667ee0 | Type::backup | layout | yes | absorbed | Copies a type's `job_time`, `tribe_mask`, `type`, the name and type-name strings, `preq[3]` and `costs[6]` into a `TypeBak`. Called through `ObjectType/UnitType/BuildType/TechType::backup` from `Game::init_type_backup@005898c0` (from `Game::run`), with `Game::restore_type_backup@005896d0` undoing it at `Game::end_game_close` and `ScenarioRead::import` — the snapshot that lets scenario triggers (`ScenarioFuncSet::enable_type`) and the editor (`IFaceEditorTypeEditor`) mutate the global type table and have it revert after the game. Covered by the crate's per-`Sim` type tables built fresh from the loader each game (`crates/rondata/src/load.rs`), which nothing outside the game mutates. Entered every run because `Game::run` takes the snapshot; it writes no simulated state. |
| 00668000 | Type::restore | layout | yes | absorbed | The inverse of `Type::backup` (writes `job_time`, `tribe_mask`, the name, `preq[3]`, `costs[6]` back). Same coverage: the crate never mutates a shared type table across games. The emulator sweep named in `docs/PARKED.md:6502` checks it; no crate fn backs it and none is needed. |
| 00668080 | Type::sum_cost | layout | no | dead | Sums the vslot `+0x78` (`get_cost`) charge over the available goods (context and escrow arguments passed through). It is vslot `+0xcc` of `Type` and only the Tech/Spell/Bonus type vtables (`vtables.txt`, `Type::vftable@00b43cbc` and nine kin); no direct call by name exists in the export, and every `+0xcc` call site found is on an `ObjectData`/`UnitData` receiver (where `+0xcc` is `UnitData::is_supply`), not a type. No lockstep run entered it. Census: no name pattern fits; it would fall under a proposed "no caller located, never entered" mark only. See Not established. |
| 006680e0 | Type::sum_rules_cost | layout | yes | twin-differs | The six cost words `+0x18..+0x2c` summed, times the constants' `unit_/build_/spell_/tech_cost_factor` chosen by the first class test that holds (units `0x32..0x19d`, buildings `0x19e..0x21e`, spells `0x275..0x2ab`, else tech). The crate carries it as `combat::CombatType::cost` (`crates/sim/src/combat.rs:343`), filled at `crates/rondata/src/load.rs:1433` (units) and `:1654` (buildings) as `c.cost.iter().sum::<i32>() * 10` — the literal ten, not `Tuning::unit_cost_factor`/`build_cost_factor` (`crates/sim/src/core/tuning.rs:573/575`, which `cost::Kind::factor` `cost.rs:32` already selects). Agree on the stock data (all four factors ship 10); differ on a mod that changes `UNIT_COST_FACTOR` or `BUILD_COST_FACTOR`. The sweep `sweep/type_sum_rules_cost.rs` re-derives it inline, not against the loader. |
| 006681f0 | Type::pay_cost | layout | yes | twin | `crates/sim/src/cost.rs:1044` `pay`: per available good, a non-escrow payment larger than `bucket - escrow` zeroes the escrow first; the bucket floors at zero after the charge; an escrow payment draws the escrow down, floored at zero. Agree on all three predicates and both clamps. The out-parameter (`param_5[good] = charge`) is the crate's `charges` array computed beforehand (`cost.rs:952`) and stored by the caller (`production.rs:98`). Sweep `sweep/type_pay_cost.rs`. |
| 00668f30 | Types::get_type_index | open | no | replaced | Linear scan of the global type table for a `type_name` (`+0xb0`) equal to the string, `-1` (with an error report) otherwise. Callers: `ScenarioRead::load_*_chunk` (scenario files), `ScenarioFuncSet::get_type_index` (the trigger VM), `Conquest*` (CtW). The XML loaders / scripting VM groups; the crate resolves names in `crates/rondata` at load and in `ai_host.rs:483` for the AI script's `get_type_index`. |
| 006691c0 | Types::good_key | open | yes | replaced | String → good `TypeIndex` (`0..0x31`), the two literals `TYPE_NONE`/`TYPE_ANY_BUILDING`, else an error. Callers are load-time: `GoodType::init`, `BuildType::init`, `Lands::init@0067e730` (the `<MAKE type>` names, `crates/sim/src/world.rs:644` records the `"none"` → `-1` answer), and `Map::init_resources`/`Map::place_resources` resolving a map script's resource names. The XML loaders group; entered every run because every game loads. |
| 00669330 | Types::build_key | open | yes | replaced | String → building `TypeIndex`, same shape. Callers `BuildType::init`, `UnitType::init`, `TechType::set_research@0066cba0` (called only from `Types::init`). XML loaders. |
| 006694a0 | Types::unit_key | open | yes | replaced | String → unit `TypeIndex`; caller `UnitType::init` (and `Types::object_key`). XML loaders. |
| 00669610 | Types::tech_key | open | yes | replaced | String → tech `TypeIndex`; callers the six `*Type::init` loaders. `crates/sim/src/tech.rs:26` records the resolved slot shape (`Preq`). XML loaders. |

## Part 2

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00669840 | Types::finalize_grafting | open | yes | twin | Three loops, run once from `Game::init_data` (and `WalkDataGame::walk_data` on load). (1) For each unique unit (`+0x2b4 & 0x1000000`) in `0x32..0x191` whose `GRAFT` (`+0x25c`) is another valid type: with the lobby's no-unique-units flag (`info.flags & 8`) clear, the graft's `tribe_mask &= ~unique's` then `|=` the mask of tribes whose `+0x58` flag is set; with it set, `|= unique's`. Twin: `crates/sim/src/tech.rs:896` `TechTree::finalize` gathers the union into `graft_unique_mask` and `tech.rs:1092` `live_tribe_mask` applies it at query time — `(mask & !c) | barbarians`, or `mask | c` under `no_unique_units`. The original's in-place repetition over several uniques reduces to the same `(m & ~(u1\|u2)) \| b`. Agree. (2)/(3) cache `get_age_slow` into `+0x278` for units `0x32..0x191` and buildings `0x19e..0x21e`, and `get_military_level_slow` into `+0x2dc` for units: `crates/sim/src/nations.rs:247` `finalize_type_ages` and `tech.rs:1005` `military_level_of`/`military_level_in`. Agree on the ranges and the lobby dependence. |
| 00668ae0 | Types::object_key | layout | yes | replaced | `unit_key(s,1)`, falling to `build_key(s,1)` when that answers the any-unit index, else an `OBJECTS` key error. Caller `SpellType::init`. XML loaders; the emulator sweep `crates/sim/tests/sweep/types_object_key.rs` asserts the fall-through. |
| 00669100 | get_unique_types2 | layout | no | ui | Returns the byte `unique_types2`. The `type_buckets` family's only non-self callers are `IFaceGarrison::update_visibility@00806170` and `IFaceSelected::update_selected@00823790`: counting the selection (and a garrison) by type for the panel. Proposed pattern: short name `^(add_type_bucket\|get_type_count\|get_unique_types\|clear_type_buckets)2?$` — catches these six and `clear_type_buckets@006691a0`, `clear_type_buckets2@00669180`. |
| 00669110 | get_type_count2 | layout | no | ui | Reads one byte of `type_buckets[7][i + 0x1bb]`. Same family, same pattern. |
| 00669120 | get_unique_types | layout | no | ui | Returns the byte `unique_types`. Same family, same pattern. |
| 00669130 | get_type_count | layout | no | ui | Reads the two-index byte table at `0xed89ce + j*0x1ed + i`. Same family, same pattern. |
| 00669140 | add_type_bucket2 | layout | no | ui | Increments `type_buckets[7][i+0x1bb]`, bumping `unique_types2` on its first count. Same family, same pattern. |
| 00669160 | add_type_bucket | layout | no | ui | The two-index twin of `add_type_bucket2`, bumping `unique_types`. Same family, same pattern. |
| 00669780 | Types::walk_data | layout | no | replaced | Visits two bytes (`+0x1e2`, `+0x1e3`) of every tech type `0x220..0x274` with a `DataWalk`. Callers `WalkDataGame::walk_data@005a2360` (from `SaveGame::do_save`/`LoadGame::do_load`). Proposed replaced group **"the data walk"**: the `DataWalk` visitors that serialise state for save/load, the recorded game's rules block (`RecordGame::read_rules`/`write_rules`) and the sync checksum (`CheckSums::check_all`, `GameLog::say_checksum`) — pattern `::walk_(rules_)?data$`. |
| 00669800 | Types::walk_rules_data | layout | yes | replaced | Calls vslot `+0xc4` (the type's own rules walk) for every type `0..0x325`. Caller `Game::walk_rules_data@00589550`, whose callers are `RecordGame::read_rules/write_rules`, `CheckSums::check_all`, `CommandManager::issue_check_sums` and `GameLog::say_checksum` — entered by the lockstep runs through the checksum/record. Proposed group "the data walk", as above. |
| 006b0180 | World::repair_world | open | no | debug | Re-marks every tile of every water cell (`land` 1 or 2, not `flags & 0x100`) `mask = mask & ~0x10 \| 0x20`, then rebuilds more world data. Its one caller is `ConsoleWin::run_cmd@007d6a70` case 0x18, the console `repair` verb (`docs/AI.md` "Debug verbs"). Census: the class `ConsoleWin`'s callees; proposed pattern `^World::repair_world$` or "only caller is `ConsoleWin::run_cmd`". |

## Part 3

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006b0bb0 | World::compute_reg_territory | open | yes | twin | 28 KB, read by its first screen, the docs and the crate's line citations, not whole. The weighted-distance Voronoi of one region's cells over every city and fort claim, the per-nation border arms, the runner-up `bordering` mark and the per-cell goods scan. Twin: `crates/sim/src/territory.rs:389` `compute_region_territory` (and `:357` `compute_all_territory_won`, from `Sim::recompute_territory` `lib.rs:6597`, paced by `crate::border_pass`), the goods scan `crates/sim/src/rares.rs:445` `claim_cell_goods` (oil → `oil_patches`, else `Leader::new_rare` behind `was_really_seen`, plus `good_seen_bits`). `docs/ATTRITION.md` §Territory states the rule and its diff (Golden chapter four, run132/133, per-cell `who` against the `WORLD` dump). Agree as diffed. Note: `docs/ECONOMY.md:523` and `:599` still say this function's `new_rare` call "is not carried"; item 1555 built it (`rares.rs:440`), so those two paragraphs are stale. |
| 006b1d60 | World::revert_coast | open | no | editor | Restores the original coast after random terrain is added; sole caller `ScenarioEditor::add_random_terrain@009a29e0` (`docs/AI.md` "editor" list). Census: class `ScenarioEditor`'s callees; proposed pattern "only callers in `ScenarioEditor`/`TerrainMap`". |
| 006b5130 | WorldData::find_slope_normal | open | yes | ui | Float cross product of three `TerrainOut::find_data_z` heights 0x14 apart, normalised: the ground normal a sprite is tilted to. Callers `GuyOut::draw@005dc490` and `DeathObjOut/draw@008d4090` only — the renderer. Its float arithmetic is itself proof it is not sim state (`CLAUDE.md`'s no-float rule mirrors the original here). Proposed pattern: every caller matches `Out::draw$`. |
| 006b58b0 | World::analyze_map | open | yes | unmodelled | Run once at setup. (a) Over regions `1..` (records from `+0x88`): `land_size` (`+0x78`) the sum of sizes — **twin** `crates/sim/src/victory.rs:110` `land_size` (item 3575, regions numbered 1..63). (b) A region where any active leader's per-region short (`LeaderData +0x1260 + 2·region`) is non-zero is a `player_reg` (`+0x50`, `Region.flags \|= 4`, size into P); any other of at least `max(1, (size*60 + d²/2)/d²)` cells (`d` the map size's `+0x144`) a `resource_reg` (`+0x54`, `flags \|= 8`, size into R). (c) The cell census `forest_size`..`sea_resources` (`+0x58..+0x74`) and `total_metal` (`MountainsData::total_gather_size`). (d) `sea_map` (`+0x34`) outside `0..4` becomes `3 + (2P ≤ R)` / 2 / `land_size < 4·size/5`, and a file 3 becomes 4 under the same test. The crate has (a) only: `sea_map` is copied from the dump (`crates/rondata/src/diff/build.rs:228–240`, "the harness carries the original's `sea_map`") and nothing computes it (`terrain_init.rs:465` stops at `Map::make`'s value); `player_reg`, `resource_reg`, the census and the `Region.flags` 4/8 bits are not carried. Lockstep-entered and agreeing because the harness takes the dump's `sea_map`; the census fields are read only by `WorldData::log_data` and `TerrainWin::on_redraw` (the only `world->field_0x58..0x74` readers in the export), so they are inert to play. The field that shows it: `WORLD` under `[Start Game]` — `sea_map`, `player_reg`, `resource_reg` (`docs/AI.md` §15.8, run20's `resource_reg 8`), on any map whose style has no `<SEA_MAP>` or writes 3. Who reads `Region.flags & 4/8` was not settled (Not established). |
| 006b5cf0 | World::walk_data | open | yes | replaced | Visits the world by part number (0..13, −1 all: header, start arrays, oil arrays, scalars, each cell's `WData[0..0x15)`, tile masks and the seen grids, `wcoord_seen`, `danger[8]`, `CollBlock`s, four terrain arrays; `docs/MAPGEN.md` §the parts table). Callers `GameLog::say_checksum`, `CheckSums::check_all`/`check_world`, `CommandManager::issue_check_sums`, `SaveGame::verify_save`, `LoadGame::verify_load`, `WalkDataGame::walk_data`. Proposed group "the data walk" (with `Types::walk_data`). The crate mirrors parts 2, 5 and 6 as a test instrument only — `crates/sim/src/mapgen.rs:330` `MapGrid::start_part` and `WDATA_WALKED` (`mapgen.rs:18`) Adler-32 the same bytes to diff the setup trace — which is the diff, not a twin of play. |
| 006b0350 | World::compute_val | name | yes | twin | `crates/sim/src/map_values.rs:295` `compute_val`: water (`flags & 0x100` or `land` 1/2) draws nothing and is `(0,0)`; rings 3–4 of the circle table count cells and their land record's `make` slots (skipping a 0x1000 centre tile); the 5×5 penalties; one draw `count/4 + r%10`; the factors; `goods = 0x20 \| bits`. `docs/MAPGEN.md` §9 diff-backs it draw for draw on 280 traces and by seed and parts 2/5/6 at 7937/7945 (item 3807), the water and region-size factors reading-only (3807a). Agree as diffed. |
| 006b2c00 | World::wipe | name | yes | twin | Every cell: `flags 0`, `land 2`, `land_sub 0`, `region 0x40`, `region2 0`, `down −1`, `down_who/val/goods/light 0`, `who/who2 −1`, `blocked/bad/solid/was_seen 0`; the four start arrays' counts zeroed; tile masks zeroed. Twin `crates/sim/src/mapgen.rs:341` `MapGrid::wipe` with `WCell::WIPED` (`mapgen.rs:50`) — agrees field for field with the stores at `+0/+2/+6/+8/+0xa/+0xe/+0xf/+0x13`. The rest of the body (world scalars zeroed, `sea_map = −1`, `clear_seen`/`clear_seen2`, `wcoord_seen`, `danger[8]`, `Mountains::clear`, `Doober::close`) is absorbed by the crate building a fresh `World` per game. Diff-backed by part 5 (`docs/MAPGEN.md` §5). |
| 006b2de0 | World::add_starting_location | name | yes | twin | Pushes x onto `start_x`, y onto `start_y`, `(x, x−1, x, x−1)` onto `start_city_x`, `(y, y, y−1, y−1)` onto `start_city_y`, and sets the four cells' bits in `start_city_locs`. Twin `crates/sim/src/mapgen.rs:377` `MapGrid::add_starting_location`, same pushes in the same order and the same four city bits. Diff-backed by part 2 (`docs/MAPGEN.md` §5, `MapGrid::start_part`). |
| 006b3230 | World::adjust_for_blocking | name | yes | twin | 9 KB: 0x200 around 0x70 mountains, forest tiles blocked, 0x40 regions released and regions re-run, `eliminate_dead_mount_spots` twice, water edges 0x2000, forest borders unblocked, the gather bit 0x8000. Twin `crates/sim/src/map_values.rs:73` `adjust_for_blocking`; `docs/MAPGEN.md` §9 diff-backs it on Great Sahara (draw-free, seed and parts 2/5/6 at 7937 on 105 files, item 3807), with the water edges, the region release, the 0x1000 tile and the cliffs' 0x8000 reading-only (3807a). Agree as diffed; the reading-only arms are the mechanic document's to settle. |
| 0043f360 | WorldData::is_valid (Coord) | layout | yes | twin | `0 ≤ x < tile_xs·0xc0 && 0 ≤ y < tile_ys·0xc0` on unit coordinates. Twins `crates/sim/src/air.rs:2008` `in_world` (`width·UNITS_PER_CELL`, `UNITS_PER_CELL = 192·4` = `tile_xs·0xc0`, the same four comparisons) and `crates/sim/src/world.rs:1785` `World::accepts` (`floor_div` to the cell, then `contains` — a negative unit coordinate floors to −1 and is refused, as the signed test refuses it). Agree. |
| 0046d220 | World::get_wdata | layout | no | absorbed | `wdata + (y·xs + x)·0x1c`, no bounds test. Covered by `crates/sim/src/world.rs:1698` `World::index` and `cell_data` (`:1229`), which add a bounds test the original's callers make beforehand. |
| 0046d250 | World::new_coll_block | layout | yes | absorbed | Lazily allocates a cell's `CollBlock` (`BitMask<768>`, `+0` 0x300, `+4` 0x60, 96 zero bytes, `+8` 1) into `WData +0x18`. Covered by `crates/sim/src/collide.rs:545`, the occupancy flattened to one bit per unit cell over the whole map, allocated once; `docs/COLLISION.md` §2 states the layout and `add_to_world`'s allocate-where-none arm (`add_to_world_view.rs:80`). Allocation is unobservable. |
| 0046ef10 | WorldData::get_goods | layout | yes | twin | The cell's `goods` byte (`WData +0xd`). One caller, `Unit::think_scout`, tests `& 2` with no city and `& 0x10` with one, halving the score. Twin: `crates/sim/src/scout.rs:640` reads `world.cell_data(c).goods` and tests the same two bits under the same `city_num` arms. Agree. |
| 0046ef50 | World::set_down | layout | yes | absorbed | Stores `down` (`+8`) and `down_who` (`+0xa`) on one cell. Callers `Terrain::move_goody@0084a3d0` and `Terrain::move_ocean_goody@0084aec0` (mapgen). Covered by the direct field writes in `crates/sim/src/halfland.rs:359` `Pass::move_goody` (`:429–430`) and `crates/sim/src/map_resources.rs:353/759/788`, diff-backed by part 5 (`docs/MAPGEN.md`, item 3890). |
| 0046f760 | WorldData::is_valid (TCoord) | layout | yes | twin | `0 ≤ x < tile_xs && 0 ≤ y < tile_ys`. Twin `crates/sim/src/world.rs:1836` `World::tile_in_bounds` (`width·4`, `height·4`, the same four comparisons); the view seam `calc_gather_type_view.rs:95` `tile_valid` routes to it. Agree. |
| 0046f8c0 | WorldData::is_cliff_at | layout | yes | twin | `(tdata[tile_xs·ty + tx].mask & 3) == 1`. Twin `crates/sim/src/path.rs:438` `is_cliff` (`tile_mask(t) & tile::OBJECT == tile::OBJECT_CLIFF`, `world.rs:576`). Agree; `docs/PATHFINDER.md:762` and `docs/SCOUT.md:388` record the same reading. Callers `UnitData::invalid_loc`, `TerrainOut::terraform_for_nuke`. |
| 00471170 | WorldData::is_valid (int) | layout | no | twin | `0 ≤ x < xs && 0 ≤ y < ys` on cell coordinates. Twin `crates/sim/src/world.rs:1694` `World::contains`. Agree. Reached from `TerrainOut::check_for_diagonal` only through a P1 pair no shipped tile set yields (`crates/sim/src/halfland.rs:809`, item 3886), which is why no lockstep run entered it. |
| 00472af0 | WorldData::is_oil_at | layout | yes | twin | `wdata[y·xs + x].flags & 0x800`. Twin: the predicate `f & cell::OIL != 0` in `crates/sim/src/world.rs:1293` `land_class` (and `:1319`, `:1333`), with `cell::OIL = 0x800` named from this function (`world.rs:398`). Agree. The only callers by name are `Console::execute_at_cursor` and `ConsoleWin::run_cmd`, yet the lockstep runs entered it — see Not established (an ICF-folded twin accessor is the likely route). |
| 0047a970 | WorldData::is_edge | layout | yes | twin | `x == 0 \|\| y == 0 \|\| x == xs−1 \|\| y == ys−1` (the function returns the negation of the all-interior test). One caller, `TerrainGroup::available_rough_tile@006a24d0` (mapgen). Twin: the same four comparisons inlined at `crates/sim/src/terrain_groups.rs:469` in the port of `available_rough_tile` (`:461`). Agree. |

## Part 4

The image scan below is `tools/trace/report.py <exe> refs` (the dead-scan of items 935/940, read-only) over this group's uncalled rows, run 2026-11-06 on the installed `riseofnations.exe`: "dead" means no call, jump or address copy anywhere in the image.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006b1b80 | World::set_orig_coast | layout | no | dead | Clears 0x400 on a cell's flags (the decompile shows `wdata[0]`, an index lost). No caller; the sixty-seventh pass ruled it dead (`docs/MAPGEN.md:2774`, `docs/audit/2026-11-05-fable-pass-67.md:81`): its one RVA match at 0x67b5ac is `AmmoOut::draw`'s rel32, and it is on the guard's dead list. The scan agrees ("pointer only", that one rva). Census: already on the dead list. |
| 006b1ba0 | World::get_was_seen | layout | no | dead | `seen2[fog_xs·y + x]` (`+0x160`). Scan: dead (no reference). An out-of-line copy of an accessor every caller inlines; the crate's fog reads (`Sim::was_really_seen`, `crates/sim/src/rares.rs:460`) carry the job. Proposed: add to the dead list. |
| 006b1c60 | World::set_tocean | layout | no | dead | A tile's mask `& ~0x10 \| 0x20` (the ocean surface). Scan: dead. Its body is the write `check_and_set_halfland(0)`'s first pass makes inline (`docs/MAPGEN.md:1206`, ported in `crates/sim/src/terrain_init.rs`). Proposed: dead list. |
| 006b1c90 | World::set_waterhalf | layout | no | dead | Sets the tile's ocean surface and the cell's HALFLAND (0x100) on `param_3`, clears an ocean surface otherwise (the decompile drops the `else`). Scan: no reference; the byte before is not padding, so the scan does not exclude a fall-through; the listing does: the byte is the immediate of `set_tocean`'s `retl $0xc` at 0x6b1c8d. Proposed: dead list. |
| 006b1d10 | World::set_coastal | layout | no | dead | A tile's 0x400 (coastal) set on `param_3`, cleared otherwise (the decompile drops the `else`). Scan: dead. `Terrain::check_and_set_coastal@0084dd70` writes 0x400 inline, and that is what the crate ports (`crates/sim/src/terrain_init.rs:405`). Proposed: dead list. |
| 006b1eb0 | World::set_cliff_at | layout | no | dead | `set_blocked_at(t, 1)` then `mask & ~2 \| 1` (the cliff object). Scan: dead. The cliffs' writes are made inline by `Cliffs::assess_wcoord` (the crate's `crates/sim/src/cliffs.rs:240` `block`, "`& 0xfffd \| 1`, read after the call"). Proposed: dead list. |
| 006b1f00 | World::set_mountain_at | layout | no | dead | `set_blocked_at(t, p)`, then the mountain object `mask & ~1 \| 2` on `p`, cleared otherwise. Scan: "pointer only", its one match an RVA inside `.rsrc` (resource bytes, not code). Proposed: dead list. |
| 006b1f80 | World::set_river_at | layout | yes (run1018, run53) | twin | Set arm: tile `\|= 0x800`, cell `\|= 0x1000`; clear arm: tile `&= ~0x800` and the cell's 0x1000 cleared when none of its 16 tiles still has 0x800. Callers `RiverSection::create_trough_edge` (mapgen) and `River::clear_world`/`ScenarioEditor::*` (the clear arm). Twin of the set arm: `crates/sim/src/map_troughs.rs:426` `mark_wet` ("the tile 0x800, the cell 0x1000"), diff-backed by parts 5/6 (`docs/MAPGEN.md:1978`). The clear arm is editor-only and not needed. Agree. |
| 006b2060 | World::set_tree_at | layout | no | dead | Forest surface on (`\|= 0x38`, the cell's `solid` −1) or off (`& ~0x30`, `solid` +1), `& ~8` after. Scan: dead. The forest is written inline by mapgen's own passes. Proposed: dead list. |
| 006b2110 | World::set_gather_edge | layout | no | dead | A tile's `\|= 0x8000` (the gather bit). Scan: dead. `adjust_for_blocking` sets 0x8000 inline (`crates/sim/src/map_values.rs:73`). Proposed: dead list. |
| 006b2390 | WorldData::is_dead_build | layout | no | dead | `flags & 2` (the closed-building bit, `crate::world::cell::CLOSED`, `crates/sim/src/world.rs:381`). Scan: dead. Its one simulation reader, `Leader::produce_building`, tests the bit inline, as the crate does; the item-1721 sweep (`docs/CENSUS.md:841`) checks the read against `cell_data(c).flags`. Proposed: dead list. |
| 006b23c0 | WorldData::is_passable | layout | no | dead | `(flags & 0x70) == 0`. Scan: dead (its live twin is `is_impassable@006b4880`, swept in item 1721). Proposed: dead list. |
| 006b2400 | WorldData::is_tocean_slow | layout | yes | ui | `(tdata.mask & 0x30) == 0x20` (the cell test before it is computed and discarded). Scan: called only from `TerrainOut::tcoord_is_surfline` (twice), the renderer's surf line; lockstep-entered because the renderer runs. `docs/SCOUT.md:388` cites it only as a reading of the tile array's stride. Proposed pattern: every caller in `TerrainOut::` — catches this one. |
| 006b2490 | WorldData::is_enemy_territory | layout | no | ctw | Owner `who` of the cell valid, not `*param_3`, not its team leader, and not mutually allied (`diplos == 2` both ways). Scan: one caller, `ReinforcementArmy::place_army+0xd4`, whose only caller is `ConquestGame::place_reinforcements`. Conquer the World. Census: class `ReinforcementArmy` (CtW); proposed pattern `^(Conquest\|Reinforcement)` on the class of every caller. The skirmish's own enemy-territory test is `crates/sim/src/territory.rs:501` `enemy_territory_by`. |
| 006b2510 | WorldData::get_who2 | layout | no | dead | The cell's `who2` (`+0x10`, signed). Scan: dead; every reader inlines it. Swept in item 1721 against `World::second` (`docs/CENSUS.md:845`). Proposed: dead list. |
| 006b2540 | WorldData::is_seen_wcoord | layout | no | dead | `wcoord_seen[x + y·xs] & ally_mask`, with the reveal-map, observer (`leader_flags & 0x800`, `num_units[0x141]`) and ally-owned-cell arms. Scan: dead. Proposed: dead list. |
| 006b25f0 | WorldData::was_seen_wcoord | layout | yes | ui | `was_seen (+0x14) & ally_mask`, with reveal-map/observer early arms and an allied-owner arm keyed on the owner's per-region shorts (`+0x125e`, `+0x12de`). Scan: called from `MouseOver::describe_border_info` and `TerrainOut::render` (twice) only — the hover text and the renderer; lockstep-entered through the renderer. The emulator sweep of item 2966 (`crates/sim/tests/sweep/world_data_was_seen_wcoord.rs`) is the only crate reference. Proposed pattern: every caller in `TerrainOut::`/`MouseOver::`. |

## Summary

60 functions, one ruling each: **twin 16**, **twin-differs 1**, **absorbed 5**, **replaced 9**,
**dead 14**, **ui 11**, **debug 1**, **editor 1**, **ctw 1**, **unmodelled 1**.

Of the 16 twins, 9 are backed by a diff or an emulator sweep the crate already runs
(`can_pay_cost`, `pay_cost`, `compute_reg_territory`, `compute_val`, `wipe`,
`add_starting_location`, `adjust_for_blocking`, `set_river_at`'s set arm, `finalize_grafting`
through the lockstep), and 7 are one-line predicates compared here by hand (`is_valid` ×3,
`get_goods`, `is_cliff_at`, `is_oil_at`, `is_edge`).

**Proposed census classes** (each a pattern and what it catches here):

- **Dead by image scan** — add to the guard's dead list, on `tools/trace/report.py <exe> refs`
  showing no call, jump or address copy: `World::get_was_seen@006b1ba0`, `set_tocean@006b1c60`,
  `set_waterhalf@006b1c90` (the 0x00 before it is `set_tocean`'s `retl $0xc`), `set_coastal@006b1d10`,
  `set_cliff_at@006b1eb0`, `set_mountain_at@006b1f00` (its only match is in `.rsrc`),
  `set_tree_at@006b2060`, `set_gather_edge@006b2110`, `WorldData::is_dead_build@006b2390`,
  `is_passable@006b23c0`, `get_who2@006b2510`, `is_seen_wcoord@006b2540`. Twelve rows, all
  out-of-line copies of accessors and setters that every caller inlines (`set_orig_coast` is already
  on the list).
- **The selection's type buckets** (ui): short name
  `^(add_type_bucket|get_type_count|get_unique_types|clear_type_buckets)2?$`. Catches the six rows
  here and `clear_type_buckets@006691a0`, `clear_type_buckets2@00669180`. The only outside callers
  are `IFaceGarrison::update_visibility` and `IFaceSelected::update_selected`.
- **`say_name`** (ui): short name `^say_name$`. Catches `TypeData::say_name@006677c0`,
  `TechTypeData::say_name@0066d440`, `ObjectData::say_name@00645310` and the `UnitData` and
  `ConquestLeader` copies.
- **Renderer-only callees** (ui): a function whose every caller (by the image scan) is in
  `TerrainOut::`, `MouseOver::` or an `…Out::draw`. Catches `WorldData::find_slope_normal`,
  `is_tocean_slow` and `was_seen_wcoord`. The last two were entered by the lockstep runs only
  because the renderer runs.
- **Output-class callees** (ui): every caller in a `.*Out$` class. Catches
  `TypeData::get_limiting_resource`, whose only caller is `LeaderOut::warn_resources`.
- **Conquer the World by caller**: every caller's class matches `^(Conquest|Reinforcement)`.
  Catches `WorldData::is_enemy_territory`.

**Proposed replaced group**: **"the data walk"**. These are the `DataWalk` visitors that
serialise state for three uses: save/load (`SaveGame`, `LoadGame`, `WalkDataGame`), the
recorded game's rules block (`RecordGame::read_rules`/`write_rules`), and the sync checksum
(`CheckSums`, `GameLog::say_checksum`, `CommandManager::issue_check_sums`). Pattern
`::walk_(rules_)?data$`. Catches `Types::walk_data`, `Types::walk_rules_data` and
`World::walk_data` here, plus the `*::walk_data` element walks `docs/DECISIONS.md:7759` counts. The
crate keeps an instrument copy of `World::walk_data`'s parts 2, 5 and 6 (`MapGrid::start_part`)
to diff the setup trace. That copy is a test, not a twin. Six other rows are `replaced` under the
existing XML-loaders group: `get_type_index`, `good_key`, `build_key`, `unit_key`, `tech_key`,
`object_key`. `get_type_index` also serves the scenario/trigger VM.

**`unmodelled`, ranked (lockstep-entered first):**

1. **`World::analyze_map@006b58b0`** (entered on every run). The crate has its `land_size`
   (`victory.rs:110`). It does not compute `sea_map`: the harness copies the dump's value
   (`crates/rondata/src/diff/build.rs:228–240`) and `terrain_init.rs:465` stops at
   `Map::make`'s. Also missing: `player_reg`/`resource_reg` and the `Region.flags` 4/8 bits. The
   rows agreed because the harness carries the original's `sea_map`. A sim standing alone (no dump)
   would keep the style file's −1 or 3 where the original computes 0–4. That matters: the AI's
   `> 2` coastal/dock and `< 4` expansion gates read it (`docs/AI.md` §15.8, East Indies 3 → 4 on
   run20). The dump field that shows it: `WORLD` under `[Start Game]` — `sea_map`, `player_reg`,
   `resource_reg` — on any style without `<SEA_MAP>` or with `<SEA_MAP value="3"/>`. The census
   fields `forest_size`…`sea_resources` it also writes are read only by `WorldData::log_data` and
   `TerrainWin::on_redraw`, so they are inert.

**`twin-differs`:** `Type::sum_rules_cost@006680e0`. The loader fills `CombatType::cost` with a
literal `* 10` (`crates/rondata/src/load.rs:1433`, `:1654`), not with `Tuning::unit_cost_factor`
or `build_cost_factor`. It agrees on stock data and differs on a mod that edits
`UNIT_COST_FACTOR`/`BUILD_COST_FACTOR`. Seen in `compare_target`'s ranking, so in `UNITDATA`'s
target fields on a modded capture only.

**Stale prose found:** `docs/ECONOMY.md:523` and `:599` say `compute_reg_territory`'s
`Leader::new_rare` call "is not carried". Item 1555 built it (`crates/sim/src/rares.rs:440–465`
`claim_cell_goods`).

## Not established

- **`Type::sum_cost@00668080`'s callers.** It is vslot `+0xcc` of `Type` and the Tech, Spell and
  Bonus type vtables. The image holds those vtable pointers and one RVA (inside
  `GameReportWin::on_button_clicked`, unconfirmed as a reference), and no direct call. A virtual
  call through `+0xcc` on a type receiver was not found among the `+0xcc` sites read: all of
  those are `ObjectData`/`UnitData` receivers. It is ruled `dead` because no lockstep run entered
  it and no caller was located. It may be a tooltip's (`ui`) caller through a type pointer the
  grep did not reach.
- **Who reads `Region.flags & 4` and `& 8`**, which `analyze_map` sets. If a simulation reader
  exists (the AI's region choice is the likely one), its effect is unmodelled too.
- **`analyze_map`'s presence test.** It reads `LeaderData +0x1260 + 2·region` per leader, a
  per-region short. Its name was not settled from `types.txt`.
- **`World::set_coastal` and `set_waterhalf`'s exact clear arms.** The decompile drops an `else`.
  Both are dead by the scan, so the listing was not read.
- **`compute_reg_territory` was not read whole** (28 KB). The twin ruling rests on
  `docs/ATTRITION.md`'s diff and the crate's line citations, not on a fresh comparison of every
  arm.
