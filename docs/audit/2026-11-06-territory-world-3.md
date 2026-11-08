# Territory reading: world-3 (2026-11-06)

The seventy-first tranche's reading fan-out, group `world-3`: 54 functions of the
`world` and `worldmap` stems — the world's cell and tile accessors and setters, and the
interface's world map (the minimap) — read by name from the decompile export, one ruling
each, by a territory reader on Opus 5.5. `entered` is the TSV's lockstep column (`-`
none, else the first run). Line numbers cite the tree at `b92475fab`. "No caller" means
neither `Class::name(` nor a bare `name(` appears in any other function of the export
(the decompiler prints a same-class call bare); an accessor with no caller is one the
compiler inlined at every reader, and its out-of-line copy is never called.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006b2b10 | `World::scale_by_area` | layout | - | dead | `(sq/2 + world+0x8 · n) / sq`, `sq` the standard size squared, at least 1. No caller in the export. The live copy is `Map::scale_by_area@006a0910`, twinned at `crates/sim/src/map_lakes.rs:268` (`scale_by_area`), which adds an `n < 1 → 0` guard this copy lacks — so the two differ only on a call nobody makes. The sweep `crates/sim/tests/sweep/world_scale_by_area.rs` holds the formula. Census: proposed mark `is_uncalled` (no caller and no vtable slot in the export) — catches this row, `World::scale_by_axis`, `World::adjust_fog`, `World::set_land`, `WorldData::is_coast`, `WorldData::get_region2` and the other inlined accessors of this group. |
| 006b2b50 | `World::set_land` | layout | - | absorbed | Writes `land_sub` (+3) and `land` (+2) each only when non-negative; with `kind >= 0` clears `flags & 0x3c`, optionally `0x500`, ORs `kind`, and ORs `0x400` when `kind == 4` (COAST). No out-of-line caller: the `0xffc3` store is inlined in `Map::make_coastlines`, `Terrain::check_and_set_halfland_wcoord`, `TerrainGroups::treeify_mountains`, `Mountains::add_mountain` and the scenario editor. The crate writes at each site: `crates/sim/src/halfland.rs:183` `set_land` (the same negative guard, `land_sub` first) and `crates/sim/src/mapgen.rs:705` (`(flags & 0xffc3) \| COAST \| 0x400`). Agree. |
| 006b3020 | `WorldData::is_coast` | layout | - | absorbed | `wdata[y·xs + x].flags & 4`. No caller (`Region::coast_here`'s `is_coast` is `Region::is_coast`). The bit is read inline: `World::land_class` `crates/sim/src/world.rs:1296` (`f & cell::COAST`), `terrain_groups.rs:515`; the sweep (item 1721, `docs/CENSUS.md:839`) drives `cell_data(c).flags & cell::COAST`. |
| 006b3050 | `WorldData::has_mountain_tcoords` | layout | run1016 | twin | Any of the cell's 16 tiles (`4x + (j&3)`, `4y + (j>>2)`) with `mask & 3 == 2`. Callers all map generation (`Map::place_region_resource`, `Map::make_riverbeds`, `TerrainGroup::available_rough_tile`, `place_oil_deposits`, `treeify_mountains`, `change_mountain_base`). Three twins, same index and predicate: `crates/sim/src/terrain_groups.rs:434`, `map_resources.rs:281`, `map_rivers.rs:99` (`mountain_tiles`). Agree. |
| 006b30e0 | `WorldData::start_city_wcoord` | layout | run1016 | twin | The bit `y·xs + x` of the start-city bitmask (`world+0xf8`, the data of `start_city_locs` at `+0xf0`). Callers `TerrainGroup::available_rough_tile`, `Mountains::verify_template` (map generation). Twin `crates/sim/src/terrain_groups.rs:407` `city` over `MapGrid::city` (`mapgen.rs:228`, the same `y·xs + x` bit). Agree. |
| 006b3120 | `WorldData::is_flat` | layout | - | absorbed | `(flags & 0x30) == 0 && !is_ocean && (flags & 0x48) == 0`. One caller, `Mountains::verify_template` (map generation). The crate computes it inline with the same three tests: `crates/sim/src/terrain_groups.rs:792` (`quick_verify`, negated) and `:729` (`clear_group`). Agree. |
| 006b3180 | `World::is_edge_of_region` | layout | run1017 | twin | An in-bounds neighbour of the eight (`move_x/move_y[1..8]`) whose region (`+4`) differs from the cell's. One caller, `TerrainGroups::nubify_forest`. Twin `crates/sim/src/terrain_groups.rs:2189` `edge_of_region`: same bounds test and region compare. Agree. |
| 006b3850 | `WorldData::start_city_rad_wcoord` | layout | run1016 | twin | For each start city (count `world+0xbc`, x `+0xc8`, y `+0xe4`), the octagonal distance (long leg + short²/(2·long), unsigned; `(short + 2·long) >> 1` when short ≥ 60000), times 4, `< city_center_radius − 1`. One caller, `Map::make_riverbeds`. Twin `crates/sim/src/map_rivers.rs:110` `near_start_city` + `:120` `octagonal`: the same leg choice (`dy < dx` picks dx long), the same unsigned arithmetic and 60000 arm, the same `4·d < radius − 1`. Agree. |
| 006b3960 | `WorldData::get_region2` | layout | - | absorbed | The signed short `region2` (+6). No caller (inlined). The field is `CellData::region2` `crates/sim/src/world.rs:225`, `−1` as `None`; the sweep (item 1721) holds it. |
| 006b3990 | `WorldData::is_building_wcoord` | layout | run1016 | ui | All four corner tiles of the cell are a building footprint (`mask & 3 == 3`) or placed (`mask & 0x80`). Callers `GoodTypeOut::draw@0066fd70` (a good's sprite is not drawn under a building, `:59`) and `ScenarioEditor::on_edit_terrain` — the lockstep entry is the renderer's. Census: proposed mark `callers_all_marked` (every caller already `ui`/editor/cut) — catches this row; no name pattern fits. |
| 006b3a80 | `World::set_resource_at` | layout | - | absorbed | Sets or clears tile bit `0x200` (`tile::AS_BUILDING`, `crates/sim/src/world.rs:595`) at `y·tile_xs + x`. No caller: its writers inline the `\| 0x200` / `& 0xfdff` (`Good::close`, `Unit::move_step`, `Object::do_damage`, `Build::process`). The crate carries the bit in the tile mask (`set_tile_bits`/`set_tile_mask`, `world.rs:1964`, `:1978`) as loaded, and reads it at `tcoord.rs:151` (`blocked_tcoord`) and `terrain.rs:118`. |
| 006b3b80 | `WorldData::is_city_at` | layout | run1016 | twin-differs | Tile bit `0x100` (city radius) else 0; then the cell's `who` (+0xf): `who > −1 && who != p && !is_ally(p, who)` → 0, else 1. One caller, `ObjectsData::find_city_at@0065b870:31`. Twin: the head of `Sim::find_city_at` `crates/sim/src/place.rs:712-719` (`CITY_RADIUS`, then `Owner::None` or an ally passes). **Differs on `who == −2`** (`Owner::Ambiguous`): the original's `-1 < who` guard lets it pass, the crate's `_ => return None` refuses — the crate's own `Owner::Ambiguous` doc (`world.rs:176`) says every consumer treats it as unowned, which is what the original does here. Shows on a frame where a claim is ambiguous and something asks for the city there (`find_city_at`'s callers: placement, `alarm`, `castable`); the dump field is the cell's territory `who` = −2 in `WorldSync`/`TerritorySync`, then a `BUILDDATA` city index or a refused placement. |
| 006b3c10 | `WorldData::get_bottom` | layout | - | dead | Walks an object chain `(who, index)` through `objects[who].list[index]` `+0x2e` (next who) and `+0x2c` (next index) until the index is negative, and answers whether the terminator is below −1. The decompiler reuses `this->xs` as the walk's index (a folded signature). No caller in the export; the crate's chain walkers (`alarm_view.rs:628` `get_down`, `capture.rs:632`) never ask the terminator's kind. Census: the `is_uncalled` mark. |
| 006b4180 | `World::set_city_at` | layout | - | absorbed | Sets or clears tile bit `0x100` at `y·tile_xs + x`. No out-of-line caller: inlined in `Wall::mask_city` (`& 0xfeff` / `\| 0x100` at its lines 26, 30). Twin of the inline: `crates/sim/src/city.rs:2135` `mask_city` → `set_tile_bits`/`clear_tile_bits(t, tile::CITY_RADIUS)` (`world.rs:593`, `0x100`). Agree. |
| 006b41d0 | `World::set_was_seen` | layout | - | absorbed | ORs `1 << who` into the cell's `was_seen` byte (`WData +0x14`) and into `seen2` (`World +0x160`) at the fog cell. No caller (inlined; `Wall::start`'s footprint `was_seen` write is the same store, `docs/CITIES.md` §3.5). The crate's `seen2` write is `World::set_seen2_only` `crates/sim/src/world.rs:989` (a mask, not a player index); `WData +0x14` is not kept, the crate's own note at `world.rs:966` (no reader in this simulation, `docs/VISION.md` §7). |
| 006b4230 | `World::set_behind` | layout | - | dead | Sets or clears tile bit `0x8` (`param_4`) or `0x4` at `y·tile_xs + x`. The executable never references it (`docs/PARKED.md:353`, pinned in `docs_guard::DEAD_CITED`); the live copy is inline in `Wall::mark_behind_tiles@0063d230` and `Mountains::add_mountain` (`docs/CENSUS.md:362`). The crate carries no `0x4`/`0x8` tile bit (`crates/sim/src/world.rs:572-610` names none) — the inlined writer's bit is the `docs/ROADS.md:1026` tile-mask residue, `Wall::mark_behind_tiles`' row, not this one's. Census: the `is_uncalled` mark, or `DEAD_CITED` read as a census class. |

## Part 2

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006b4290 | `World::get_seen` | layout | - | twin | `seen[fy·fog_xs + fx]` (`World +0x15c`, `+0xc` the fog width). No caller (inlined). Twin `World::seen` `crates/sim/src/world.rs:957` — `fog_now[fy·fog_xs + fx]`, the same plane and index; the crate adds a bounds check answering `None`. Agree on the grid. |
| 006b42c0 | `WorldData::is_really_seen` | layout | - | unmodelled | For `p > 7` → 1; leader flag `0x800` or `num_units[0x141] != 0` → 1; flag `0x2000` with the cell's owner an ally of `p` → 1; else `seen[fy·fog_xs + fx] & ally_mask` (the **current** plane, not `seen2`). One caller, `BuildTypeData::in_enemy_territory@00638b40:50`, called with the tile's *owner* as `p`: its last arm, "the owner really sees the tile", reached only when the placer has no city (`+0x3f8 == 0`) and none in that region. The port `Sim::in_enemy_territory` `crates/sim/src/place.rs:624` takes that arm false (`docs/PARKED.md:5226`, item 2250). The body is `WorldData::is_seen` less its `reveal_map == 3` arm, so the crate's `Sim::world_is_seen_fog` (`crates/sim/src/fight.rs:1537`) minus that arm is the whole of a port. Not entered by any lockstep run — no captured game placed a building on enemy ground with no city of its own. The difference shows as a placement refused (`ENEMY_TERRITORY2 0x19`, `docs/CITIES.md:389`) where the port accepts: a `BUILDDATA` row present in the port and absent in the original, on a capture of a cityless player building in a rival's sight. |
| 006b4380 | `WorldData::is_rocks` | layout | run1016 | absorbed | `wdata[y·xs + x].flags & 8`. Callers `WorldData::get_land@006b4730:36` and `ConsoleWin::run_cmd` (debug). Inlined in the crate's `get_land` twin `World::land_class` `crates/sim/src/world.rs:1302` (`f & cell::ROCK`, then oil for 7), in the same order (`world.rs:2273`'s test). Agree. |
| 006b4530 | `World::set_started2_at` | layout | - | absorbed | Sets or clears tile bit `0x40` (a second building placed). No caller: inlined in `BuildType::start_me`. Crate: `tile::PLACED_TWICE` `crates/sim/src/world.rs:589`, set at `city.rs:1546` (`start_me(1)`: a tile already `PLACED` takes `0x40`) and cleared with `PLACED` at `city.rs:2025`, `:2053`, `:2629`. Agree. |
| 006b4570 | `World::set_started_at` | layout | - | absorbed | Sets or clears tile bit `0x80` (a building placed, not started). No caller: inlined in `BuildType::start_me`. Crate: `tile::PLACED` `crates/sim/src/world.rs:591`, set at `city.rs:1548`, cleared at `city.rs:2025`/`:2053`/`:2629`. Agree. |
| 006b45c0 | `World::set_building_at` | layout | - | absorbed | On: `mask \|= 3` (the object field to building). Off: clears the field **only when it reads 3**. No caller: inlined in `BuildType::mask_me@006312a0:127-130`. Crate: `city.rs:2027` `set_tile_field(t, OBJECT, OBJECT_BUILDING)` on, and `city.rs:2050` `set_tile_field(t, OBJECT, 0)` off **without the `== 3` guard** — inert in a skirmish, since placement refuses a footprint on a mountain (2) or cliff (1) tile, so the field always reads 3 when a building unmasks; the gap is `BuildType::mask_me`'s row, not this one's. |
| 006b46b0 | `World::set_gathered_at` | layout | - | absorbed | Sets or clears tile bit `0x1000` (gathered from). No caller survives (`docs/AI.md:3177`). Crate: `tile::GATHERED_FROM` `crates/sim/src/world.rs:609`, set by the crate's gather-site pass `gather.rs:566` (`set_tile_bits`), read by `World::gather_at` (`world.rs:1374`) and `gather.rs:481`/`:515`. The writer's own call site is unknown; the crate's is a reading. |
| 006b4880 | `WorldData::is_impassable` | layout | - | absorbed | `(flags & 0x70) != 0` (MOUNTAIN \| FOREST \| `0x40`). No caller (inlined). The crate tests the same mask inline: `crates/sim/src/ai_census.rs:1095`, `ai_place.rs:175`; the sweep (item 1721) holds it. Agree. |
| 006b48c0 | `WorldData::is_detected` | layout | - | absorbed | `seen3[fy·fog_xs + fx] & leaders[p].+0x6929` (the detection plane `World +0x164` against the leader's ally mask). No caller: `UnitData::is_seen`'s bare `is_detected` is `UnitData::is_detected`, which inlines this read. Crate: `crates/sim/src/fight.rs:1467` (`world.seen3(..) & seen_ally_mask(who) == 0`), `seen_ally_mask` `vision.rs:730` the `+0x6929` byte. Agree. |
| 006b4bb0 | `World::set_seen2` | layout | run1017 | twin | `seen2 \|= mask`; with `param_4 == 0` also `seen \|= mask` and the cell twin `+0x168`; always `WData +0x14 \|= mask`; answers whether `seen2` or `seen` changed and on a change queues `WorldMapFog::add_update` (the minimap's redraw). Callers `Unit::update_local_seen` (`param_4` 0), `Wall::update_local_seen` (`param_4 = !wonder`), `Object::update_seen` (`+0x3a`, 0), `Ammo::do_damage` (`0xff`, 0) — none reads the return. Twins: `param_4` set → `World::set_seen2_only` `crates/sim/src/world.rs:989`; `param_4` 0 → `World::set_seen` `world.rs:970`; called from `vision.rs:894` (`update_local_seen_unit`) and `:960-963` (`update_local_seen_build`, the wonder arm). The two planes agree; `+0x168` and `WData +0x14` are not kept (no reader, `vision.rs:872`); the redraw queue is the interface's. `Object::update_seen`'s `+0x3a` call is the infiltrator's plane, a live SEAM at `cast.rs:894`. |
| 006b50f0 | `WorldData::is_detected_by_enemy` | layout | - | absorbed | `seen3[fy·fog_xs + fx] & ~leaders[p].+0x6929` — detected by someone outside `p`'s alliance. No caller (inlined). Crate: `crates/sim/src/cast.rs:118` (`world.seen3(..) & !seen_ally_mask(owner) != 0`), the same plane, index and complement. Agree. |
| 006b5350 | `WorldData::get_coll_block` | layout | run1016 | absorbed | The cell's `CollBlock*` (`+0x18`), or `−1` when a region `>= 0` is asked and the cell's (`+4`) differs. One caller, `CollCheck::move_unit@00682ad0:60` (its one-block head), which returns on `−1` or a null block. The crate flattens the per-cell blocks into one map-wide bitmask (`crates/sim/src/collide.rs:545`) and carries the region gate as `coll_region_ok` `collide.rs:1425`, called from `coll_move` `:1679` (the one-block head at `:1708`); `docs/PATHFINDER.md:2294` read both against the port and found them agreeing. Two arms differ in form and are inert: a null block (a cell no unit was ever added to) has no flat-bitmap analogue, and a region-less cell passes the crate's `is_none_or` where the original's `−1 != region` refuses — every generated cell carries a region. |
| 006b5590 | `WorldData::is_mountains` | layout | - | absorbed | `wdata[y·xs + x].flags & 0x10`. No caller (inlined). The crate tests `cell::MOUNTAIN` inline: `World::land_class` `crates/sim/src/world.rs:1300` (with `0x40`), `terrain_groups.rs:473`; the sweep (item 1721) holds it. |
| 006b55c0 | `WorldData::is_seen` | layout | run1016 | twin | `p > 7` or `reveal_map == 3` → 1; leader flag `0x800` or `num_units[0x141]` → 1; flag `0x2000` with the cell's `who` an ally → 1; else `seen[fy·fog_xs + fx] & ally_mask`. Callers `Unit::set_attacking`, `Unit::do_cast`, `WallData::is_now_seen`, `ItemData::is_seen` and a dozen renderers. Twin `Sim::world_is_seen_fog` `crates/sim/src/fight.rs:1537`: the same five arms in order (`sees_every_unit` `ai_sites.rs:263` is the `0x800`/Fouché pair, `territory_reveal_preq` the `0x2000` bit), the current plane `World::seen`, `seen_ally_mask`. Agree; the crate answers 1 off the grid where the original reads past it. |
| 006b56b0 | `WorldData::is_pass_land` | layout | - | dead | `(flags & 0x30) == 0 && !is_ocean` (no mountain, no forest, not water). No caller in the export, and no inlined copy found among the `& 0x30` cell readers (`has_gather_access`, `get_tregion`, `is_tocean` read the tile field, not this). The sweep `crates/sim/tests/sweep/world_data_is_pass_land.rs` holds it. Census: the `is_uncalled` mark. |
| 006b76a0 | `World::adjust_fog` | layout | - | dead | Empty body (`return;`); no caller. The sweep `crates/sim/tests/sweep/world_adjust_fog.rs` holds the no-op. Census: the `is_uncalled` mark, or a proposed `is_empty_body` (an export body of `return;` alone). |
| 006b76b0 | `World::scale_by_axis` | layout | - | dead | `(div/2 + world.xs · n) / div`, at least 1 (the decompiler prints `n` as `this`). No caller; the live copy is `Map::scale_by_axis@006a08d0` (its sweep `crates/sim/tests/sweep/map_scale_by_axis.rs`). Census: the `is_uncalled` mark. |

## Part 3 — the world map

`WorldMap`, `WorldMapFog` and `WorldMapBackground` are the interface's minimap: a window
(`Window::mouse_capture`, `client_to_screen`), a camera, vertex arrays and two textures. No
row below writes a field the simulation reads; the minimap's only tie to the simulation is
that `World::set_seen`/`set_seen2`/`clear_seen` queue redraws into it
(`WorldMapFog::add_update`). One census class catches all twenty-one: **proposed `is_ui_body`
extension, class pattern `^WorldMap(Fog|Background)?$`** (today only `on_right_down` is
caught, by `on_right`). The `entered` runs are the renderer's, which the traced games run.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 008c7000 | `WorldMap::on_right_down` | layout | - | ui | Takes the mouse capture and records the press (`dragging`, `down_x/y`, `down_x2/y2`). The existing `UI_BODY` `on_right` mark catches it; the sweep `crates/sim/tests/sweep/world_map_on_right_down.rs` lifts it to `layout` (the `territory_kind` order the terrain-1 report names). |
| 008c7810 | `WorldMap::on_extra_up` | open | - | ui | The minimap's extra-button release: bumps the console player's `accum_clicks` and `accum_minimap_clicks` (lobby statistics, read by no simulation rule) and calls `Console::map_click` — the command a click becomes is the command layer's, replaced by design. Only reference a vftable slot (`docs/AI.md:39479`). Census: the class pattern, or `on_extra` added to `UI_BODY`'s `on_(…)` list. |
| 008c7cb0 | `WorldMap::check_mouse_accept` | open | - | ui | Whether the cursor lies in the minimap's diamond: the capture test, then a float barycentric test against `mouse_check_verts`. Census: the class pattern, or `check_mouse` in `UI_BODY`. |
| 008c8120 | `WorldMap::get_view_rect` | open | run1016 | ui | The main view's four corners projected to the ground (`RenderWindow::mouse_to_xy_plane` at the console window's corners) into `view_rect`, for the minimap's view outline. Census: the class pattern. |
| 008c81f0 | `WorldMap::on_redraw` | open | run1016 | ui | Gated on the scene's draw flag and `enable_minimap`; brings the window to top and translates its float vertices to screen space, then draws. Census: the class pattern, or `on_redraw` in `UI_BODY` (it would catch every window's redraw). |
| 008c8f70 | `WorldMap::add_texture_ping` | open | run1017 | ui | Builds the vertices of a `TexPing` (a ping's animated texture) in floats; reads no game state. Read by its first screen and its globals (14 KB). Census: the class pattern. |
| 008cb8a0 | `WorldMap::prepare_background` | open | run1016 | ui | Waits on the graphics device (`Sleep`, `device_lost_reset`), renders `WorldMapBackground` and restores the camera. Census: the class pattern. |
| 008cbcd0 | `WorldMap::compute_fog_destination_verts` | open | run1016 | ui | Projects the world's corners through `MiscAccess::world_to_console` with a temporary camera (`set_dir`, `adjust_tilt`) to place the fog quad. One caller, `WorldMap::init`. Census: the class pattern, or `_verts$` in `UI_BODY`. |
| 008ce500 | `WorldMap::add_blip` | open | run1016 | ui | Appends a unit or building blip's quad (size from `blip_sizes`, colour, `blip_verts[layer]`) — vertex counts only. One caller, `WorldMap::render_blips`. Read by its first screen, its stores and its globals (25 KB). Census: the class pattern. |
| 008d0b70 | `WorldMapBackground::setup_camera` | open | run1016 | ui | Places the background's camera over the world's extent (`set_dir`, `set_auto_tilt`, `compute_clip_planes`). Census: the class pattern, or `setup_camera` in `UI_BODY`. |
| 008d1070 | `WorldMapFog::fill_colors_who` | open | run1016 | ui | Writes `fog.texture_who`, the territory-colour texture, from each cell's owner filtered by `can_see_mask`. Its only reader is the minimap. Census: the class pattern. |
| 008d1420 | `WorldMapFog::fill_colors_fog` | open | run1016 | ui | Drains `fog.fx_update`/`fy_update`, reads `WorldData::is_seen`/`was_seen` for the console player and writes `fog.texture_fog` and `fog_lookup` (`set_fog_lookup`), invalidating `TerrainOut`'s fog cache. Census: the class pattern. |
| 008d1300 | `WorldMapFog::set_fog_lookup` | layout | run1016 | ui | Writes `fog_lookup` at `(y+1)·(fog_xs+2) + x+1` and mirrors the border into the one-cell apron. One caller, `fill_colors_fog`. The sweep `world_map_fog_set_fog_lookup.rs` lifts it to `layout`. Census: the class pattern. |
| 008d17b0 | `WorldMapFog::fill_colors` | layout | run1016 | ui | Builds `can_see_mask` from `LeaderData::can_see_borders` for the console player, refills both textures on a change, then the fog. One caller, `Scene::render`. Census: the class pattern. |
| 008d1830 | `WorldMapFog::get_fog_lookup` | layout | - | ui | Reads `fog_lookup[(y+1)·(fog_xs+2) + x+1]`. No caller in the export; the sweep (item 1942) holds it. Census: the class pattern. |
| 008d26a0 | `WorldMap::get_mouse_loc` | open | - | ui | Swaps in the minimap's camera, projects the last mouse position to the ground, restores the main camera. Callers the minimap's click handlers. Census: the class pattern, or `mouse` in `UI_BODY`. |
| 008d27f0 | `WorldMap::full_update` | open | - | ui | Marks both cameras changed, re-inits the world map through its vtable, re-inits the fog. Census: the class pattern. |
| 008d2760 | `WorldMapFog::add_update` | layout | run1016 | ui | Appends a fog cell to `fog.fx_update`/`fy_update` unless a full refill is pending (`flags & 2`). Callers `World::set_seen`, `World::set_seen2`, `World::clear_seen` — the simulation's fog writes queue the redraw; the queue's one reader is `fill_colors_fog`. The crate's twins of those three omit the call, rightly. Census: the class pattern. |
| 008d2b60 | `WorldMap::reinit_all` | layout | - | ui | Re-inits the world map (`init`, two vtable calls) and the fog. Census: the class pattern (`reinit` misses `DEAD_WEIGHT`'s `init` anchor at the name's start). |
| 008d2ba0 | `WorldMap::update_world_map` | open | run1016 | ui | Moves the main camera (`Camera::set_loc`) to the minimap point under the mouse while dragging. Callers `Game::loop_render` and `WorldMap::on_left_down`. The camera is not simulation state. Census: the class pattern. |
| 008d3870 | `WorldMap::prep_buffers` | open | run1016 | ui | Allocates the minimap's vertex and texture buffers (`Buffer::init/close`, `WorldMapBackground::init`), retrying through a lost device. Callers `WorldMap::init`, `Game::run`, `MiscAccess::device_lost_reset`. Census: the class pattern. |

## Summary

**Counts (54):** twin 7 · twin-differs 1 · absorbed 17 · unmodelled 1 · dead 6 · ui 22.
No row is `replaced`, `editor`, `ctw`, `debug`, `saveload`, `network` or `sound`.

**What the group is.** The `world` half is accessors and setters of the cell (`WData`, 0x1c
bytes at `World +0x134`), the tile mask (`World +0x138`) and the fog planes (`+0x15c` seen,
`+0x160` seen2, `+0x164` seen3). Most have **no caller**: the compiler inlined them at
every reader, and the crate does the same bit test or store at its own readers — hence
seventeen `absorbed`. The `twin` rows are the ones with a live caller and a named crate
function, filed `layout` only because the crate's fn name is not the snake case of the
original's (`mountain_tiles`, `city`, `edge_of_region`, `near_start_city`, `World::seen`,
`set_seen2_only`/`set_seen`, `world_is_seen_fog`). The `worldmap` half is the minimap,
all `ui`.

**Proposed census classes:**

- **`is_uncalled`** — no caller and no vtable slot in the export (a mechanical mark from
  the call graph). In this group it would catch the six `dead` rows (`World::scale_by_area`,
  `World::scale_by_axis`, `World::adjust_fog`, `World::set_behind`, `WorldData::get_bottom`,
  `WorldData::is_pass_land`), `WorldMapFog::get_fog_lookup`, and the uncalled inlined
  accessors among the `absorbed` (`World::set_land`, `set_resource_at`, `set_city_at`,
  `set_was_seen`, `set_started_at`, `set_started2_at`, `set_building_at`,
  `set_gathered_at`, `WorldData::is_coast`, `get_region2`, `is_impassable`,
  `is_detected`, `is_detected_by_enemy`, `is_mountains`) — those are covered by the crate's
  own inline tests, so the mark should file them *absorbed-by-inlining*, not dead. Twenty
  of this group's 33 `world` rows (six dead, fourteen absorbed).
- **`is_empty_body`** — an export body of `return;` alone: `World::adjust_fog`.
- **`UI_BODY` class extension `^WorldMap(Fog|Background)?$`** — all 21 `worldmap` rows
  (today only `WorldMap::on_right_down` is caught). Narrower alternatives on the short name:
  `on_extra`, `on_redraw`, `check_mouse`, `setup_camera`, `_verts$`, `mouse`.
- **`callers_all_marked`** — a function whose every caller is already `ui`, editor or cut:
  `WorldData::is_building_wcoord` (`GoodTypeOut::draw`, `ScenarioEditor::on_edit_terrain`).
- **A rubric spelling fix, not a class**: accept `Class::method@va` in any `crates/sim` fn's
  doc comment as the port's spelling regardless of the fn's name. It would lift the seven
  `twin` rows from `layout` toward `port`; three already carry the exact spelling in the
  doc comment (`terrain_groups.rs:432`, `map_rivers.rs:97`/`:106`, `terrain_groups.rs:2187`).

**Proposed replaced groups:** none.

**`unmodelled`, ranked** (none is lockstep-entered):

1. `WorldData::is_really_seen@006b42c0` — not entered. The last arm of
   `BuildTypeData::in_enemy_territory`: a cityless placer's footprint on enemy ground
   is refused when that ground's owner currently sees it. The port takes the arm false
   (`crates/sim/src/place.rs:624`, `docs/PARKED.md:5226`). The field that would show it: a
   `BUILDDATA` row (a placed building) present in the port and absent in the original, on a
   capture where a player with no city places a building in a rival's territory and sight.
   The port is one call: `Sim::world_is_seen_fog` (`fight.rs:1537`) without its
   `reveal_map == 3` arm.

**`twin-differs`:**

- `WorldData::is_city_at@006b3b80` (entered, run1016) vs `Sim::find_city_at`
  `crates/sim/src/place.rs:715-719`: on a cell whose territory `who` is `−2`
  (`Owner::Ambiguous`) the original lets the city search go on and the crate refuses. Seen
  in the dump as the cell's `who` = −2 in the territory dump, then a city index or a refused
  placement in `BUILDDATA`. The lockstep agreeing says no captured frame asked
  `find_city_at` on an ambiguous cell.

**Inert differences recorded in the `absorbed` rows** (not rulings of their own):
`BuildType::mask_me`'s inline of `set_building_at` clears the object field without the
`== 3` guard (`city.rs:2050`); `get_coll_block`'s null-block and region-less-cell arms have
no flat-bitmap analogue (`collide.rs:1425`).

## Not established

- **Callers by name only.** "No caller" was settled by grepping the export for
  `Class::name(` and a bare `name(`; a call through a function pointer, or one the
  decompiler printed under a COMDAT-folded name, would not show. For `World::set_behind`
  the executable-wide check exists (`docs_guard::DEAD_CITED`); for the other uncalled rows
  it does not.
- **`World::set_gathered_at`'s live writer** stays unknown (`docs/AI.md:3177`); the crate's
  gather-site pass (`gather.rs:566`) is a reading, not a twin of a located call.
- **Tile bits `0x4`/`0x8`** (`Wall::mark_behind_tiles`, the live inline of
  `set_behind`): whether any simulation rule reads them was not settled here; the crate
  carries neither, and `docs/ROADS.md:1026` books their residue to that function's row.
- **`num_units[0x141]`** is read as a Fouché standing on the crate's word
  (`ai_sites.rs:259`), not re-derived here.
- The `worldmap` bodies over 14 KB (`add_blip`, `add_texture_ping`) were ruled by their
  first screen, their globals and their stores, as the brief allows; neither reads a
  `GameAccess` global at all.
