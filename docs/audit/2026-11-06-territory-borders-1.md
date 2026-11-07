# Territory reading — group `borders-1` (2026-11-06)

A territory reader's rulings on the 60 functions of the `borders`, `terrainborder` and `wall`
stems (`/Users/rf-studio/.claude/jobs/d388900a/tmp/t71/groups/group-borders-1.tsv`), one row per
function, read by name from the decompile export. Reader: Opus 5.5, the seventy-first tranche's
reading fan-out. `entered` is the TSV's `entered_by_lockstep` column (`y` = some lockstep run
entered it, `-` = none). Nothing here is transcribed; only the expression a ruling turns on is quoted.

**The family's shape, read first.** `Borders` (the `MiscAccess::borders` singleton) and its `Border`,
`StripBuilder`, `LinkVert`, `BorderNodes`, `MorphSideStrip`, `BorderMorphRender`,
`BorderSplineRender` and `BorderSpline` are the **drawn national-border line**: `Scene::render@008b3270`
calls `Borders::process_all@00882490`, which per region border runs `Border::process@0087fe30` —
gated on `camera->camera_changed`, stamped with `gamec->graphic_tick`, ending in `Border::draw`. The
simulation's only contact is `World::compute_reg_territory@006b0bb0` (lines ~287/294/589) and
`Region::walk_data@00680e20` calling `add_border_region` / `remove_border_region` /
`save_last_who`, which write only into the `Borders` singleton and the region's `border_id` byte
(`Region+0x30`) — a handle into the render list. Territory ownership itself (`WorldData` tile `+0xf`
`who`, `+0x10`) is written by `compute_reg_territory` after the call, and is ported in
`crates/sim/src/territory.rs`. `TerrainBorder` is the terrain-type blend edge renderer (no
national meaning). `Wall` is the wall building class (`WallData` its data side), which is simulation.

## Part 1 — the drawn border (`borders` stem, 37 rows)

