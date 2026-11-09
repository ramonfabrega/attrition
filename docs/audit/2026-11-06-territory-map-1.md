# Territory reading, group `map-1` (2026-11-06)

The seventy-first tranche's reading fan-out: 60 functions of the `map` stem
(the mask, region, continent, river and resource passes of map generation,
plus `MapFairness` and `ResourceDivvyPool`), each read by name in the
decompile export (`~/ghidra-projects/decomp`) and ruled against `crates/sim`.
Reader: Opus 5.5. No other file touched, nothing committed.

**Frame for every ruling.** The simulation does not generate its start map: it
loads the original's `WORLD=6` block and `DUMP_ALL` height grid from the start
dump (`docs/MAPGEN.md` header; `rondata::diff::setup`). A generator exists in
`crates/sim` for four styles only — Great Sahara and Southwest Mesa
(`sim::mapgen`), Great Lakes (`sim::map_lakes`), East Indies
(`sim::map_east_indies`) — and the shared passes behind them (`sim::mapgen`,
`sim::terrain_groups`, `sim::map_resources`, `sim::map_rivers`,
`sim::map_values`, `sim::terrain_init`, `sim::map_heights`, `sim::map_troughs`,
`sim::halfland`, `sim::cliffs`), each pinned by the setup checksum trace
(`docs/MAPGEN.md` §3, §15). So "entered by a lockstep run and unmodelled" in
this group is never a parting on a dumped field: the start dump carries the
function's product into every run. The falsifier for an unmodelled generator
function is the setup checksum line at its checkpoint (`Mapgen` 7807 for a
mask, 7854 for the zero-draw passes, 7927 resources; `docs/MAPGEN.md` §2–§3)
and the start dump's `WORLD=6` cells, on a run of its style.

**`twin` against `twin-differs` here.** `twin` means the crate's function
agrees with the original on every input a ported caller can hand it; an arm
only an unported style reaches is named in the evidence, not counted as a
difference. `twin-differs` means a reachable lobby of a ported style would
part the two.

