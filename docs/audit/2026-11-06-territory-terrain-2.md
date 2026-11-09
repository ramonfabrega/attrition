# Territory reading: group `terrain-2` (the `terrain`, `terraingroups`, `terrainmap` stems)

The seventy-first tranche's reading fan-out (READER_BRIEF, 2026-11-06), read on **Opus 5.5**. Sixty
functions: the height grid and half-land (`Terrain::*`), the terrain groups (`TerrainGroup(s)::*`)
and the scenario editor's terrain window (`TerrainMap::*`). Each row is one ruling; `entered`
counts the lockstep runs of the group file that entered the function (`-` none). The decompile is
`~/ghidra-projects/decomp`; crate citations are `crates/sim/src/<file>.rs:<line>` at
`4314ac355`. `docs/MAPGEN.md` §7, §10, §12 and §16 already rule much of this family; a row that
leans on one says so.

## Part 1 — `TerrainMap`, the editor's terrain window

All seventeen are members of `TerrainMap`, which `IFaceEditorTerrainMap` alone builds
(`docs/AI.md` "The scenario editor", `docs/MAPGEN.md` §12, DECISIONS 107 §3). Proposed census
mark for every row: **class `^TerrainMap$`** (with `IFaceEditorTerrainMap`, `ScenarioEditor`,
`RiverEditor` beside it, `ctw`-like `is_cut` by class rather than by name).

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 009b3340 | TerrainMap::remove_cliff | open | - | editor | Erases a cliff under the cursor: `Cliffs::delete_cliff`, then re-enables the window's button; called only by `TerrainMap::remove_feature`. Mark: class `TerrainMap`. |
| 009b3590 | TerrainMap::remove_river | open | - | editor | `River::clear_world` + `River::close` on the river under the cursor; caller `remove_feature` alone. Mark: class `TerrainMap`. |
| 009b37a0 | TerrainMap::remove_mountain | open | - | editor | `Mountains::remove_mountain` and `ScenarioEditor::new_mountain` bookkeeping; caller `remove_feature` alone. Mark: class `TerrainMap`. |
| 009b4da0 | TerrainMap::on_redraw | open | - | editor | The terrain window's paint: camera, `RenderWindow::mouse_to_xy_plane`, fonts; called from `IFaceEditorTerrainMap::on_redraw` (vtable +0xb8). Mark: class `TerrainMap`; `is_ui_body`'s `on_(…paint…)` misses `on_redraw`, so `on_redraw` is a proposed addition to that regex (it would catch every `*::on_redraw`). |
| 009b59f0 | TerrainMap::map_change | open | - | editor | Rebuilds the window's `terrain_type` byte grid from the world (`WorldData::num_waterhalf` per cell); callers `ScenarioEditor::generate_map`, `modify_terrain_map`, `reset_terrain_map`, `load_file`, `add_random_features`. Mark: class `TerrainMap`. |
| 009b5d20 | TerrainMap::river_stop | open | - | editor | Ends an editor-drawn river: `RiversOut::start_river`, spline points, `River::calculate`, `map_change_river`; caller `ScenarioEditor::modify_terrain_map`. Already `RESIDUE` on the editor's cut (DECISIONS 107 §3, PARKED 14125). Mark: class `TerrainMap`. |
| 009b5e00 | TerrainMap::erase_highlight | open | - | editor | Clears the window's highlight cells; caller `ScenarioEditor::modify_terrain_map`. Mark: class `TerrainMap`. |
| 009b5f20 | TerrainMap::synch_mountain_data | open | - | editor | Copies the world's mountains into `ScenarioEditor`'s mountain list (`new_mountain`); same callers as `map_change`. Mark: class `TerrainMap`. |
| 009b32f0 | TerrainMap::river_point | layout | - | editor | One `RiverOut::add_spline_point` on the river being drawn. Mark: class `TerrainMap`. |
| 009b3320 | TerrainMap::start_river | layout | - | editor | `RiversOut::start_river` into `cur_river`. Mark: class `TerrainMap`. |
| 009b39e0 | TerrainMap::remove_feature | layout | - | editor | Dispatch on the window's `terrain_type` byte (3/6 mountain, 0x18 river, 4 cliff) to the three `remove_*`; callers `TerrainMap::draw`, `on_left_down`, `Options::picked_spot`. Mark: class `TerrainMap`. |
| 009b4710 | TerrainMap::map_change_river | layout | - | editor | Writes 0x18 into the window's byte grid on every world cell flagged 0x1000 (river path); caller `river_stop`. Mark: class `TerrainMap`. |
| 009b4780 | TerrainMap::on_mouse_leave | layout | - | ui | Resets the cursor fields and sets `grid_changed`. Already matched by `is_ui_body`'s `on_(mouse…)` regex; listed `layout` only because a sweep test asserts its offsets. |
| 009b4820 | TerrainMap::on_right_up | layout | - | ui | `drawing = 0`. Already matched by `is_ui_body` (`on_right`). |
| 009b4930 | TerrainMap::on_left_up | layout | - | ui | `drawing = 0; stamping = false`. Already matched by `is_ui_body` (`on_left`). |
| 009b5ec0 | TerrainMap::get_terrain_type | layout | - | editor | Reads the window's own `terrain_type` byte at `(x, y)`; no simulation state. Mark: class `TerrainMap`. |
| 009b5ee0 | TerrainMap::cancel_button_enable | layout | - | editor | `Window::set_enabled` on the window's cancel button. Mark: class `TerrainMap`. |