Proposed census class for every row ruled `ui` here, **`BORDER_RENDER`**: class matches
`^(Borders?|BordersOut|BorderNodes|BorderMorphRender|BorderSplineRender|BorderSpline|MorphSideStrip|StripBuilder|LinkVert|TerrainBorder)$`
— the classes the scene's render pass reaches through `Borders::process_all`. Exceptions named
in their rows: the three entry points the simulation calls (`add_border_region`,
`remove_border_region`, `save_last_who`) are render-state writes from inside
`compute_reg_territory`, ruled `ui` with that caller stated, since nothing they write is read back
by any simulation function (the `Borders` singleton is read only by the classes above,
`MouseOver::describe_border_info` and `BorderSplineRender::init_border_render`; `Region+0x30` is
read only as the handle's own nonzero test).

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0087f5b0 | Border::check_ends | open | y | ui | Picks the owners either side of a border strip's end from tile `who` (`world+0x134` cell `+0xf`), masked by the viewer's `borders+0x218` visibility bits, then `check_border_blocked`/`no_border`; feeds only strip geometry. `BORDER_RENDER`. |
| 0087f8c0 | Border::generate_strip | open | y | ui | Builds a spline from `BordersData::wcoord_list_1` with float tangents (`fVar15..18`), `Spline::calc_spline`, `BorderSplineRender::make_bbox`, `MiscAccess::terrain` heights. Pure render geometry. `BORDER_RENDER`. |
| 0087fbb0 | Border::walk_regular_strips | open | y | ui | Walks the strip builder's link lists into render strips (`StripBuilder::clear`). Called from `Border::process`/`build_border` only. `BORDER_RENDER`. |
| 0087ff20 | Border::singul_morph | open | y | ui | Morph-animation helper (no globals, no callees beyond itself); part of the border's change animation keyed on `graphic_tick`. `BORDER_RENDER`. |
| 008800a0 | Border::walk_clump_strips | open | y | ui | Walks clump strips into `MorphSideStrip::set_bits`; morph animation geometry. `BORDER_RENDER`. |
| 00880280 | Border::sort_clump_list | open | y | ui | Sorts changed tiles into clumps for the morph, reading tile `who` and the viewer's `console->who` (`leadersc`). Viewer-relative, render only. `BORDER_RENDER`. |
| 008804a0 | Border::build_morph | open | y | ui | Builds the border-change morph from `who_changed`, testing `is_border_hidden` (viewer-relative); called from `Border::process` with `change_state` 0x40/0x80 and `gamec->graphic_tick`. `BORDER_RENDER`. |
| 00880960 | Border::sort_regular | open | y | ui | Collects border edges between differently-owned tiles into the strip builder, testing the viewer's mask and `check_border_blocked`. Render only. `BORDER_RENDER`. |
| 00880e50 | BorderMorphRender::inc_morph | open | y | ui | Advances one morph render against `gamec->graphic_tick`; returns done. Graphic clock, not the sim clock. `BORDER_RENDER`. |
| 008811e0 | BorderMorphRender::make_interp_spline | open | y | ui | Interpolated spline between two morph states (`Spline::calc_spline`, terrain heights); float geometry. `BORDER_RENDER`. |
| 008816d0 | BorderMorphRender::check_circular | open | y | ui | Tests whether a morph strip closes on itself (no globals). Geometry. `BORDER_RENDER`. |
| 008818f0 | MorphSideStrip::set_bits | open | y | ui | Recursively sets `who_bits` on a morph side strip from `borders` tile flags. Render state. `BORDER_RENDER`. |
| 00881b30 | StripBuilder::walk_strip | open | y | ui | Walks linked vertices of one strip into an output vertex array (`Error::report` on a broken link). Geometry; the crate's `walk_strip` hit is an unrelated sweep/name. `BORDER_RENDER`. |
| 00881cf0 | StripBuilder::add_strip | open | y | ui | Adds an edge to the strip builder: `BorderNodes::find`, `LinkVert::init/append`, `remove_end`. Geometry. `BORDER_RENDER`. |
| 00882530 | Borders::make_texture | open | y | ui | Builds the border line texture from the leaders' colours (`leadersc`); called only from `Borders::init@008833a0`. `BORDER_RENDER`. |
| 00882990 | BorderSplineRender::map_texture | open | y | ui | Texture coordinate mapping (the export also folds `TileSet`/`Ping*Spline` names onto this address — COMDAT). Render. `BORDER_RENDER`. |
| 00883780 | Borders::add_border_region | open | y | ui | Allocates a `Border` (0xcc) for a region, appends it to the singleton's list and returns its 1-based index, which the caller stores as `Region+0x30`. Called by `World::compute_reg_territory@006b0bb0:294` and `Region::walk_data@00680e20` (load). Writes only render state; the crate has no render list and needs none (`territory.rs` computes ownership without it). `BORDER_RENDER`. |
| 004a1ca0 | BorderSpline::on_field_change | layout | - | ui | Sets `field_change = 1` on a tuning-params object (the debug/tuning field editor's callback); PARKED 3479b/3861a already find no registration path but the editor. `BORDER_RENDER`, or a proposed `on_field_change$` pattern (editor callbacks: also `CircleRender`, `Storm`). |
| 004a1cb0 | BorderSpline::get_name | layout | - | editor | Returns the literal `"Border Spline Params"` for the params editor. Proposed pattern `get_name$` on a `*Spline`/params class, or `BORDER_RENDER`. |
| 0087fd00 | Border::process_morph | layout | y | ui | Ticks each morph render (`inc_morph`), recycles finished ones, flags `need_rebuild`. `BORDER_RENDER`. |
| 0087fd70 | Border::rebuild_visible_list | layout | y | ui | Filters render strips by `TerrainOut::is_region_visible` (camera). `BORDER_RENDER`. |
| 0087fe00 | Border::build_border | layout | - | ui | `close`, `sort_regular`, `walk_regular_strips`, `rebuild_visible_list`; no callers in the export. `BORDER_RENDER`. |
| 0087fe30 | Border::process | layout | y | ui | Per-border render driver: rebuild on `need_rebuild`/`recalc_borders`/`camera_changed`, morph twice on `graphic_tick`, then `draw`. Called only from `Borders::process_all`. `BORDER_RENDER`. |
| 00880050 | Border::generate_morph_side | layout | - | ui | Pops a `MorphSideStrip`, walks it, pushes it back and returns null (a vestigial body). No callers. `BORDER_RENDER`. |
| 00882050 | StripBuilder::check_last_connect | layout | - | ui | Tests whether a node table's last link points back to the first. No callers (inlined into `add_strip`). `BORDER_RENDER`. |
| 008820a0 | StripBuilder::remove_end | layout | y | ui | Removes a strip end from the builder; called from `add_strip`. `BORDER_RENDER`. |
| 00882170 | BorderNodes::find | layout | y | ui | Hash lookup of a border node by coordinate; called only from `add_strip`. PARKED 2221 already notes its port mark is a name collision. `BORDER_RENDER`. |
| 008821e0 | LinkVert::append | layout | y | ui | Links a vertex into a strip's chain; called from `add_strip`. `BORDER_RENDER`. |
| 00882300 | Borders::is_border_hidden | layout | y | ui | 1 when leader `p` has tribe bonus 0x13 and is neither the viewer (`console->who`) nor mutually at diplomatic state 2 with the viewer; callers `build_morph`, `save_last_who`, `init_border_render` — all render. A nation whose borders are not drawn to others; the territory itself is unaffected. `BORDER_RENDER`. |
| 00882370 | Borders::check_border_blocked | layout | y | ui | Reads the singleton's per-tile flag bytes (`borders+0x2c`, stride 8) to drop an owner from a strip end; callers `sort_regular`, `check_ends`. `BORDER_RENDER`. |
| 00882490 | Borders::process_all | layout | y | ui | Called only from `Scene::render@008b3270`: recomputes the viewer's `can_see_borders` mask into `borders+0x218` and runs `Border::process` on each border. `BORDER_RENDER`. |
| 008826c0 | BorderSplineRender::is_visible | layout | - | ui | `TerrainOut::is_region_visible` on the strip's bbox. No callers. `BORDER_RENDER`. |
| 008826f0 | BorderSplineRender::make_bbox | layout | y | ui | Short-int bbox over a spline's points; called from `generate_strip`. `BORDER_RENDER`. |
| 00883190 | Borders::get_who | layout | - | ui | Tile owner read (`world+0x134` cell `+0xf`, negative → -1); no callers in the export. The crate's equivalent read is `World::owner_at` (`crates/sim/src/world.rs:1825`), not a twin of a used function. `BORDER_RENDER`. |
| 008831d0 | BordersOut::no_border | layout | y | ui | 1 when the tile's flag `0x100` is clear and its byte `+2` is 1 or 2; callers `check_ends` and `MouseOver::describe_border_info` (UI). Already caught by `is_ui_body` (`cls.endswith("Out")`). |
| 008835e0 | Borders::save_last_who | layout | y | ui | Called once per tile from `World::compute_reg_territory@006b0bb0:589` just before the new owner is stored: if the owner changes (and neither side is hidden from the viewer), sets flag 0x40/0x80 in `borders+0x2c` and queues the tile on the region's `Border+0x58` change list — the morph's input. Writes only render state; the crate computes the owner and needs no change list. `BORDER_RENDER`. |
| 008836d0 | Borders::remove_border_region | layout | - | ui | Closes and frees region border `n`, swaps the last into its slot and rewrites that region's `Region+0x30` handle; called from `compute_reg_territory@006b0bb0:287` when a region loses its city. Render list bookkeeping. `BORDER_RENDER`. |

## Part 2 — terrain edges and the wall building (`terrainborder`, `wall` stems, 23 rows)

`Wall` is the base class of `Build` (`Wall::Wall@0063e390` is reached only from `BuildData::BuildData@0062f370`
and the destructors; no live object keeps `Wall::vftable`), so `Wall::*` methods are every building's.
The tile-mask bit `0x4` that `mark_behind_tiles` writes (`world+0x138`, the TCoord mask) has no
simulation reader: a scan of every function touching `field_0x138` for `& 4`, `& 0xc`, `>> 2 & 1`,
`| 4` and `& 0xfffb` finds only `mark_behind_tiles` itself (the sim readers mask `0x30`, `3`, `1`,
`0x100`, `0x200`, `0x300`, `0x2000`, `0x4000`, `0x6000`). `docs/ROADS.md` §6–§7 already carries the
resulting tile-mask residue as bit `0x4` alone.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 008adda0 | TerrainBorder::build_edge | open | y | ui | 34,994 bytes, read by its first screen, its callees and its callers: float vertex/face/texcoord emission (`FaceArray::add` ×86, `TexArray::add` ×56, `MiscAccess::terrain`), callers `TerrainOut::render@008558b0` and `TerrainOut::add_border_piece@00863910`. The terrain-type blend edge mesh; no national meaning. `BORDER_RENDER`. |
| 008af520 | TerrainBorder::create_borders | open | y | ui | Reads terrain types from `worldc` to decide which blend edges to build (`build_edge`); called from the `TerrainOut` render path only. `BORDER_RENDER`. |
| 008afdb0 | TerrainBorder::add_render_bucket | open | y | ui | Buckets edge meshes for drawing against `MiscAccess::camera`. `BORDER_RENDER`. |
| 0063d0a0 | Wall::refresh_nearby_tiles | open | y | ui | Called only from `Wall::close@0063f6e0`: scans a ring of cells around the closing building and, for every **active** building object on them, calls `mark_behind_tiles(1)` to restore the bit `0x4` the closing one cleared. Only bit `0x4` moves (see above); render occlusion. Proposed census pattern **`BEHIND_MASK`**: short name `^(mark_behind_tiles|refresh_nearby_tiles|set_behind\w*|tell_behind)$` — catches this row, `Wall::mark_behind_tiles@0063d230`, `World::set_behind@006b4230`, `Mountains::set_behind_flags@00898050`, `ConsoleWin::tell_behind@007e57d0`. |
| 0063d440 | Wall::kill_at_tile | open | y | twin-differs | Twin: the per-tile body inside `Sim::wall_start` (`crates/sim/src/city.rs:1896`, the `victims` loop at :1906). Both gate on the TCoord mask's `0x40` (`tile::PLACED_TWICE`) and disband the rival with a full refund (`Object::disband(…, 1)` / `disband_building(o, true)`). **They differ on a started rival**: the original asks `ObjectsData::find_building_placed_at(x, y, -1, o, who)` for the first player building (`who < 8`, `(o, who)` excluded) whose footprint covers the tile, and `call *0x50(%eax); jne` at `0063d4b8` (the listing; the decompiler prints an empty `if`) **ends the tile** when that first one is started; the crate skips started buildings and still disbands later unstarted ones. Also the crate takes any owner, the original only `who < 8`. Reachable only when a started building and an unstarted one share a double-placed tile at a third building's start — `docs/CITIES.md` §12 leaves exactly this `UNSURE`. The console sound on the disband is `sound`. Would show in `BUILDDATA` (a site that survives in the original and is disbanded here) on a capture with three overlapping placements. |
| 0063d560 | Wall::update_construct_time | open | y | twin | `crates/sim/src/build.rs:518` `construct_base`: same seven layers in the same order — Maya unless wonder (`vslot +0x2c`), `BUILDINGS_CREATED_FASTER` `×3>>2`, Versailles (`has_wonder 0x218`), tobacco (`+0x6da5`/`+0x6dcd & 0x20`), British on `is(0x20b)` (Air Defense), Dutch and Roman on `is_fort` (`+0xfc`) unless wonder, `capital_build_time` when `city_num` (`+0x3f8`) is 0, then `(10 − upgrade)·c/10`. Each step `c·100/(k+100)` on both sides; unsigned there, positive `i32` here. Agree. `docs/CITIES.md` §3.2, `docs/NATIONS.md` rows cite it. |
| 0063d790 | Wall::start_me | open | y | twin-differs | Twins: `start_me(1)` at `crates/sim/src/city.rs:1542` (placement: `0x80` if clear, else `0x40`) — agrees; `start_me(0)` at `city.rs:2625` ("release the reservation") — **differs**: the original clears `0x80` only when `0x40` is clear, and on a `0x40` tile clears both bits **only if** `find_building_placed_at` finds no other building placed there; the crate clears `PLACED | PLACED_TWICE` unconditionally. So cancelling one of two overlapping unstarted sites leaves the survivor's shared tiles reserved (`0x80|0x40`) in the original and free here — visible to every `tile::PLACED` reader (`roads.rs:285/625`, `ai_place.rs:244/309`, `caravan.rs:932`, `transport.rs:1726`). `start_me(1)`'s tail `Doober::expire_cover_doobers` is render. Would show in the world dump's tile mask (`world+0x138`, bits `0x80`/`0x40`; the run72/run73 tile-mask diff of `docs/ROADS.md`) on a capture that cancels one of two overlapping sites. |
| 0063fb60 | Wall::inc_time | open | y | twin | `crates/sim/src/air.rs:3838` `walls_inc_time` → `wall_anti_air` (the `recharging +0x7a` cycle on `get_game_frames(8)`/`(0xc)`, gated on `is_active`, `has_objmask(0x80000000)` and the `is(0x209)`/`is(0x20a)` returns, then `execute_game_events` at `cur_anim 0xc`), and `city.rs:2929` `refresh_site_hits` (the `update_hits(this, 0)` of an inactive site). The rest is `GraphicPieces::emit_*_particles` (render). The branch order is the listing's (the decompile flattens it); `docs/COMBAT.md` §84 is diff-backed on run404's 1,157 Radar blocks. One stated difference, documented in the crate: site hits are refreshed in a pass after all objects, not interleaved per leader. |
| 00642da0 | Wall::decrement_stats | open | y | twin-differs | Mirror of `increment_stats`: inactive → `num_queued[type]` −1 (saturating at 0); active → `num_buildings[type]` and `reg_buildings[region of the building's tile][type]` −1 (saturating, region `< 64`), and `SimpleArray<int>::remove` from `mil_trainers`. Twins: `mil_trainer_close` (`crates/sim/src/ai_units.rs:2113`, order-preserving remove — agrees) and the counted readers `num_buildings_of`/`num_sites_of` (`ai_types.rs:45/53`), `reg_buildings_raw` (`ai_build.rs:217`), which recount live state. Differs with `increment_stats` (next row). |
| 00643270 | Wall::increment_stats | open | y | twin-differs | Inactive arm: `num_queued[type]` +1 (the unit-counter tail `+0xa10..0xa24` needs `is_unit_type`, type `0x32..0x19d`, never a building type — inert here). Active arm: `num_buildings[type]`, `reg_buildings[reg][type]` (reg `< 64`), and append to `mil_trainers` if `is_military_trainer`. Twins: `mil_trainer_open` (`ai_units.rs:2091`, gated as `Wall::activate@0063e4b0:56` is — `build_flags 0x10` or in a city — agrees), but **the counts differ**: `num_buildings_of`, `reg_buildings_raw` count every `alive && active` building, while the original counts only those `Wall::activate:56` passed (`NO_CITY` or in a city) and moves them on `Build::add_to_city`/`remove_from_city`/`City::close`. A finished non-`NO_CITY` building outside any city counts here, not there (`docs/CITIES.md` §4.2 states the rule). Dump: `LEADERDATA` `num_buildings[scan]` and `reg_buildings[scan][scan2]` are printed and pinned **uncompared** (`crates/rondata/src/diff/coverage.rs:275`). |
| 0041bfe0 | Window::on_app_focus | layout | y | dead | `ret 4` — an empty one-argument `thiscall`; the name is one of many COMDAT-folded bodies at this address, and the lockstep entry is some simulation stub folded onto it. Nothing to port. Proposed census mark: a body of one `ret`/`ret N` (or the `Window::` class, `is_ui_body`). |
| 0041c000 | GraphicWinXML::get_graphicwinxml | layout | y | twin | `mov eax, ecx; ret` — returns `this`; folded identity cast. `crates/sim/src/flag_reads.rs:145` `graphic_win_xml` (identity) — agrees; it has no simulation consumer beyond the sweep, so `dead` is equally defensible. |
| 0041dbe0 | Window::on_focus | layout | y | dead | `mov eax, 1; ret 4` — a folded "answer 1" stub. Window focus is UI; the lockstep entry is a folded sim stub. Census: `Window::` → `ui`, or the one-instruction-body mark. |
| 0046cda0 | SubObjectData::is_active | layout | y | twin | `flags & 1`; `crates/sim/src/flag_reads.rs:22` `sub_object_is_active` and the many `SubObjectData::is_active` readings (`fight.rs:1943`, `airbase.rs:199`) — agree. |
| 00471360 | WallData::can_capture | layout | y | twin | Listing: if the type's vslot `+0x64` is `BuildTypeData::is_city` (`0x472bc0`) answer `is(0x19e, 0)` (the `VILLAGE` line) else tail-jump to that slot. Twin `Sim::building_is_city` (`crates/sim/src/city.rs:331`, `building_is(b, Ident::Village)`), read by `capture_eligible` (`city.rs:2945`) — agrees (`docs/CITIES.md` §7.1). |
| 004722f0 | WallData::is_sea_trade | layout | - | twin | `ObjectData::is(0x1b0, 0)` (the Dock line) through the object's vslot `+0xb8`. Twin `build::is_dock` (`crates/sim/src/build.rs:466`, `is(types, t, Ident::Dock)`) — agrees; the sweep's `obj_is_sea_trade` (`trade_view.rs:73`) is the trade search's slot `+0x28`. |
| 00472370 | WallOut::draw | layout | - | ui | Forwards to `WallOut::draw(uchar)@00641930`. Caught by `is_ui_body` (`*Out`). |
| 00472410 | WallData::has_repeat_air | layout | y | twin | `build_masks & 0x80`; the crate's `Building::repeat_air` (`crates/sim/src/lib.rs:1738`), read by `repeat_air` (`air.rs:2462`) — agrees. |
| 00472430 | BuildData::can_make | layout | - | absorbed | `jmp *0x198(%eax)` — the overload at vslot `+0x194` forwarding to the one at `+0x198`, which on `Build::vftable` is `BuildData::can_make@0062db10`, the one the crate models (`lib.rs:4076`, `ai_host.rs:1127`). An adapter, no logic. |
| 00472cb0 | WallData::can_make | layout | - | dead | `xor eax, eax; ret 8`: `Wall::vftable`'s `+0x198`; every live object is a `Build`, whose vtable overrides the slot. Never dispatched. |
| 00473650 | WallData::is_defensive | layout | y | twin | Listing: `is(0x1b7)` ‖ `is(0x1bb)` ‖ `is(0x1bf)` ‖ type `+0x1e8 != 0`, short-circuit in that order. `crates/sim/src/build.rs:444` `is_defensive_by` (Tower, Fort, Airbase, `attack != 0`) — agrees. |
| 0063d230 | Wall::mark_behind_tiles | layout | y | ui | Sets/clears bit `0x4` on the TCoord mask over a band `buildtype+0x2dc` (`behind_height`) rows deep in front of the footprint; callers `Wall::start`, `Wall::close`, `refresh_nearby_tiles`, `SpellType::cast_bribe`. No simulation reader of bit `0x4` (scan above). Not modelled; `docs/ROADS.md` §6 pins the residue as `0x4`-only. `BEHIND_MASK`. |
| 0063d3a0 | Wall::kill_competing_buildings | layout | y | twin | The footprint loop (`tile_corner`, `x_size` `+0x234` × `y_size` `+0x238`, in-bounds tiles) calling `kill_at_tile`; twin the `footprint(ty, corner)` loop in `Sim::wall_start` (`city.rs:1906`) — agrees on the tiles; the per-tile body differs as `kill_at_tile`'s row says. Called only from `Wall::start@0063e810`. |

## Summary

**Counts (60):** `ui` 42 · `twin` 9 · `twin-differs` 4 · `dead` 3 · `editor` 1 · `absorbed` 1 ·
`unmodelled` 0. Of the 37 `borders` rows, 36 are the drawn border line and one its params editor;
none computes territory. Territory itself is `World::compute_reg_territory@006b0bb0`, ported in
`crates/sim/src/territory.rs`, which only *notifies* this family (`add_border_region`,
`remove_border_region`, `save_last_who`).

**Proposed census classes** (Ramon's to accept):

- **`BORDER_RENDER`** — class `^(Borders?|BordersOut|BorderNodes|BorderMorphRender|BorderSplineRender|BorderSpline|MorphSideStrip|StripBuilder|LinkVert|TerrainBorder)$`.
  Catches all 37 `borders` rows and the 3 `terrainborder` rows here (and the family's
  constructors, `init`, `draw`, `close`, already dead weight). Evidence: the only root is
  `Scene::render@008b3270` → `Borders::process_all`, clocked by `graphic_tick` and
  `camera_changed`; the sim's three calls write only the `Borders` singleton and the `Region+0x30`
  handle, which no simulation function reads.
- **`BEHIND_MASK`** — short name `^(mark_behind_tiles|refresh_nearby_tiles|set_behind\w*|tell_behind)$`.
  Catches `Wall::mark_behind_tiles@0063d230`, `Wall::refresh_nearby_tiles@0063d0a0`,
  `World::set_behind@006b4230`, `Mountains::set_behind_flags@00898050`,
  `ConsoleWin::tell_behind@007e57d0`. Evidence: bits `0x4`/`0x8` of `world+0x138` have no reader
  among the simulation functions touching that array.
- **A one-instruction-body mark** (or the `Window::` class into `is_ui_body`): `Window::on_app_focus@0041bfe0`
  (`ret 4`), `Window::on_focus@0041dbe0` (`mov eax,1; ret 4`). COMDAT-folded stubs whose name is an accident of the fold.

**Proposed replaced groups:** none.

**`unmodelled`:** none. In their place, the four `twin-differs`, ranked (all lockstep-entered):

1. **`Wall::increment_stats@00643270` / `Wall::decrement_stats@00642da0`** — the crate's
   `num_buildings` and `reg_buildings` count every live active building; the original counts only
   those `Wall::activate:56` filed (`NO_CITY` or in a city) and moves them with city membership.
   Field: `LEADERDATA` `num_buildings[scan]`, `reg_buildings[scan][scan2]` — printed and pinned
   uncompared (`crates/rondata/src/diff/coverage.rs:275`); comparing them is a widening, not a capture.
2. **`Wall::start_me@0063d790`** (`start_me(0)`) — cancelling one of two overlapping unstarted
   sites keeps the survivor's shared tiles `0x80|0x40` in the original; the crate clears them.
   Field: world tile mask (`world+0x138`) bits `0x80`/`0x40`, on a capture that cancels an
   overlapping site.
3. **`Wall::kill_at_tile@0063d440`** — the original's tile sweep stops at a started first-found
   rival (`0063d4b8`, `jne` on vslot `+0x50`), the crate's skips it; and the crate takes owners
   `>= 8`. Field: `BUILDDATA` presence of the site, on a three-way overlap. Very likely
   unreachable in play (`docs/CITIES.md` §12's `UNSURE`).

## Not established

- **Bit `0x4` readers beyond `field_0x138` spellings.** The scan read every function that names
  `field_0x138`; a reader through another alias of the TCoord mask pointer (a cached copy, a
  `World` method the decompiler types differently) would be missed. The render side
  (`GuyOut`, `TerrainOut`) is the expected reader and was not traced.
- **`Wall::inc_time`'s branch order** was taken from the crate's listing-based reading and
  `docs/COMBAT.md` §84's run404 diff, not re-derived here; the decompile flattens the `recharging`
  arms.
- **Severity of `start_me(0)`** depends on whether `BuildType::mask_me@006312a0` clears
  `0x80`/`0x40` when the survivor later starts (the crate's `mask_building` does); not read.
- **`kill_at_tile`'s first-found order** (`move_x`/`move_y` neighbour cells, then each cell's
  object chain) was not compared with the crate's building-index order; it matters only on the
  reachability question above.
- **`Borders::is_border_hidden`'s tribe bonus `0x13`** — which nation it is was not looked up; it
  changes only the drawn line, not territory.
- **`Borders::get_who@00883190`** has no callers in the export (inlined or folded); ruled with its
  family.