**The eleven held-out candidates.** `docs/MAPGEN.md` §22 reads eleven
styles' `make_continents` to their lobby and stops at the entry on purpose:
the held-out style (3888c) is not yet named, and the held-out style is
scored, never read against (`CLAUDE.md` phase 3, DECISIONS 54). This report
keeps that fence. The eleven rows, and British Isles beside them, are ruled
from their dispatch, their callers and their callees' names, not from their
bodies; the shared helpers they call (`fill_cont`, `place_start_in_region`,
`find_region_centroid`, `eliminate_edge_canals`) are read in full, since a
helper is not a style.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0068b1e0 | Map::scale_land_area | name | run1016,1018,1019,1021,1062,1176 | twin-differs | `crates/sim/src/mapgen/map_lakes.rs:419` `scale_land_area`. Gate agrees: `get_player_overage@0068b280` returns 0 for `0 < players ≤ allowance` (map size 0/1→2, 2→3, 3→4, 4→6, else 8) and the value passes through; port `players <= allowance`. Past the allowance the original returns `(int)(param_1 / (edx / allowance))` in floats; the port returns `None` and the Great Lakes / East Indies generator refuses (a lobby of 4+ nations on a size-0 map reaches it). `players == 0` takes the original's float path (overage returns the allowance itself) where the port passes the value; no game has 0 nations. Callers: GL and EI `make_continents` (ported) and the unported styles. |
| 0068b890 | Map::eliminate_pools | name | run1018,run53 | twin | `crates/sim/src/mapgen/map_lakes.rs:446` `eliminate_pools`. `ENTIRE_WORLD` arm compared: regions relabelled (`clear_all`, `find_all`), every region > 0x40 collected in first-seen scan order, the smallest by record count (+0x14) picked with strict `<` from `xs*xs` (first of equals), its cells merged into the first neighbour region `< 0x41` found in `move_x/y[1..8]` order, `land`/`land_sub` zeroed (16-bit store at +2), region rewritten; both sides agree. Not ported: the `EDGES_ONLY` arm (candidates only on the outer ring) and `CORNERS_ONLY` (first and last column), called with `EDGES_ONLY` by three unported styles; GL calls `ENTIRE_WORLD` only. |
| 0068e8f0 | Map::check_player_forest | name | run1016..1021 | twin | `crates/sim/src/mapgen/terrain_groups.rs:2041` `check_player_forest`. Compared: disc `circle_radius[0]..circle_radius[ccr/4 − 1]`, skip when ≥ 3 cells carry 0x20; patch sizes 3, 2, 1 from `circle_radius[2]`; grow predicate `flags & 0x30 == 0`, (`0x100` or `land` not 1/2), `flags & 0x48 == 0`, start-city bit clear, `flags & 4 == 0`; orthogonal neighbours joined; on the exact size `flags = (flags & 0xffc3) | 0x20`. Agree; diff-backed on East Indies (item 3828) and Southwest Mesa (3878). The COAST → `land` 2 copy is unreachable (a COAST cell cannot grow), as the port's comment says. |
| 0068ef00 | Map::check_player_land | name | run1018,run53 | twin | `crates/sim/src/mapgen/map_lakes.rs:629` `check_player_land`. Compared on Great Lakes' call (every `region` 0 after `invert_land`): radius `< 0 → 5`, every cell of `circle_radius[0]..circle_radius[r]` round each start-city cell made `land` 0 and region 0; agree, diff-backed at 7807 (item 3815: shrinking the ring parts part 5). Not ported: the adoption arm (a start whose region is 0 adopts the first neighbour region 1..0x3f and moves its cells' coords) and the `ecx ≠ 0` protect test (a disc cell next to another region with > 4 coords is skipped); both need live regions, which only the eight unported callers (Nile Delta, Sea Power, EMW, Colonial, New World, Mediterranean, British Isles, Warring States) hand it. |
| 0068a100 | MapFairness::recompute | layout | - | dead | Argmin/argmax of `fairness[teams[i]]` into `lowest_player`/`highest_player`. `report.py refs`: no call, jump or address copy (byte before 0xcc). The same loop is inlined in `MapWarringStates::make_continents` (decompile lines ~711–717, ~754–760), which is what runs. Census: proposed `UNCALLED` frozenset (see Summary). |
| 0068a150 | MapFairness::unapply_distances | layout | - | dead | `fairness[t] −= dists[t]` per team member. `refs`: dead (byte before is 0xc3, the end of `recompute`'s `ret`). Inlined in Warring States' mask (~line 746). Census: `UNCALLED`. |
| 0068a180 | MapFairness::apply_distances | layout | - | dead | `fairness[t] += dists[t]`. `refs`: dead. Inlined in Warring States' mask (~line 703). Census: `UNCALLED`. |
| 0068a1b0 | MapFairness::point_lowers_highest | layout | - | dead | Returns `lowest_dist == highest_player`. `refs`: dead. Inlined as Warring States' loop break (~line 694, the compare `docs/MAPGEN.md` §18.2 names). Census: `UNCALLED`. |
| 0068a370 | MapFairness::enable_team_mode | layout | - | dead | `teams[i] = i / max(n/k, (n+1)/k)` when `k < n`. `refs`: dead. Inlined in Warring States (~line 531). Census: `UNCALLED`. |
| 0068a440 | ResourceDivvyPool::refresh_water | layout | - | dead | Zeroes `water_bits`. `refs`: dead; no name caller in the export. `get_water` inlines the same clear. Census: `UNCALLED`. |
| 0068a460 | ResourceDivvyPool::refresh_late | layout | - | dead | Zeroes `late_bits`. `refs`: dead. Census: `UNCALLED`. |
| 0068a480 | ResourceDivvyPool::refresh_early | layout | - | dead | Zeroes `early_bits`. `refs`: dead. Census: `UNCALLED`. |
| 0068a4a0 | ResourceDivvyPool::get_water | layout | - | twin | `crates/sim/src/mapgen/map_resources.rs:105` `Pool::get` with `k = 2`. Body identical to `get_early` up to the list (diffed modulo names): draw `Random::get(0,0xffff) % count` only when `count ≥ 2`, retry until the index's bit is clear, set it, clear all bits once every index is taken (no draw); port `c − 1 > 0`, `taken`, clear-on-full; agree. One caller, `place_region_resource+0x24a`; never entered, since no divvy row is water (`site::GET_WATER`'s note). |
| 0068a570 | ResourceDivvyPool::get_late | layout | run1016..1021 | twin | `crates/sim/src/mapgen/map_resources.rs:105` `Pool::get` with `k = 1`; same predicate as `get_water` above; diff-backed draw for draw (`site::GET_LATE`, MAPGEN §8, §15 step 4). |
| 0068a640 | ResourceDivvyPool::get_early | layout | run1016..1021 | twin | `crates/sim/src/mapgen/map_resources.rs:105` `Pool::get` with `k = 0`; same predicate; diff-backed (`site::GET_EARLY`). |
| 0068a700 | ResourceDivvyPool::done_adding_goods | layout | run1016..1021 | absorbed | Sizes the three `DynamicBitMask`s to the lists and zeroes them. One caller, `place_resources+0xa2`. Covered by the port's `Pool` construction (`crates/sim/src/mapgen/map_resources.rs:147–153`: `taken` = `vec![false; len]` per list). |