## Part 2 — `Terrain`: half-land, land and sub, goodies, small bodies

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0084d8a0 | Terrain::verify_land | name | 6 | twin | `crates/sim/src/mapgen/halfland.rs:194` `Pass::verify_land`. Compared: depth static `++`, return past 500 without the decrement; `move[1..9]` in bounds; `is_combo_acceptable(l, s, nl, ns) == 0` gates; neighbour land 1/2 → self to `0/COAST_BASE` and recurse on self, out-params updated; else step `ns ± 1` when `|s − ns| > 1` (`ns − 1` when `s <= ns`), written on land 0 only when `ns != step && ns != s`, land 1/2 → neighbour `0/COAST_BASE`; marked neighbours recursed after the loop. Agree. Not held: the `SimpleArray` at `+0x66f0` remove and the render lists (the transition list §19 rules out). |
| 0084fa50 | Terrain::spot_update_land | name | 6 | twin | `halfland.rs:249` `Pass::spot_update_land` (world half). Compared: bounds, `land != l \|\| sub != s` gate, `sub` then `land` each only when `>= 0`, then `verify_land(l, s)`. Agree. The tail (edges reset to 0x38, `mark_trans_flags`, `create_transition_renderables`) is the transition arm `docs/MAPGEN.md` §19 rules out on every grid on file, plus `recalc_builds`. |
| 0084d540 | Terrain::spot_update_land_batch | name | 6 | twin | `halfland.rs:673` `Pass::spot_update_land_batch`. Compared: pass 1 writes every listed cell whose pair differs (`sub`, `land`, negative skipped); pass 2 `verify_land` on every listed cell from the batch's own pair (the listing resets `param_2/3` after each call). Agree. Same render tail as `spot_update_land`. |
| 0084b8e0 | Terrain::convert_beach_to_sandy | name | 6 | twin | `halfland.rs:697` `Pass::convert_beach_to_sandy` (and the predicate-only `terrain_init.rs:440`). Compared: y outer, x inner; `land == 0`; first in-bound `move[1..9]` with `land == 2` (loop `< 0x20` from `move+4`); `spot_update_land(x, y, 1, 0)`; `CoordInfo +0x1c &= 0xdfff`; break. Agree. |
| 0084dd70 | Terrain::check_and_set_coastal | name | 6 | twin | `terrain_init.rs:409` `check_and_set_coastal`. Compared: 16 tiles row outer; wet = `mask & 0x30 == 0x20`; first in-bound non-wet `move[1..9]` (off-map skipped, not stopping); every in-bound wet tile in its `move[1..49]` (`< 0xc0`) gains 0x400. Agree. |
| 0084e610 | Terrain::check_and_set_halfland | name | 6 | twin | `terrain_init.rs:270` (height-free) and `:324` `check_and_set_halfland_heights` with `halfland.rs:730` `Pass::sweeps`. Compared: under `param_1 == 0` a sweep of `_wcoord` then the not-HALFLAND `SANDY/OCEAN` tiles `& 0xffef \| 0x20`; the queue re-swept until empty; `check_and_set_coastal` on every cell; relabel of `land 0, region > 0x40` (region2 nonzero → swap; else first `move[1..25]` (`< 0x60`) land-0 neighbour in a region `< 0x41` not its own); 0x80 tail on `flags & 4 \|\| land == 2` with HALFLAND at `orthog[0..5]`. Agree; the `Regions` lists' `add_unique`/`remove` are not held (no part walks them, `halfland.rs` header). Diff-backed (§10, §16). |
| 0084dec0 | Terrain::check_and_set_halfland_wcoord | open | 6 | twin | `halfland.rs:571` `Pass::wcoord`, with `clear_halfland_wcoord` at `:551`. The gate `CoordInfo & 0x84`, the region scan over `move[1..25]`, the strict `< −30.0f` four-corner test (`WATERLINE`, `halfland.rs:40`), the wet and dry arms: `docs/MAPGEN.md` §16, diff-backed cell-exact on all 185 Great Lakes files (`great_lakes_s_heights_are_the_start_dump_s_grid`, the 720 records). Not re-derived here beyond the gate and the threshold, which agree. |
| 0084ece0 | Terrain::mark_halfwater | name | 6 | twin | `halfland.rs:507` `Pass::mark_halfwater`. Compared: first touch (no 0x100) runs `move_ocean_goody`; tile `& 0xffef \| 0x20`; cell `\|= 0x100`; `num_waterhalf == 16`; regions swapped when `param_5 > 0x40`; a later touch returns unless full, and full → clear 0x100, `region = water`, `region2 = 0`, `land 2`, `verify_land(2, 0)`; first touch → `region/region2`, the three half-land lists appended, `land 0 sub 0`, `verify_land(0, 0)`. Agree. The `Regions` list moves are not held (as above). |
| 0084a3d0 | Terrain::move_goody | name | 2 (run1018, run53) | twin | `halfland.rs:359` `Pass::move_goody` → `move_any(rocky = true)` and `move_good` at `:371`; callers in `crates/sim/src/mapgen/map_troughs.rs:459` and `cliffs.rs:216`. Compared: entry lands read once over `move[0..49]` (−1 off map); edge test `x == 0 \|\| y == 0 \|\| x == xs−1 \|\| y == xs−1` (both against `xs − 1`, as the listing); box arm `move[9..25]` dry land 0 free of `0x100\|0x10\|0x28\|0x800\|0x1000\|0x40\|0x8000`, no good, not blocked, `CoordInfo` without 0x4, no south forest; good arm closes (vtable +0x90) and re-inits at `snap_center`; oil arm `move[9..49]` of the cell's land class, rock bit only when the old cell was dry, `add_rocky_patch` (the returned cell). Agree. |
| 0084aec0 | Terrain::move_ocean_goody | name | 6 | twin-differs | `halfland.rs:349` `Pass::move_ocean_goody` → `move_any(rocky = false)`. Box and oil arms agree as in `move_goody`. **The good arm is not built**: the original's (0x84b1b0..) closes the good and re-inits it on the first `move[9..]` cell whose entry land is **1 or 2** (wet), not HALFLAND, free of `0x10`, `0x68`, `0x1800`, `0x8000`, no good, `blocked` 0, no south forest — a water target, unlike `move_goody`'s dry one; the crate pushes the cell to `Unported::Goody` and refuses by name (`docs/MAPGEN.md` §16 "Not established"). No corpus map refuses. A capture would show it as a start-dump `GOOD` record's `x/y` moved off a cell that turned wet at `Terrain.cpp` 720. |
| 0084fd90 | Terrain::eliminate_dead_mount_spots | name | 6 | twin | `crates/sim/src/mapgen/map_values.rs:165` `eliminate_dead_mount_spots`. Compared: two passes (counter 2), y outer, x inner, one `xs·xs` visited bitmask not cleared between passes; mountain (0x10) cells visit `move[0..9]` once; a cell with no in-bound `orthog[1..5]` neighbour with `flags & 0x30 == 0` blocks every tile without 0x4000 and rewrites flags `& 0xffc3 \| 0x10` per block; land stored back unchanged. Agree. |
| 0084b8d0 | Terrain::convert_errant_sandy | layout | - | dead | A bare `ret` (journal 2026-10-13-item-1866: the emulator sweep agrees, no byte changes); no caller in the export. Proposed mark: an empty-body detector (a body of `return;` alone), which `tools/census.py` could read from the export rather than a name. |
| 0043c2f0 | TerrainOut::get_name | layout | - | ui | Returns the literal `"Terrain"` for the renderer's registration name. Already caught by `is_ui_body` (class ends `Out`); `layout` only because a sweep asserts it. |
| 0049f320 | OceanVars::get_name | layout | - | debug | Returns a string-table name for `OceanVars`, the renderer's tweakable ocean variables (it has a `registration` and lives at `+0x5524` of the renderer's settings). Proposed mark: class regex `Vars$` (`OceanVars::*`; and any other `*Vars` tweak block), or short name `get_name` on a class with a `registration` method. |
| 0084b9e0 | TerrainList::walk_data | layout | - | saveload | Hands every `CoordInfo`'s `flags`/`+0x1e` pair to a `DataWalk` callback (the save/load walker). Proposed mark: short name `^walk_\w*data$\|^walk_roads$` under a proposed **"save and load"** replaced group (callers are `WalkDataGame::walk_data`, `LoadGame::verify_load`, `SaveGame::verify_save`). |
| 0084dd60 | Terrain::build_beach_data | layout | - | dead | One call, `TerrainOil::create_doobers` on `this + 0x4d5c`; no caller in the export (no direct call, no vtable slot in `vtables.txt`). `Terrain::init` calls `create_doobers` itself (738), whose twins are `crates/sim/src/mapgen/map_heights.rs:789` and `halfland.rs:753`. Mark: an export-side "no caller" detector. |
| 00850ef0 | Terrain::walk_coord_data | layout | - | saveload | `DataWalk` over the terrain's `CoordInfo` hash buckets (`+0x47d8`), regenerating the land lists first on input; caller `WalkDataGame::walk_data` alone. Mark: as `TerrainList::walk_data`. |
| 0084bdd0 | TerrainList::delete_items | open | 6 | absorbed | Frees each `CoordInfo` bucket chain and the bucket array: teardown of the renderer-side coordinate hash. The crate holds `CoordInfo.flags` as a plain `Vec<u16>` (`terrain_init.rs:147` `coord_flags`), which needs no free. Mark: short name `delete_items` belongs in `DEAD_WEIGHT` beside `clear`/`reset`. |
| 0084ba10 | TerrainList::add_new_coord_info | open | 6 | twin | `crates/sim/src/mapgen/terrain_init.rs:147` `coord_flags` (one `CoordInfo.flags` per cell). Compared: land read as `NONE` (3) when world flags carry 0x4; `NONE` → 0x8000 with 0x4; HALFLAND or 0x400 → 0x4; `OCEAN` → 0x1000, and 0x80 with a `NONE`/0x4 cell in `move[1..25]`; `BASELAND` → 0x20 with one in `move[1..9]`. Agree with `docs/MAPGEN.md` §10's reading; the decompile prints the 0x4 arm with an empty `else if` body (a misprint the doc read past), and the 734–706 records on 185 Great Lakes files back it. The bucket hash, `FaceArray` and render coordinates are not held. |
| 0084bef0 | CoordInfo::add_geometry | open | 6 | ui | 36,690 bytes, read by its first screen and calls: `TexArray::add`, `RenderHelper::new_data`, `TerrainVis::add_visible_wcoord`, float vertex data; sole caller `TerrainOut::render@008558b0`. The terrain mesh's per-cell geometry. Proposed mark: short name `add_geometry` (or class `CoordInfo` whole, the renderer's per-cell record). Entered by lockstep runs only because the original renders. |
| 00850120 | Terrain::add_weathering | open | 6 | ui | Renderer decals (`TexArray::add`, `TerrainOut::rotate_tex_coords`, float texture coordinates); callers `WallOut::draw_weathering`, `Doober::draw_weathering` (`docs/MAPGEN.md` §12). Proposed mark: short name `\w*weathering$`. |
| 00850d40 | Terrain::mark_forest | open | 6 | unmodelled | After `Terrain::init`, from `Setup::build_game@005ac190:495` alone: for every forest cell (`flags & 0x20`), every tile with `mask & 0x30 == 0x30` sets tile bit **0x8** on twelve in-bound tiles in a band up and to the left of it (offsets from four `.rdata` xmm constants: (−1,0) (−2,−1) (−1,−1) (0,−1) (−3,−2) (−2,−2) (−1,−2) (−4,−3) (−3,−3) (−2,−3) (−4,−4) (−3,−4)). No `crates/sim` function writes it; the map chain stops at 7870 and the start world is loaded from the dump (`crates/rondata/src/diff/setup.rs:370` `world_tiles`), which already carries the bits, so every captured game absorbs it. Shows as the start dump's `WORLD` tile scan, bit 0x8, against a generated grid. Who reads tile bit 0x8 is not established (crate `world.rs` names no such tile bit; `map_values.rs:150` clears it). |
| 00852a60 | Terrain::walk_data | open | - | saveload | `DataWalk` over the height array and lists, and on input rebuilds the renderer (`Surf::init`, `adjust_for_mountains`, `generate_water`, normals, `Cliffs::init` for patch < 4); `docs/MAPGEN.md` §12 calls it the save game. Proposed group **"save and load"** with mark short name `^walk_\w*$` on a `DataWalk` argument. |
| 00852b00 | Terrain::walk_roads | open | - | saveload | Road elements through the walker; callers `LoadGame::verify_load`, `WalkDataGame::walk_data`, `SaveGame::verify_save`. Same proposed group and mark. |
| 00850b60 | Terrain::refresh_good_z | layout | 6 | unmodelled | Every live good and item (`flags & 1`) gets its `z` (`+0xc`, XOR `0x63637`) from `TerrainOut::find_data_z(x, y)`. Callers: `Terrain::init` (894 → 933), `Wall::init@0063e9b0` after a terraform, `Nuke::add_nuke`, `NukeOut::setup_terrain_terraforming`, `ScenarioEditor::remove_invalid_objects`. The crate carries no good or item `z` (`crates/sim/src/cities/city.rs:1576` names the call only to say it is skipped with the farm test). The dump prints it: the per-frame `GOOD` record's `SUBOBJECT` `z_internal`, "printed and not read" (`crates/rondata/src/diff/coverage.rs:68`). Inert on every captured game unless a rule reads a good's `z`; none found here. |
| 00850c40 | Terrain::object_placed | layout | 6 | unmodelled | Under a building footprint (`param_5 != 0`, from `Wall::start@0063e810`), every in-bound cell with `land == 0` goes through `spot_update_land(x, y, 0, weatherbase)`, a `land_sub` rewrite to the tileset's `weatherbase` with `verify_land`'s flood; then `TerrainVis::invalidate_wcoord`. The crate calls it the renderer's (`city.rs:1937`, `docs/ROADS.md` §7.4), true of its height (none moves) but not of `WData.land_sub`, which it writes. Its effect is on file: the 47 start-dump `land_sub` rows reading 2 in start-city areas (`docs/MAPGEN.md` §16, "The value diff at 720"), absorbed by loading the start dump; mid-game it would show as `land_sub` in a `WORLD` scan's cells or checksum part 5. Inert to every rule read so far: §19.4 shows its writes cannot reach the transition arm that blocks tiles. |
| 00850f70 | Terrain::init | layout | 6 | absorbed | 38,772 bytes, read by its first screen, its checkpoints and `docs/MAPGEN.md` §10, which reads it whole. No single twin; the port chain splits it (§15): `crates/sim/src/mapgen/terrain_init.rs:461` `terrain_init_without_heights` (`sea_map`, `CoordInfo`, the height-free half-land), `map_heights.rs:907` `heights`, `halfland.rs` (720), `map_heights.rs:789` `create_doobers` (738), `halfland.rs:697` the beach, `map_rivers`, `map_troughs`, `map_heights.rs:736` `adjust_for_mountains`, `cliffs.rs`. Not covered: the `+0x6460` list batch between 782 and 815 (3809a, no part moves on Great Sahara), `refresh_good_z` (row above), the process-lifetime Fractal cache (`last_map_size`) and the `Prefs` flatten (`docs/MAPGEN.md` §10's quirks; fresh-process captures are unaffected). |

## Part 3 — `TerrainGroup` and `TerrainGroups`, the third phase of map generation

Every row here sits in `crates/sim/src/mapgen/terrain_groups.rs` (or `mapgen.rs`), whose draw stream is
the trace's site for site and word for word on all 280 Great Sahara, 222 Great Lakes and 418 East
Indies traces, with the 7859–7870 checkpoints on 105, 185 and 202 files (`docs/MAPGEN.md` §7). A
`twin` row names the predicate re-checked against the decompile here; the stream is the stronger
evidence.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006a24d0 | TerrainGroup::available_rough_tile | open | 6 | twin | `terrain_groups.rs:463` `Pass::available`. Compared: in bounds; refuse wet (no 0x100 and `land` 1/2); `flags & 0x28`; `is_edge`; 0x10; a start-city cell; `flags & 0x44`; then rings `circle_radius[0]..[r]` of mountain (0x10, `mount_space`), forest (0x20, `forest_space`), rock (0x8, `rock_space`) not in the group's own tiles, and any wet cell in `coast_space`; for kind 8 a `move[1..9]` neighbour with mountain tiles or foreign rock. Agree (the decompile's empty `if` bodies are its `return 0`s). Mutation-held: the coast ring emptied turns Great Lakes red. |
| 006a2d90 | TerrainGroup::place_oil_deposits | open | 6 | twin | `terrain_groups.rs:750` `Pass::place_oil`. Compared: type 6 and `n != 0`; `n` capped at the tile count; tiles in order, skipping one already oil (0x800); the first in-bound `move[1..9]` neighbour without 0x28 and without mountain tiles places oil on the tile. The decompile reads as if every free neighbour placed again; the listing breaks to the outer loop after one `set_oil_at` (0x6a2ee0..0x6a2eee, `incl; cmpl; je` then the checksum line and the next tile). Agree with the listing. Returns `count == n`. |
| 006a9a60 | TerrainGroups::change_forest_base | open | 6 | twin | `terrain_groups.rs:2211` `change_base(…, Base::Forest)`. Compared: x outer, y inner; source `flags & 0x20` takes `land 0`, `land_sub = forest_base` (if ≥ 0); each in-bound `orthog[1..5]` neighbour not forest and `land != 2` draws `r % 100 < forest_growth_prob`. The `forest_base ≥ 0` gate is the caller's (`Map::make@0068bc90`), held in `change_bases`. Agree; stream-backed (370/436/364 draws). |
| 006a9c10 | TerrainGroups::change_mountain_base | open | 6 | twin | Same fn, `Base::Mountain`: source and neighbour exclusion are `WorldData::has_mountain_tcoords` (any of 16 tiles `mask & 3 == 2`), `mountain_growth_prob`. Agree. |
| 006a9dc0 | TerrainGroups::change_coast_base | open | 6 | twin | Same fn, `Base::Coast`: source and exclusion `flags & 4` (COAST), `coast_growth_prob`. Agree; first run on Great Lakes (384 draws from 722 COAST cells), mutation-held. |
| 006a1540 | TerrainGroups::add_doobers | name | 6 | twin | `terrain_groups.rs:1831` `add_doobers`. Compared: forest cells without 0x1000, south then east neighbour not forest, each a `% 100 < bush_clump_prob` roll and a size draw; then non-wet cells without 0x38/0x1000 next to a mountain at `move[8]` then `move[2]`, one rock roll. The writes are `Doober::add_doober`, which is `TexArray::add` (renderer decoration), so the crate keeps the draws and drops the objects: absorbed for the writes. |
| 006a1cc0 | TerrainGroups::treeify_mountains | name | 6 | twin | `terrain_groups.rs:1871` `treeify_mountains`. A cell with mountain tiles but no 0x10 and no mountain south/east and no 0x404 draws `% 100 < prob` to forest (`& 0xffc3 \| 0x20`, `land 0`); a mountain cell with a non-mountain east (else south) neighbour draws for forest-over-mountain. Stream-backed; mutation "a fringe cell's lost FOREST bit" turns 7859 red. Skipped on style 9 by the caller. |
| 006a1f30 | TerrainGroups::nubify_transitions | name | 6 | twin | `terrain_groups.rs:2294` `nubify_transitions`. Columns: a run of five land-0 non-COAST cells at fertility `n` draws `% 3`, raising the west (else east) cell 1..3 back when below `n`; rows (bounded by `xs`, as the original): a run of four draws `& 3`, raising above (else below). Stream-backed (432/708/183 draws). |
| 006a2320 | TerrainGroup::randomize_orthogs | name | 6 | twin | `terrain_groups.rs:776` `Pass::randomize_orthogs`. The original re-seats the circular `orthog_xlist/ylist` at `r % len` when `len > 1`, then `corner_xlist/ylist` likewise; both lists hold four, so `% 4` twice, orthogonal first. Agree. |
| 006a2ab0 | TerrainGroup::drop_tile | name | 6 | twin | `terrain_groups.rs:521` `Pass::drop_tile`. Compared the switch: 4 `available_rough_tile(0x20)` and `vector_dist` to the first tile `> maxd` refuses, COAST → `land 2`, flags `& 0xffc3 \| 0x20`, 16 tiles blocked; 5 `Mountains::add_mountain` (excluding); 6 as 4 with 0x8 and no block; 7 `set_oil_at(1)`; 8 `Cliffs::position_cliff`. Agree. |
| 006a2f60 | TerrainGroup::place_region_group | name | 6 | twin | 38,933 bytes, read by its first screen, its caller (`place_all` alone) and `docs/MAPGEN.md` §7. Twin `terrain_groups.rs:1224` `Pass::place_region_group`: one start draw, the 16-bit Galois LFSR walk (`(s >> 1) ^ 0xB400`), the type-7 wet-disc start test to `circle_radius[rock_space]`. Stream-backed; not re-derived here. |
| 006a4190 | TerrainGroup::place_player_group | name | 6 | twin | 50,751 bytes, read by its first screen, its caller (`place_all` alone) and §7. Twin `terrain_groups.rs:985` `Pass::place_player_group` (with `player_candidate`, `player_place`, `player_grow`, `player_mountain`): ring-start draw, growth draws, the mirror `r & 3`, the cliff arm. Stream-backed; not re-derived here. |
| 006a6f90 | TerrainGroups::fill_fertile | name | 6 | twin | `crates/sim/src/mapgen.rs:730` `MapGrid::fill_fertile`. Compared: x outer, y inner; `land == 0`; count of leading partitions until the fractal byte is below one (`break` on `byte < p`); `land_sub = count`, `land = 0`. Agree. |
| 006a70d0 | TerrainGroups::place_all | name | 6 | twin | 62,740 bytes, read by its first screen, its caller (`Map::make@0068bc90:149`, under `semaphore & 2` clear) and §7. Twin `terrain_groups.rs:1412` `place_all`. Stream-backed on every corpus trace. The pattern-2/3 weight (`clumps > 6 && regions < 6`) never ran with more than one region (3803b); `sim::single` holds it (§11). |
| 006a93b0 | TerrainGroups::nubify_forest | name | 6 | twin | `terrain_groups.rs:2126` `nubify_forest`: columns then rows, a run of four forest-edge cells draws `r & 3` and re-forests the side cell that many back, skipping a start-city cell, ocean and a region edge. Stream-backed (5 GS, 3 GL draws). Known inert difference recorded in §7: keeping COAST/ROCK/MOUNTAIN bits turns nothing red. |
| 006a9f70 | TerrainGroups::fix_transitions | name | 6 | twin | `terrain_groups.rs:2254` `fix_transitions`: `nubify_transitions(3)`, step 3 → 2 over the eight neighbours (not wet unless COAST, not already 3/2), `nubify_transitions(2)`, step 2 → 1, `nubify_transitions(1)`. Stream-backed through 7870. |

