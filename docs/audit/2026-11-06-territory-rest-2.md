# Territory reading — group `rest-2` (2026-11-06)

A territory reader's rulings on the 60 functions of the group `rest-2`
(`/Users/rf-studio/.claude/jobs/d388900a/tmp/t71/groups/group-rest-2.tsv`): the small stems
`gameaccess`, `good`, `herd(s)`, `hero(es)`, `item`, `itemtype`, `lands`, `oilwell(s)`,
`reinforcearmy`, `river`, `riverdoober(s)`, `rivers`, `search`, `signposts`, `sort`, `special(s)`,
`storm`, `supplies`, `supply` and `techtype`. One row per function, read by name from the
decompile export. Reader: Opus 5.5, the seventy-first tranche's reading fan-out. `entered` is the
TSV's `entered_by_lockstep` column (`y` = some lockstep run entered it, `-` = none); a lockstep
trace covers the whole process, so `y` on a render or interface body says only that the viewer ran.
Nothing is transcribed; only the expression a ruling turns on is quoted.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00583480 | GameAccessConst::get_scary_message_win | layout | - | replaced | Returns the global `MiscAccess::message_win`. No static caller in the export (the `GameAccessConst` getters are reached through the scripting VM's accessor table). Spelled at `crates/sim/src/core/flag_reads.rs:104-108` as a `global_cell` word (item 1766). Proposed extension of **the scripting VM** group: pattern `^GameAccessConst::get_scary_` (catches this row, the next two, and the eleven `get_scary_*` getters `flag_reads.rs:90-201` already spells). The object returned is a UI window, so `ui` would catch it as well. |
| 00583490 | GameAccessConst::get_scary_graphic_pieces | layout | - | replaced | Returns the global `MiscAccess::graphic_pieces` (the render piece table). No static caller; `flag_reads.rs:107` spells the cell `0x00c06214`. Same proposed pattern `^GameAccessConst::get_scary_`. |
| 005834a0 | GameAccessConst::get_scary_console_who | layout | - | replaced | `MiscAccess::console->who`: the viewing player's index. No static caller; `flag_reads.rs:110-115` (`console_who`, `+0x298`). The console is the viewer, never a rule's input. Same proposed pattern. |
| 005834b0 | GameAccessConst::find_tcoord_z | layout | y | twin | Forwards to `TerrainOut::find_tcoord_z(div_3_table[x>>6], div_3_table[y>>6])`: a coord's tile height. One static caller, `Leader::compute_site_stats@006cd040:188`, at the cell centre `wx*0x300+0x180`, divided by 25. Twin: `crates/sim/src/ai_sites.rs:518-521` — `(q >> 2) + self.world.tile_z(cc.centre_tile()) / 25`, `World::tile_z` (`crates/sim/src/world/mod.rs:1044`) being the table `World::retile_z` (`world.rs:1203`, the anti-diagonal mean, 0 on ocean) fills from `TerrainOut::find_tcoord_z@008544a0`. Same tile (centre `/0x300` == `div3(>>6)` of `0x180`-offset coords), same `/25`: agree. |
| 005834f0 | GameAccessConst::find_data_z | layout | - | replaced | Forwards to `TerrainOut::find_data_z(x, y)`. No static caller in the export (VM accessor). The underlying sampler has its twin, `World::data_z` (`crates/sim/src/world/mod.rs:1116`), used by `Sim::ground_z` (`fight.rs:2263`); this wrapper itself is the accessor surface. Proposed pattern for the VM's terrain accessors: `^GameAccessConst::find_(tcoord\|data)_z$` — but `find_tcoord_z` has a simulation caller (row above), so only `find_data_z` takes the mark. |
| 0066d850 | Good::inc_time | layout | - | dead | Empty body (`return;`). Swept, no write (`docs/CENSUS.md:877`, item 1735). A good (deposit) has no per-tick work. Census: an empty-body class (`is_dead_weight` on a body that is a bare `return`) would catch it and `Supply::deplete` (below). |
| 0066e4c0 | GoodData::is_seen | layout | y | unmodelled | Vslot `+0x48`: `ever_seen & ally_mask` → 1; else for a kind other than 6/0x1f, a non-negative leader, and (kind 5, or tribe bonus 9 or 4) `WorldData::was_seen` at the half-cell; otherwise `was_really_seen`. Swept only (`crates/sim/tests/sweep/good_data_is_seen.rs`, NATIONS table line 425). Callers: `Leader::calc_gather@006ceee0:88` — **the Porcelain Tower's pass** (`has_wonder(0x215)`), which `crates/sim/src/rares.rs:36` and `docs/ECONOMY.md` name not modelled (PARKED 2600); `Leader::plan_strategy@006b9620:234` (an object scan through vslot `0xb0`, kind range from `obj_base[1]`, not established to include goods); the rest are viewer (`ObjectsOut::sort_goods`, `Console::find_good`). `ever_seen` is left unwritten (`ai_census.rs:77`, AI.md §55.1). Entered because the viewer draws goods; the rule's one sim reader is inert without a Porcelain Tower. Field that would show it: the leader's `rare_owned`/resource rates in `LeadersSync` on a game where a leader holds the Porcelain Tower with an unmanned rare in its territory. |
| 0066e5b0 | GoodData::get_gpiece | layout | y | ui | `type->index(+0x18 → +4) + MiscAccess::graphic_pieces->first_good_piece`: the render piece index of a deposit. Entered because the viewer draws goods. No rule reads a graphic piece. Census: proposed render pattern `::get_gpiece$` (catches this and `ItemData::get_gpiece` below; journals 1824/1851 swept both). |
| 0066e5d0 | Good::walk_data | layout | - | saveload | `DataWalk` of the good: a section label (`int_str_array+0x11544`), the bytes `+0x20..+0x21`, then `SubObject::walk_data`. Serialisation/checksum only. Census class **`walk`** (siblings' proposal, `::walk_(rules_)?data$`), which catches every `walk_data` row of this group. |
| 00741a40 | Herd::walk_data | layout | - | saveload | `DataWalk` over the herd record's bytes `this..+0x1b`. Census `walk`. |
| 00741b50 | Herds::find_herd | layout | - | dead | Linear search of `herds` for the live (`+0x1a & 1`) herd whose `+0x14` equals the argument; -1 if none. No caller in the export (DECISIONS' unreferenced-row scan is the mark that would confirm it in the image). The crate's herd is reached by index (`Unit::herd`, `crates/sim/src/anim.rs:2651`), never searched. |
| 00741cc0 | Herds::process | layout | - | twin | Every 64th frame, herd `(frame>>6) % max(n, 5)`; if it exists and is alive (`+0x1a & 1`), `Herd::process`. Twin: `Sim::process_gaia` gate `frame & 0x3f == 0` (`crates/sim/src/gaia.rs:127`) → `Sim::herd_walk` (`gaia.rs:442`): `idx = (frame >> 6) % count.max(5)`, return when `idx >= count \|\| !alive`. The original's signed `>>6` (bias `frame>>31 & 0x3f`) equals the crate's on non-negative frames. Agree; the two `Herd::process` draws are diffed in the draw stream (`docs/SYNC.md:2047`, ORACLE draws 108–109). Not entered in the TSV only because the trace credits the inlined call site, not this body. |
| 00741d10 | Herds::start_herds | layout | - | editor | Calls `Herd::create_units` on every live herd. One caller: `ScenarioEditor::load_file@0099f450:35`. A skirmish founds its herds elsewhere (the founding is `herd_create_view.rs`'s `Herd::create_units` view, swept and unwired; the harness seeds herds from the dump, `crates/rondata/src/diff/setup.rs:1055-1064`). Census: proposed class "sole caller is `ScenarioEditor::`" (an `is_cut` on callers rather than names). |
| 00741d50 | Herds::walk_data | layout | - | saveload | Section label, then `PtrArray<Herd>::walk_data`. Census `walk`. |
| 00739e20 | Hero::walk_data | layout | - | saveload | `Array<ActiveSpell>::walk_data` of the hero's spells, then bytes `+0x24..+0x2a`. A `Hero` is a skirmish general's record (`Heroes::init_hero` at `Unit::init`, `cast.rs:1426`), not Conquer the World's; this body is still only its serialisation. Census `walk`. |
| 00739e50 | HeroData::get_radius | layout | y | twin | A general's aura radius in tiles: `general_radius × (get_general_upgrade + 3) / 2`; Parmenio (`is(0x168,0)`) `× parmenio_radius_adjust >> 8` (signed, toward zero); Wellington (`0x170`) `× wellington_radius / 100`; Kutosov (`0x176`) `× kutosov_radius / 100`; Terra Cotta Army (`has_wonder(0x211)`) `+ terra_cotta_range`; military patriot (`is(0x160\|0x162\|0x164, 1)`) `+ mil_patriot_radius_bonus`; economic patriot (`0x161\|0x163\|0x165`) `+ econ_patriot_radius_bonus`. Twin: `Sim::hero_radius` (`crates/sim/src/cast.rs:1397`) → `supply::general_radius` (`crates/sim/src/supply.rs:107`): same order, same wrapping arithmetic, `/256` ≡ the biased `>>8`. Agree, under two SEAMs the doc comment names: the wonder arm hard-wired false (inert: `TERRA_COTTA_RANGE` ships 0) and `is(t, 1)` read as the type itself (the patriot sets are already the full upgrade lines, so the sets coincide on shipped data). Swept by `hero_radius_view.rs` (item 3396). Not Conquer the World's. |
| 0073a510 | Heroes::walk_data | open | y | saveload | 7.4 KB: a section label, then `PtrArray<Hero>::walk_data` inlined (length, size, the `"Array::walk_data_ptr: write len exceeds size"` guard, `Hero::Hero` on read, `Hero::walk_data` per element). Entered on runs 1016/1497/53/54 by the checksum/save walk. No state produced. Census `walk`. |
| 00677150 | Item::walk_data | layout | - | saveload | Section label, bytes `+0x20..+0x21` (`ever_seen` and its neighbour), `SubObject::walk_data`. Census `walk`. |
| 00677590 | ItemOut::draw_overlays | layout | - | ui | `return 1`: the item's overlay draw (slot `+0x5c`), empty. `docs/GOODY.md:187` (trace, run1094): a box is drawn by `ItemOut::draw`, never this. Census: `is_ui_body` by class (`*Out::draw*`), pattern `Out::draw_` on the class+name. |
| 00677850 | ItemData::is_seen | layout | y | twin | Vslot `+0x48`: `ever_seen & ally_mask` → 1; leader ≥ 0 with `has_tribe_bonus(9)` and the item's cell owned (`WData +0xf`) by `who` → `was_seen` at the half-cell, returned; else `WorldData::is_seen` at the half-cell. Twin: `Sim::goody_item_is_seen` (`crates/sim/src/goody.rs:313`), whose comment lays out the same three arms (the Spanish arm item 3318, the line-of-sight fall-through item 1558, the item's own `ever_seen` byte item 3335). Agree, under the twin's stated assumption that every box sits at its cell centre (half-cell `(2x+1, 2y+1)`), which `ItemTypeData::snap_center` (below) makes true for any box placed through it. |
| 00677970 | ItemData::get_gpiece | layout | y | ui | `first_item_piece − 0x21f + type index`: the render piece. Entered by the viewer. Census: proposed `::get_gpiece$`. |
| 00677df0 | ItemTypeData::snap_center | layout | - | absorbed | Vslot of `TypeData::snap_center`: an item type snaps to the cell centre, `wx × 0x300 + 0x180`, `wy × 0x300 + 0x180`. The crate has no item placement (boxes load from the start dump, `goody.rs`'s SEAM), and `goody_item_is_seen` assumes exactly this centre (`goody.rs:291-292`). The building/good `snap_center` variants are twinned in `city.rs:1474` and `transport.rs:3798`; this item variant is covered by the centre assumption. |

## Part 2

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0067e610 | Lands::find_key | layout | y | replaced | Case-insensitive search of the nine `Land` records (stride `0x138`, `+0x11c` the `key` String) for a name; index or -1. Callers: `Map::init_rivers@0069e520:114` (an `XMLElement::get_attrib` value resolved to a land, which then sets `river_mask` `+0x40`), `TerrainGroups::init_terrain_data@006a6540:163`, `TerrainGroups::init_tileset_data@006a61f0`. A name resolved while a parsed document is walked: **the XML loaders** group (proposed addition by name). The crate carries the land table by index, `LANDS` (`crates/sim/src/world/mod.rs:741`), re-derived from `rules.xml` by `rondata`. Entered on run53/54 at map load. |
| 0067e680 | Land::walk_data | layout | - | saveload | Section label, bytes `this..+0x108`, the `name` and `key` Strings. Census `walk`. |
| 0067e6d0 | LandData::get_amount | layout | y | twin | `make[i] == good` for i in 0..4 → `num_make[i]`, else 0. One caller, `BuildTypeData::blocked_tcoord@00636db0:327` (a flat gatherer's tile with no resource on its land → `0x24`/`9`). Twin: the `land_amount` of `blocked_tcoord_steps`' view (`crates/sim/src/tcoord.rs:359`): `LANDS[land].good.iter().position(== good).map(amount[k])`, else 0. Same search over the four slots, same zero default; called at the same arm (`tcoord.rs:209-213`, `gathers() && flat()`). Agree. (`gather.rs:331`'s `Sim::land_amount` answering 1 is a different reader, the flat-site richness sum, and is a stated SEAM there.) |
| 0067e700 | Lands::walk_data | layout | - | saveload | Section label, `ObjectArray<Land>::walk_data`. Census `walk`. |
| 0073f650 | OilWell::walk_data | layout | - | saveload | `DataWalk` over the oil-well record `this..+0x8`. Census `walk`. |
| 0073f890 | OilWells::walk_data | open | y | saveload | 7.4 KB: section label, then `PtrArray<OilWell>::walk_data` inlined (the same shape as `Heroes::walk_data`: length, size, the write-length guard, construct on read). Entered on runs 1016/1497/53/54 by the checksum/save walk. Census `walk`. |
| 007aa400 | ReinforcementArmy::check_edge | open | - | ctw | Scans one map edge (1..4: a row or column band) for a cell whose region (`WData +4`) is neither leader 0's nor 1's capital-region word and whose owner (`+0xf`) is unowned or `this->who` — a landing edge for a Conquer-the-World reinforcement army. Called by `place_army` alone (`docs/AI.md:39428`). Census: proposed class `^ReinforcementArmy::` (catches both rows; the constructor helpers `Array<ReinforcementArmy>::init`, `ArrayBaseSimpleCopy<ReinforcementArmy>::increase_size` too), or a "sole caller under `ConquestGame::`" mark. |
| 007aa560 | ReinforcementArmy::place_army | open | - | ctw | 14.5 KB, read by its first screen, callers and the names it calls: nine reads of `GameAccess::conquest_game`, `ScenarioData::reinforcements_arrived`, `ScenarioEditor::add_unit`, `WorldData::is_enemy_territory`, `MessageWin::add_message`/`IFaceMainBase::do_notice` — spawns a CtW reinforcement army's units on an edge cell from `check_edge` and announces it. Callers `ConquestGame::place_reinforcements@00798880` (`Game::do_frame:155` under semaphore bit 17, set only while `conquest_game->initialized`, `docs/AI.md:39424-39427`). Proposed class `^ReinforcementArmy::`; proposed replaced group **"Conquer the World"** (CLAUDE.md's v1 cut). |
| 00883950 | River::walk_data | open | - | saveload | Writes/reads whether the river has a spline (`+0x8`); on read pops or clears a `Spline` from the recycler and calls `River::init`, then `SplineData::walk_data`. Serialisation of the river's control spline. Census `walk`. |
| 008858e0 | River::calculate | open | - | twin | One river's build: unless already done (`+0xd1`), on the generator's call (`param_2 == 0`) `RiverOut::ensure_ocean_end`, then the spline's step count (`+0x62` = `length(+0xc) × rivers+0x100`) and order 3 (`+0x60`), `Spline::calc_spline`, `RiverOut::create_sections`, and `Terrain::spot_update_land_batch(+0x284 list, 0, riverbase)`; sets `+0xd1`. Twin: the per-river loop of `map_troughs::calculate_all` (`crates/sim/src/mapgen/map_troughs.rs:661`): `ensure_ocean_end` (line ~684), the clamped cubic B-spline of `length · 1.7` steps, `create_sections`' trough walk, then `p.spot_update_land_batch(&spot, 0, riverbase)` (line 767) — the same five steps in the same order (module doc `map_troughs.rs:1-27`, `docs/MAPGEN.md` §10/§15 step 8). Agree by reading; **no lockstep run entered it** (no traced map laid a river; first entry is the console's `river calc`, run1499, MAPGEN §21), so the twin is reading-backed for this body. Its other callers are the console (`ConsoleWin::run_cmd` case 0x65), `ScenarioEditor::init_terrain`, `RiverEditor::place`, `TerrainMap::river_stop` — editor/debug arms (`param_2 != 0` or semaphore bit `ptr[1] & 8`). |
| 00885b70 | RiverDoober::add_doob_for_render | layout | - | ui | Empty body. A render hook. Census: `UI_BODY`'s `add_for_render` does not match `add_doob_for_render`; proposed widening `add_\w*for_render` (catches this and `RiverDoobers::add_doobers_for_render`). |
| 00885b80 | RiverDoober::add_for_render | layout | - | ui | Under the graphics scalability flag `section_render+6`, `RiverDoobers::add_for_render(this)`. Pure render. `UI_BODY` already matches `^add_for_render` (`tools/census.py:875`); the `layout` mark is taken first, so the exclusion ordering is what leaves it open. |
| 00887760 | RiverDoobers::inc_time | open | y | ui | Per frame over `doobs_for_update`: locate the doober on its river (`RiversOut::get_location`), and on its cadence (`+0x28 & frame`) four `internal_random` draws and an `Underlays::add` (a water-ripple decal); then advances the float `tv_modifier` by `rivers+0x108` mod 1. Callers `Game::loop_render@005917e0`, `Scene::render@008b3270` — the render loop, and `internal_random` is the non-sync stream (`docs/SYNC.md:64`), so no sync draw. Entered because the viewer runs. Census: proposed class `^RiverDoobers?::` (the four rows of the two stems), or `inc_time` callers all under `loop_render`/`Scene::`. |
| 00887700 | RiverDoobers::add_doobers_for_render | layout | - | ui | Empty body. Proposed `add_\w*for_render`. |
| 00437900 | RiversOut::get_name | layout | - | debug | Returns the literal `L"Rivers"`: the subsystem's display name (the profiler/registry label; PARKED 2606 groups the `get_name`s). `is_ui_body` should already catch it by class (`cls.endswith("Out")`); same ordering issue as above. Proposed pattern for the non-`Out` siblings: `::get_name$` returning a literal (catches `Storm::get_name` below). |
| 008890e0 | Rivers::walk_data | layout | - | saveload | On read, `RiverDoobers::close_class`/`init_class`; then `PtrArray<River>::walk_data`. Census `walk`. |
| 0067e600 | Search::walk_data | layout | - | saveload | Empty body: the search object has nothing to serialise. Census `walk` (and an empty body). |
| 00897a50 | Signposts::cur_y | layout | - | editor | Returns the `y_coords` list's cursor value. `Signposts` (a global of three linked lists: label, x, y) is the scenario's named map points: its only users are `ScenarioWrite::save_signposts_chunk@009a5950` and `ScenarioRead::load_signposts_chunk@009a9900`. The scenario and its triggers are CLAUDE.md's v1 cut. Census: proposed class `^Signposts::` (catches all seven rows of this stem). |
| 00897a60 | Signposts::cur_x | layout | - | editor | The `x_coords` cursor value. No caller outside the scenario chunk code. Proposed `^Signposts::`. |
| 00897a70 | Signposts::cur_name | layout | - | editor | Copies the `labels` cursor's String. Caller `ScenarioWrite::save_signposts_chunk` (twice). Proposed `^Signposts::`. |
| 00897ac0 | Signposts::next | layout | - | editor | Advances the three list cursors in step. Caller `save_signposts_chunk`. Proposed `^Signposts::`. |
| 00897b50 | Signposts::head | layout | - | editor | Resets the three cursors to the list heads. Caller `save_signposts_chunk`. Proposed `^Signposts::`. |
| 00897bc0 | Signposts::get_num | layout | - | editor | Returns the label list's count. No caller in the export beyond the chunk code's inlined reads. Proposed `^Signposts::`. |
| 00897e20 | Signposts::add | layout | - | editor | Appends a (label, x, y) triple to the three lists. Reached from the scenario chunk reader (`load_signposts_chunk` adds through the list templates directly) and the editor. Proposed `^Signposts::`; or the replaced group **"save and load / the scenario"**. |
| 0092d5c0 | sort | layout | - | ui | A restart-on-first-swap bubble sort of three parallel int arrays, keyed descending on the first, ties descending on the second (the third carried). One caller, `StatWin::setup_normal_entries@007ecc40:155` — the post-game statistics window's leader rows. Census: proposed "sole caller is an `is_ui_body`/`*Win::` class" mark; by name the free function `sort` is ambiguous (two bodies). |
| 0092d680 | sort | layout | y | absorbed | The two-array form: stable descending sort of `a` (strict `<` swaps only, restart after each), `b` carried. Callers: `Regions::sort_regions@0067fb90` (ranks land regions 0..63 and water 64..126 by `Region::size`, writing `Region::rank` `+0x18`) and `Game::init_teams@0058ae70:358,397` (orders lobby seats by `elo` for team balancing). Neither effect reaches a rule: `Region::rank` is written only by `sort_regions` and read only by `Region::log_data`/`Regions::log_data` and the console (grep of the export: `->rank`/`* 0x88 + 0x18`), and the teams come from the dump's `GAME INFO`. MAPGEN §4 (line 429: "`sort_regions` (rank only) … None of the three writes a cell"). Entered because every game's setup ranks regions. Field that would show a difference if one existed: `REGION`'s `rank` in the `DUMP_ALL` `BEGIN REGIONS` block — printed and **not parsed** (`RegionDump` keeps `region`, `flags`, `size`, `crates/rondata/src/gamelog.rs:3077`). |

## Part 3

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00740030 | Special::walk_data | layout | - | saveload | `Array<ActiveSpell>::walk_data`, then bytes `+0x24..+0x2a` (the same record shape as `Hero`). Census `walk`. |
| 00740060 | SpecialData::get_radius | layout | - | dead | `general_radius × 2` (tiles). No caller in the export and no vtable slot (`vtables.txt` has no `SpecialData::get_radius`): its one use is inlined into `SpecialsData::find_special` as `general_radius × 0x180` (= `× 2 × 0xc0`). Swept (journal 1819). DECISIONS' unreferenced-row scan is the mark. |
| 007403b0 | Specials::walk_data | open | y | saveload | 7.5 KB: section label, then the per-leader `PtrArray<Special>::walk_data` inlined (the `Heroes::walk_data` shape). Entered on runs 1016/1497/53/54 by the checksum/save walk. Census `walk`. |
| 00740070 | SpecialsData::find_special | layout | y | unmodelled | For leader `param_3`, the first live special (`+0x28 & 1`) whose unit is on the map (vslot `+0xbc`), within `vector_dist ≤ general_radius × 0x180` of the point, and `is(0x4b, 0)`; else -1. One caller, `ObjectData::is_jammed@00653660`, which asks it only when the asking unit's leader has `leader_flags & 0x40000`, the unit's type `+0x218 != 2` and `has_objmask(0x80000000)` (ANTI_AIR), for each live non-ally enemy with `leader_flags & 0x20000`. `is_jammed` is the radar-jam half of `Unit::fight`'s ANTI_AIR roll (`JAM_UNIT_RADAR_PROB`); the crate's `Sim::is_jammed` returns `false` (`crates/sim/src/air.rs:4177-4181`, SEAM at `air.rs:4023`: "nothing in this crate sets either"). **Entered by lockstep runs 1016, 1019–1022, 1062**: so on those games both leader flags were set, and the lockstep held because every call returned -1 (no type-`0x4b` special in range) — inert, not undumped: a found jammer swaps `CHAR_ATTACK1` for animation 0, which the unit's animation fields in `UnitsSync`/`UNITDATA` and the shot's timing would show. A capture with an enemy's type-`0x4b` unit standing within `2 × GENERAL_RADIUS` tiles of an ANTI_AIR unit as it fires is the test. Also unported: `PARKED` 2851 (no stub shape for `vector_dist`'s register arguments). |
| 00499c10 | Storm::get_name | layout | - | dead | Returns `String("Storm")`. Nothing constructs a `Storm` (only its two destructors write `Storm::vftable`; journal 2026-11-05-item-3861, `RESIDUE`). Census: proposed class `^Storm(Out)?::` (catches this, `on_field_change`, both `registration`s). |
| 00842050 | Storm::on_field_change | layout | - | dead | `return 0`. Unreachable by the same predicate (item 3861: `RESIDUE`, both readings). Proposed `^Storm(Out)?::`. |
| 0073ae90 | Supplies::walk_data | open | y | saveload | 7.8 KB: section label, then the per-leader `PtrArray<Supply>::walk_data` inlined. Entered on runs 1016/1497/53/54 by the checksum/save walk. Census `walk`. |
| 0073aba0 | Supplies::find_supply | layout | y | twin | Over leader `param_3`'s supply list (count `LeaderData +0x434`): the first slot whose record is live (`+4 & 1`) and whose unit is active (vslot `+8`) and on the map (vslot `+0xbc`), with `vector_dist ≤ SupplyData::get_radius × 0xc0`; else -1. Twin: `SupplyList::find` (`crates/sim/src/supply.rs:228`): slot order, `Wagon::supplies()` = `active && on_map` (`supply.rs:158`), `vector_dist(..) <= radius_tiles × UNITS_PER_TILE` (192 = `0xc0`, `world.rs:58`), first match. The radius hoisted out of the loop is the same number per candidate (every slot is the same leader's). Agree; `docs/SUPPLY.md` §"find_supply" (line 186) and the attrition diffs back it. |
| 0073b540 | Supply::walk_data | layout | - | saveload | `DataWalk` over the record `this..+0x6`. Census `walk`. |
| 0073b560 | SupplyData::get_radius | layout | y | twin | `supply_radius + supply_radius_upgrade × get_supply_upgrade(who)`, `+ terra_cotta_range` under `has_wonder(0x211)`. Twin: `supply::supply_radius` (`crates/sim/src/supply.rs:57`) called by `Network::supplies` (`supply.rs:293-296`) with the upgrade level asked on the frame (`Sim::supply_upgrade_level`, item 3641) and `PlayerSupply::terra_cotta`. Same sum, same order. Agree (`TERRA_COTTA_RANGE` ships 0, so the wonder arm is inert either way). |
| 0073b5d0 | Supply::deplete | layout | - | dead | Empty body; swept, no write (`docs/CENSUS.md:877`, item 1735). Census: an empty-body class (with `Good::inc_time`). |
| 0066cba0 | TechType::set_research | open | y | replaced | The loader for a tech's `WHERE`: `+0x40 = Types::build_key(<text>)`; every building type `is(where, 0)` gets `build_flags \|= 0x8000000`; and the epoch line `+0x14` = 2/1/0 for `(id − 0x227) / 7` = 1/2/3, else 3. Walks a parsed `XMLElement`: **the XML loaders** group (proposed addition by name, `::set_research$`). Both products are carried: the bit by `build::init_derived_flags` (`crates/sim/src/build.rs:333-346`, `RESEARCH_HERE` over the `WHERE`'s lineage; `docs/DATALAYER.md:464`), the line by `tech::Kind::Epoch { line }` (`crates/sim/src/tech.rs:110`, `docs/TECH.md:113-121`). Entered because every game loads the rules. |

## Summary

**Counts (60):** saveload 17 · twin 8 · ui 8 · editor 8 · replaced 6 · dead 6 · unmodelled 2 ·
absorbed 2 · ctw 2 · debug 1. No `twin-differs`. The stems the brief flagged as possibly Conquer
the World's split: `reinforcearmy` is (2 `ctw`); `hero` is **not**: `Hero`/`HeroData` are a
skirmish general's records, and `HeroData::get_radius` is a `twin`; `item` is the goody box
(`twin`, `ui`, `absorbed`, `saveload`), not CtW; `storm` is dead code (nothing constructs a
`Storm`). `gameaccess`: four rows are the accessor surface (`replaced`), one
(`find_tcoord_z`) has an AI caller and a twin.

**Proposed census classes** (pattern → rows of this group it catches):

- **`walk`**, `::walk_(rules_)?data$` (the siblings' proposal) → all 17 `saveload` rows:
  `Good`, `Herd`, `Herds`, `Hero`, `Heroes`, `Item`, `Land`, `Lands`, `OilWell`, `OilWells`,
  `River`, `Rivers`, `Search`, `Special`, `Specials`, `Supplies`, `Supply` `::walk_data`.
- **`^GameAccessConst::get_scary_`** → `get_scary_message_win`, `get_scary_graphic_pieces`,
  `get_scary_console_who` (and the eleven siblings `flag_reads.rs` spells); as an extension of
  the scripting VM group, or as `ui` (the objects returned are viewer state).
- **`::get_gpiece$`** (render piece index) → `GoodData::get_gpiece`, `ItemData::get_gpiece`.
- **`add_\w*for_render`** (a widening of `UI_BODY`'s `add_for_render`) →
  `RiverDoober::add_doob_for_render`, `RiverDoobers::add_doobers_for_render`; and
  `RiverDoober::add_for_render`, which `UI_BODY` already matches but the `layout` mark
  outranks.
- **`^RiverDoobers?::`** (river ripple decoration, render loop, non-sync RNG) → the four
  `riverdoober(s)` rows, `RiverDoobers::inc_time` among them.
- **`^Signposts::`** (scenario named points) → the seven `signposts` rows.
- **`^ReinforcementArmy::`** → both `reinforcearmy` rows (and the `Array<ReinforcementArmy>`
  helpers).
- **`^Storm(Out)?::`** → `Storm::get_name`, `Storm::on_field_change` (and the two
  `registration`s item 3861 already ruled `RESIDUE`).
- **An empty-body class** (a body that is a bare `return;`/`return 0;`, not under `*Out`) →
  `Good::inc_time`, `Supply::deplete`, `Search::walk_data`, `Storm::on_field_change`.
- **"Sole caller under `ScenarioEditor::`"** (a caller-based `is_cut`) → `Herds::start_herds`.
- **A literal-name getter**, `::get_name$` whose body returns a string constant →
  `RiversOut::get_name` (also caught by `*Out`), `Storm::get_name`.
- **The `layout`-before-excluded ordering**: three rows here (`RiverDoober::add_for_render`,
  `ItemOut::draw_overlays`, `RiversOut::get_name`) already match `is_ui_body` and are open only
  because the sweep's `layout` mark is applied first. Worth a look at `tools/census.py`'s order.

**Proposed replaced groups:**

- **Conquer the World** (CLAUDE.md's v1 cut): `ReinforcementArmy::check_edge`, `place_army`.
- **The XML loaders, by name**: add `::set_research$` (`TechType::set_research`) and
  `^Lands::find_key$` (a name resolved while `Map::init_rivers`/`TerrainGroups::init_*` walk a
  parsed document).
- **The scripting VM's accessor surface**: `^GameAccessConst::(get_scary_\w+|find_data_z)$`.
- **Save, load and the scenario**: the `walk` family and `^Signposts::`, if Ramon prefers a
  group to an exclusion.

**`unmodelled`, ranked (both entered by a lockstep run):**

1. **`SpecialsData::find_special@00740070`** — entered on runs 1016, 1019–1022, 1062 through
   `ObjectData::is_jammed`, the radar-jam half of `Unit::fight`'s ANTI_AIR roll; the crate's
   `is_jammed` is `false` (`air.rs:4177`). Entered means both leader flags (`0x40000` on the
   shooter's leader, `0x20000` on an enemy) held on those games; the lockstep held because no
   type-`0x4b` special stood within `2 × GENERAL_RADIUS` tiles — **inert, not undumped**. Field:
   the ANTI_AIR unit's animation (`UnitsSync`/`UNITDATA`: `CHAR_ATTACK1` vs animation 0) and its
   shot timing, on a capture with an enemy type-`0x4b` unit in range as it fires.
2. **`GoodData::is_seen@0066e4c0`** — entered (the viewer draws goods). Its one simulation
   reader is the **Porcelain Tower's pass** at the top of `Leader::calc_gather@006ceee0:84-88`,
   which the crate names not modelled (`rares.rs:36`, `docs/ECONOMY.md`, PARKED 2600), and
   `GoodData::ever_seen` is left unwritten (`ai_census.rs:77`). Inert without a Porcelain Tower.
   Field: the leader's rare income (`rare_owned`, resource rates in `LeadersSync`) on a game
   where a leader holds the Porcelain Tower with an unmanned rare in its territory.

## Not established

- **`Leader::plan_strategy@006b9620:234`** calls vslot `+0x48` (`is_seen`) on objects reached
  through vslot `+0xb0` from `obj_base[1]` onward; whether that walk includes goods (and so
  whether `GoodData::is_seen` has a second simulation reader) was not settled.
- **`Herds::find_herd`, `SpecialData::get_radius`, `Signposts::get_num`** are "no caller in the
  export"; the image scan (`report.py <exe> refs`) that DECISIONS used to prove an unreferenced
  row was not run here.
- **`River::calculate`'s twin** is reading-backed: no traced game laid a river through
  `Terrain::init`, so `map_troughs::calculate_all` has not been diffed against this body's
  output on a lockstep run (MAPGEN §21 entered it only through the console, and its grid did
  not change).
- **The type `0x4b`** that `find_special` requires was not named from the type table.
- **`TechType::set_research`'s `+0x40`** (the `WHERE` type key) is assumed carried by the
  crate's tech table through `rondata`; only the two derived products were checked by name.