## Part 2

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0069d000 | Map::grow_valid | name | run1016,1018,1019,1021,1062,1176 | twin | `crates/sim/src/mapgen/map_lakes.rs:128` `Grower::grow_valid`. Compared: `avoid_continent > 0x40` refuses; any other live record 1..63 with the point within `avoid_center` of the centre refuses (no draw); any cell of `circle_radius[0..avoid_continent]` holding a region `≠ 0, ≠ id` refuses (the original's two branches, unchecked well inside and bounds-checked near an edge, give one answer); then per non-zero `avoid_edge` slot one draw, `avoid_modify = clamp(avoid_modify + r%3 − 1, 0, 4)`, edge `(base_edge + slot) & 3` tested `x <`, `y <`, `xs − x ≤`, `ys − y ≤`. Agree; diff-backed (5,568 GL draws on 222 traces, EI on 418). Not ported: the exemption when both records' flags (+8) carry bit 0, which `make_region` copies from `Map::region_flags`; GL and EI never set it (EI's writes are +0, +4, +0xc and `\|= 2`). The `GLOG_MAPMAKE` `Log::say` on refusal is a log. |
| 0069d3f0 | Map::make_region | name | run1016,1018,1019,1021,1062,1176 | twin | `crates/sim/src/mapgen/map_lakes.rs:258` `Grower::make_region`. Compared: the cell's 16-bit store at +2 = 0, region = id, record's first coord, count 1; agree. The port does not hold the record's flags/common/goody factors (+8, +0, +4 from `Map::region_flags`, `common_factor`, `goody_factor`) nor `Regions.land = max(land, id)`: absorbed, since the zero-draw passes' `clear_all`/`find_all` rebuild the records before any reader (diff-backed through 7945 on GL and EI). |
| 0069c600 | Map::grow_region | name | run1016,1018,1019,1021,1062,1176 | twin | `crates/sim/src/mapgen/map_lakes.rs:295` `Grower::grow_region`, as GL and EI call it (centre −1,−1, `param_6` 0). Disc ring `k` (first `k ≥ 3` with `3k² ≥ size/4`), the `scale_by_area(512)` newest-eighth pick, the ring-`k` pick, cross vs `large_stamp` within 64 of size, the `8·size` / `size`-in-a-row stall; diff-backed draw for draw (MAPGEN §5). Not ported: an explicit centre (`(… ,WVar11, local_20, 0)` from two callers) and `param_6 = 1` (two callers), both in unported styles. |
| 0069cb20 | Map::large_stamp | name | run1016,1018,1019,1021,1062,1176 | twin | `crates/sim/src/mapgen/map_lakes.rs:206` `Grower::large_stamp`. Compared the nine shapes' stamp offsets case by case (0: (−1,−1),(−2,−2),(−4,−1),(−4,0),(−5,−1),(−5,−2); 2: (1,0),(1,−2),(1,2),(2,−4),(3,−4),(4,1); 4's two extra side draws; 5–8 two stamps; `& 15` ≥ 9 none); agree; diff-backed. |
| 0069c250 | Map::fix_diag_land | name | run1016..1021 | twin | `crates/sim/src/mapgen/mod.rs:633` `MapGrid::fix_diag_land`. Compared: x outer, y inner; a `land` 0 cell with a `land` 0 corner neighbour (`corner_x/y[1..4]`) whose two flanking cells (the switch's per-corner pairs, which are `(cx, y)` and `(x, cy)`) are on the grid and both non-zero takes the 16-bit store 2 (`land` 2, `land_sub` 0) and stops; agree; diff-backed at 7854 on four styles (MAPGEN §15 step 2). |
| 0069c470 | Map::fix_lakes | name | run1016..1021 | twin | `crates/sim/src/mapgen/mod.rs:659` `MapGrid::fix_lakes`. Compared: return when `ys − 1 < 2`; interior, y outer, x inner; an `is_ocean` cell (0x100 clear, `land` 1 or 2) with more than five of eight neighbours not ocean takes the 16-bit store 0; repeat while a sweep changed a cell; agree. |
| 006947a0 | Map::make_coastlines | name | run1016..1021 | twin | `crates/sim/src/mapgen/mod.rs:691` `MapGrid::make_coastlines`. Compared: `fix_lakes`; y outer x inner, a non-ocean non-COAST cell with an on-grid ocean neighbour lacking COAST takes `& 0xffc3`, `\| 4`, `\| 0x400`; then a COAST cell with no on-grid neighbour that is land-without-COAST takes `& 0xfac3` and `land` 2; agree; diff-backed at 7854. |
| 00694530 | Map::make_riverbeds | name | run1016..1021 | twin | `crates/sim/src/mapgen/map_rivers.rs:175` `make_riverbeds`. Compared: clear `& 0xdffd` (0x2000, 0x2); forest `flags & 0x21 == 0x20`; the 7×7 (`move[1..0x31]`) refusal; `river_thresh ≤` qualifying neighbours; `\| 0x2000`; agree; diff-backed 5012–5118 on four styles (MAPGEN §15 step 8). |
| 00693820 | Map::make_riverlist | name | run1018,run53 | twin | `crates/sim/src/mapgen/map_rivers.rs:491` `make_riverlist`. Compared: the LFSR widths (0xb8 under 251, 0xca0 under 4001 — the listing's `< 0xfa1` — else 0xb400) and the 1..399 path bound (`iVar10 − 1U < 399`); agree; diff-backed on Great Lakes, the only style with a `RIVERLIST`, river draws whole on all 222 traces (`rondata::mapgen` `great_lakes_s_rivers_are_5012_to_5118`). |
| 00692fb0 | Map::make_rivers | name | run1016..1021 | twin | `crates/sim/src/mapgen/map_rivers.rs:643` `make_rivers`: `make_riverbeds`, then each `RIVER` element through `make_riverlist` with lengths scaled by width (cap 399), checkpoints 5012/5014/5076/5116/5118; diff-backed on all four ported styles (§15 step 8). |
| 0068f4f0 | Map::place_resources | name | run1016..1021 | twin | `crates/sim/src/mapgen/map_resources.rs:142` `place_resources`. Compared the ladder: a group change (or group 0) draws `r % 100` and clears `selected`; skip on `roll < 0` unless chance 0 and selected; `roll −= chance`; `numrare` 0 places nothing; pattern 0 → player placer, else region placer; diff-backed draw for draw and at 7927 on four styles (`…_resources_are_the_traced_stream_and_7927`). The function's XML half (`XML::init`, `String::ignore`, `Types::good_key`, `add_good`) is `rondata::mapgen`'s reader (replaced by design, the XML loaders). |
| 0068f380 | Map::find_avail_regions | open | run1016..1021 | twin | `crates/sim/src/mapgen/map_resources.rs:494` `Pass::avail_regions`. Compared: land ids 1..0x40 or water 0x40..0x7f; record count > 5; pattern 3 drops a region holding a start; the list in descending id (`LinkList::add` prepends); cursor `r % count` drawn only for count ≥ 2; agree; diff-backed (`site::FIND_AVAIL`). |
| 00690480 | Map::place_region_resource | open | run1016..1021 | twin | Over 40 KB: read by its callees and the port. `crates/sim/src/mapgen/map_resources.rs:545` `Pass::region` with `region_ok` (`:651`). Callees agree with the port's structure: `get_early/late/water`, one `Random::get` (the LFSR start, `site::REGION_START`), `has_mountain_tcoords`, `has_south_forest`, `Objects::init_good/init_item`; diff-backed draw for draw at 7927 on four styles. Its eleven `Log::say` calls are logs. |
| 00691f70 | Map::place_player_resource | open | run1016..1021 | twin | Over 40 KB: read by its callees and the port. `crates/sim/src/mapgen/map_resources.rs:371` `Pass::player` with `player_ok` (`:426`). One `Random::get` (the ring start, `site::PLAYER_RING`), `get_early/late`, `has_south_forest`, `init_good/init_item`; diff-backed at 7927. |
| 006a0320 | Map::scale_number | open | run1016..1021 | replaced | The XML loaders: it reads `global_cur_element2`'s attribute, `convert_int`, and scales by the word after the space (`AREA`, `SCALE`, `SCALEWATER`, `SCALELAND`). The twin is in the data layer, `crates/rondata/src/mapgen.rs:345` `scale_number`: negative × `mult`, `max(1, (std/2 + n·v)/std)` for `SCALE`/`AREA`, 0 under 1; agrees on those. Differs on `SCALEWATER`/`SCALELAND`: the original's float path over the regions' land/water counts (the `regions->sea` loop at the shown branch); the reader refuses. No stock `mapstyles` file uses either (MAPGEN §4, §11). |

## Part 3

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0068a1c0 | MapFairness::calc_distances | open | - (run1122, a coverage trace) | unmodelled | Warring States' start scorer: per team member `M + m²/(2M)` (or `(m + 2M)/2` past 60000; 0 when M is 0) times `w = 1.0f`, so integer-exact; argmin/argmax into `lowest_dist`/`highest_dist` with strict compares (MAPGEN §18.2). One caller, `MapWarringStates::make_continents` (0x699a8a). No crate counterpart (`grep calc_distances crates` empty); the style is unported. No lockstep run entered it; run1122 (style 15, `cover=1`) did. Would show: the setup checksum's seed and part 2 at 7807 on a Warring States run at `CHECKSUM` detail 1, `check_all_level ≥ 14`, with a start `WORLD` dump (§18.2's lobby). |
| 0068a960 | Map::fill_cont | open | - | unmodelled | Assigns teams to continents for team-layout masks: counts each team's live leaders (`leaders.list[i] & 1`, `get_team`), gives team 8 (no team) one continent per player, then for each pair of continents of equal head count one draw `r & 1` (sign-fixed) swaps their slots in the team → continent map. Draws `game_random`. Callers (`refs`): `MapEastMeetsWest::make_continents+0x22a`, `MapWarringStates::make_continents+0x7e3`; both unported. Would show: the seed and part 2 at 7807 on a style 15 or 19 run with two or more teams. |
| 0068ac00 | Map::place_start_in_region | open | - (run316, British Isles) | unmodelled | Finds a start cell in region `id`: one draw `r % count` for where to begin in the region's coords, then a walk; pass 1 keeps 5..`xs − 6` from the edges, pass 2 3..`xs − 4`; each candidate must be at least `param_4` (pass 2 `min(param_4, 6)`) by the `M + m²/2M` distance from every start already placed, and pass `Map::is_near_ocean`; writes the cell. Callers (`refs`): `MapBritishIsles::make_continents+0x87a`, `MapEastMeetsWest::make_continents+0x9a3`. No crate counterpart. Entered by run316 (`docs/CENSUS.md` §residue), not by a lockstep run. Would show: part 2 (the start arrays) at 7807 on a British Isles (22) or East Meets West (19) setup trace. |
| 0068ae50 | Map::find_region_centroid | open | - (run1125) | unmodelled | The truncating integer mean of a region's coords (`sum x / count`, `sum y / count`, the decompile's 8-way unrolled sum). One caller, `MapEastMeetsWest::make_continents+0x642` (MAPGEN §22.2). No crate counterpart. Would show: part 2 / part 5 at 7807 on a style 19 setup trace. Pure: a candidate for an emulator sweep (`tools/emu/callfn.py`) if the style is ever ported. |
| 0068b360 | Map::eliminate_edge_canals | open | - (run1098,1125,1126) | unmodelled | On each of the four map edges, where three consecutive edge cells are ocean (`land` 2), a land cell one in becomes ocean (16-bit store 2) and is marked, and where the second ring is then likewise ocean-flanked the cell two in follows; it uses `xs` for both axes and 100-byte stack rows (a non-square or > 100 map would misbehave — no stock size reaches it). Callers (`refs`): Nile Delta `+0x444`, Sea Power `+0x591`, East Meets West `+0x6cf` (MAPGEN §22.2). No crate counterpart. Would show: part 5 at 7807 on a style 19/20/21 setup trace. |
| 0068d260 | MapConquest::add_additional_players | open | - | ctw | Adds start locations on a Conquer-the-World map (`Regions::get_num_sea`, one draw, `add_starting_location`, `check_starting_loc`). Callers: `MapScenario::make+0x550` and `MapConquest::make_continents+0x986`, both minted only by `ConquestGame::build_conquest_game` or the editor (MAPGEN §22.3, run1515 did not enter it). Census: proposed class mark `^MapConquest::` (catches this row, `make_continents`, `check_starting_loc`, `check_sea_map`). |
| 0068d7e0 | MapConquest::check_starting_loc | open | - (run1515, a console `!eval`) | ctw | A start-cell predicate: on the grid; optionally no water in the disc to `min(edx, 0x40)`; that far from every edge; at least `ecx` from the starts (all, or one) and from a passed list; optionally no COAST, forest, water or rock/0x40 in the disc (the decompile drops each `return 0`). Callers: `add_additional_players` (twice, CtW) and `ScenarioFuncSet::find_extra_starting_loc` (a scenario script function, the trigger system cut from v1). A skirmish reaches it only through a typed `!eval` (MAPGEN §23). Census: `^MapConquest::`. |
| 0068dba0 | MapConquest::make_continents | open | - | ctw | The Conquer-the-World mask: loads a territory image (`ImageIO::load/save`, `PlayerProfile::get_app_directory`, `SplashScreen::refresh`) and calls the editor's `import_image`; styles 0x18/0x19, minted only by `ConquestGame::build_conquest_game` (MAPGEN §18.1; `blind::RESIDUE`). Census: `^MapConquest::`. |
| 0068e6a0 | Map::force_continent_spacing | open | - | editor | Separates touching continents on an imported image map: an edge cell (`World::is_edge_of_region`) of a land region with a non-water neighbour of another region becomes water (16-bit store 2) and leaves the region's coords. Callers: `ScenarioEditor::import_image` (0x9a4420) and, through it, `MapConquest::make_continents` (MAPGEN §18.1; `blind::RESIDUE`). Census: add to `CUT_FROM_V1`, or a proposed callers-only mark "reached only from `ScenarioEditor::` or `MapConquest::`". |
| 00694ab0 | Map::make_continents | open | - | dead | The base mask: four private-seeded `Fractal`s, water where A < 55, no `game_random` draw. In two vtables only, `Map::vftable` (no `new_map` case leaves it installed) and `MapScenario::vftable`, whose `make` never dispatches +0x40 (MAPGEN §18.1, two readings agree; `blind::RESIDUE`). Census: an explicit name in `CUT_FROM_V1` beside `MapScenario::make`. |
| 00695050 | MapGrass::make_continents | open | - | editor | The blank map: `wipe`, regions cleared and found, starts mirrored by one loop (`docs/MOVEMENT.md` cites `6950cb`, `6950e5`). Style 23 (`BLANK_MAP`); the only caller passing 23 is `ScenarioEditor::generate_map`, and the lobby's list stops at 22 (`docs/CENSUS.md` residue row). `refs`: pointer only (the vtable). Census: `CUT_FROM_V1`. |
| 006a0950 | Map::new_map | open | run1016..1021 | absorbed | The style factory: a style below 6 goes through `set_map`; then a switch allocates the style's class (0x2f0 bytes), runs `Map::Map` and installs the vtable whose +0x40 is that style's mask (MAPGEN §22.1). Covered by the port choosing the style's Rust generator by the dump's `MAP_STYLE` (`rondata::mapgen` calls `great_sahara_make_continents`, `great_lakes_make_continents`, … per file); no object needs building. The cases with no generator are the styles ruled `unmodelled` in Part 4. |
| 006a1340 | Map::set_map | open | run1016..1021 | unmodelled | A random-style lobby: for style 0..5 it draws `r % 17 + 6` until a 19-byte table accepts it (0/3 any non-zero byte, 1/4 land 2, 2/5 water 1), refusing 15, 16, 17 by team style; for style ≥ 6 it returns the style with no draw (listing: `cmp $6, %ecx; jge 6a1503` → `mov %ecx, %eax; ret`). Called by `Game::init_rules_and_teams` (every game; writes `World+0x30`) and `new_map`. **Entered by lockstep and inert**: every captured lobby names a concrete style, so it is the identity and draws nothing. Would show: `GAMEINFO`'s `MAP_STYLE` in the start dump and the seed at the first checkpoint (7805) on a lobby set to a random style. |

## Part 4

The style masks, `+0x40` of each `new_map` class (MAPGEN §22.1). For every
unported style the falsifier is the same take, one setup trace at its own
`--map` (`golden_capture.sh … --map <style> --dump-all-start`, MAPGEN §22.2),
compared on the seed, part 2 and part 5 at 7807. None is a lockstep rule: the
start dump carries the mask into every run, so no `UNITDATA`/`LEADERDATA`
field can part on it.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0069bf70 | MapGreatSahara::make_continents | open | run1017,1020,1022,1024,1177,1178 | twin | `crates/sim/src/mapgen/mod.rs:406` `great_sahara_make_continents`. Compared: `wipe`, `clear_all`, `invert_land`; radius `min(m/2 − 4, (m/2)·80/100)`, `m` the shorter side; step `0xffffffff / players`; angle `(r%0xffff)·0x10000 + r%0xffff`; centre `xs/2 + (r&1)`, `ys/2 + (r&1)`; `start_dist = max(4, xs/16)` at +0x28; first point a step past the angle; agree; diff-backed at 7807 on 105 files (item 3776). |
| 0069bb20 | MapSouthwestMesa::make_continents | open | - (run1493) | twin | `crates/sim/src/mapgen/mod.rs:433` `southwest_mesa_make_continents`, which calls Great Sahara's: the listing is Great Sahara's instruction for instruction (item 3869); diff-backed at 7807 on run1493 (MAPGEN §15 step 1). |
| 00699e40 | MapGreatLakes::make_continents | open | run1018,run53 | twin | `crates/sim/src/mapgen/map_lakes.rs:510` `great_lakes_make_continents`. Compared: first lake `max(90, cells/20)`, `small = max(24, cells/40)`, lake count `xs/12 + (r&1)`, the 1,000-miss shrink and `small·13/16` while over 19 (`0x13 <`), `avoid_continent = max(6, scale_land_area(…))`, then `eliminate_pools(ENTIRE_WORLD)`, `invert_land`, the starts walked in from `3·xs/2` by 3, `check_player_land`; agree; diff-backed 5,568 draws on 222 traces and 7807 on 185 files (item 3815). |
| 00698b60 | MapWarringStates::make_continents | open | - (run1122) | unmodelled | Style 15, a corpus style already read (MAPGEN §18.2, not one of §22's eleven): team continents via `fill_cont`, `MapFairness` scoring of start candidates (`calc_distances` and the inlined `apply`/`unapply`/`recompute`/`enable_team_mode`), `land_dist`, `make_region`/`grow_region` twice, `find_all`, `check_player_land`; six `Random::get` sites. No crate generator. Entered by run1122 (coverage), never by a lockstep run; no setup trace of it is on disk. |
| 00695130 | MapBritishIsles::make_continents | open | - (run316, run1127) | unmodelled | Style 22. Body not read (the fence above). Callees by name: `scale_land_area` ×2, `get_player_overage`, `make_region`, `grow_region` ×2, `eliminate_pools`, `check_player_land`, `place_start_in_region` (`refs` +0x87a), four `Random::get` sites, and a `NetDaemon::process_all` (the network pump inside a mask). No crate generator. Entered by run316 and run1127 (coverage). |
| 0069c0e0 | MapOldWorld::make_continents | open | - (run1117) | unmodelled | Style 6. Body not read (held-out candidate). 2,089 bytes, the size of Great Sahara's 2,104; MAPGEN §5's "reads the same" is unchecked (§22.4). If a listing compare shows Great Sahara's instructions, as item 3869 did for Mesa, this becomes `twin` of `great_sahara_make_continents`. |
| 0069be00 | MapAmazonBasin::make_continents | open | - (run1099) | unmodelled | Style 8. Body not read (held-out candidate). 2,104 bytes, Great Sahara's size exactly; the same listing compare would settle a twin. |
| 0069bc90 | MapHimalayas::make_continents | open | - (run1128) | unmodelled | Style 9. Body not read (held-out candidate). 2,094 bytes; the same listing compare applies. |
| 0069b650 | MapAfricanWateringHole::make_continents | open | - (run1119) | unmodelled | Style 11. Body not read (held-out candidate). A `grow_region` caller (name grep). |
| 0069add0 | MapMediterranean::make_continents | open | - (run1120) | unmodelled | Style 12. Body not read (held-out candidate). Calls `grow_region`, `eliminate_pools`, `check_player_land` (name grep). |
| 0069a650 | MapAustralianOutback::make_continents | open | - (run1121) | unmodelled | Style 13. Body not read (held-out candidate). Calls `grow_region`, `eliminate_pools` (name grep). |
| 006984d0 | MapNewWorld::make_continents | open | - (run1123) | unmodelled | Style 16. Body not read (held-out candidate). Calls `grow_region`, `eliminate_pools`, `check_player_land` (name grep). |
| 00697dd0 | MapColonialPowers::make_continents | open | - (run1124) | unmodelled | Style 17. Body not read (held-out candidate). Calls `grow_region`, `eliminate_pools`, `check_player_land` (name grep). |
| 00696640 | MapEastMeetsWest::make_continents | open | - (run1125) | unmodelled | Style 19. Body not read (held-out candidate). The widest helper set in the group: `fill_cont`, `find_region_centroid`, `eliminate_edge_canals`, `place_start_in_region`, `grow_region`, `eliminate_pools`, `check_player_land` (`refs` and name grep). |
| 00696080 | MapSeaPower::make_continents | open | - (run1126) | unmodelled | Style 20. Body not read (held-out candidate). Calls `eliminate_edge_canals` (+0x591), `grow_region`, `eliminate_pools`, `check_player_land`. |
| 00695bf0 | MapNileDelta::make_continents | open | - (run1098) | unmodelled | Style 21. Body not read (held-out candidate). Calls `eliminate_edge_canals` (+0x444), `grow_region`, `eliminate_pools`, `check_player_land`. |

## Summary

**Counts (60 rows).** twin 23 · twin-differs 1 · absorbed 2 · replaced 1 ·
unmodelled 19 · dead 9 · ctw 3 · editor 2.

Every row whose `entered` column names a lockstep run is a `twin`, `absorbed`,
`replaced` or the one inert `unmodelled` (`set_map`). The 23 twins are the
four ported styles' chain and are diff-backed by the setup checksum trace
(MAPGEN §15); this reading re-compared each one's predicate on both sides and
found them agreeing. It also named what each twin leaves out, which is only
what an unported style reaches.

**Proposed census marks.**
- **`UNCALLED`**, a frozenset beside `CUT_FROM_V1`, or better a mark taken
  from `report.py <exe> refs … ` "dead" output rather than a name: out-of-line
  copies that nothing calls, because the compiler inlined them. It catches 8
  here: `MapFairness::recompute`, `unapply_distances`, `apply_distances`,
  `point_lowers_highest`, `enable_team_mode` (all inlined in
  `MapWarringStates::make_continents`), and
  `ResourceDivvyPool::refresh_water`, `refresh_late`, `refresh_early`. As a
  name regex:
  `^(MapFairness::(recompute|(un)?apply_distances|point_lowers_highest|enable_team_mode)|ResourceDivvyPool::refresh_(water|late|early))$`.
- **`^MapConquest::`** as a class mark, `ctw`. It catches
  `add_additional_players`, `check_starting_loc` and `make_continents`, plus
  `check_sea_map@0068d1a0`, which is outside this group.
- **`CUT_FROM_V1` additions**: `Map::make_continents` (the base mask, which
  no installed vtable dispatches), `MapGrass::make_continents` (style 23,
  the editor's blank map) and `Map::force_continent_spacing` (the editor's
  image import and CtW). All three are already `blind::RESIDUE` rows by
  reason, and no census mark names them.

**Proposed replaced groups.** None new. `Map::scale_number` joins the existing
XML loaders: its twin is the data layer's `rondata::mapgen::scale_number`.
`place_resources`' XML half is the same group's.

**`unmodelled`, ranked.**
1. `Map::set_map@006a1340`: **entered by every lockstep run, and inert** —
   for a concrete style it is the identity with no draw (the listing's
   `jge` to `mov %ecx,%eax; ret`). Its rule is the random-style lobby (styles
   0..5: `r % 17 + 6` against the land/water table, with team-style
   refusals). It would show in `GAMEINFO` `MAP_STYLE` in the start dump, and
   in the seed at 7805, on a lobby set to a random style.
2. `Map::fill_cont@0068a960`: team → continent assignment with `r & 1` swaps
   between equal continents (Warring States, East Meets West). Not entered.
   It would show in the seed and part 2 at 7807 on a style 15/19 team lobby.
3. `Map::place_start_in_region@0068ac00`: start placement inside a region
   (British Isles, East Meets West). Entered by run316 only, which is
   coverage. It would show in part 2 at 7807, style 22 or 19.
4. `Map::eliminate_edge_canals@0068b360`: the edge-water widening (styles
   19, 20, 21). It would show in part 5 at 7807.
5. `MapFairness::calc_distances@0068a1c0`: Warring States' start scorer,
   integer-exact. It would show in the seed and part 2 at 7807, style 15
   (§18.2's lobby).
6. `Map::find_region_centroid@0068ae50`: an integer mean (style 19). It
   would show in part 2 and part 5 at 7807.
7. The thirteen unported masks: Warring States, British Isles and §22's
   eleven. Each would show in the seed, part 2 and part 5 at 7807 on one
   setup trace at its own `--map`. No setup trace of any of them is on
   disk; their entries are coverage traces only.

**Two differences inside ported twins that a port of the next style would hit
first.**
- `scale_land_area` refuses its float path. A **reachable lobby of a ported
  style** takes it: Great Lakes or East Indies with more nations than the
  map size's allowance, for example three nations at map size 0 or 1. It is
  the group's only `twin-differs`.
- `grow_valid` ignores the region-flag bit-0 exemption. `make_region` copies
  that flag from `Map::region_flags`, so it matters for any style whose
  `region_flags` sets bit 0. Neither GL nor EI does.

## Not established

- **The bodies of §22's eleven styles and British Isles were not read**, to
  keep the held-out fence (3888c). Whether Old World (2,089 bytes), Amazon
  Basin (2,104) and Himalayas (2,094) are Great Sahara's listing, which would
  make them `twin`, takes a listing compare like item 3869's. That compare
  is the commander's to book once the held-out style is named.
- `place_region_resource` and `place_player_resource` are over 40 KB. They
  were read by their callees and the port's structure; their `twin` rests on
  the 7927 diff, not on a predicate-by-predicate re-reading.
- `check_player_land`'s protect arm: the decompile drops the statement after
  the `4 < coords` test. The reading that it skips the cell is inferred from
  the control flow, not from the listing.
- `check_starting_loc`: every `return 0` is elided in the decompile, so the
  predicate's polarity is inferred.
- `eliminate_pools`: when a merged region has no neighbour region `< 0x41`,
  the original merges into a stale `EVar10` and the port merges into itself
  (`into = s`). No ported grid reaches that case, and this reading did not
  settle it in the listing.
- The `refs` dead scan cannot see a computed call target or a fall-through
  past a non-padding byte. `unapply_distances`' preceding byte is 0xc3,
  `recompute`'s `ret`.
- `set_map`'s table and team-style arms are reading-only, as MAPGEN §22.4
  already says.