## Summary

**Counts by ruling (60):** twin 27 · twin-differs 1 · absorbed 2 · unmodelled 3 · editor 14 ·
ui 6 · saveload 4 · dead 2 · debug 1.

The family is the most ported in the territory: every `TerrainGroup(s)` function and every
`Terrain` half-land, land and goodie function a playing game reaches has a twin, and most are
backed by stream or checkpoint diffs on the whole corpus (`docs/MAPGEN.md` §7, §10, §16). The
`name`/`layout` marks on them mean only that the rubric's spelling test did not match; the
doc comments cite the addresses.

**Proposed census marks** (each a regex or class for `tools/census.py`):
- **class `^TerrainMap$`** (with `IFaceEditorTerrainMap`, `ScenarioEditor`, `RiverEditor`) as
  `is_cut`-by-class: `remove_cliff`, `remove_river`, `remove_mountain`, `on_redraw`,
  `map_change`, `river_stop`, `erase_highlight`, `synch_mountain_data`, `river_point`,
  `start_river`, `remove_feature`, `map_change_river`, `get_terrain_type`,
  `cancel_button_enable` (14). The three `on_*_up`/`on_mouse_leave` already match `is_ui_body`.
- **`is_ui_body` += `on_redraw`**: `TerrainMap::on_redraw` (and every `*::on_redraw`).
- **short name `add_geometry`, or class `CoordInfo`**: `CoordInfo::add_geometry`.
- **short name `\w*weathering$`**: `Terrain::add_weathering`.
- **class `Vars$`**: `OceanVars::get_name`.
- **`DEAD_WEIGHT` += `delete_items`**: `TerrainList::delete_items`.
- **an export-side "empty body or no caller" detector** (a name cannot catch these):
  `Terrain::convert_errant_sandy` (bare `ret`), `Terrain::build_beach_data` (no caller).

