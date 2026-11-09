# Territory reading — group `world-1` (2026-11-06)

A territory reader's rulings on the 60 functions of the `players`, `regions`, `tribes` and `type`
stems (`/Users/rf-studio/.claude/jobs/d388900a/tmp/t71/groups/group-world-1.tsv`), one row per
function, read by name from the decompile export. Reader: Opus 5.5, the seventy-first tranche's
reading fan-out. `entered` is the TSV's `entered_by_lockstep` column (`y` = some lockstep run
entered it, `-` = none). Nothing here is transcribed; only the expression a ruling turns on is quoted.

**The family's shape, read first.** `Player` (size 0x8c, `GameInfo::player[8]` at `game+0x44`) is
the lobby **seat**, not the nation: `flags` (`+0x30`: 1 occupied, 4 shared-leader team play, 0x10
left, 0x40 resigned, 0x80 lost, 0x100 won), `tribe`, `who` (the leader it drives), `play`, `name`,
`platform`, `net_player`. The simulation's nation is `Leader`/`LeaderData`, which this crate carries
as `Sim::defeated`, `nation`, `tech`… The seat's flags are read by the viewer (`Console`'s seat:
`can_see_borders`, `list_rules`), the network (`CommandManager`, `DropControl`) and the team-play
early return in `leave_game`; no rule of a playing skirmish reads them.