**Proposed replaced group: "save and load"** — the `DataWalk` walkers, short name `^walk_\w*$`
taking a `DataWalk *`: `TerrainList::walk_data`, `Terrain::walk_data`, `Terrain::walk_roads`,
`Terrain::walk_coord_data` (callers `WalkDataGame::walk_data`, `LoadGame::verify_load`,
`SaveGame::verify_save`).

**`unmodelled`, ranked (all three entered by lockstep runs 1016–1021):**
1. **`Terrain::refresh_good_z@00850b60`**: each live good's and item's `z` from
   `find_data_z`, at `Terrain::init` 894 → 933 and after every building terraform
   (`Wall::init`). It was entered and never mattered because no field the port compares holds
   it: the per-frame `GOOD` record's `SUBOBJECT` **`z_internal`** is printed and not read
   (`crates/rondata/src/diff/coverage.rs:68`). That key is the falsifier, and it costs a parser
   line, no capture. Inert unless some rule reads a good's `z`.
2. **`Terrain::object_placed@00850c40`**: under each started building, `BASELAND` cells' `land_sub`
   goes to the tileset's `weatherbase`, with `verify_land`'s flood. Entered and inert because the
   start world is loaded from the dump (which carries it: `docs/MAPGEN.md` §16's 47 start-city
   `land_sub` rows reading 2), and no rule read so far reads `land_sub` mid-game (§19.4 closes the
   transition arm). The falsifier is **`WData.land_sub`** in a mid-game `WORLD` scan
   (`diff/world.rs`'s whole-scan blocks), or checksum part 5, after a building starts on a cell
   whose sub is not `weatherbase`.
3. **`Terrain::mark_forest@00850d40`**: tile bit **0x8** on a twelve-tile band up and to the left
   of every forest tile, from `Setup::build_game`. Entered and absorbed by loading the start
   dump's tile masks. The falsifier is the start dump's `WORLD` tile scan, bit 0x8, against a
   generated (not loaded) world. This one matters on the day the port generates its own start.

**`twin-differs` (1):** `Terrain::move_ocean_goody@0084aec0`: the good arm (moving a good to a
**wet** `move[9..]` cell) is refused by name (`Unported::Goody`), never guessed; no corpus map
refuses.

## Not established

- **Who reads tile bit 0x8** (row `mark_forest`). `crates/sim/src/world.rs`'s `tile` module
  names no such bit; a grep of the export for `& 8` on the tile array (`+0x138`) hit cell-flag
  reads (`+0x134`) only, in the files checked (`World::gather_at`,
  `PathFinder::calc_road_cost`, `BuildTypeData::blocked_tcoord`).
- **Whether any rule reads a good's or item's `z`** (row `refresh_good_z`), or `WData.land_sub`
  outside the transitions (row `object_placed`). Both are inert as far as these rows read.
- **`Terrain::build_beach_data` and `Terrain::convert_errant_sandy` callers through a pointer.**
  No direct call or `vtables.txt` slot; a call through a data pointer was not searched (the
  linker map would settle it).
- **`check_and_set_halfland`'s relabel arm when `region2 == 0`**: its `LAB_0084ea46` tail (the
  `Regions` list move) was not read for a `region2` write; the crate writes none
  (`terrain_init.rs:352`). §16's diff passes on Great Lakes, where the arm runs.
- **`place_region_group`, `place_player_group` and `place_all`** were read by their first screen
  and callers only (over 38 KB each); their twins rest on the stream diffs, not on this reading.
- **`move_ocean_goody`'s box arm** predicate was taken from the crate's doc comment and the
  shared `move_any`, not re-derived from the decompile's box loop.