## Part 1 — the seat (`players` stem, 14 rows)

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006edc00 | Player::quit | open | y | absorbed | Menu quit: `resign(this,1)`, then semaphore byte 1 bit 0x80 and `flags`, `game->playing = 0` (unless `semaphore[0]&0x10` and no arg) and `GameLog::create_report`. Callers `CommandManager::issue_quit`, `CommandPackage::process_quit` (replaced: command packaging), `UnitBalance::next` (cut, 3404a). Its one rule is the resign, covered by `Sim::resign` (`crates/sim/src/resign.rs:43`) which `rondata::diff::shutdown` (`crates/rondata/src/diff/shutdown.rs:97`) calls on the quit frame; the rest is the match lifecycle and the end report. |
| 006ede80 | Player::drop | open | - | network | A seat dropped by the network: message + notice + sound (`S_ALLY_RESIGNED`/`S_ENEMY_RESIGNED`), then `leave_game(this,7)`. Callers `DropControl::process_drop`, `DropControl::on_player_timed_out` only. Census: proposed class `DropControl` callers / name `^Player::drop$` under a **network seat** pattern `^Player::(drop\|is_(active_)?connection)$`. A skirmish never drops a seat. |
| 006ee010 | Player::leave_game | open | y | twin-differs | Twin: `Sim::resign` (`crates/sim/src/resign.rs:43`), which always runs `Leader::defeat(6,−1,0)`. Original: `flags \|= 0x10`; (a) under `semaphore[0]&4` (shared-leader team play) returns early when another occupied, live seat (`flags&5==5`, no 0x100/0xd0) drives the same `who`; (b) when the leader's `leader_flags&3==3` and `lost_capital_timer` (`LeaderData+0x418`) is nonzero, it is **`defeat(1, captor, 0)`** — captor the holder of its capital from `LeaderData::find_capital` under `elimination==1`, else the leader's own `+0x8` (`who`) — not `defeat(param,−1,0)`. Both agree on every capture on file (one human, never quit while the capital timer ran). The crate has no `lost_capital_timer` (only `CityTally::lost_capital_frame`, `crates/sim/src/city.rs:125`). Would show: `LEADERDATA` `defeated_by` (`+0x10`) and the reason on the frame after a quit by a leader whose capital is lost (`LeadersSync`), and the captor arm of `defeat_by` (units/buildings changing owner). |
| 006ee1d0 | Player::nameWithPlatformSymbol | open | y | ui | The seat's `name`, with a platform glyph appended when `platform == "XboxLive"`. Callers are messages, chat, achievement panes, `LeaderData::get_name`, `MPDropWin`. Proposed census class **`DISPLAY_NAME`**: short name `^(\w*name\w*\|nameWithPlatformSymbol\|sub_nation_name\|lookup_display_name)$` excluding `find_*`; catches this row, `generate_name`, `Tribe::sub_nation_name`, `Tribe::set_pseudonym`, `Tribe::get_name`, `Type::lookup_display_name`, `Type::set_*name`. |
| 006ee3c0 | Player::generate_name | open | y | ui | Gives a computer seat a name: the tribe's name list from XML, pick `(game.info.seed + count of earlier occupied non-shared seats of the same tribe) % n`, falling back to the tribe's name. No RNG draw (the seed is read, not advanced) and no simulation reader of `Player::name`. Callers `Game::init_rules_and_teams`, `ScenarioEditor::new_leader`, `ConquestGame::setup_*`. `DISPLAY_NAME`. |
| 006edcb0 | Player::resign | name | y | twin | Twin `Sim::resign` (`crates/sim/src/resign.rs:43`; its module doc spells `Player::resign@006edcb0`, its fn doc only `Leader::defeat`, hence the census's `name`). Both reach `leave_game(6)` → `Leader::defeat(6,−1,0)`: defeated set, `Armies::leader_defeated`, planes die (`GOLDEN.md` §152.15, run1468b frame 2600). Its own body beyond that is `flags \|= 0x40`, a message, a notice, a sound (ally/enemy by the `leaders+0x74` diplomacy pair), and semaphore byte 1 0x80 for the console's seat: no simulation reader. The `leave_game` arms it skips are under that row. |
| 0043e220 | Player::is_over | layout | y | ui | `flags & 0x100 \|\| flags & 0xd0` (swept: `crates/sim/src/core/flag_reads.rs:49`, no consumer). Read with `console->play`'s seat only: `LeaderData::can_see_borders` (a viewer who is out sees every border unless tournament), `Game::list_rules`, and ~20 `*Out`/`*Win`/`Scene` draw bodies. The simulation's own "out" is `Sim::defeated`. Census: covered by the proposed **`SEAT_STATE`** pattern `^Player::(is_over\|is_active\|end_game\|win\|lose)$`. |
| 0047e0e0 | Player::is_active | layout | - | ui | `flags&1 && !(flags&0x100) && !(flags&0xd0)` (swept, `crates/sim/src/core/flag_reads.rs:208`, no consumer). Callers `Options::exec`, `ReplayWin::on_button_clicked` only. `SEAT_STATE`. |
| 004b71d0 | Player::is_active_connection | layout | - | network | `is_active` and `is_connection`. Callers `CommandManager::check_accept_issue`, `CommandManager::send_local_package` (replaced: the command packaging and manager). Network seat pattern above. |
| 006edb90 | Player::is_connection | layout | - | network | True when `net_player != 0` or `play` is in `drop_control.player_pendings` / `player_drops`. Callers `DropControl::*`, `CommandManager::process_turn/issue_*` (replaced). Network seat pattern above. |
| 006ee110 | Player::end_game | layout | y | ui | For the console's seat only: `scene->gameover_time = graphic_tick`, `Camera::outdate`, `Options::reset`, semaphore byte 1 `\|= 0x80`. Callers `Player::win/lose`, `Leader::defeat`. Graphic tick and camera: not simulation. `SEAT_STATE`. |
| 006ee250 | Player::lose | layout | y | ui | `flags \|= 0x80`, `end_game`, then a lose sound unless resigned (`flags&0x40`). Called from `Leader::defeat` (whose rule is in `Sim::resign` / the defeat path) and `EndGameWin`/`ConquestFinalWin`. The seat bit is read only by `SEAT_STATE` readers and the network. `SEAT_STATE`. |
| 006ee290 | Player::win | layout | y | ui | `flags \|= 0x100`, `end_game`, a win sound for the console's unresigned seat. Called from `Leader::victory`, which the crate stands in for as a seat (`crates/sim/src/territory/victory.rs` `seams`: the game never ends by territory); the seat flag itself has no simulation reader. `SEAT_STATE`. |
| 006ee2d0 | Player::walk_data | layout | - | saveload | The save/load walker: a tag, `flags..tribe`, and when occupied the bytes to `+0x39` and `name`. Proposed census pattern **`WALKER`**: short name `^walk_(rules_)?data$` (`walk_rules_data` checked below); catches every `*::walk_data` in this group (`Player`, `Region`, `Regions`, `Tribes`, `Tribe`) and the two `walk_rules_data` if they prove to be the same walker. Proposed replaced group "save and load". |

## Part 2 — the map's regions (`regions` stem, 11 rows)

`Regions` (`GameAccess::regions`, 128 records of 0x88: land 0..0x3f, sea 0x40..0x7f) is labelled
at map generation and read by the AI and transport (`coast`, `coords`, `borders`, `site`). This
crate's twins live in `crates/sim/src/mapgen/mod.rs` (the labelling) and `crates/sim/src/world/mod.rs` (the
read side).

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0067f800 | Regions::rebuild_coords | open | y | twin | Twin `World::region_coords_strided` (`crates/sim/src/world/mod.rs:2023`). Compared: the original resizes each nonempty record's `coords` to `size`, then sweeps `y` in `0..world+0x4` outer, `x` in `0..world+0x0` inner, appending `(x, y)` to `coords[cell+0x4 region]` (and `Error::report`s on overrun or short count); the crate filters the same row-major order by region and strides it. They agree (the order is `docs/SCOUT.md` §11, diff-backed by the scout walk, East Indies 1373 → 1570, `docs/SYNC.md` §3.20). The crate keeps no list, so the free/realloc and the mismatch report are absorbed. |
| 0067fb90 | Regions::sort_regions | open | y | dead | Ranks land records 0..0x3f and sea records 0x40..0x7e by `size` with a library `sort` and writes each one's position to `Region::rank` (`+0x18`). `rank`'s readers by name are `Region::log_data`, `ConsoleWin::run_cmd`, and the copy `Region::operator=`; an offset grep (`* 0x88 + 0x18`) finds only this writer. No RNG draw. Write-only for a skirmish; the crate holds no rank (`crates/sim/src/mapgen/mod.rs:466` says so). Census: a single-name mark `^Regions::sort_regions$` (a write-only field's writer). |
| 0067fd70 | Regions::set_coastals | open | y | twin | Twin `World::rebuild_coasts` (`crates/sim/src/world/mod.rs:838`) for the **`coast`** mask: for each cell of a land region (`< 0x40`), each `move_x/y[1..8]` neighbour in a sea region (`region − 0x40 < 0x40`) sets `L.coast[S − 0x40]`. Agree, with one reading difference at the map's west edge: the original's neighbour column is `div_3_table[((x+dx)·0x300 + 0x180) >> 8]` (the row is not so mapped), which for `x+dx = −1` reads `div_3_table[−2]`, before the table; the crate bounds-checks `x+dx` directly. The second half writes **`coastal`** (`+0x44`): from each coastal cell, rings 2–3 (`move[9..48]`) in a different land region `L2` set `L.coastal[L2]` and `L2.coastal[L]`; then `finalize_coastals`. The crate carries no `coastal` — see the next row for why that is not a gap. |
| 006805b0 | Region::finalize_coastals | open | y | dead | Closes `coastal` over shared seas: for every land `L2` already in `this.coastal`, any third land `L3` in `L2.coastal` that shares one of the first 63 sea bits with both `this.coast` and `L2.coast` is added both ways, restarting the scan from the lowest index added. Writes only `coastal`. No reader of `Region::coastal` was found outside `log_data`, `walk_data`, the constructor, `operator=` and this writer (grep by name; offset grep `0x88 + 0x50` hits only this body; `Army::find_target`, `go_here`, `is_coast`, `coast_here`, `num_coasts` read `coast`, not `coastal`), so `docs/TRANSPORT.md` §9.1's "`coastal` is `find_target`'s" is not borne out by the export. Census: single-name mark with `sort_regions`. Not settled: an indirect read the greps cannot see (`## Not established`). |
| 0067eff0 | Regions::find_all | name | y | twin | Twin `Regions::find_all` (`crates/sim/src/mapgen/mod.rs:503`), with `Regions::find` and the land/sea bins `LAND_BIN`/`SEA_BIN` (`:462`): y-outer scan, adoption from the first `move[1..8]` neighbour of the same wetness (`is_ocean`, label not 0/0x7f), land from the first free record from 1 clamped to 0x3e, sea pre-incremented from 0x40 clamped to 0x7d, eviction into 0x3f/0x7e, then the x-outer `region2` pass for HALFLAND cells. Read by its first screen, its tail (`set_coastals`, `sort_regions`, free `coords`, `rebuild_coords`, `do_all_non_input`) and `docs/MAPGEN.md` §6. Diff-backed: `find_all_relabels_every_start_grid` re-labels 494 of 496 start grids; the two that part are parked 3798b (a `mark_halfwater` row, not this function). The census's `name` is a citation-format artefact: the fn doc spells it in bold. |
| 0067eed0 | Regions::get_num_sea | layout | - | ctw | Counts sea records `0x40..0x7e` with `size != 0` (SIMD-unrolled). Callers `MapConquest::add_additional_players`, `MapConquest::check_sea_map` (Conquer the World) and `ScenarioFuncSet::find_extra_starting_loc` (the scenario trigger set). Census: callers-only-CtW; a proposed class pattern on callers is not expressible in the census, so the single-name mark `^Regions::get_num_sea$`. |
| 0067f7d0 | Regions::fix_all_borders | layout | y | twin | Twin `Sim::fix_borders` (`crates/sim/src/city.rs:983`): zero `Region::borders` (`+0x2c`) on the 64 land records, which the crate turns into its `borders_fixed` flag and the next `check_borders`' zero of `reg_known_rares`. Callers `Leader::gain_tech` (Civic), `Leader::calc_gather` (the gem bit), `Leader::fix_tech_flags`; the crate's call sites are named at `crates/sim/src/city.rs:962` and `crates/sim/src/lib.rs:6823`. Agree (`docs/ECONOMY.md`, `docs/ATTRITION.md` "Territory"). |
| 00680500 | Regions::concat | layout | - | dead | Merges region `a` into `b`: `size[b] += size[a]`, `size[a] = 0`, every cell labelled `a` relabelled `b`. No caller in the export (grep `concat` over `funcs/`: only its own file). Census: proposed **`NO_CALLER`** mark — a function with no caller in the export and no vtable slot. |
| 00680560 | Region::release | layout | - | unmodelled | Zeroes `region` on every cell of the record's `coords` and sets `size` 0. One caller: `World::adjust_for_blocking@006b3230`, on every land record whose `flags & 0x40`, before re-running `find_all` (also reached from `ScenarioRead`/`ScenarioEditor`). The crate's `adjust_for_blocking` (`crates/sim/src/mapgen/map_values.rs`, `RELEASED_REGION` at `:42`) holds no region flags and releases nothing; Great Sahara's 7937 agrees with none released. Not entered by any lockstep trace (map generation runs before the first frame). Would show: the `map.cpp` 7937 checkpoint's cell `region` labels (`rondata::mapgen`, the start dump's `WORLD` region map) on a map whose generator sets `Region.flags 0x40` — the flag's writer is unread. |
| 00680e20 | Region::walk_data | layout | y | saveload | The DataWalk walker: `borders`, the three masks (`flags = 2` on input), `coords`, and — when not a checksum walk — the `border_id` presence, re-registering `Borders::add_border_region` on load. `WALKER`. Entered by run1497 only (a planted-cell capture), which is the walker's checksum or save path, not a rule (`## Not established`). |
| 00681240 | Regions::walk_data | layout | - | saveload | Walks `sea..coords`, the record array and the coordinate list. `WALKER`. |

## Part 3 — the nations (`tribes` stem, 9 rows)

`Tribes` (`tribes.list`, 24 records of 0x5f0) holds each nation's file, names and its `graft`
block (the nation powers that `docs/NATIONS.md` reads from `rules.xml`, walked into the rules
checksum by `Game::walk_rules_data@00589550`, which inlines `Tribe::walk_rules_data`'s two walks per
tribe). No row here carries a nation power; the powers are the data, and their consumers are ported
elsewhere (`docs/NATIONS.md`).

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006efb50 | Tribe::sub_nation_name | open | - | ctw | Rewrites each string of a list, replacing the tribe's `old_name` with the Conquer-the-World tribe's name (`conquest_gamec->ctw_tribes[tribe]+0x18`). Caller `Scene::set_help_type@008be140` (help text; `docs/AI.md` "Interface" rules it so). A CtW pseudonym in help strings. `DISPLAY_NAME`. |
| 006efda0 | Tribe::set_substitute_tribe | open | - | ctw | Sets `file` and re-reads the tribe's XML from it. Called from `ConquestGame::setup_scenario@007999e0` alone (`docs/AI.md` "Conquer the World"). Census: proposed **`CTW_ONLY`** single-name marks for `Tribe::set_substitute_tribe`, `Tribe::set_barbarian`, `Tribes::find`, `Regions::get_num_sea`. |
| 006eff70 | Tribe::set_barbarian | open | - | ctw | `barbarian = 1`, `file` from `conquest_gamec->barbarian_files[n]`, re-reads the XML (fallback file, `Error::report`). Called from `ConquestGame::setup_scenario` alone. `CTW_ONLY`. |
| 006ef2d0 | Tribes::find | layout | - | ctw | Linear name → tribe index over the 24 records (an inlined `String ==`). Callers: `ConquestColonies`, `ConquestDiploOffer`, `ConquestLeader`, `ConquestStyle` and ~30 `ScenarioFuncSet::ctw_*`/scenario verbs; none outside CtW and the scenario trigger set. A skirmish's nation is chosen by index in the lobby. `CTW_ONLY`. |
| 006efb20 | Tribes::walk_data | layout | - | saveload | Tag + `ObjectArray<Tribe>::walk_data`. `WALKER`. |
| 006f0200 | Tribe::set_pseudonym | layout | - | dead | Saves `name` to `old_name` once, then sets `name`. No caller in the export (grep over `funcs/` finds its own file only); the CtW pseudonym is applied by `sub_nation_name`. `NO_CALLER`. |
| 006f0270 | Tribe::walk_data | layout | - | saveload | Walks `tribe..graft`, the graft block, `file`, `name`, `eng_name`, `old_name`. `WALKER`. |
| 006f0530 | Tribe::get_name | layout | y | ui | A copy of `name`. Callers: `LeaderData::say_full_name`, `IFaceSelected::draw_tribe`, `AchieveChartPane`, `SetupWin`, `GameSpyQuick`, `Conquest*` text — display only. `DISPLAY_NAME`. |
| 006f1270 | Tribe::walk_rules_data | layout | - | replaced | The rules walk of one tribe (`tribe..text_substitute`, the graft block). Not called as a function (inlined into `Game::walk_rules_data`, whose callers are `RecordGame::write_rules/read_rules`, `CheckSums::check_all`, `CommandManager::issue_check_sums`, `GameLog::say_checksum`, `Game::dump/restore_rules_data`). Proposed replaced group **"the rules walk"** (`walk_rules_data` on every class: the recorded game's rules section, the lockstep checksum, CtW's rules restore); this crate loads the rules by `rondata` and has no checksum peer. |

## Part 4 — the type base class, predicates (`type` stem, 14 rows)

`TypeData` (vftable `00b43e48`) is the base of every type record; `BonusTypeData`, `TechTypeData`
and `SpellTypeData` take its slots `+0x60..+0x8c` unchanged, while `UnitTypeData` overrides
`research_time`, `type_graft` and `get_kill_value`, `BuildTypeData` `get_kill_value`, and
`ObjectTypeData` `is` (`funcs/<Class>/` holds the overrides). So the base slots below are the
**tech/bonus/spell** behaviour, and `time`, `count_discovered`, `get_cost`, `get_score_value` and
`avail` are every type's. The non-virtual `is_peasant`..`is_merchant` are type-index tests. This
crate's twin of the range tests is `tech::Kind` (`crates/sim/src/tech.rs:100`).

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0042db70 | TypeData::is_peasant | layout | y | twin | `type == 0x32 \|\| type == 0x33` (`BASE_UNITTYPES`, `PEASANTSKOREAN`). Twins `Sim::is_peasant_type` (`crates/sim/src/ai_make.rs:77`) and `Sim::track_civilian_type`'s `Worker::Citizen` arm (`crates/sim/src/lib.rs:6266`), the loader assigning `Worker::Citizen` by the same two ids. Callers `Leader::track_unit_type`, `create_units`, `make_stuff`, `Options::describe`. Agree. |
| 0042db90 | TypeData::is_scholar | layout | y | twin | `type == 0x34 \|\| 0x35`. Twin `track_civilian_type`'s `Worker::Scholar` arm (`crates/sim/src/lib.rs:6282`). Callers `track_unit_type`, `create_units`, `GroupData::count`. Agree. |
| 0042dbb0 | TypeData::is_worker | layout | y | twin | `type ∈ {0x32,0x33,0x34,0x35}`. Twin `Sim::is_worker` (`crates/sim/src/groups/group.rs:3447`, `0x32..=0x35` on the unit's type index — the same test, asked of a unit). Callers `create_units`, `Options::describe`, `StatWin`, `Scene::set_help_type`. Agree. |
| 0042dbd0 | TypeData::is_merchant | layout | y | twin | `type ∈ {0x3d MERCHANT, 0x3e MERCHANTDUTCH, 0x190 FURTRAPPER}`. Twin the `MERCHANTS` table (`crates/sim/src/ai_census.rs:53`, `calc_gather.rs:40`, `army.rs:76`). Callers `ObjectData::train_time`, `create_units`, `StatWin`, `GraphicPieces`. Agree. |
| 004704d0 | TypeData::is_cancel_alliance | layout | y | unmodelled | `0x2a3 < type < 0x2ac` (vslot `+0x5c`). Its one simulation reader is `TypeData::get_cost@00664090:1464`: a cancel-alliance spell's price for a player is `× LeaderData::cost_factor_dow`. That price is paid only by `Leader::action_respond` (`docs/AI.md` §98.6: not modelled). Entered on the lockstep runs through `get_cost` (a price asked, never paid on a capture). Would show: the human's goods `bucket` (`LEADERDATA`, `LeadersSync`) on the frame after an accepted treaty that breaks an alliance. |
| 004704f0 | TypeData::is_declare_war | layout | y | unmodelled | `0x29b < type < 0x2a4` (vslot `+0x58`). Readers: `get_cost:1447` (the declaration's price `× cost_factor_dow`, less `10 ×` a leader table word when not allied, floored at 0) and `LeaderData::cost_factor_dow@006da590` (`(ally + is_dow) × (ages + 1)`). Paid by `Leader::pay_dow`, which a human's declaration takes (`docs/AI.md` §98.6, not modelled; `docs/RUNS.md` recon 3460: leader 0's third bucket −640 on block 912). The crate's `Sim::declare_war` (`crates/sim/src/lib.rs:6487`) sets the war without the price. Would show: `LEADERDATA` goods `bucket` on the frame after a human `process_declare` (`COMMANDMANAGER=1` names it). |
| 004705d0 | TypeData::is_plain_tech_type | layout | y | twin | Tech (`0x220..0x274`) and not age (`0x220..0x226`) and not epoch (`0x227..0x242`), each through its vslot. Twin `Kind::is_plain_tech` (`crates/sim/src/tech.rs:135`: `Final \| Plain \| Gov`, `0x243..0x274`). Agree. |
| 004706e0 | TypeData::is_object_type | layout | y | absorbed | Unit (`0x32..0x19d`) or building (`0x19e..0x21e`) or good (`< 0x32`), each through its vslot. No single twin; `tech::Kind`'s `Good`/`Unit`/`Building` variants (`crates/sim/src/tech.rs:100`) answer it at every reader. The crate's `Unit` stops at `0x191` (`0x192..0x19d` carry no shipped unit). |
| 00470820 | TypeData::is_age_type | layout | y | twin | `0x21f < type < 0x227`. Twin `Kind::Age` (`crates/sim/src/tech.rs:109`). Agree. |
| 00470870 | TypeData::is_epoch_type | layout | y | twin | `0x226 < type < 0x243`. Twin `Kind::Epoch` (`crates/sim/src/tech.rs:111`). Agree. |
| 00472a50 | TypeData::type_graft | layout | - | twin | `return type` (the non-unit base; units override at `UnitTypeData::type_graft@0061d790`). Twin `TechTree::get_graft`'s fall-through `Some(t)` for a non-unit (`crates/sim/src/tech.rs:1137`). Agree. |
| 004770d0 | TypeData::avail | layout | - | dead | `type != TYPE_ANY_BUILDING`. Ruled unreachable by two blind readings and recorded as `RESIDUE` (`docs/audit/2026-10-22-residue-3505.md:48`); no named caller in the export (`GoodTypeData::avail_drop`/`avail_ctw` are different functions). |
| 004771c0 | TypeData::is | layout | y | twin | `type == param_1`, the strict flag ignored — the base for techs, bonuses and spells (objects take `ObjectTypeData::is@0065f7d0`, lineage). Twin `TechTree::is` (`crates/sim/src/tech.rs:1141`), whose equality head answers a tech; its `is_list`/`graft`/`from` arms could answer more only if a tech record carried them (`## Not established`). `docs/AI.md` line 6149 states the same exactness. |
| 00663a00 | TypeData::get_kill_value | layout | - | dead | `get_score_value(0)` through vslot `+0x7c`: the base, reached only for a non-unit, non-building type, while kills are of units (`UnitTypeData::get_kill_value@0061d3d0`) and buildings (`BuildTypeData::get_kill_value@00639790`, the crate's `cost::kill_value_by`, `crates/sim/src/cost.rs:1258`, swept, no consumer). Not entered. |

## Part 5 — the type base class, names, times and prices (`type` stem, 12 rows)

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006677f0 | Type::lookup_display_name | open | y | replaced | Type vslot `+0xb0`, called only from `Type::set_name`: looks the type's name up in a strings XML (`XML::init`, `XMLNode::get_element`, `get_text`) and, when absent, **adds** it (`add_element`, `set_attrib`, `set_text`, `XML::save`) — a loader with a string-table maintenance side. Read by its first screen and its call list (10,660 bytes). The display name it returns goes to `Type+0x74`, which the rules walk skips under `checksum` (`Type::walk_rules_data`). The XML loaders group; also `DISPLAY_NAME`. |
| 00668380 | Type::find_preq | open | y | twin | Type vslot `+0xdc`: does any of the type's `num_preq` slots equal `t` — slot 0 as written; slots 1 and 2 through `get_preq`'s lobby remaps inlined (the epoch/age line tests, `starting_technology`/`ending_technology`, `game_rules == 8`'s lower start). Twins: `preqs_of` (`crates/sim/src/ai_load.rs:655`, through `TechTree::get_preq(setup, …)` for `compute_ai_values`'s `depends`) and `Sim::transport_bonus_needs` (`crates/sim/src/ai_research.rs:313`, `research_techs`' `types[0x2ae]->find_preq(t)` on a many-landmass map), which compares slot 0 only. Agree on the shipped data: `TRANSPORT_BONUS`'s slots 1 and 2 are empty, and an empty slot (`< 0`) stays −1 whatever the lobby (`docs/AI.md` §304, item 3515). |
| 00663190 | Type::walk_rules_data | layout | y | replaced | Type vslot `+0xc4`: walks `+0x4..+0x5e` and, unless a checksum walk, the display name `+0x74`. Reached from `Types::walk_rules_data` under `Game::walk_rules_data` (the recorded game's rules, `CheckSums::check_all`, `issue_check_sums`, `GameLog::say_checksum`). Proposed replaced group "the rules walk" (with `Tribe::walk_rules_data`). |
| 006639a0 | TypeData::research_time | layout | y | twin | Spell (`0x275..0x2ab`) → `res_time × 100`; else `(time(who) × research_tick_premium) >> 8`, product wrapping, shift logical. Units override (`UnitTypeData::research_time@0061d760`), so this is the tech's. Twin `Sim::tech_time` (`crates/sim/src/lib.rs:3883`: `job_time × TIME_SCALE` then `production::research_tick`), and `production::research_time_by` (`crates/sim/src/production.rs:376`, swept, both arms). Agree; the spell arm has no consumer in the crate (spells are priced by `do_cast`). |
| 00663ae0 | Type::set_name | layout | y | replaced | Sets `+0x60` (name), `+0xb0` (type name) and `+0x74` from `lookup_display_name`. Callers `UnitType::init`, `BuildType::init`, `TechType::init`, `ItemType::init` — the XML loaders. |
| 00663b60 | Type::set_graph_name | layout | - | dead | `+0x88 = name`. No caller in the export (grep over `funcs/`: its own file only). `NO_CALLER`; the graphics name is the renderer's in any case. |
| 00663b70 | Type::set_type_name | layout | - | dead | `+0xb0 = name`, or `+0x60` when empty. No caller in the export. `NO_CALLER`. |
| 00663b90 | Type::set_help_name | layout | - | dead | `+0x9c = name`. No caller in the export. `NO_CALLER`; help text is UI in any case. |
| 00663f20 | TypeData::time | layout | y | twin | Spell not held (`LeaderData::has_spell`) → `res_time × 100`; else `job_time × 100`. Every type's (no override). Twins: `production::time_by` (`crates/sim/src/production.rs:365`, swept, both arms) and the inlined `job_time × TIME_SCALE` of `Sim::tech_time` (`crates/sim/src/lib.rs:3887`), `production::base_time`, `Sim::disband_time` (`crates/sim/src/queue_disband.rs:152`). Agree on every arm the crate reaches. |
| 00663f80 | TypeData::count_discovered | layout | y | twin | Over the 8 leader slots with `leader_flags & 1`: a good, `TYPE_NONE` or `TYPE_ANY_BUILDING` counts; a unit or building needs `has_preq`, then the type's bit in `tech`. Twin `Sim::held_elsewhere` (`crates/sim/src/lib.rs:3920`), the plain-tech arm only (its sole caller in the crate, `ObjectData::train_time`'s held-elsewhere term), counting `tech[t]` over `players.len().min(8)` slots, defeated seats included. Agree (`docs/GOLDEN.md` §149.9, item 3783: who=1's Despotism 37,700 → 30,000 on 958); also swept (`crates/sim/tests/sweep/type_data_count_discovered.rs`). |
| 00664090 | TypeData::get_cost | layout | y | twin-differs | 68,662 bytes, read by its first screen, its spell tail (`:1440–1470`) and `docs/COSTS.md`, its specification. Twin `cost::cost_of` (`crates/sim/src/cost.rs:638`) with `Sim::price_of` / `price_with` / `tech_price` (`crates/sim/src/lib.rs:2477`, `:3298`, `:3572`) and the arms cited at `lib.rs:2582–3725`; diff-backed on run40/run41/run59 (`docs/COSTS.md` "What is diff-backed"). Differs in one arm beyond `docs/COSTS.md` "What is not established": the diplomacy spells' price for a player (`is_declare_war` × `cost_factor_dow`, less 10 × a leader-pair word when not allied; `is_cancel_alliance` likewise) is not carried — see those rows. |
| 006673b0 | TypeData::get_score_value | layout | y | unmodelled | Σ the type's six base costs (Militia, Minuteman, Partisan priced as the Citizen when `param 0`) × `unit_cost_factor` (with `param 1`, further `× research_premium >> 8 × unittypes+0x2e0 >> 8`), `build_cost_factor`, `tech_cost_factor` or `spell_cost_factor` by range. Callers (vslot `+0x7c`): `Leader::compute_unit_score`/`build_score`/`research_score`/`unit_upgrades_score` and every `get_kill_value` (`Object::take_damage`/`do_damage`: `kill_value / 5` to the killer's score). Entered on the lockstep runs and unmodelled: its effect is dumped — `LEADERDATA` `score` (and its `score_*` parts) — but excluded from the diff (`crates/rondata/src/diff/leader.rs:42`, "no score is kept"). Its rule readers are `Leader::diplomacy`'s top-scorer arm (from frame 40,500, or 27,000 at difficulty ≥ 3) and team style 7 (`crates/sim/src/diplomacy.rs:154`, `:381` SEAMs), and `GameDaemon::process_victory`'s score arms; inert before those frames on every capture on file. |

## Summary

**Counts (60 rows).** twin 18 · twin-differs 2 · absorbed 2 · unmodelled 4 · replaced 4 · dead 9 ·
ui 8 · saveload 5 · ctw 5 · network 3.

**Proposed census classes** (none is in `tools/census.py` today; each is a name pattern):

- **`DISPLAY_NAME`** — short name `^(generate_name|nameWithPlatformSymbol|sub_nation_name|lookup_display_name|get_name|set_pseudonym|set_(graph|type|help)_name)$`
  on the classes `Player|Tribe|Type`: `Player::nameWithPlatformSymbol`, `Player::generate_name`,
  `Tribe::get_name`, `Tribe::sub_nation_name`, `Type::lookup_display_name` (also an XML loader),
  plus the `NO_CALLER` rows `Tribe::set_pseudonym`, `Type::set_graph_name/set_type_name/set_help_name`.
  Scoped by class because a bare `get_name` elsewhere may not be a string.
- **`SEAT_STATE`** — `^Player::(is_over|is_active|end_game|win|lose)$`: the lobby seat's flags, read by
  the viewer and the network only.
- **network seat** — `^Player::(drop|is_(active_)?connection)$`: `drop`, `is_active_connection`,
  `is_connection`.
- **`WALKER`** — short name `^walk_data$`: `Player::walk_data`, `Region::walk_data`,
  `Regions::walk_data`, `Tribes::walk_data`, `Tribe::walk_data`. `DEAD_WEIGHT` already takes `log`,
  `dump`, `print` but not the DataWalk walker.
- **`NO_CALLER`** — a function with no caller in the export and no vtable slot (a mechanical check,
  not a name): `Regions::concat`, `Tribe::set_pseudonym`, `Type::set_graph_name`,
  `Type::set_type_name`, `Type::set_help_name`.
- **`CTW_ONLY`** — single names whose callers are all `Conquest*`/`ScenarioFuncSet`:
  `Tribe::set_substitute_tribe`, `Tribe::set_barbarian`, `Tribes::find`, `Regions::get_num_sea`,
  `Tribe::sub_nation_name`.
- **write-only field** — single names: `Regions::sort_regions` (`Region::rank`),
  `Region::finalize_coastals` (`Region::coastal`).

**Proposed replaced groups** (Ramon's to accept):

- **"the rules walk"** — `walk_rules_data` on every class (`Tribe::walk_rules_data`,
  `Type::walk_rules_data` here): the recorded game's rules section, the lockstep checksum
  (`CheckSums::check_all`, `issue_check_sums`, `GameLog::say_checksum`) and CtW's rules restore.
- **"save and load"** — the `WALKER` rows, if save/load is not taken as `saveload` exclusion.
- The existing **XML loaders** group takes `Type::set_name` and `Type::lookup_display_name`.

**`unmodelled`, ranked** (lockstep-entered first):

1. `TypeData::get_score_value@006673b0` — **entered**. The score: every type's cost sum × the class
   factor, feeding `Leader::compute_*_score` and every kill's `kill_value / 5`. Dumped as
   `LEADERDATA` `score` (and `score_*`), excluded from the diff (`crates/rondata/src/diff/leader.rs:42`).
   Rule readers: `Leader::diplomacy`'s top-scorer arm from frame 40,500 (27,000 at difficulty ≥ 3),
   team style 7, `GameDaemon::process_victory`'s score arms — inert before those frames on every
   capture on file.
2. `TypeData::is_declare_war@004704f0` — **entered** (through `get_cost`, a price asked). The
   declaration's price, paid by a human's `Leader::pay_dow`: `LEADERDATA` goods `bucket` on the frame
   after a human `process_declare` (`COMMANDMANAGER=1`; recon 3460 showed −640).
3. `TypeData::is_cancel_alliance@004704d0` — **entered** (the same way). The cancel-alliance price,
   paid by `Leader::action_respond` on an accepted treaty: the human's `bucket` the frame after.
4. `Region::release@00680560` — not entered (map generation). Zeroes a flagged region's cells before
   `find_all` re-runs in `World::adjust_for_blocking`; would show in the `map.cpp` 7937 checkpoint's
   cell `region` on a map whose generator sets `Region.flags & 0x40`.

**`twin-differs`**: `Player::leave_game@006ee010` (entered) — the crate always defeats with reason 6
and no captor; the original makes a quit during a running `lost_capital_timer` a `defeat(1, captor)`
and skips the defeat in shared-leader team play when another live seat drives the leader
(`LEADERDATA` `defeated_by` the frame after such a quit). `TypeData::get_cost@00664090` (entered) —
the diplomacy-spell arm above.

## Not established

- **`set_coastals`' west edge.** The neighbour column is `div_3_table[((x+dx)·0x300 + 0x180) >> 8]`,
  which at `x+dx = −1` reads `div_3_table[−2]`; whether that word is negative (and so rejected, as the
  crate's bounds check does) was not read from the PE.
- **`Region::coastal` has no reader** by name or by the offset greps tried (`0x88 + 0x50`, `coastal.ptr`);
  an indirect read through a `Region *` with a computed offset would escape both. `docs/TRANSPORT.md`
  §9.1's "`coastal` is `find_target`'s" is contradicted by `find_target`'s body, which calls
  `is_coast` (the `coast` mask).
- **Why run1497 entered `Region::walk_data`** — the walker's checksum or save path; not traced.
- **`TypeData::is` vs `TechTree::is`** agree only while no tech, bonus or spell record carries an
  `is_list`, `graft` or `from`; not checked over the shipped tables.
- **`leave_game`'s non-elimination captor** is the leader's `+0x8`, read as `who` by the other uses of
  `leaders.list[i].field_0x8`; `Leader` (0x6eec) and `LeaderData` (0x6ee4) differ by 8 bytes and the
  base offset was not settled from the type record.
- **The writer of `Region.flags & 0x40`** that `adjust_for_blocking` releases on is unread (as
  `crates/sim/src/mapgen/map_values.rs:38` already says).
- **Which leader field `take_damage`'s `kill_value / 5` lands in** (`local_8[0x20d].field_0x29d8`) was
  not mapped to a `LEADERDATA` key.
